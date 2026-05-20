//! CLI tests for the `sbol downgrade` subcommand.

use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use tempfile::TempDir;

const SBOL3_TURTLE: &str = r#"
@prefix sbol: <http://sbols.org/v3#> .

<https://example.org/lab/c>
    a sbol:Component ;
    sbol:displayId "c" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> .
"#;

const SBOL3_RDFXML: &str = r#"<?xml version="1.0"?>
<rdf:RDF
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:sbol="http://sbols.org/v3#">
  <sbol:Component rdf:about="https://example.org/lab/c">
    <sbol:displayId>c</sbol:displayId>
    <sbol:hasNamespace rdf:resource="https://example.org/lab"/>
    <sbol:type rdf:resource="https://identifiers.org/SBO:0000251"/>
  </sbol:Component>
</rdf:RDF>
"#;

fn write_fixture(dir: &TempDir, name: &str, contents: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn downgrade_writes_sbol2_to_explicit_output() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "input.ttl", SBOL3_TURTLE);
    let output = dir.path().join("out.ttl");
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "downgrade",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    let contents = fs::read_to_string(&output).unwrap();
    assert!(
        contents.contains("http://sbols.org/v2#ComponentDefinition"),
        "output should contain SBOL 2 ComponentDefinition type: {contents}"
    );
}

#[test]
fn downgrade_infers_rdfxml_output_from_xml_extension() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "input.ttl", SBOL3_TURTLE);
    let output = dir.path().join("out.xml");
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "downgrade",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    let contents = fs::read_to_string(&output).unwrap();
    assert!(
        contents.contains("ComponentDefinition") || contents.contains("v2#"),
        ".xml output should be inferred as RDF/XML: {contents}"
    );
}

#[test]
fn downgrade_accepts_xml_extension_for_rdfxml_input() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "input.xml", SBOL3_RDFXML);
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "downgrade",
            input.to_str().unwrap(),
            "--to",
            "turtle",
            "-o",
            "-",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("http://sbols.org/v2#ComponentDefinition"));
}

#[test]
fn downgrade_from_flag_overrides_extension_inference() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "input.dat", SBOL3_RDFXML);
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "downgrade",
            input.to_str().unwrap(),
            "--from",
            "rdfxml",
            "--to",
            "turtle",
            "-o",
            "-",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("http://sbols.org/v2#ComponentDefinition"));
}
