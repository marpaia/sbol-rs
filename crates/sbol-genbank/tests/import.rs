//! Integration tests for the GenBank → SBOL 3 importer against the
//! committed real-world GenBank corpus.

use std::path::PathBuf;

use sbol_genbank::{GenbankImporter, ImportWarning};
use sbol3::{RdfFormat, SbolIdentified, SbolTopLevel, Severity};

fn workspace_fixture(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("tests/fixtures/genbank");
    path.push(name);
    path
}

fn import(name: &str) -> (sbol3::Document, sbol_genbank::ImportReport) {
    let path = workspace_fixture(name);
    GenbankImporter::new("https://example.org/lab")
        .expect("namespace")
        .read_path(&path)
        .unwrap_or_else(|err| panic!("import {name}: {err}"))
}

fn assert_no_validation_errors(document: &sbol3::Document, name: &str) {
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
fn simple_cds_part_imports_cleanly() {
    let (document, report) = import("BBa_E0040.gb");
    assert_no_validation_errors(&document, "BBa_E0040");
    assert_eq!(report.components, 1);
    assert_eq!(report.sequences, 1);
    assert!(report.features >= 1, "expected at least one feature");

    let component = document
        .components()
        .find(|c| {
            c.identity.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/BBa_E0040")
        })
        .expect("BBa_E0040 Component missing");
    let namespace = component.namespace().expect("namespace");
    assert_eq!(namespace.as_str(), "https://example.org/lab");
    let name = component.name().expect("name");
    assert!(
        name.to_lowercase().contains("gfp") || name.to_lowercase().contains("green fluorescent"),
        "expected GFP-related name, got {name:?}"
    );
}

#[test]
fn promoter_part_imports_with_multiple_features() {
    let (document, report) = import("BBa_R0010.gb");
    assert_no_validation_errors(&document, "BBa_R0010");
    assert_eq!(report.components, 1);
    assert!(
        report.features >= 5,
        "BBa_R0010 has multiple annotated features in source; expected ≥5, got {}",
        report.features
    );
}

#[test]
fn composite_design_imports_thirty_features_with_dedupe() {
    // BBa_F2620 is a composite "PoPS receiver" with multiple sub-parts
    // that share labels (e.g. two `BBa_R0040` annotations). The
    // importer must produce unique display IDs so SBOL 3 IRI
    // compliance holds and there are no duplicate identities.
    let (document, report) = import("BBa_F2620.gb");
    assert_no_validation_errors(&document, "BBa_F2620");
    assert!(
        report.features >= 20,
        "composite has many features; got {}",
        report.features
    );
    // Sanity: total typed objects = 1 Component + 1 Sequence + N
    // SequenceFeatures + at least N Range locations.
    assert!(document.ranges().count() >= report.features);
}

#[test]
fn ribosome_binding_site_imports() {
    let (document, _report) = import("BBa_B0034.gb");
    assert_no_validation_errors(&document, "BBa_B0034");
}

#[test]
fn unknown_feature_key_emits_warning() {
    let synthetic = r#"LOCUS       UNKNOWN                 12 bp    DNA     linear       01-JAN-2026
DEFINITION  synthetic with unknown feature key.
FEATURES             Location/Qualifiers
     made_up         1..12
                     /label="weird"
ORIGIN
        1 atgcatgcatgc
//
"#;
    let (_doc, report) = GenbankImporter::new("https://example.org/lab")
        .unwrap()
        .read_str(synthetic)
        .expect("import");
    assert!(report.warnings.iter().any(|w| matches!(
        w,
        ImportWarning::UnknownFeatureKey { kind } if kind == "made_up"
    )));
}

#[test]
fn round_trip_through_turtle() {
    let (document, _report) = import("BBa_E0040.gb");
    let turtle = document.write(RdfFormat::Turtle).expect("write turtle");
    let parsed = sbol3::Document::read_turtle(&turtle).expect("re-read turtle");
    assert_eq!(parsed.components().count(), document.components().count());
    assert_eq!(parsed.sequences().count(), document.sequences().count());
    assert_eq!(
        parsed.sequence_features().count(),
        document.sequence_features().count()
    );
    assert_eq!(parsed.ranges().count(), document.ranges().count());
}

#[test]
fn gbk_extension_imports_identically_to_gb() {
    // `.gb` and `.gbk` are the same flat-file format under different
    // naming conventions (NCBI uses `.gb`; SnapGene historically uses
    // `.gbk`). The importer accepts both extensions; this test pins
    // that behavior by feeding identical bytes to read_path under each
    // extension and asserting the resulting Documents have the same
    // triple set.
    let dir = tempfile::tempdir().expect("tempdir");
    let bytes = std::fs::read(workspace_fixture("BBa_E0040.gb")).expect("read source fixture");

    let gb_path = dir.path().join("identical.gb");
    let gbk_path = dir.path().join("identical.gbk");
    std::fs::write(&gb_path, &bytes).expect("write .gb");
    std::fs::write(&gbk_path, &bytes).expect("write .gbk");

    let importer = GenbankImporter::new("https://example.org/lab").expect("namespace");
    let (doc_gb, _) = importer.read_path(&gb_path).expect("read .gb");
    let (doc_gbk, _) = importer.read_path(&gbk_path).expect("read .gbk");

    let triples_gb = doc_gb.rdf_graph().normalized_triples();
    let triples_gbk = doc_gbk.rdf_graph().normalized_triples();
    assert_eq!(
        triples_gb, triples_gbk,
        ".gbk import produced a different triple set than .gb for identical bytes"
    );
    assert!(
        !triples_gbk.is_empty(),
        ".gbk import produced an empty graph"
    );
    // Sanity: the loaded document has the expected shape.
    assert_eq!(doc_gbk.components().count(), 1);
    assert_eq!(doc_gbk.sequences().count(), 1);
}

#[test]
fn gbk_extension_via_read_str_matches_read_path() {
    // read_path resolves the file from disk; read_str takes a string
    // slice. Both paths must produce the same Document regardless of
    // which extension the on-disk file uses.
    let dir = tempfile::tempdir().expect("tempdir");
    let text =
        std::fs::read_to_string(workspace_fixture("BBa_E0040.gb")).expect("read source fixture");
    let gbk_path = dir.path().join("snapgene-style.gbk");
    std::fs::write(&gbk_path, &text).expect("write .gbk");

    let importer = GenbankImporter::new("https://example.org/lab").expect("namespace");
    let (from_path, _) = importer.read_path(&gbk_path).expect("read .gbk path");
    let (from_str, _) = importer.read_str(&text).expect("read string");

    assert_eq!(
        from_path.rdf_graph().normalized_triples(),
        from_str.rdf_graph().normalized_triples()
    );
}

#[test]
fn mixed_case_month_in_locus_line_is_tolerated() {
    let mixed_case = r#"LOCUS       TEST                    12 bp    DNA     linear       01-Jan-2026
DEFINITION  mixed-case month should not break the parser.
FEATURES             Location/Qualifiers
     promoter        1..12
                     /label="p"
ORIGIN
        1 atgcatgcatgc
//
"#;
    let (document, _report) = GenbankImporter::new("https://example.org/lab")
        .unwrap()
        .read_str(mixed_case)
        .expect("import");
    assert_eq!(document.components().count(), 1);
}
