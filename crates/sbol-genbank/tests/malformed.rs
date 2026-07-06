//! Negative and edge-input coverage for the GenBank importer. Each test
//! pins the importer's documented behavior — a specific warning or a
//! typed error — rather than a panic.

use sbol_genbank::{GenbankImporter, ImportError, ImportWarning};

fn importer() -> GenbankImporter {
    GenbankImporter::new("https://example.org/lab").expect("namespace")
}

#[test]
fn truncated_record_is_a_parse_error() {
    // A record cut off mid-FEATURES with no ORIGIN and no `//` terminator.
    let truncated = "\
LOCUS       TRUNC                   60 bp    DNA     linear       01-JAN-2026
DEFINITION  cut off before the sequence.
FEATURES             Location/Qualifiers
     CDS             1..6
";
    match importer().read_str(truncated) {
        Err(ImportError::Parse(_)) => {}
        other => panic!("expected a Parse error for a truncated record, got {other:?}"),
    }
}

#[test]
fn record_without_accession_or_locus_name_synthesizes_an_identifier() {
    // The LOCUS line carries no name and there is no ACCESSION line. The
    // importer still emits a Component, using a synthesized identifier
    // and recording a SynthesizedIdentifier warning.
    let no_id = "\
LOCUS                               12 bp    DNA     linear       01-JAN-2026
FEATURES             Location/Qualifiers
     promoter        1..12
ORIGIN
        1 atgcatgcatgc
//
";
    let (document, report) = importer().read_str(no_id).expect("import");
    assert_eq!(document.components().count(), 1);
    assert!(
        report
            .warnings
            .iter()
            .any(|w| matches!(w, ImportWarning::SynthesizedIdentifier)),
        "expected a SynthesizedIdentifier warning, got {:?}",
        report.warnings
    );
    let identity = document
        .components()
        .next()
        .unwrap()
        .identity
        .as_iri()
        .map(|i| i.as_str().to_owned())
        .unwrap();
    assert_eq!(identity, "https://example.org/lab/imported_record");
}

#[test]
fn non_utf8_input_is_an_io_error() {
    // The reader path decodes the whole stream as UTF-8 up front; invalid
    // bytes surface as an Io error rather than a panic or silent
    // corruption.
    let mut bytes =
        b"LOCUS       X   12 bp DNA linear 01-JAN-2026\nORIGIN\n        1 atgc".to_vec();
    bytes.push(0xFF);
    bytes.push(0xFE);
    match importer().read(&bytes[..]) {
        Err(ImportError::Io { .. }) => {}
        other => panic!("expected an Io error for non-UTF-8 bytes, got {other:?}"),
    }
}

#[test]
fn empty_input_produces_an_empty_document() {
    // No records at all: not an error, just an empty document.
    let (document, report) = importer().read_str("").expect("import empty");
    assert_eq!(document.components().count(), 0);
    assert_eq!(report.components, 0);
    assert_eq!(report.sequences, 0);
}
