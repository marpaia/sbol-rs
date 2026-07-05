//! Shared helpers for the SBOL 3 → SBOL 2 downgrade integration tests.
#![allow(dead_code)]

use std::path::PathBuf;

pub fn workspace_fixture(relative: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push(relative);
    path
}

pub fn has_triple(graph: &sbol3::RdfGraph, subject: &str, predicate: &str, object: &str) -> bool {
    graph.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some(subject)
            && t.predicate.as_str() == predicate
            && t.object.as_iri().map(|i| i.as_str()) == Some(object)
    })
}

pub fn has_literal_triple(
    graph: &sbol3::RdfGraph,
    subject: &str,
    predicate: &str,
    object: &str,
) -> bool {
    graph.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some(subject)
            && t.predicate.as_str() == predicate
            && t.object.as_literal().map(|l| l.value()) == Some(object)
    })
}

pub fn count_triples(
    graph: &sbol3::RdfGraph,
    subject: &str,
    predicate: &str,
    object: &str,
) -> usize {
    graph
        .triples()
        .iter()
        .filter(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some(subject)
                && t.predicate.as_str() == predicate
                && t.object.as_iri().map(|i| i.as_str()) == Some(object)
        })
        .count()
}
