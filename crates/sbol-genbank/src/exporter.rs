//! SBOL 3 → GenBank conversion engine.

use std::borrow::Cow;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use gb_io::seq::{Feature as GbFeature, Location, Seq, Topology};
use sbol3::constants::{ORIENTATION_REVERSE_COMPLEMENT, SBO_PROTEIN, SBO_RNA, SO_CIRCULAR};
use sbol3::{Component, Document, Range, SbolIdentified, SbolObject, SequenceFeature};

use crate::feature_map::{GENERIC_GENBANK_KEY, so_to_feature_key};

/// Predicate that preserves a GenBank feature key with no Sequence
/// Ontology mapping. When present on a SequenceFeature its literal value
/// is the exact original GenBank key, which the exporter reproduces
/// verbatim so an unmapped key survives a GenBank → SBOL 3 → GenBank
/// round-trip.
const GENBANK_FEATURE_KIND: &str = "http://sboltools.org/backport#genbank_feature_kind";

/// GenBank division code for synthetic sequences, the default for SBOL 3
/// records that carry no explicit division.
const DEFAULT_DIVISION: &str = "SYN";

/// Renders SBOL 3 [`Component`] + [`Sequence`] pairs back to GenBank
/// flat-file text, the inverse of [`GenbankImporter`].
///
/// Each [`Component`] that references a [`Sequence`] with nucleotide
/// elements becomes one GenBank record: the component's `displayId`
/// seeds the LOCUS name, its `type` terms set the molecule type and
/// topology, and each attached [`SequenceFeature`] becomes a GenBank
/// feature whose key is recovered from the feature's Sequence Ontology
/// role (or from a preserved original key). Protein components and
/// components without sequence elements have no GenBank nucleotide
/// representation and are skipped with a warning.
///
/// ```no_run
/// use sbol_genbank::{GenbankExporter, GenbankImporter};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let (document, _) =
///     GenbankImporter::new("https://example.org/lab")?.read_path("plasmid.gb")?;
/// let genbank = GenbankExporter::new().to_string(&document)?;
/// println!("{genbank}");
/// # Ok(())
/// # }
/// ```
///
/// [`GenbankImporter`]: crate::GenbankImporter
/// [`Component`]: sbol3::Component
/// [`Sequence`]: sbol3::Sequence
/// [`SequenceFeature`]: sbol3::SequenceFeature
#[derive(Clone, Debug)]
pub struct GenbankExporter {
    division: String,
}

impl Default for GenbankExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl GenbankExporter {
    /// Builds a new exporter emitting the `SYN` (synthetic) division.
    pub fn new() -> Self {
        Self {
            division: DEFAULT_DIVISION.to_owned(),
        }
    }

    /// Overrides the GenBank division code written to every LOCUS line
    /// (e.g. `BCT`, `PLN`). Defaults to `SYN`.
    pub fn division(mut self, division: impl Into<String>) -> Self {
        self.division = division.into();
        self
    }

    /// Renders every eligible [`Component`] in `document` to GenBank
    /// flat-file text.
    pub fn to_string(&self, document: &Document) -> Result<String, ExportError> {
        let (bytes, _report) = self.render(document)?;
        String::from_utf8(bytes).map_err(|err| ExportError::NonUtf8(err.to_string()))
    }

    /// Writes GenBank flat-file text for `document` to `writer`, returning
    /// an [`ExportReport`] tallying what was produced.
    pub fn write<W: Write>(
        &self,
        document: &Document,
        mut writer: W,
    ) -> Result<ExportReport, ExportError> {
        let (bytes, report) = self.render(document)?;
        writer.write_all(&bytes).map_err(|source| ExportError::Io {
            path: PathBuf::from("<writer>"),
            source,
        })?;
        Ok(report)
    }

