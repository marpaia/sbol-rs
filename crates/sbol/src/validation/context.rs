use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(feature = "http-resolver")]
use std::io::Read;

use sbol_ontology::{Ontology, OntologyRegistry};

use crate::validation::options::ValidationOptions;
use crate::{Document, Iri, Object, Resource};

/// Controls whether validation may inspect resources outside the primary document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExternalValidationMode {
    /// Do not perform external document or content resolution.
    #[default]
    Off,
    /// Resolve only caller-provided documents and explicitly configured providers.
    ProvidedOnly,
    /// Resolve caller-provided data and configured external providers such as HTTP.
    ExternalAllowed,
}

/// Resolver-aware validation inputs.
#[derive(Default)]
#[non_exhaustive]
pub struct ValidationContext<'a> {
    options: ValidationOptions,
    ontology_registry: OntologyRegistry,
    external_mode: ExternalValidationMode,
    documents: Vec<&'a Document>,
    document_resolvers: Vec<&'a dyn DocumentResolver>,
    content_resolvers: Vec<&'a dyn ContentResolver>,
}

impl<'a> ValidationContext<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_options(mut options: ValidationOptions) -> Self {
        let extensions = options.take_ontology_extensions();
        let ontology_registry = if extensions.is_empty() {
            OntologyRegistry::bundled_only()
        } else {
            OntologyRegistry::bundled_with(extensions)
        };
        Self {
            options,
            ontology_registry,
            ..Self::default()
        }
    }

    pub fn options(&self) -> &ValidationOptions {
        &self.options
    }

    /// Returns the ontology view used by validation. Bundled-only by default;
    /// extension snapshots layered in via
    /// [`ValidationOptions::with_ontology_extension`] are merged on top.
    pub fn ontology(&self) -> &Ontology {
        self.ontology_registry.ontology()
    }

    /// Returns the underlying ontology registry.
    pub fn ontology_registry(&self) -> &OntologyRegistry {
        &self.ontology_registry
    }

    pub fn external_mode(&self) -> ExternalValidationMode {
        self.external_mode
    }

    pub fn documents(&self) -> &[&'a Document] {
        &self.documents
    }

    pub fn document_resolvers(&self) -> &[&'a dyn DocumentResolver] {
        &self.document_resolvers
    }

    pub fn content_resolvers(&self) -> &[&'a dyn ContentResolver] {
        &self.content_resolvers
    }

    pub fn with_external_mode(mut self, external_mode: ExternalValidationMode) -> Self {
        self.external_mode = external_mode;
        self
    }

    pub fn with_document(mut self, document: &'a Document) -> Self {
        self.documents.push(document);
        self
    }

    pub fn with_document_resolver(mut self, resolver: &'a dyn DocumentResolver) -> Self {
        self.document_resolvers.push(resolver);
        self
    }

    pub fn with_content_resolver(mut self, resolver: &'a dyn ContentResolver) -> Self {
        self.content_resolvers.push(resolver);
        self
    }
}

/// A set of in-memory SBOL documents indexed by object identity.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct DocumentSet<'a> {
    documents: Vec<&'a Document>,
    objects: BTreeMap<Resource, &'a Object>,
}

impl<'a> DocumentSet<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_documents(
        documents: impl IntoIterator<Item = &'a Document>,
    ) -> Result<Self, DocumentSetError> {
        let mut set = Self::new();
        for document in documents {
            set.add_document(document)?;
        }
        Ok(set)
    }

    pub fn add_document(&mut self, document: &'a Document) -> Result<(), DocumentSetError> {
        for identity in document.objects().keys() {
            if self.objects.contains_key(identity) {
                return Err(DocumentSetError::duplicate(identity.clone()));
            }
        }

        for (identity, object) in document.objects() {
            self.objects.insert(identity.clone(), object);
        }
        self.documents.push(document);
        Ok(())
    }

    pub fn documents(&self) -> &[&'a Document] {
        &self.documents
    }

    pub fn get(&self, identity: &Resource) -> Option<&'a Object> {
        self.objects.get(identity).copied()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DocumentSetError {
    identity: Resource,
}

impl DocumentSetError {
    fn duplicate(identity: Resource) -> Self {
        Self { identity }
    }

    pub fn identity(&self) -> &Resource {
        &self.identity
    }
}

