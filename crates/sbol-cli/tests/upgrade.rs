//! CLI tests for the `sbol upgrade` subcommand.

use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use tempfile::TempDir;

const SBOL2_MINIMAL: &str = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .
@prefix so: <https://identifiers.org/SO:> .

<https://example.org/lab/J23100/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/J23100> ;
    sbol:displayId "J23100" ;
    sbol:version "1" ;
    sbol:type biopax:Dna ;
    sbol:role so:0000167 .
"#;

const SBOL3_INPUT: &str = r#"
@prefix sbol: <http://sbols.org/v3#> .

<https://example.org/lab/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> .
"#;

fn write_fixture(dir: &TempDir, name: &str, contents: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn upgrade_writes_sbol3_to_explicit_output() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "input.ttl", SBOL2_MINIMAL);
    let output = dir.path().join("out.ttl");
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "upgrade",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    let contents = fs::read_to_string(&output).unwrap();
    assert!(
        contents.contains("http://sbols.org/v3#Component"),
        "output should contain SBOL 3 Component type: {contents}"
    );
    // SBOL 2 IRIs only appear as object values of backport predicates
    // (sbol2type, sbol2persistentIdentity, sbol2version) — never as subject
    // identities or active predicates.
    for line in contents.lines() {
        if let Some(predicate_start) = line.find("http://sbols.org/v2#") {
            let before = &line[..predicate_start];
            assert!(
                before.contains("http://sboltools.org/backport#"),
                "non-backport SBOL2 reference in line: {line}"
            );
        }
    }
}

#[test]
fn upgrade_infers_rdfxml_output_from_xml_extension() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "input.ttl", SBOL2_MINIMAL);
    let output = dir.path().join("out.xml");
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "upgrade",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    let contents = fs::read_to_string(&output).unwrap();
    assert!(
        contents.contains("Component") || contents.contains("v3#"),
        ".xml output should be inferred as RDF/XML: {contents}"
    );
}

#[test]
fn upgrade_to_stdout_requires_explicit_format() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "input.ttl", SBOL2_MINIMAL);
    Command::cargo_bin("sbol")
        .unwrap()
        .args(["upgrade", input.to_str().unwrap()])
        .assert()
        .code(2);
}

#[test]
fn upgrade_writes_to_stdout_with_explicit_format() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "input.ttl", SBOL2_MINIMAL);
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["upgrade", input.to_str().unwrap(), "--to", "turtle"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("http://sbols.org/v3#Component"));
}

#[test]
fn upgrade_rejects_sbol3_input() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "input.ttl", SBOL3_INPUT);
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "upgrade",
            input.to_str().unwrap(),
            "--to",
            "turtle",
            "-o",
            "-",
        ])
        .assert()
        .code(2);
}

#[test]
fn upgrade_exits_two_on_missing_file() {
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "upgrade",
            "/nonexistent/sbol2.ttl",
            "--to",
            "turtle",
            "-o",
            "-",
        ])
        .assert()
        .code(2);
}

#[test]
fn upgrade_accepts_xml_extension_for_sbol2_input() {
    // SBOLTestSuite, SynBioHub, and iGEM all ship RDF/XML with a `.xml`
    // extension. The library's RdfFormat::from_path is strict and refuses
    // `.xml`, so the upgrade command tolerates it locally.
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("tests/fixtures/sbol2/real/implementation_example.xml");
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "upgrade",
            path.to_str().unwrap(),
            "--to",
            "turtle",
            "-o",
            "-",
        ])
        .assert()
        .success();
}

#[test]
fn upgrade_from_flag_overrides_extension_inference() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "weird.dat", SBOL2_MINIMAL);
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "upgrade",
            input.to_str().unwrap(),
            "--from",
            "turtle",
            "--to",
            "turtle",
            "-o",
            "-",
        ])
        .assert()
        .success();
}

#[test]
fn upgrade_validate_flag_succeeds_on_well_formed_input() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "input.ttl", SBOL2_MINIMAL);
    let output = dir.path().join("out.ttl");
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "upgrade",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--validate",
        ])
        .assert()
        .success();
}

#[test]
fn upgrade_strict_succeeds_when_no_warnings() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "input.ttl", SBOL2_MINIMAL);
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "upgrade",
            input.to_str().unwrap(),
            "--to",
            "turtle",
            "-o",
            "-",
            "--strict",
        ])
        .assert()
        .success();
}
