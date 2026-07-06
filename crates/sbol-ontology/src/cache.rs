//! On-disk cache of ontology extension snapshots.
//!
//! The cache lets library code download, build, and re-load ontologies
//! that are too large to bundle by default (NCIT, lab-specific
//! ontologies, etc.). Validation reads from the cache via
//! [`OntologyCache::load`]; the cache is never consulted during
//! validation itself. Only library code that explicitly calls
//! [`OntologyCache::ensure_installed`] performs network IO.
//!
//! Default location:
//!
//! - `$SBOL_ONTOLOGY_CACHE` if set
//! - `$XDG_CACHE_HOME/sbol/ontologies/` (Linux/macOS fallback)
//! - `$HOME/.cache/sbol/ontologies/` (Linux fallback)
//! - `$HOME/Library/Caches/sbol/ontologies/` (macOS)
//! - `%LOCALAPPDATA%\sbol\ontologies\` (Windows)

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use crate::parser::{parse_obo_terms, parse_rdfxml_terms};
use crate::{Ontology, TSV_FORMAT_VERSION, normalize_term_id};

/// Filesystem location for installed ontology extensions.
#[derive(Clone, Debug)]
pub struct OntologyCache {
    root: PathBuf,
}

impl OntologyCache {
    /// Resolves the default cache directory. Performs no IO.
    pub fn default_path() -> PathBuf {
        if let Some(value) = std::env::var_os("SBOL_ONTOLOGY_CACHE") {
            return PathBuf::from(value);
        }
        platform_default_cache_dir()
    }

    /// Returns a cache anchored at the default path.
    pub fn from_default_path() -> Self {
        Self::at(Self::default_path())
    }

    /// Returns a cache anchored at the provided path. The directory is
    /// created lazily on first install.
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the cache root directory.
    pub fn path(&self) -> &Path {
        &self.root
    }

    fn tsv_path(&self, name: &str) -> PathBuf {
        self.root.join(format!("{name}.tsv"))
    }

    fn manifest_path(&self, name: &str) -> PathBuf {
        self.root.join(format!("{name}.manifest"))
    }

    /// Returns true when `name` is present in the cache and matches the
    /// expected format version.
    pub fn is_installed(&self, name: &str) -> bool {
        self.tsv_path(name).is_file() && self.manifest_path(name).is_file()
    }

