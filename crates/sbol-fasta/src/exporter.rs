//! SBOL 3 → FASTA conversion engine.
//!
//! FASTA is a lossy target: it carries only a header and sequence body, so
//! feature annotations, roles, and structure do not survive the trip. What the
//! exporter guarantees is that every [`sbol3::Sequence`] with `elements`
//! becomes one FASTA record, headed by the display ID (and description, when
//! present) of the `Component` that references it.

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use sbol3::{Document, Resource, SbolIdentified, SbolObject};

/// Serializes SBOL 3 [`Document`]s to FASTA.
///
/// Records are emitted in document order: each `Component`'s referenced
/// sequences first (headed by the component's display ID and description),
/// then any sequences not referenced by a component (headed by their own
/// display ID). Sequences without `elements` are skipped.
#[derive(Clone, Debug)]
pub struct FastaExporter {
    line_width: usize,
}

impl Default for FastaExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl FastaExporter {
    /// Builds an exporter that wraps sequence bodies at 70 columns.
    pub fn new() -> Self {
        Self { line_width: 70 }
    }

    /// Sets the column at which sequence bodies wrap. A width of 0 is treated
    /// as 1 to avoid emitting empty lines.
    pub fn with_line_width(mut self, width: usize) -> Self {
        self.line_width = width.max(1);
        self
    }

    /// Renders the document to a FASTA string.
    pub fn to_string(&self, document: &Document) -> String {
        let mut out = String::new();
        for record in self.records(document) {
            out.push('>');
            out.push_str(&record.header);
            out.push('\n');
            wrap_into(&record.elements, self.line_width, &mut out);
        }
        out
    }

    /// Writes FASTA to an arbitrary writer, returning a tally of records.
    pub fn write<W: Write>(
        &self,
        document: &Document,
        writer: &mut W,
    ) -> Result<ExportReport, ExportError> {
        let text = self.to_string(document);
        let records = text.bytes().filter(|byte| *byte == b'>').count();
        writer
            .write_all(text.as_bytes())
            .map_err(|source| ExportError::Io {
                path: PathBuf::from("<writer>"),
                source,
            })?;
        Ok(ExportReport { records })
    }

    /// Writes FASTA to a file on disk.
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

    fn records(&self, document: &Document) -> Vec<FastaRecord> {
        let mut out = Vec::new();
        let mut referenced: HashSet<Resource> = HashSet::new();

        for component in document.components() {
            for sequence_ref in &component.sequences {
                let Some(SbolObject::Sequence(sequence)) = document.resolve(sequence_ref) else {
                    continue;
                };
                referenced.insert(sequence.identity.clone());
                let Some(elements) = sequence.elements.as_deref() else {
                    continue;
                };
                if elements.is_empty() {
                    continue;
                }
                out.push(FastaRecord {
                    header: header(
                        component.display_id(),
                        &component.identity,
                        component.description(),
                    ),
                    elements: elements.to_string(),
                });
            }
        }

        for sequence in document.sequences() {
            if referenced.contains(&sequence.identity) {
                continue;
            }
            let Some(elements) = sequence.elements.as_deref() else {
                continue;
            };
            if elements.is_empty() {
                continue;
            }
            out.push(FastaRecord {
                header: header(
                    sequence.display_id(),
                    &sequence.identity,
                    sequence.description(),
                ),
                elements: elements.to_string(),
            });
        }

        out
    }
}

struct FastaRecord {
    header: String,
    elements: String,
}

/// Builds a single-line FASTA header: an id token (display ID, else IRI)
/// optionally followed by the description with newlines flattened to spaces.
fn header(display_id: Option<&str>, identity: &Resource, description: Option<&str>) -> String {
    let id = display_id
        .map(str::to_string)
        .unwrap_or_else(|| identity.to_string());
    match description.map(str::trim).filter(|text| !text.is_empty()) {
        Some(text) => format!("{id} {}", text.replace(['\n', '\r'], " ")),
        None => id,
    }
}

/// Appends `elements` to `out`, wrapping every `width` characters.
fn wrap_into(elements: &str, width: usize, out: &mut String) {
    let chars: Vec<char> = elements.chars().collect();
    for chunk in chars.chunks(width) {
        out.extend(chunk.iter());
        out.push('\n');
    }
}

/// Tally of what a [`FastaExporter`] run wrote.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ExportReport {
    /// Number of FASTA records written.
    pub records: usize,
}

/// Fatal errors from [`FastaExporter`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ExportError {
    /// Filesystem or writer failure.
    Io {
        /// The path (or `<writer>`) that failed.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "failed to write {}: {source}", path.display()),
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests;
