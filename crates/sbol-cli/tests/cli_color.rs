//! End-to-end CLI tests: global color flag handling and ANSI escape behavior.

mod common;
use common::*;

use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn global_color_flag_accepted_before_subcommand() {
    Command::cargo_bin("sbol")
        .unwrap()
        .args(["--color", "never", "rules", "list"])
        .assert()
        .success();
}

#[test]
fn validate_color_always_paints_severity_label_and_error_prefix() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "bad.ttl", TTL_INVALID);
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["--color", "always", "validate", path.to_str().unwrap()])
        .assert()
        .code(1);
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("\x1b[1;31merror\x1b[0m["),
        "expected bold-red `error` before `[rule]`: {stdout}"
    );
}

#[test]
fn error_label_is_painted_red_on_io_failure() {
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["--color", "always", "validate", "/nonexistent.ttl"])
        .assert()
        .code(2);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.starts_with("\x1b[1;31merror\x1b[0m:"),
        "expected bold-red `error:` prefix on stderr: {stderr}"
    );
}

#[test]
fn color_never_suppresses_ansi_on_stderr_errors() {
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["--color", "never", "validate", "/nonexistent.ttl"])
        .assert()
        .code(2);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("\x1b["),
        "--color never should suppress ANSI on stderr too: {stderr}"
    );
    assert!(
        stderr.starts_with("error:"),
        "expected plain `error:` prefix: {stderr}"
    );
}
