//! Integration tests for the FASTA → SBOL 3 importer.

use std::path::PathBuf;

use sbol::{RdfFormat, SbolIdentified, SbolTopLevel, Severity};
use sbol_fasta::{Alphabet, FastaImporter, ImportError, ImportWarning};

fn workspace_fixture(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("tests/fixtures/fasta");
    path.push(name);
    path
}

fn assert_no_validation_errors(document: &sbol::Document, name: &str) {
    let report = document.validate();
    let errors: Vec<_> = report
        .issues()
        .iter()
        .filter(|i| matches!(i.severity, Severity::Error))
        .collect();
    assert!(
        errors.is_empty(),
        "{name}: {} validation error(s): {:?}",
        errors.len(),
        errors
    );
}

#[test]
fn nucleic_acid_record_imports_as_dna() {
    let (document, report) = FastaImporter::new("https://example.org/lab")
        .unwrap()
        .read_path(workspace_fixture("pUC19.fasta"))
        .expect("import pUC19");
    assert_no_validation_errors(&document, "pUC19");
    assert_eq!(report.components, 1);
    assert_eq!(report.sequences, 1);
    assert_eq!(report.dna_records, 1);
    assert_eq!(report.protein_records, 0);

    let component = document.components().next().unwrap();
    let name = component.name().expect("name");
    // NCBI FASTA defline: ">M77789.2 Cloning vector pUC19, complete sequence"
    assert!(
        name.starts_with("M77789"),
        "expected NCBI-style id as name, got {name:?}"
    );
    let namespace = component.namespace().expect("namespace");
    assert_eq!(namespace.as_str(), "https://example.org/lab");
}

#[test]
fn protein_record_imports_as_protein() {
    let (document, report) = FastaImporter::new("https://example.org/lab")
        .unwrap()
        .read_path(workspace_fixture("GFP_protein.fasta"))
        .expect("import protein");
    assert_no_validation_errors(&document, "GFP_protein");
    assert_eq!(report.protein_records, 1);
    assert_eq!(report.dna_records, 0);
}

#[test]
fn multi_record_file_produces_multiple_components() {
    let (document, report) = FastaImporter::new("https://example.org/lab")
        .unwrap()
        .read_path(workspace_fixture("multi_protein.fasta"))
        .expect("import multi");
    assert_no_validation_errors(&document, "multi_protein");
    assert!(
        report.components >= 2,
        "expected ≥2 components from the multi-record file, got {}",
        report.components
    );
    assert_eq!(report.components, report.sequences);
}

#[test]
fn rna_record_imports_via_u_in_alphabet() {
    let synthetic = ">test_rna\nACGUACGUACGU\n";
    let (document, report) = FastaImporter::new("https://example.org/lab")
        .unwrap()
        .read_str(synthetic)
        .expect("import");
    assert_no_validation_errors(&document, "rna");
    assert_eq!(report.rna_records, 1);
}

#[test]
fn forced_alphabet_overrides_detection() {
    // A sequence that looks like DNA but is actually a peptide.
    // Without forcing, the alphabet detector would call this DNA.
    let synthetic = ">ambiguous\nACGT\n";
    let (document, report) = FastaImporter::new("https://example.org/lab")
        .unwrap()
        .with_alphabet(Alphabet::Protein)
        .read_str(synthetic)
        .expect("import");
    assert_no_validation_errors(&document, "forced");
    assert_eq!(report.protein_records, 1);
    assert_eq!(report.dna_records, 0);
}

#[test]
fn empty_input_is_an_error() {
    match FastaImporter::new("https://example.org/lab")
        .unwrap()
        .read_str("")
    {
        Err(ImportError::Empty) => {}
        other => panic!("expected ImportError::Empty, got {other:?}"),
    }
}

#[test]
fn record_with_no_sequence_emits_warning_but_succeeds() {
    let synthetic = ">empty_record description\n>another\nACGT\n";
    let (document, report) = FastaImporter::new("https://example.org/lab")
        .unwrap()
        .read_str(synthetic)
        .expect("import");
    assert_eq!(report.components, 2);
    assert!(report.warnings.iter().any(|w| matches!(
        w,
        ImportWarning::EmptyRecord { record_id } if record_id == "empty_record"
    )));
    let _ = document;
}

#[test]
fn duplicate_ids_are_deduplicated() {
    // Two records with the same header id — the importer must
    // synthesize unique display IDs to keep SBOL 3 IRI compliance.
    let synthetic = ">dup\nACGT\n>dup\nGGGG\n";
    let (document, _report) = FastaImporter::new("https://example.org/lab")
        .unwrap()
        .read_str(synthetic)
        .expect("import");
    assert_eq!(document.components().count(), 2);
    let identities: Vec<_> = document
        .components()
        .map(|c| c.identity.as_iri().map(|i| i.as_str().to_owned()).unwrap())
        .collect();
    assert!(
        identities[0] != identities[1],
        "duplicate IDs should produce distinct identities: {identities:?}"
    );
}

#[test]
fn round_trip_through_turtle() {
    let (document, _report) = FastaImporter::new("https://example.org/lab")
        .unwrap()
        .read_path(workspace_fixture("pUC19.fasta"))
        .expect("import");
    let turtle = document.write(RdfFormat::Turtle).expect("write turtle");
    let parsed = sbol::Document::read_turtle(&turtle).expect("re-read turtle");
    assert_eq!(parsed.components().count(), document.components().count());
    assert_eq!(parsed.sequences().count(), document.sequences().count());
}

#[test]
fn handles_fa_extension() {
    let dir = tempfile::tempdir().unwrap();
    let bytes = std::fs::read(workspace_fixture("pUC19.fasta")).unwrap();
    let path = dir.path().join("genome.fa");
    std::fs::write(&path, &bytes).unwrap();

    let (document, _) = FastaImporter::new("https://example.org/lab")
        .unwrap()
        .read_path(&path)
        .expect("import .fa");
    assert_eq!(document.components().count(), 1);
}

#[test]
fn handles_fna_and_faa_extensions() {
    let dir = tempfile::tempdir().unwrap();
    let dna = std::fs::read(workspace_fixture("pUC19.fasta")).unwrap();
    let protein = std::fs::read(workspace_fixture("GFP_protein.fasta")).unwrap();
    let fna = dir.path().join("genome.fna");
    let faa = dir.path().join("proteome.faa");
    std::fs::write(&fna, &dna).unwrap();
    std::fs::write(&faa, &protein).unwrap();

    let importer = FastaImporter::new("https://example.org/lab").unwrap();
    let (doc_fna, report_fna) = importer.read_path(&fna).expect("import .fna");
    let (doc_faa, report_faa) = importer.read_path(&faa).expect("import .faa");
    assert_eq!(doc_fna.components().count(), 1);
    assert_eq!(report_fna.dna_records, 1);
    assert_eq!(doc_faa.components().count(), 1);
    assert_eq!(report_faa.protein_records, 1);
}
