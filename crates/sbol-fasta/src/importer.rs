//! FASTA → SBOL 3 conversion engine.

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use sbol3::SbolObject;
use sbol3::constants::{EDAM_IUPAC_DNA, EDAM_IUPAC_PROTEIN, SBO_DNA, SBO_PROTEIN, SBO_RNA};
use sbol3::{BuildError, Component, Document, Iri, Sequence as SbolSequence};

use crate::alphabet::Alphabet;
use crate::parser::{Record, parse_records};

/// Imports FASTA records and emits SBOL 3 [`Document`]s.
///
/// `FastaImporter::new` takes the namespace IRI that the resulting
/// SBOL 3 top-level objects will be rooted under, typically the
/// owning lab or repository (e.g. `https://example.org/lab`).
/// Component identities are derived as `{namespace}/{record-id}`.
///
/// By default, the alphabet of each record is detected automatically
/// from the sequence text. Call [`FastaImporter::with_alphabet`] to
/// override that detection when the data is ambiguous.
#[derive(Clone, Debug)]
pub struct FastaImporter {
    namespace: Iri,
    forced_alphabet: Option<Alphabet>,
}

impl FastaImporter {
    /// Builds a new importer scoped to the supplied namespace IRI.
    pub fn new(namespace: impl AsRef<str>) -> Result<Self, ImportError> {
        let namespace = Iri::new(namespace.as_ref().to_owned())
            .map_err(|err| ImportError::Namespace(err.to_string()))?;
        Ok(Self {
            namespace,
            forced_alphabet: None,
        })
    }

    /// Skips alphabet detection and forces every record to be
    /// imported as the supplied [`Alphabet`].
    pub fn with_alphabet(mut self, alphabet: Alphabet) -> Self {
        self.forced_alphabet = Some(alphabet);
        self
    }

    /// Reads every record from the supplied reader and returns one
    /// SBOL 3 [`Document`] containing the emitted Components +
    /// Sequences plus an [`ImportReport`] tallying what was produced.
    pub fn read<R: Read>(&self, mut reader: R) -> Result<(Document, ImportReport), ImportError> {
        let mut buffer = String::new();
        reader
            .read_to_string(&mut buffer)
            .map_err(|err| ImportError::Io {
                path: PathBuf::from("<reader>"),
                source: err,
            })?;
        self.read_str(&buffer)
    }

    /// Reads from a string slice.
    pub fn read_str(&self, input: &str) -> Result<(Document, ImportReport), ImportError> {
        let records = parse_records(input);
        if records.is_empty() {
            return Err(ImportError::Empty);
        }

        let mut objects: Vec<SbolObject> = Vec::new();
        let mut report = ImportReport::default();
        let mut used_display_ids: HashSet<String> = HashSet::new();

        for (index, record) in records.iter().enumerate() {
            self.append_record(
                record,
                index,
                &mut used_display_ids,
                &mut objects,
                &mut report,
            )?;
        }

        let document = Document::from_objects(objects).map_err(ImportError::Build)?;
        Ok((document, report))
    }

