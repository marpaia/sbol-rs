use super::*;
use crate::FastaImporter;

const NS: &str = "https://example.org/lab";

#[test]
fn exports_one_record_per_sequence_with_wrapped_body() {
    let fasta = ">gene1 a test gene\nACGTACGTAC\n>gene2\nTTTT\n";
    let (document, _) = FastaImporter::new(NS).unwrap().read_str(fasta).unwrap();

    let out = FastaExporter::new().with_line_width(4).to_string(&document);

    // Two records, body wrapped at 4 columns.
    assert_eq!(out.matches('>').count(), 2);
    assert!(out.contains(">gene1 a test gene\n"));
    assert!(
        out.contains("acgt\nacgt\nac\n"),
        "wrapped body missing in:\n{out}"
    );
    assert!(out.contains(">gene2\ntttt\n"));
}

#[test]
fn round_trips_sequence_content() {
    let fasta = ">alpha\nACGTACGTAC\n>beta\nGGGGCCCCTTTT\n>prot\nMVSKGEEL\n";
    let importer = FastaImporter::new(NS).unwrap();
    let (document, first) = importer.read_str(fasta).unwrap();

    let exported = FastaExporter::new().to_string(&document);
    let (reimported, second) = importer.read_str(&exported).unwrap();

    assert_eq!(first.sequences, second.sequences);
    assert_eq!(first.components, second.components);

    // Every original sequence's elements survive the export→import trip.
    let mut original: Vec<_> = document
        .sequences()
        .filter_map(|s| s.elements.clone())
        .collect();
    let mut round_tripped: Vec<_> = reimported
        .sequences()
        .filter_map(|s| s.elements.clone())
        .collect();
    original.sort();
    round_tripped.sort();
    assert_eq!(original, round_tripped);
}

#[test]
fn write_path_reports_record_count() {
    let fasta = ">one\nAAAA\n>two\nCCCC\n";
    let (document, _) = FastaImporter::new(NS).unwrap().read_str(fasta).unwrap();

    let dir = std::env::temp_dir();
    let path = dir.join("sbol_fasta_export_test.fasta");
    let report = FastaExporter::new().write_path(&document, &path).unwrap();
    assert_eq!(report.records, 2);

    let written = std::fs::read_to_string(&path).unwrap();
    assert_eq!(written.matches('>').count(), 2);
    let _ = std::fs::remove_file(&path);
}
