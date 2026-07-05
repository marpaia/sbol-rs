//! Round-trip fidelity: parse → typed → serialize must reproduce the input
//! triple set (including retained extension triples) for the vendored SBOL 2
//! fixtures.

use std::collections::BTreeSet;

use sbol2::{Document, RdfFormat, Triple};

mod common;

fn triple_set(document: &Document) -> BTreeSet<Triple> {
    document.rdf_graph().normalized_triples().into_iter().collect()
}

fn assert_round_trip(input: &str, format: RdfFormat, label: &str) {
    let document = Document::read(input, format)
        .unwrap_or_else(|error| panic!("{label} did not parse: {error}"));
    let rebuilt = Document::from_objects(document.typed_objects().to_vec())
        .unwrap_or_else(|error| panic!("{label} did not rebuild from typed objects: {error}"));

    let original = triple_set(&document);
    let round_tripped = triple_set(&rebuilt);

    if original != round_tripped {
        let lost: Vec<_> = original.difference(&round_tripped).collect();
        let gained: Vec<_> = round_tripped.difference(&original).collect();
        panic!(
            "{label} did not round-trip.\n  lost {} triples: {:#?}\n  gained {} triples: {:#?}",
            lost.len(),
            lost,
            gained.len(),
            gained
        );
    }
}

const TURTLE_FIXTURES: &[&str] = &[
    "single_cd.ttl",
    "md_simple.ttl",
    "collection.ttl",
    "cd_with_subparts.ttl",
    "cd_with_annotation.ttl",
    "mapsto_merge.ttl",
    "urn_design.ttl",
];

#[test]
fn turtle_fixtures_round_trip() {
    for fixture in TURTLE_FIXTURES {
        let input = common::read_fixture(fixture);
        assert_round_trip(&input, RdfFormat::Turtle, fixture);
    }
}

fn xml_fixtures() -> Vec<String> {
    let root = common::fixture_root().join("real");
    let mut out = Vec::new();
    collect_xml(&root, &common::fixture_root(), &mut out);
    out.sort();
    out
}

fn collect_xml(dir: &std::path::Path, base: &std::path::Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_xml(&path, base, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("xml") {
            let relative = path.strip_prefix(base).unwrap().to_string_lossy().into_owned();
            out.push(relative);
        }
    }
}

#[test]
fn real_xml_fixtures_round_trip() {
    let fixtures = xml_fixtures();
    assert!(!fixtures.is_empty(), "expected vendored SBOL 2 XML fixtures");
    for fixture in fixtures {
        let input = common::read_fixture(&fixture);
        assert_round_trip(&input, RdfFormat::RdfXml, &fixture);
    }
}