impl fmt::Display for DocumentSetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "duplicate SBOL object identity `{}` in document set",
            self.identity
        )
    }
}

impl std::error::Error for DocumentSetError {}

/// Resolved byte content for an Attachment or Model source.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolvedContent {
    pub bytes: Vec<u8>,
    pub media_type: Option<String>,
}

impl ResolvedContent {
    pub fn new(bytes: impl Into<Vec<u8>>, media_type: Option<String>) -> Self {
        Self {
            bytes: bytes.into(),
            media_type,
        }
    }
}

/// Coarse class of a resolution failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolutionErrorKind {
    UnsupportedScheme,
    NotFound,
    InvalidData,
    Io,
    Http,
    Parse,
}

/// A resolver failure with a stable kind and human-readable context.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolutionError {
    kind: ResolutionErrorKind,
    message: String,
}

impl ResolutionError {
    pub fn new(kind: ResolutionErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ResolutionErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ResolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ResolutionError {}

impl From<io::Error> for ResolutionError {
    fn from(error: io::Error) -> Self {
        Self::new(ResolutionErrorKind::Io, error.to_string())
    }
}

/// Resolves an external resource into an SBOL document.
pub trait DocumentResolver {
    fn resolve_document(&self, resource: &Resource) -> Result<Document, ResolutionError>;
}

/// Resolves an Attachment or Model source into bytes.
pub trait ContentResolver {
    fn resolve_content(&self, source: &Iri) -> Result<ResolvedContent, ResolutionError>;
}

/// File-backed resolver for opt-in local validation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct FileResolver {
    roots: Vec<PathBuf>,
}

impl FileResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.roots.push(root.into());
        self
    }

    pub fn add_root(&mut self, root: impl Into<PathBuf>) {
        self.roots.push(root.into());
    }

    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    fn candidate_paths(&self, iri: &Iri) -> Result<Vec<PathBuf>, ResolutionError> {
        let value = iri.as_str();
        if let Some(path) = file_iri_path(value)? {
            return Ok(vec![path]);
        }

        let Some(relative_path) = iri_path_suffix(value) else {
            return Err(ResolutionError::new(
                ResolutionErrorKind::UnsupportedScheme,
                format!("unsupported source URI scheme for `{value}`"),
            ));
        };

        if self.roots.is_empty() {
            return Err(ResolutionError::new(
                ResolutionErrorKind::UnsupportedScheme,
                format!("no filesystem root configured for `{value}`"),
            ));
        }

        Ok(self
            .roots
            .iter()
            .map(|root| root.join(&relative_path))
            .collect())
    }

    fn read_bytes(&self, iri: &Iri) -> Result<Vec<u8>, ResolutionError> {
        let candidates = self.candidate_paths(iri)?;
        let mut saw_candidate = false;
        for path in candidates {
            saw_candidate = true;
            match fs::read(&path) {
                Ok(bytes) => return Ok(bytes),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(ResolutionError::new(
                        ResolutionErrorKind::Io,
                        format!("could not read `{}`: {error}", path.display()),
                    ));
                }
            }
        }

        let kind = if saw_candidate {
            ResolutionErrorKind::NotFound
        } else {
            ResolutionErrorKind::UnsupportedScheme
        };
        Err(ResolutionError::new(
            kind,
            format!("no filesystem content found for `{}`", iri.as_str()),
        ))
    }
}

impl ContentResolver for FileResolver {
    fn resolve_content(&self, source: &Iri) -> Result<ResolvedContent, ResolutionError> {
        let bytes = self.read_bytes(source)?;
        Ok(ResolvedContent::new(bytes, media_type_for(source.as_str())))
    }
}

impl DocumentResolver for FileResolver {
    fn resolve_document(&self, resource: &Resource) -> Result<Document, ResolutionError> {
        let Some(iri) = resource.as_iri() else {
            return Err(ResolutionError::new(
                ResolutionErrorKind::UnsupportedScheme,
                format!("blank node `{resource}` cannot be resolved as a document"),
            ));
        };
        let bytes = self.read_bytes(iri)?;
        let text = String::from_utf8(bytes).map_err(|error| {
            ResolutionError::new(
                ResolutionErrorKind::InvalidData,
                format!("resolved document `{iri}` was not UTF-8: {error}"),
            )
        })?;
        Document::read_turtle(&text).map_err(|error| {
            ResolutionError::new(
                ResolutionErrorKind::Parse,
                format!("resolved document `{iri}` was not valid Turtle: {error}"),
            )
        })
    }
}

