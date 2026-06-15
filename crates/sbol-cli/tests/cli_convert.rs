//! End-to-end CLI tests: convert subcommand serialization and output routing.

mod common;
use common::*;

use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn convert_ttl_to_jsonld_roundtrips_through_validator() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "ok.ttl", TTL_VALID);
    let out_path = dir.path().join("ok.jsonld");
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "convert",
            path.to_str().unwrap(),
            "--output",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(out_path.exists());
    Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", out_path.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn convert_to_flag_overrides_output_extension_inference() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "ok.ttl", TTL_VALID);
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["convert", path.to_str().unwrap(), "--to", "ntriples"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("<https://example.org/lab/c>"),
        "expected N-Triples-style serialization on stdout: {stdout}"
    );
}

#[test]
fn convert_stdout_without_to_flag_errors() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "ok.ttl", TTL_VALID);
    Command::cargo_bin("sbol")
        .unwrap()
        .args(["convert", path.to_str().unwrap()])
        .assert()
        .code(2);
}
