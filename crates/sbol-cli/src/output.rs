use std::fs;
use std::path::Path;
use std::process::ExitCode;

use sbol::v3::{Document, RdfFormat, ReadError, Severity, ValidationIssue};

use crate::style::{Styles, paint, severity_code};

pub(crate) fn format_issue(issue: &ValidationIssue, path: &Path, color: bool) -> String {
    let severity_label = match issue.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        _ => "issue",
    };
    let severity_painted = match severity_code(issue.severity) {
        Some(code) => paint(color, code, severity_label),
        None => severity_label.to_string(),
    };
    let property = issue
        .property
        .map(|property| format!(" <{property}>"))
        .unwrap_or_default();
    format!(
        "{}: {severity_painted}[{}] [{}]{property}: {}",
        path.display(),
        issue.rule,
        issue.subject,
        issue.message,
    )
}

pub(crate) fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

pub(crate) fn read_document(path: &Path, styles: Styles) -> Result<Document, ExitCode> {
    match Document::read_path(path) {
        Ok(document) => Ok(document),
        Err(ReadError::Io { source, .. }) => {
            eprintln!(
                "{}: failed to read {}: {source}",
                styles.err_label(),
                path.display()
            );
            Err(ExitCode::from(2))
        }
        Err(ReadError::UnknownFormat { extension, .. }) => {
            let ext = extension.as_deref().unwrap_or("<none>");
            eprintln!(
                "{}: unsupported extension `{ext}` for {} — supported: .ttl, .rdf, .jsonld, .nt",
                styles.err_label(),
                path.display()
            );
            Err(ExitCode::from(2))
        }
        Err(error) => {
            eprintln!(
                "{}: failed to parse {}: {error}",
                styles.err_label(),
                path.display()
            );
            Err(ExitCode::from(2))
        }
    }
}

pub(crate) fn read_document_with_format(
    path: &Path,
    format: RdfFormat,
    styles: Styles,
) -> Result<Document, ExitCode> {
    let input = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!(
                "{}: failed to read {}: {err}",
                styles.err_label(),
                path.display()
            );
            return Err(ExitCode::from(2));
        }
    };
    match Document::read(&input, format) {
        Ok(document) => Ok(document),
        Err(error) => {
            eprintln!(
                "{}: failed to parse {} as {format}: {error}",
                styles.err_label(),
                path.display()
            );
            Err(ExitCode::from(2))
        }
    }
}

/// Maps a path to an RDF format, treating `.xml` as RDF/XML for SBOL
/// conversion commands. The library's strict `from_path` rejects `.xml` as
/// ambiguous, but SBOL files from SynBioHub, iGEM, and the SBOLTestSuite
/// commonly use that extension for RDF/XML.
pub(crate) fn infer_conversion_rdf_format(path: &Path) -> Option<RdfFormat> {
    if let Some(format) = RdfFormat::from_path(path) {
        return Some(format);
    }
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    if extension == "xml" {
        return Some(RdfFormat::RdfXml);
    }
    None
}