#[cfg(feature = "http-resolver")]
/// HTTP(S) resolver for opt-in external validation.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct HttpResolver {
    agent: ureq::Agent,
}

#[cfg(feature = "http-resolver")]
impl Default for HttpResolver {
    fn default() -> Self {
        Self {
            agent: ureq::Agent::new(),
        }
    }
}

#[cfg(feature = "http-resolver")]
impl HttpResolver {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "http-resolver")]
impl ContentResolver for HttpResolver {
    fn resolve_content(&self, source: &Iri) -> Result<ResolvedContent, ResolutionError> {
        let value = source.as_str();
        if !value.starts_with("http://") && !value.starts_with("https://") {
            return Err(ResolutionError::new(
                ResolutionErrorKind::UnsupportedScheme,
                format!("unsupported HTTP source URI scheme for `{value}`"),
            ));
        }

        let response = self.agent.get(value).call().map_err(http_error)?;
        let media_type = response
            .header("content-type")
            .and_then(|header| header.split(';').next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let mut bytes = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut bytes)
            .map_err(ResolutionError::from)?;
        Ok(ResolvedContent::new(bytes, media_type))
    }
}

#[cfg(feature = "http-resolver")]
impl DocumentResolver for HttpResolver {
    fn resolve_document(&self, resource: &Resource) -> Result<Document, ResolutionError> {
        let Some(iri) = resource.as_iri() else {
            return Err(ResolutionError::new(
                ResolutionErrorKind::UnsupportedScheme,
                format!("blank node `{resource}` cannot be resolved as a document"),
            ));
        };
        let content = self.resolve_content(iri)?;
        let text = String::from_utf8(content.bytes).map_err(|error| {
            ResolutionError::new(
                ResolutionErrorKind::InvalidData,
                format!("resolved document `{iri}` was not UTF-8: {error}"),
            )
        })?;
        Document::read_turtle(&text).map_err(|error| {
            ResolutionError::new(
                ResolutionErrorKind::Parse,
                format!("resolved document `{iri}` was not valid Turtle: {error}"),
            )
        })
    }
}

/// Decorator that caches `HttpResolver` results on disk.
///
/// The cache directory mirrors:
/// `<cache_dir>/<sha256(source_iri)>/{content,media_type}`.
///
/// Cache hits are deterministic across CI runs (matters for sbol3-12805
/// hash verification, where re-fetching the source on every run would
/// produce flaky results when upstream content changes). Writes are
/// atomic via tmp-file + rename so a crashed run never leaves a corrupt
/// cache entry.
#[cfg(feature = "http-resolver")]
#[derive(Debug)]
pub struct CachingHttpResolver {
    inner: HttpResolver,
    cache_dir: PathBuf,
}

#[cfg(feature = "http-resolver")]
impl CachingHttpResolver {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            inner: HttpResolver::new(),
            cache_dir: cache_dir.into(),
        }
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    fn cache_paths(&self, source: &Iri) -> (PathBuf, PathBuf) {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(source.as_str().as_bytes());
        let digest = hex_digest_static(&hasher.finalize());
        let dir = self.cache_dir.join(&digest);
        (dir.join("content"), dir.join("media_type"))
    }

    fn read_cached(&self, source: &Iri) -> Option<ResolvedContent> {
        let (content_path, media_type_path) = self.cache_paths(source);
        let bytes = fs::read(&content_path).ok()?;
        let media_type = fs::read_to_string(&media_type_path).ok().and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            }
        });
        Some(ResolvedContent::new(bytes, media_type))
    }

    fn write_cached(&self, source: &Iri, content: &ResolvedContent) -> io::Result<()> {
        let (content_path, media_type_path) = self.cache_paths(source);
        if let Some(parent) = content_path.parent() {
            fs::create_dir_all(parent)?;
        }
        write_atomic(&content_path, &content.bytes)?;
        let media_type = content.media_type.as_deref().unwrap_or("");
        write_atomic(&media_type_path, media_type.as_bytes())
    }
}

#[cfg(feature = "http-resolver")]
impl ContentResolver for CachingHttpResolver {
    fn resolve_content(&self, source: &Iri) -> Result<ResolvedContent, ResolutionError> {
        if let Some(cached) = self.read_cached(source) {
            return Ok(cached);
        }
        let content = self.inner.resolve_content(source)?;
        let _ = self.write_cached(source, &content);
        Ok(content)
    }
}

