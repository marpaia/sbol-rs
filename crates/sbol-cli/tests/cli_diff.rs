//! End-to-end CLI tests for `sbol diff`.

mod common;
use common::*;

use assert_cmd::Command;
use tempfile::TempDir;

const TTL_OLD: &str = r#"
PREFIX sbol: <http://sbols.org/v3#>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX SO: <https://identifiers.org/SO:>

<https://example.org/lab/j23119> a sbol:Component;
    sbol:displayId "j23119";
    sbol:name "J23119";
    sbol:hasNamespace <https://example.org/lab>;
    sbol:type SBO:0000251;
    sbol:role SO:0000167 .
"#;

const TTL_NEW: &str = r#"
PREFIX sbol: <http://sbols.org/v3#>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX SO: <https://identifiers.org/SO:>

<https://example.org/lab/j23119> a sbol:Component;
    sbol:displayId "j23119";
    sbol:name "J23119 constitutive promoter";
    sbol:hasNamespace <https://example.org/lab>;
    sbol:type SBO:0000251;
    sbol:role SO:0000167;
    sbol:role SO:0000804 .
"#;

fn run(dir: &TempDir, old: &str, new: &str, extra: &[&str]) -> assert_cmd::assert::Assert {
    let old_path = write_fixture(dir, "old.ttl", old);
    let new_path = write_fixture(dir, "new.ttl", new);
    let mut args = vec![
        "--color",
        "never",
        "diff",
        old_path.to_str().unwrap(),
        new_path.to_str().unwrap(),
    ];
    args.extend_from_slice(extra);
    Command::cargo_bin("sbol").unwrap().args(args).assert()
}

#[test]
fn identical_documents_report_no_differences() {
    let dir = TempDir::new().unwrap();
    let assertion = run(&dir, TTL_OLD, TTL_OLD, &[]).success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout.trim(), "no differences");
}

#[test]
fn changed_object_reports_property_moves() {
    let dir = TempDir::new().unwrap();
    let assertion = run(&dir, TTL_OLD, TTL_NEW, &[]).success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("~ https://example.org/lab/j23119"),
        "expected the changed component: {stdout}"
    );
    assert!(
        stdout.contains("+ \"J23119 constitutive promoter\""),
        "expected the added name: {stdout}"
    );
    assert!(
        stdout.contains("- \"J23119\""),
        "expected the removed name: {stdout}"
    );
    assert!(
        stdout.contains("+ https://identifiers.org/SO:0000804"),
        "expected the added role: {stdout}"
    );
    assert!(
        stdout.contains("0 added, 0 removed, 1 changed"),
        "expected a summary line: {stdout}"
    );
}

#[test]
fn exit_code_flag_signals_differences() {
    let dir = TempDir::new().unwrap();
    run(&dir, TTL_OLD, TTL_NEW, &["--exit-code"]).code(1);
}

#[test]
fn exit_code_flag_is_zero_when_identical() {
    let dir = TempDir::new().unwrap();
    run(&dir, TTL_OLD, TTL_OLD, &["--exit-code"]).success();
}

#[test]
fn without_exit_code_flag_differences_still_exit_zero() {
    let dir = TempDir::new().unwrap();
    run(&dir, TTL_OLD, TTL_NEW, &[]).success();
}

#[test]
fn json_output_carries_structured_changes() {
    let dir = TempDir::new().unwrap();
    let assertion = run(&dir, TTL_OLD, TTL_NEW, &["--format", "json"]).success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(value["added"].as_array().unwrap().len(), 0);
    assert_eq!(value["removed"].as_array().unwrap().len(), 0);
    let changed = value["changed"].as_array().unwrap();
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0]["identity"], "https://example.org/lab/j23119");
    let role = &changed[0]["properties"]["http://sbols.org/v3#role"];
    assert_eq!(
        role["added"][0]["resource"],
        "https://identifiers.org/SO:0000804"
    );
}

#[test]
fn cross_version_documents_are_rejected() {
    let dir = TempDir::new().unwrap();
    let v3 = write_fixture(&dir, "v3.ttl", TTL_OLD);
    let v2 = write_fixture(
        &dir,
        "v2.ttl",
        r#"
PREFIX sbol: <http://sbols.org/v2#>
<https://example.org/lab/c/1> a sbol:ComponentDefinition;
    sbol:displayId "c";
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
    );
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "--color",
            "never",
            "diff",
            v2.to_str().unwrap(),
            v3.to_str().unwrap(),
        ])
        .assert()
        .code(2);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("diff compares documents of one version"),
        "expected a version-mismatch error: {stderr}"
    );
}

#[test]
fn missing_file_exits_two() {
    let dir = TempDir::new().unwrap();
    let present = write_fixture(&dir, "present.ttl", TTL_OLD);
    Command::cargo_bin("sbol")
        .unwrap()
        .args(["diff", "/nonexistent.ttl", present.to_str().unwrap()])
        .assert()
        .code(2);
}