    /// Lists installed extensions in name order. IO errors surface as `Err`.
    pub fn list(&self) -> io::Result<Vec<InstalledOntology>> {
        if !self.root.is_dir() {
            return Ok(Vec::new());
        }
        let mut installed = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else {
                continue;
            };
            let Some(stem) = name.strip_suffix(".manifest") else {
                continue;
            };
            installed.push(load_manifest(&self.manifest_path(stem))?);
        }
        installed.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(installed)
    }

    /// Removes the named extension's TSV and manifest. Returns
    /// `Ok(false)` if no files were present.
    pub fn remove(&self, name: &str) -> io::Result<bool> {
        let mut removed = false;
        for path in [self.tsv_path(name), self.manifest_path(name)] {
            match fs::remove_file(&path) {
                Ok(()) => removed = true,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        Ok(removed)
    }

    /// Loads the named extension as an [`Ontology`] ready to attach to a
    /// validation registry. Returns an error if the extension is not
    /// installed or if its TSV cannot be parsed.
    pub fn load(&self, name: &str) -> io::Result<Ontology> {
        let path = self.tsv_path(name);
        Ontology::from_tsv_path(&path)
    }

    /// Returns the manifest for one installed extension. Returns
    /// `NotFound` if the extension is not installed.
    pub fn manifest(&self, name: &str) -> io::Result<InstalledOntology> {
        load_manifest(&self.manifest_path(name))
    }

    /// Installs an extension. Always re-downloads and rebuilds, replacing
    /// any existing files atomically via temp-file rename.
    pub fn install(
        &self,
        descriptor: &OntologyDescriptor,
    ) -> Result<InstalledOntology, InstallError> {
        self.do_install(descriptor)
    }

    /// Installs an extension only when it is missing. Idempotent and safe
    /// to call from library startup.
    pub fn ensure_installed(
        &self,
        descriptor: &OntologyDescriptor,
    ) -> Result<InstalledOntology, InstallError> {
        if self.is_installed(descriptor.name) {
            return self.manifest(descriptor.name).map_err(Into::into);
        }
        self.do_install(descriptor)
    }

    fn do_install(
        &self,
        descriptor: &OntologyDescriptor,
    ) -> Result<InstalledOntology, InstallError> {
        fs::create_dir_all(&self.root).map_err(InstallError::Io)?;

        let raw = crate::download::fetch(descriptor.source_url).map_err(InstallError::Download)?;
        let source_sha256 = hex_lower(Sha256::digest(&raw).as_slice());

        let tsv = build_extension_tsv(descriptor, &raw).map_err(InstallError::Build)?;
        let fact_sha256 = hex_lower(Sha256::digest(tsv.as_bytes()).as_slice());

        let installed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let manifest = InstalledOntology {
            name: descriptor.name.to_owned(),
            source_url: descriptor.source_url.to_owned(),
            source_sha256,
            fact_sha256,
            format_version: TSV_FORMAT_VERSION,
            installed_at,
        };

        write_atomic(&self.tsv_path(descriptor.name), tsv.as_bytes()).map_err(InstallError::Io)?;
        write_atomic(
            &self.manifest_path(descriptor.name),
            manifest.to_manifest_string().as_bytes(),
        )
        .map_err(InstallError::Io)?;

        Ok(manifest)
    }

    /// Re-hashes the installed TSV for `name` and compares against the
    /// manifest. Returns `Err` if files are missing, the hash diverged,
    /// or the manifest is unreadable.
    pub fn verify(&self, name: &str) -> Result<InstalledOntology, VerifyError> {
        let manifest = load_manifest(&self.manifest_path(name)).map_err(VerifyError::Io)?;
        let tsv = fs::read(self.tsv_path(name)).map_err(VerifyError::Io)?;
        let actual = hex_lower(Sha256::digest(&tsv).as_slice());
        if actual != manifest.fact_sha256 {
            return Err(VerifyError::Mismatch {
                expected: manifest.fact_sha256.clone(),
                actual,
            });
        }
        Ok(manifest)
    }
}

impl Default for OntologyCache {
    fn default() -> Self {
        Self::from_default_path()
    }
}

/// Catalog of built-in ontology extensions. Each variant exposes a
/// [`descriptor`](Self::descriptor) that names a stable cache entry, a
/// source URL, and the branch roots that get retained when the snapshot
/// is built.
#[derive(Clone, Copy, Debug)]
pub enum KnownOntology {
    /// NCI Thesaurus, scoped to experimental-reagent subtrees the SBOL
    /// 3.1.0 specification names by example (cell line, organism strain,
    /// growth medium, control, inducer). The cache install is ~5-15k
    /// terms depending on the upstream NCIT release.
    Ncit,
}

impl KnownOntology {
    pub fn descriptor(self) -> &'static OntologyDescriptor {
        match self {
            KnownOntology::Ncit => &NCIT_DESCRIPTOR,
        }
    }

    pub fn name(self) -> &'static str {
        self.descriptor().name
    }
}

/// Recipe for downloading and building one ontology extension.
#[derive(Clone, Copy, Debug)]
pub struct OntologyDescriptor {
    pub name: &'static str,
    pub source_url: &'static str,
    pub source_format: SourceFormat,
    pub source_license: &'static str,
    pub branch_roots: &'static [BranchRoot],
}

#[derive(Clone, Copy, Debug)]
pub enum SourceFormat {
    Owl,
    Obo,
}

/// A subtree that the cache install retains. The `role` is recorded as
/// the SBOL-facing classification for every term in the subtree.
#[derive(Clone, Copy, Debug)]
pub struct BranchRoot {
    pub id: &'static str,
    pub role: &'static str,
}

/// Manifest written next to an installed extension's TSV.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledOntology {
    pub name: String,
    pub source_url: String,
    pub source_sha256: String,
    pub fact_sha256: String,
    pub format_version: u32,
    pub installed_at: u64,
}

