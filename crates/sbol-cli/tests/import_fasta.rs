//! CLI tests for the `sbol import-fasta` subcommand.

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
    workspace_root().join("tests/fixtures/fasta").join(name)
}

#[test]
fn imports_dna_fasta_to_turtle() {
    let dir = TempDir::new().unwrap();
    let output = dir.path().join("pUC19.ttl");
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "import-fasta",
            fixture("pUC19.fasta").to_str().unwrap(),
            "--namespace",
            "https://example.org/lab",
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("1 DNA"),
        "expected DNA tally in stderr: {stderr}"
    );
    let body = std::fs::read_to_string(&output).unwrap();
    assert!(body.contains("http://sbols.org/v3#Component"));
}

#[test]
fn imports_protein_fasta_with_correct_type() {
    let dir = TempDir::new().unwrap();
    let output = dir.path().join("gfp.ttl");
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "import-fasta",
            fixture("GFP_protein.fasta").to_str().unwrap(),
            "--namespace",
            "https://example.org/lab",
            "-o",
            output.to_str().unwrap(),
            "--validate",
        ])
        .assert()
        .success();
    let body = std::fs::read_to_string(&output).unwrap();
    // SBO_PROTEIN
    assert!(body.contains("SBO:0000252"));
}

#[test]
fn forced_alphabet_overrides_detection() {
    let dir = TempDir::new().unwrap();
    // Sequence is pure ACGT — auto-detect would call DNA, but we force protein.
    std::fs::write(dir.path().join("ambig.fasta"), ">test\nACGT\n").unwrap();
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "import-fasta",
            dir.path().join("ambig.fasta").to_str().unwrap(),
            "--namespace",
            "https://example.org/lab",
            "--alphabet",
            "protein",
            "--to",
            "turtle",
            "-o",
            "-",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("1 protein"),
        "expected protein tally with --alphabet protein, got: {stderr}"
    );
}

#[test]
fn missing_namespace_is_an_error() {
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "import-fasta",
            fixture("pUC19.fasta").to_str().unwrap(),
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
            "import-fasta",
            "/nonexistent/missing.fasta",
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
