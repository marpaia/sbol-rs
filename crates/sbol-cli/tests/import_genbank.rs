//! CLI tests for the `sbol import-genbank` subcommand.

use std::path::PathBuf;

use assert_cmd::Command;
use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path
}

fn fixture(name: &str) -> PathBuf {
    workspace_root().join("tests/fixtures/genbank").join(name)
}

#[test]
fn imports_real_genbank_to_turtle_file() {
    let dir = TempDir::new().unwrap();
    let output = dir.path().join("BBa_E0040.ttl");
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "import-genbank",
            fixture("BBa_E0040.gb").to_str().unwrap(),
            "--namespace",
            "https://example.org/lab",
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("imported:"),
        "expected import summary on stderr, got: {stderr}"
    );
    let body = std::fs::read_to_string(&output).unwrap();
    assert!(body.contains("http://sbols.org/v3#Component"));
    assert!(body.contains("https://example.org/lab/BBa_E0040"));
}

#[test]
fn imports_to_stdout_with_explicit_format() {
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "import-genbank",
            fixture("BBa_E0040.gb").to_str().unwrap(),
            "--namespace",
            "https://example.org/lab",
            "--to",
            "turtle",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("http://sbols.org/v3#Component"));
}

#[test]
fn validate_flag_succeeds_on_clean_import() {
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "import-genbank",
            fixture("BBa_E0040.gb").to_str().unwrap(),
            "--namespace",
            "https://example.org/lab",
            "--to",
            "turtle",
            "-o",
            "-",
            "--validate",
        ])
        .assert()
        .success();
    let _ = assertion;
}

#[test]
fn missing_namespace_is_an_error() {
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "import-genbank",
            fixture("BBa_E0040.gb").to_str().unwrap(),
            "--to",
            "turtle",
            "-o",
            "-",
        ])
        .assert()
        .failure();
}

#[test]
fn missing_input_file_exits_two() {
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "import-genbank",
            "/nonexistent/missing.gb",
            "--namespace",
            "https://example.org/lab",
            "--to",
            "turtle",
            "-o",
            "-",
        ])
        .assert()
        .code(2);
}
