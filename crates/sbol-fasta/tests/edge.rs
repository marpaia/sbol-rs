//! Edge-input coverage for the FASTA importer: ambiguity codes, the
//! RNA/DNA/protein detection boundary, empty records, and non-UTF-8
//! bytes. Each test pins documented behavior rather than a panic.

use sbol3::Severity;
use sbol_fasta::{FastaImporter, ImportError, ImportWarning};

fn importer() -> FastaImporter {
    FastaImporter::new("https://example.org/lab").expect("namespace")
}

fn assert_no_validation_errors(document: &sbol3::Document) {
    let errors: Vec<_> = document
        .validate()
        .issues()
        .iter()
        .filter(|i| matches!(i.severity, Severity::Error))
        .cloned()
        .collect();
    assert!(errors.is_empty(), "validation errors: {errors:?}");
}

#[test]
fn nucleotide_ambiguity_codes_import_as_dna() {
    // IUPAC nucleotide ambiguity codes (N, R, Y, K, M, S, W, B, D, H, V)
    // are valid nucleotide FASTA; none of them are protein-only letters,
    // so the record detects as DNA and imports cleanly.
    let (document, report) = importer()
        .read_str(">amb ambiguity codes\nACGTNRYKMSWBDHVacgtn\n")
        .expect("import");
    assert_no_validation_errors(&document);
    assert_eq!(report.dna_records, 1);
    assert_eq!(report.protein_records, 0);
    assert_eq!(report.rna_records, 0);
    // DNA elements are lowercased per the SBOL 3 convention.
    let seq = document.sequences().next().unwrap();
    assert_eq!(
        seq.elements.as_deref().expect("elements"),
        "acgtnrykmswbdhvacgtn"
    );
}

#[test]
fn uracil_switches_detection_from_dna_to_rna() {
    // The only difference between these two records is T vs U; that flips
    // the detected alphabet.
    let (_dna_doc, dna) = importer().read_str(">d\nACGTACGT\n").expect("dna");
    assert_eq!((dna.dna_records, dna.rna_records), (1, 0));

    let (_rna_doc, rna) = importer().read_str(">r\nACGUACGU\n").expect("rna");
    assert_eq!((rna.dna_records, rna.rna_records), (0, 1));
}

#[test]
fn empty_record_warns_but_still_emits_objects() {
    // A header with no sequence body: the Component and Sequence are
    // still emitted (the Sequence simply carries no elements) and an
    // EmptyRecord warning is recorded.
    let (document, report) = importer()
        .read_str(">lonely_header a record with no bases\n")
        .expect("import");
    assert_eq!(report.components, 1);
    assert_eq!(report.sequences, 1);
    assert!(
        report.warnings.iter().any(|w| matches!(
            w,
            ImportWarning::EmptyRecord { record_id } if record_id == "lonely_header"
        )),
        "expected an EmptyRecord warning, got {:?}",
        report.warnings
    );
    let seq = document.sequences().next().unwrap();
    assert!(
        seq.elements.is_none(),
        "an empty record's Sequence should carry no elements"
    );
}

#[test]
fn non_utf8_input_is_an_io_error() {
    // The reader path decodes the whole stream as UTF-8 up front; invalid
    // bytes surface as an Io error, not a panic.
    let mut bytes = b">rec\nACGT".to_vec();
    bytes.push(0xFF);
    bytes.push(0xFE);
    match importer().read(&bytes[..]) {
        Err(ImportError::Io { .. }) => {}
        other => panic!("expected an Io error for non-UTF-8 bytes, got {other:?}"),
    }
}