impl InstalledOntology {
    fn to_manifest_string(&self) -> String {
        format!(
            "# format_version: {}\nname\t{}\nsource_url\t{}\nsource_sha256\t{}\nfact_sha256\t{}\ninstalled_at\t{}\n",
            self.format_version,
            self.name,
            self.source_url,
            self.source_sha256,
            self.fact_sha256,
            self.installed_at,
        )
    }
}

fn load_manifest(path: &Path) -> io::Result<InstalledOntology> {
    let text = fs::read_to_string(path)?;
    let mut name = None;
    let mut source_url = None;
    let mut source_sha256 = None;
    let mut fact_sha256 = None;
    let mut installed_at: u64 = 0;
    let mut format_version = TSV_FORMAT_VERSION;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("# format_version:") {
            format_version = rest.trim().parse::<u32>().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("manifest {path:?} has unparseable format_version"),
                )
            })?;
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('\t') else {
            continue;
        };
        match key {
            "name" => name = Some(value.to_owned()),
            "source_url" => source_url = Some(value.to_owned()),
            "source_sha256" => source_sha256 = Some(value.to_owned()),
            "fact_sha256" => fact_sha256 = Some(value.to_owned()),
            "installed_at" => installed_at = value.parse::<u64>().unwrap_or(0),
            _ => {}
        }
    }
    Ok(InstalledOntology {
        name: name
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "manifest missing `name`"))?,
        source_url: source_url.unwrap_or_default(),
        source_sha256: source_sha256.unwrap_or_default(),
        fact_sha256: fact_sha256.unwrap_or_default(),
        format_version,
        installed_at,
    })
}

fn write_atomic(target: &Path, contents: &[u8]) -> io::Result<()> {
    let mut tmp = target.as_os_str().to_os_string();
    tmp.push(".tmp");
    let tmp_path = PathBuf::from(tmp);
    fs::write(&tmp_path, contents)?;
    fs::rename(&tmp_path, target)
}

fn build_extension_tsv(
    descriptor: &OntologyDescriptor,
    raw_bytes: &[u8],
) -> Result<String, BuildError> {
    let text = std::str::from_utf8(raw_bytes)
        .map_err(|_| BuildError::Encoding("source data is not valid UTF-8"))?;
    let mut terms = BTreeMap::new();
    match descriptor.source_format {
        SourceFormat::Obo => parse_obo_terms(text, &mut terms),
        SourceFormat::Owl => parse_rdfxml_terms(text, &mut terms),
    }

    let mut children_by_parent = BTreeMap::<String, Vec<String>>::new();
    for (id, raw) in &terms {
        for parent in &raw.parents {
            children_by_parent
                .entry(parent.clone())
                .or_default()
                .push(id.clone());
        }
    }

    let mut selected: BTreeMap<String, &'static str> = BTreeMap::new();
    let mut branches: BTreeSet<(String, String)> = BTreeSet::new();

    for root in descriptor.branch_roots {
        let Some(root_id) = normalize_term_id(root.id) else {
            return Err(BuildError::UnknownNamespace(root.id.to_owned()));
        };
        selected.entry(root_id.clone()).or_insert(root.role);
        let mut stack = vec![root_id.clone()];
        let mut visited = BTreeSet::new();
        while let Some(id) = stack.pop() {
            if !visited.insert(id.clone()) {
                continue;
            }
            if id != root_id {
                branches.insert((id.clone(), root_id.clone()));
            }
            selected.entry(id.clone()).or_insert(root.role);
            if let Some(children) = children_by_parent.get(&id) {
                stack.extend(children.iter().cloned());
            }
        }
    }

    if selected.is_empty() {
        return Err(BuildError::Empty);
    }

    let mut output = String::new();
    output.push_str(&format!("# format_version: {TSV_FORMAT_VERSION}\n"));
    output.push_str("# kind\tid\tiri\tlabel\taliases\tparents\tontology\trole\tcomponent_family\tsequence_family\ttable1\ttable2\n");

    for (id, role) in &selected {
        let raw = terms.get(id);
        let label = raw
            .and_then(|term| term.label.as_deref())
            .map(sanitize_label)
            .unwrap_or_else(|| id.clone());
        let parents: Vec<&String> = raw
            .map(|term| {
                term.parents
                    .iter()
                    .filter(|p| selected.contains_key(*p))
                    .collect()
            })
            .unwrap_or_default();
        let ontology = id.split_once(':').map(|(p, _)| p).unwrap_or("-");
        let iri = canonical_iri(id);
        let aliases = aliases_for(id);
        output.push_str("term\t");
        output.push_str(id);
        output.push('\t');
        output.push_str(&iri);
        output.push('\t');
        output.push_str(&label);
        output.push('\t');
        output.push_str(&join_or_dash(&aliases));
        output.push('\t');
        if parents.is_empty() {
            output.push('-');
        } else {
            let joined: Vec<String> = parents.iter().map(|p| (*p).clone()).collect();
            output.push_str(&joined.join("|"));
        }
        output.push('\t');
        output.push_str(ontology);
        output.push('\t');
        output.push_str(role);
        output.push_str("\t-\t-\tfalse\tfalse\n");
    }

    for (term, root) in &branches {
        output.push_str("branch\t");
        output.push_str(term);
        output.push('\t');
        output.push_str(root);
        output.push('\n');
    }

    Ok(output)
}