#[cfg(feature = "http-resolver")]
impl DocumentResolver for CachingHttpResolver {
    fn resolve_document(&self, resource: &Resource) -> Result<Document, ResolutionError> {
        let Some(iri) = resource.as_iri() else {
            return Err(ResolutionError::new(
                ResolutionErrorKind::UnsupportedScheme,
                format!("blank node `{resource}` cannot be resolved as a document"),
            ));
        };
        let content = self.resolve_content(iri)?;
        let text = String::from_utf8(content.bytes).map_err(|error| {
            ResolutionError::new(
                ResolutionErrorKind::InvalidData,
                format!("resolved document `{iri}` was not UTF-8: {error}"),
            )
        })?;
        Document::read_turtle(&text).map_err(|error| {
            ResolutionError::new(
                ResolutionErrorKind::Parse,
                format!("resolved document `{iri}` was not valid Turtle: {error}"),
            )
        })
    }
}

#[cfg(feature = "http-resolver")]
fn hex_digest_static(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    use std::fmt::Write as _;
    for byte in bytes {
        write!(out, "{byte:02x}").unwrap();
    }
    out
}

#[cfg(feature = "http-resolver")]
fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)
}

#[cfg(feature = "http-resolver")]
fn http_error(error: ureq::Error) -> ResolutionError {
    match error {
        ureq::Error::Status(404, _) => {
            ResolutionError::new(ResolutionErrorKind::NotFound, "HTTP resource was not found")
        }
        ureq::Error::Status(status, response) => ResolutionError::new(
            ResolutionErrorKind::Http,
            format!("HTTP {status} while resolving {}", response.get_url()),
        ),
        ureq::Error::Transport(error) => ResolutionError::new(
            ResolutionErrorKind::Http,
            format!("HTTP transport error: {error}"),
        ),
    }
}

fn file_iri_path(value: &str) -> Result<Option<PathBuf>, ResolutionError> {
    let Some(rest) = value.strip_prefix("file://") else {
        return Ok(None);
    };
    if rest.is_empty() {
        return Err(ResolutionError::new(
            ResolutionErrorKind::InvalidData,
            "empty file URI",
        ));
    }
    if rest.starts_with('/') {
        return Ok(Some(PathBuf::from(percent_decode(rest)?)));
    }
    if let Some(path) = rest.strip_prefix("localhost/") {
        return Ok(Some(PathBuf::from(format!("/{}", percent_decode(path)?))));
    }
    Err(ResolutionError::new(
        ResolutionErrorKind::UnsupportedScheme,
        format!("unsupported non-local file URI `{value}`"),
    ))
}

fn iri_path_suffix(value: &str) -> Option<PathBuf> {
    let path = if let Some((_, rest)) = value.split_once("://") {
        rest.split_once('/').map(|(_, path)| path)?
    } else {
        value
    };
    let path = path.trim_start_matches('/');
    if path.is_empty() || path.contains("..") {
        return None;
    }
    Some(Path::new(path).to_path_buf())
}

fn percent_decode(value: &str) -> Result<String, ResolutionError> {
    let mut decoded = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let Some(hex) = value.get(index + 1..index + 3) else {
                return Err(ResolutionError::new(
                    ResolutionErrorKind::InvalidData,
                    format!("invalid percent escape in `{value}`"),
                ));
            };
            let byte = u8::from_str_radix(hex, 16).map_err(|error| {
                ResolutionError::new(
                    ResolutionErrorKind::InvalidData,
                    format!("invalid percent escape in `{value}`: {error}"),
                )
            })?;
            decoded.push(byte as char);
            index += 3;
        } else {
            decoded.push(bytes[index] as char);
            index += 1;
        }
    }
    Ok(decoded)
}

fn media_type_for(value: &str) -> Option<String> {
    let extension = value.rsplit('.').next()?.to_ascii_lowercase();
    let media_type = match extension.as_str() {
        "csv" => "text/csv",
        "json" => "application/json",
        "nt" => "application/n-triples",
        "ttl" | "turtle" => "text/turtle",
        "txt" => "text/plain",
        "xml" => "application/xml",
        _ => return None,
    };
    Some(media_type.to_owned())
}
