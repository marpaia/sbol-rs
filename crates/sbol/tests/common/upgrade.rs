//! Shared helpers for the SBOL 2 → SBOL 3 upgrade integration tests.
#![allow(dead_code)]

use std::path::PathBuf;

use sbol::{Document, RdfFormat};

pub fn workspace_fixture(relative: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("tests/fixtures/sbol2");
    path.push(relative);
    path
}

pub fn upgrade_fixture(name: &str) -> Document {
    let path = workspace_fixture(name);
    let input = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    let (document, report) = Document::upgrade_from_sbol2(&input, RdfFormat::Turtle)
        .unwrap_or_else(|err| panic!("upgrade {}: {err}", path.display()));
    assert!(
        report.is_clean(),
        "{}: unexpected warnings: {:?}",
        name,
        report.warnings()
    );
    document
}

pub fn real_fixture(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("tests/fixtures/sbol2/real");
    path.push(name);
    path
}

pub fn upgrade_real(name: &str) -> (Document, sbol::UpgradeReport) {
    let path = real_fixture(name);
    let input = std::fs::read_to_string(&path).unwrap();
    Document::upgrade_from_sbol2(&input, RdfFormat::RdfXml).expect("real fixture upgrade")
}