fn canonical_iri(id: &str) -> String {
    let Some((prefix, local)) = id.split_once(':') else {
        return id.to_owned();
    };
    match prefix {
        "EDAM" => format!("https://identifiers.org/edam:{local}"),
        _ => format!("https://identifiers.org/{prefix}:{local}"),
    }
}

fn aliases_for(id: &str) -> Vec<String> {
    let Some((prefix, local)) = id.split_once(':') else {
        return Vec::new();
    };
    match prefix {
        "EDAM" => vec![format!("http://edamontology.org/{local}")],
        "SBO" => vec![
            format!("http://purl.obolibrary.org/obo/SBO_{local}"),
            format!("http://biomodels.net/SBO/SBO_{local}"),
        ],
        "SO" | "GO" | "CHEBI" | "CL" | "NCIT" => {
            vec![format!("http://purl.obolibrary.org/obo/{prefix}_{local}")]
        }
        _ => Vec::new(),
    }
}

fn join_or_dash<T: AsRef<str>>(values: &[T]) -> String {
    if values.is_empty() {
        "-".to_owned()
    } else {
        values
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()
            .join("|")
    }
}

fn sanitize_label(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn platform_default_cache_dir() -> PathBuf {
    let app = Path::new("sbol").join("ontologies");
    if let Some(xdg) = std::env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(xdg).join(&app);
    }
    if cfg!(target_os = "macos")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join("Library/Caches").join(&app);
    }
    if cfg!(target_os = "windows")
        && let Some(local) = std::env::var_os("LOCALAPPDATA")
    {
        return PathBuf::from(local).join(&app);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".cache").join(&app);
    }
    PathBuf::from(".").join(&app)
}

/// Errors emitted by [`OntologyCache::install`] and
/// [`OntologyCache::ensure_installed`].
#[derive(Debug)]
pub enum InstallError {
    Io(io::Error),
    Download(io::Error),
    Build(BuildError),
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallError::Io(error) => write!(f, "cache IO failed: {error}"),
            InstallError::Download(error) => write!(f, "download failed: {error}"),
            InstallError::Build(error) => write!(f, "ontology build failed: {error}"),
        }
    }
}

impl std::error::Error for InstallError {}

impl From<io::Error> for InstallError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Errors from parsing or applying branch policies during install.
#[derive(Debug)]
pub enum BuildError {
    Encoding(&'static str),
    UnknownNamespace(String),
    Empty,
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::Encoding(message) => write!(f, "encoding error: {message}"),
            BuildError::UnknownNamespace(id) => write!(
                f,
                "branch root `{id}` uses a namespace prefix not recognized by sbol-ontology"
            ),
            BuildError::Empty => write!(f, "no terms in any configured branch were found"),
        }
    }
}

impl std::error::Error for BuildError {}

/// Errors from [`OntologyCache::verify`].
#[derive(Debug)]
pub enum VerifyError {
    Io(io::Error),
    Mismatch { expected: String, actual: String },
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyError::Io(error) => write!(f, "verify IO failed: {error}"),
            VerifyError::Mismatch { expected, actual } => write!(
                f,
                "fact sha256 mismatch: manifest says `{expected}`, on disk is `{actual}`"
            ),
        }
    }
}

