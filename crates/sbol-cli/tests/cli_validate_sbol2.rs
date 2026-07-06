//! End-to-end CLI tests for the SBOL 2 validation path: version detection,
//! dispatch, and the SBOL 2 flag surface.

use std::path::PathBuf;

use assert_cmd::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sbol2")
        .join(name)
}

#[test]
fn sbol2_rdfxml_fixture_validates_through_the_sbol2_path() {
    let path = fixture("valid.rdf");
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("0 errors"),
        "expected a clean SBOL 2 report: {stdout}"
    );
}

#[test]
fn sbol2_turtle_fixture_validates() {
    let path = fixture("valid.ttl");
    Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn sbol2_incomplete_fixture_fails_under_default() {
    let path = fixture("incomplete.ttl");
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .code(1);
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("error["),
        "expected a completeness error: {stdout}"
    );
}

#[test]
fn sbol2_incomplete_fixture_passes_under_incomplete_flag() {
    let path = fixture("incomplete.ttl");
    Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", path.to_str().unwrap(), "--incomplete"])
        .assert()
        .success();
}

#[test]
fn sbol_version_override_forces_sbol2_validator() {
    let path = fixture("valid.ttl");
    Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", path.to_str().unwrap(), "--sbol-version", "2"])
        .assert()
        .success();
}

#[test]
fn sbol2_json_output_uses_the_shared_report_schema() {
    let path = fixture("valid.ttl");
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", path.to_str().unwrap(), "--format", "json"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(value["schema_version"].as_u64().unwrap(), 1);
}

#[test]
fn sbol3_only_flags_warn_on_an_sbol2_document() {
    let path = fixture("valid.ttl");
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "validate",
            path.to_str().unwrap(),
            "--severity-floor",
            "error",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("SBOL 3-only"),
        "expected an SBOL 3-only warning: {stderr}"
    );
}