    /// Writes GenBank flat-file text for `document` to a file on disk.
    pub fn write_path(
        &self,
        document: &Document,
        path: impl AsRef<Path>,
    ) -> Result<ExportReport, ExportError> {
        let path = path.as_ref();
        let file = File::create(path).map_err(|source| ExportError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let mut writer = BufWriter::new(file);
        let report = self.write(document, &mut writer)?;
        writer.flush().map_err(|source| ExportError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(report)
    }

    fn render(&self, document: &Document) -> Result<(Vec<u8>, ExportReport), ExportError> {
        let mut report = ExportReport::default();
        let mut out: Vec<u8> = Vec::new();
        for component in document.components() {
            if let Some(seq) = self.build_seq(document, component, &mut report) {
                seq.write(&mut out).map_err(|source| ExportError::Io {
                    path: PathBuf::from("<memory>"),
                    source,
                })?;
                report.records += 1;
            }
        }
        Ok((out, report))
    }

    fn build_seq(
        &self,
        document: &Document,
        component: &Component,
        report: &mut ExportReport,
    ) -> Option<Seq> {
        let display_id = component.display_id().unwrap_or("record").to_owned();

        if component
            .types
            .iter()
            .any(|iri| iri.as_str() == SBO_PROTEIN.as_str())
        {
            report.warnings.push(ExportWarning::SkippedComponent {
                component: display_id,
                reason: "protein components have no GenBank nucleotide representation".to_owned(),
            });
            return None;
        }

        let elements =
            component
                .sequences
                .iter()
                .find_map(|resource| match document.resolve(resource) {
                    Some(SbolObject::Sequence(sequence)) => sequence
                        .elements
                        .as_deref()
                        .filter(|elements| !elements.is_empty()),
                    _ => None,
                });
        let elements = match elements {
            Some(elements) => elements,
            None => {
                report.warnings.push(ExportWarning::SkippedComponent {
                    component: display_id,
                    reason: "component references no sequence with elements".to_owned(),
                });
                return None;
            }
        };

        let mut seq = Seq::empty();
        seq.name = Some(display_id);
        seq.seq = elements.as_bytes().to_ascii_uppercase();
        seq.division = self.division.clone();
        seq.topology = if component
            .types
            .iter()
            .any(|iri| iri.as_str() == SO_CIRCULAR.as_str())
        {
            Topology::Circular
        } else {
            Topology::Linear
        };
        seq.molecule_type = Some(
            if component
                .types
                .iter()
                .any(|iri| iri.as_str() == SBO_RNA.as_str())
            {
                "RNA".to_owned()
            } else {
                "DNA".to_owned()
            },
        );
        if let Some(definition) = component.description().or_else(|| component.name()) {
            seq.definition = Some(definition.to_owned());
        }

        for feature_resource in &component.features {
            if let Some(SbolObject::SequenceFeature(feature)) = document.resolve(feature_resource)
                && let Some(gb_feature) = build_feature(document, feature, report)
            {
                seq.features.push(gb_feature);
                report.features += 1;
            }
        }

        Some(seq)
    }
}

/// Recovers the GenBank feature key for a SequenceFeature. A preserved
/// original key takes precedence; otherwise the first role that maps
/// back to a curated key wins, falling back to `misc_feature`.
fn feature_kind(feature: &SequenceFeature) -> Cow<'static, str> {
    for extension in feature.extensions() {
        if extension.predicate.as_str() == GENBANK_FEATURE_KIND
            && let Some(literal) = extension.object.as_literal()
        {
            return Cow::Owned(literal.value().to_owned());
        }
    }
    for role in &feature.feature.roles {
        if let Some(key) = so_to_feature_key(role.as_str()) {
            return Cow::Borrowed(key);
        }
    }
    Cow::Borrowed(GENERIC_GENBANK_KEY)
}

fn build_feature(
    document: &Document,
    feature: &SequenceFeature,
    report: &mut ExportReport,
) -> Option<GbFeature> {
    let display_id = feature.display_id().unwrap_or("feature").to_owned();
    let location = build_location(document, feature, &display_id, report)?;

    let mut qualifiers: Vec<(Cow<'static, str>, Option<String>)> = Vec::new();
    if let Some(name) = feature.name() {
        qualifiers.push((Cow::Borrowed("label"), Some(name.to_owned())));
    }
    if let Some(description) = feature.description() {
        qualifiers.push((Cow::Borrowed("note"), Some(description.to_owned())));
    }

    Some(GbFeature {
        kind: feature_kind(feature),
        location,
        qualifiers,
    })
}

/// Assembles the GenBank location for a SequenceFeature from its Range
/// children. A single Range becomes a plain range; several ranges are
/// joined end-to-end.
fn build_location(
    document: &Document,
    feature: &SequenceFeature,
    display_id: &str,
    report: &mut ExportReport,
) -> Option<Location> {
    let mut parts: Vec<Location> = Vec::new();
    for location_resource in &feature.locations {
        if let Some(SbolObject::Range(range)) = document.resolve(location_resource) {
            match range_to_location(range) {
                Some(location) => parts.push(location),
                None => report.warnings.push(ExportWarning::LossyFeature {
                    feature: display_id.to_owned(),
                    reason: "range lacks a usable start/end".to_owned(),
                }),
            }
        }
    }

    match parts.len() {
        0 => {
            report.warnings.push(ExportWarning::LossyFeature {
                feature: display_id.to_owned(),
                reason: "no Range locations could be lowered to GenBank".to_owned(),
            });
            None
        }
        1 => parts.pop(),
        _ => Some(Location::Join(parts)),
    }
}

/// Converts an SBOL 3 [`Range`] to a `gb_io` [`Location`]. SBOL uses
/// 1-based inclusive coordinates; `gb_io` uses 0-based half-open ranges,
/// so the start shifts down by one and the end is unchanged. A
/// reverse-complement orientation wraps the range in `Complement`.
fn range_to_location(range: &Range) -> Option<Location> {
    let start = range.start?;
    let end = range.end?;
    let gb_start = start - 1;
    let gb_end = end;
    if gb_start < 0 || gb_end < gb_start {
        return None;
    }
    let base = Location::simple_range(gb_start, gb_end);
    let reverse = range
        .location
        .orientation
        .as_ref()
        .is_some_and(|orientation| orientation.as_str() == ORIENTATION_REVERSE_COMPLEMENT.as_str());
    Some(if reverse {
        Location::Complement(Box::new(base))
    } else {
        base
    })
}

/// Tally of what a [`GenbankExporter`] run produced. Useful for
/// summarizing CLI output and writing structural tests.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ExportReport {
    pub records: usize,
    pub features: usize,
    pub warnings: Vec<ExportWarning>,
}

impl ExportReport {
    pub fn is_clean(&self) -> bool {
        self.warnings.is_empty()
    }
}

/// Non-fatal issues encountered while exporting GenBank.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExportWarning {
    /// A [`Component`] had no GenBank nucleotide representation (a protein
    /// component, or one that references no sequence with elements) and
    /// was omitted from the output.
    SkippedComponent { component: String, reason: String },
    /// A SequenceFeature's locations couldn't be lowered to a GenBank
    /// location; the feature was skipped.
    LossyFeature { feature: String, reason: String },
}

/// Fatal errors from [`GenbankExporter`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ExportError {
    /// Filesystem write failure (for [`GenbankExporter::write_path`] and
    /// [`GenbankExporter::write`]).
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The rendered GenBank bytes were not valid UTF-8 (only reachable
    /// through [`GenbankExporter::to_string`]).
    NonUtf8(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "failed to write {}: {source}", path.display())
            }
            Self::NonUtf8(msg) => write!(f, "rendered GenBank was not UTF-8: {msg}"),
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::NonUtf8(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;