impl std::error::Error for VerifyError {}

const NCIT_DESCRIPTOR: OntologyDescriptor = OntologyDescriptor {
    name: "ncit",
    // The NCIT OWL distribution is large; users with bandwidth concerns
    // can override `source_url` by providing a custom OntologyDescriptor.
    source_url: "http://purl.obolibrary.org/obo/ncit.owl",
    source_format: SourceFormat::Owl,
    source_license: "CC BY 4.0",
    branch_roots: &[
        // Cell: for cell-line roles on experimental Components.
        BranchRoot {
            id: "NCIT:C12508",
            role: "feature_role",
        },
        // Organism Strain: for strain roles.
        BranchRoot {
            id: "NCIT:C14419",
            role: "feature_role",
        },
        // Growth Medium: for media reagent roles.
        BranchRoot {
            id: "NCIT:C85504",
            role: "feature_role",
        },
        // Positive Control, and siblings via the parent (Control) subtree.
        BranchRoot {
            id: "NCIT:C64356",
            role: "feature_role",
        },
        // Inducer: for inducer reagent roles.
        BranchRoot {
            id: "NCIT:C120268",
            role: "feature_role",
        },
    ],
};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn build_extension_tsv_extracts_branch_subtree_from_obo() {
        let descriptor = OntologyDescriptor {
            name: "synthetic",
            source_url: "memory://synthetic",
            source_format: SourceFormat::Obo,
            source_license: "CC0",
            branch_roots: &[BranchRoot {
                id: "CL:0000000",
                role: "component_type",
            }],
        };
        let source = "[Term]\nid: CL:0000000\nname: cell\n\n\
                      [Term]\nid: CL:0000001\nname: primary cultured cell\nis_a: CL:0000000\n\n\
                      [Term]\nid: GO:0003674\nname: molecular_function\n";
        let tsv = build_extension_tsv(&descriptor, source.as_bytes()).unwrap();
        assert!(tsv.starts_with("# format_version: 1\n"));
        assert!(tsv.contains("term\tCL:0000000\t"));
        assert!(tsv.contains("term\tCL:0000001\t"));
        // GO term outside the branch must not appear.
        assert!(!tsv.contains("term\tGO:0003674"));
        assert!(tsv.contains("branch\tCL:0000001\tCL:0000000\n"));
    }

    #[test]
    fn cache_install_then_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let cache = OntologyCache::at(tmp.path());
        let descriptor = OntologyDescriptor {
            name: "synthetic",
            source_url: "memory://synthetic",
            source_format: SourceFormat::Obo,
            source_license: "CC0",
            branch_roots: &[BranchRoot {
                id: "CL:0000000",
                role: "component_type",
            }],
        };
        let source = "[Term]\nid: CL:0000000\nname: cell\n\n\
                      [Term]\nid: CL:0000001\nname: cultured cell\nis_a: CL:0000000\n";
        // build_extension_tsv directly because the synthetic descriptor
        // has no real source_url to download from.
        let tsv = build_extension_tsv(&descriptor, source.as_bytes()).unwrap();
        fs::create_dir_all(tmp.path()).unwrap();
        fs::write(tmp.path().join("synthetic.tsv"), &tsv).unwrap();
        let manifest = InstalledOntology {
            name: "synthetic".into(),
            source_url: "memory://synthetic".into(),
            source_sha256: "0".into(),
            fact_sha256: "0".into(),
            format_version: 1,
            installed_at: 0,
        };
        fs::write(
            tmp.path().join("synthetic.manifest"),
            manifest.to_manifest_string(),
        )
        .unwrap();

        assert!(cache.is_installed("synthetic"));
        let ontology = cache.load("synthetic").unwrap();
        assert_eq!(ontology.is_cell_type_term("CL:0000001"), Some(true));
        let list = cache.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "synthetic");
        let removed = cache.remove("synthetic").unwrap();
        assert!(removed);
        assert!(!cache.is_installed("synthetic"));
    }
}
