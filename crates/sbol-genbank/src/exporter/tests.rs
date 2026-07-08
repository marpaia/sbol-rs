use super::*;
use crate::GenbankImporter;
use sbol3::Document;

const NAMESPACE: &str = "https://example.org/lab";

const FIXTURE: &str = "\
LOCUS       ROUNDTRIP               30 bp    DNA     linear       01-JAN-2026
DEFINITION  round-trip export fixture.
ACCESSION   ROUNDTRIP
FEATURES             Location/Qualifiers
     source          1..30
                     /organism=\"synthetic construct\"
     promoter        1..10
                     /label=\"prom\"
     CDS             11..20
                     /label=\"cds\"
     gene            complement(21..30)
                     /label=\"rev\"
ORIGIN
        1 atgcatgcat gcatgcatgc atgcatgcat
//
";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct FeatureDescriptor {
    roles: Vec<String>,
    ranges: Vec<(i64, i64, Option<String>)>,
}

/// Collapses a document into an order-independent view of its sequence
/// elements and feature roles/ranges, the content a GenBank round-trip
/// must preserve.
fn describe(document: &Document) -> (Vec<String>, Vec<FeatureDescriptor>) {
    let mut sequences: Vec<String> = document
        .sequences()
        .filter_map(|sequence| sequence.elements.clone())
        .collect();
    sequences.sort();

    let mut features: Vec<FeatureDescriptor> = Vec::new();
    for feature in document.sequence_features() {
        let mut roles: Vec<String> = feature
            .feature
            .roles
            .iter()
            .map(|role| role.as_str().to_owned())
            .collect();
        roles.sort();

        let mut ranges: Vec<(i64, i64, Option<String>)> = Vec::new();
        for location in &feature.locations {
            if let Some(SbolObject::Range(range)) = document.resolve(location)
                && let (Some(start), Some(end)) = (range.start, range.end)
            {
                let orientation = range
                    .location
                    .orientation
                    .as_ref()
                    .map(|iri| iri.as_str().to_owned());
                ranges.push((start, end, orientation));
            }
        }
        ranges.sort();

        features.push(FeatureDescriptor { roles, ranges });
    }
    features.sort();

    (sequences, features)
}

#[test]
fn round_trip_preserves_sequence_and_features() {
    let importer = GenbankImporter::new(NAMESPACE).unwrap();
    let (original, import_report) = importer.read_str(FIXTURE).unwrap();

    // The `source` feature is molecule metadata, not an annotation, so
    // the importer drops it: three annotated features survive.
    assert_eq!(import_report.features, 3);

    let genbank = GenbankExporter::new().to_string(&original).unwrap();
    let (round_tripped, _) = importer.read_str(&genbank).unwrap();

    assert_eq!(describe(&original), describe(&round_tripped));
}

#[test]
fn export_emits_locus_features_and_origin() {
    let importer = GenbankImporter::new(NAMESPACE).unwrap();
    let (document, _) = importer.read_str(FIXTURE).unwrap();

    let exporter = GenbankExporter::new();
    let genbank = exporter.to_string(&document).unwrap();

    assert!(genbank.contains("LOCUS"), "missing LOCUS line:\n{genbank}");
    assert!(
        genbank.contains("SYN"),
        "missing default division:\n{genbank}"
    );
    assert!(
        genbank.contains("FEATURES             Location/Qualifiers"),
        "missing FEATURES header:\n{genbank}"
    );
    assert!(
        genbank.contains("ORIGIN"),
        "missing ORIGIN block:\n{genbank}"
    );

    // Roles are reverse-mapped to their canonical GenBank keys.
    assert!(genbank.contains("CDS"), "missing CDS feature:\n{genbank}");
    assert!(
        genbank.contains("promoter"),
        "missing promoter feature:\n{genbank}"
    );
    // Forward and reverse-complement locations both survive.
    assert!(
        genbank.contains("11..20"),
        "missing forward range:\n{genbank}"
    );
    assert!(
        genbank.contains("complement(21..30)"),
        "missing reverse-complement range:\n{genbank}"
    );
    // The residues are emitted uppercased in the ORIGIN block.
    assert!(
        genbank.contains("atgcatgcat") || genbank.contains("ATGCATGCAT"),
        "missing sequence residues:\n{genbank}"
    );

    let report = exporter.write(&document, Vec::new()).unwrap();
    assert_eq!(report.records, 1);
    assert_eq!(report.features, 3);
    assert!(
        report.is_clean(),
        "unexpected warnings: {:?}",
        report.warnings
    );
}