    /// Reads from a file on disk (`.fasta` / `.fa` / `.fna` / `.faa`;
    /// the importer doesn't actually care about the extension).
    pub fn read_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(Document, ImportReport), ImportError> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|err| ImportError::Io {
            path: path.to_path_buf(),
            source: err,
        })?;
        self.read(BufReader::new(file))
    }

    fn append_record(
        &self,
        record: &Record,
        index: usize,
        used_display_ids: &mut HashSet<String>,
        objects: &mut Vec<SbolObject>,
        report: &mut ImportReport,
    ) -> Result<(), ImportError> {
        let raw_id = if record.id.is_empty() {
            format!("record_{index}")
        } else {
            record.id.clone()
        };
        let base_display_id = sanitize_display_id(&raw_id);
        let display_id = dedupe(base_display_id, used_display_ids);

        if record.sequence.is_empty() {
            report.warnings.push(ImportWarning::EmptyRecord {
                record_id: raw_id.clone(),
            });
        }

        let alphabet = self
            .forced_alphabet
            .unwrap_or_else(|| Alphabet::detect(&record.sequence));
        let (component_type, encoding, elements_case) = match alphabet {
            Alphabet::Dna => (SBO_DNA, EDAM_IUPAC_DNA, ElementsCase::Lower),
            Alphabet::Rna => (SBO_RNA, EDAM_IUPAC_DNA, ElementsCase::Lower),
            Alphabet::Protein => (SBO_PROTEIN, EDAM_IUPAC_PROTEIN, ElementsCase::Upper),
        };

        // Sequence
        let sequence_display_id = format!("{display_id}_sequence");
        let mut sequence_builder =
            SbolSequence::builder(self.namespace.as_str(), sequence_display_id.as_str())
                .map_err(ImportError::Build)?;
        if !record.sequence.is_empty() {
            let elements = match elements_case {
                ElementsCase::Lower => record.sequence.to_ascii_lowercase(),
                ElementsCase::Upper => record.sequence.to_ascii_uppercase(),
            };
            sequence_builder = sequence_builder.elements(elements);
        }
        sequence_builder = sequence_builder.encoding(encoding);
        let sequence = sequence_builder.build().map_err(ImportError::Build)?;
        let sequence_resource = sequence.identity.clone();
        objects.push(SbolObject::Sequence(sequence));
        report.sequences += 1;

        // Component
        let mut component_builder =
            Component::builder(self.namespace.as_str(), display_id.as_str())
                .map_err(ImportError::Build)?;
        component_builder = component_builder.types([component_type]);
        if let Some(description) = record.description.as_deref().map(str::trim)
            && !description.is_empty()
        {
            // Headers commonly read like `>id description text…`. We
            // use the original record id (not the sanitized display
            // id) as a human-friendly name, with the description
            // following.
            component_builder = component_builder.name(&raw_id).description(description);
        } else {
            component_builder = component_builder.name(&raw_id);
        }
        component_builder = component_builder.add_sequence(sequence_resource);
        let component = component_builder.build().map_err(ImportError::Build)?;
        objects.push(SbolObject::Component(component));
        report.components += 1;

        match alphabet {
            Alphabet::Dna => report.dna_records += 1,
            Alphabet::Rna => report.rna_records += 1,
            Alphabet::Protein => report.protein_records += 1,
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
enum ElementsCase {
    Lower,
    Upper,
}

fn dedupe(base: String, used: &mut HashSet<String>) -> String {
    if used.insert(base.clone()) {
        return base;
    }
    for suffix in 2.. {
        let candidate = format!("{base}_{suffix}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!("display ID space exhausted");
}

fn sanitize_display_id(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        return "record".to_owned();
    }
    if out
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        out.insert(0, '_');
    }
    out
}

/// Tally of what a [`FastaImporter`] run produced.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ImportReport {
    pub components: usize,
    pub sequences: usize,
    pub dna_records: usize,
    pub rna_records: usize,
    pub protein_records: usize,
    pub warnings: Vec<ImportWarning>,
}

impl ImportReport {
    pub fn is_clean(&self) -> bool {
        self.warnings.is_empty()
    }
}

/// Non-fatal issues encountered while importing FASTA.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImportWarning {
    /// A record had a header but no sequence body. The Component +
    /// Sequence are still emitted; the Sequence simply has no
    /// `elements`.
    EmptyRecord { record_id: String },
}

/// Fatal errors from [`FastaImporter`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ImportError {
    /// The namespace IRI was invalid.
    Namespace(String),
    /// The input contained no `>` records.
    Empty,
    /// Filesystem read failure (for [`FastaImporter::read_path`]).
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// SBOL 3 object construction failed (typically an invalid
    /// displayId or namespace).
    Build(BuildError),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Namespace(msg) => write!(f, "invalid namespace: {msg}"),
            Self::Empty => write!(
                f,
                "input contained no `>` records — was the file truncated?"
            ),
            Self::Io { path, source } => {
                write!(f, "failed to read {}: {source}", path.display())
            }
            Self::Build(err) => write!(f, "SBOL object construction failed: {err}"),
        }
    }
}

impl std::error::Error for ImportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Build(err) => Some(err),
            _ => None,
        }
    }
}
