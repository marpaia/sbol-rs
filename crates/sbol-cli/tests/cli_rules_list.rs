//! End-to-end CLI tests: rules list subcommand output, filtering, and formatting.

mod common;

use assert_cmd::Command;

#[test]
fn rules_list_text_contains_known_rule_and_header() {
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["rules", "list", "--color", "never"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let first_line = stdout.lines().next().unwrap();
    assert!(
        first_line.starts_with("rule"),
        "expected header to start with `rule`, got: {first_line}"
    );
    for header in ["status", "normative", "section", "blocker", "note"] {
        assert!(
            first_line.contains(header),
            "header missing `{header}`: {first_line}"
        );
    }
    assert!(
        stdout.contains("sbol3-10102"),
        "expected sbol3-10102 in rules listing: {stdout}"
    );
    assert!(
        stdout.contains("rules"),
        "expected summary line in rules listing: {stdout}"
    );
    assert!(
        !stdout.contains("\x1b["),
        "--color never should suppress ANSI escapes: {stdout}"
    );
}

#[test]
fn rules_list_truncates_long_notes_by_default() {
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["rules", "list", "--color", "never"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains('…'),
        "expected at least one truncated note (ellipsis): {stdout}"
    );
}

#[test]
fn rules_list_uses_columns_env_for_note_width_when_piped() {
    let narrow = Command::cargo_bin("sbol")
        .unwrap()
        .args(["rules", "list", "--color", "never"])
        .env("COLUMNS", "80")
        .assert()
        .success();
    let narrow_stdout = String::from_utf8(narrow.get_output().stdout.clone()).unwrap();
    let narrow_widest = narrow_stdout
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    let wide = Command::cargo_bin("sbol")
        .unwrap()
        .args(["rules", "list", "--color", "never"])
        .env("COLUMNS", "180")
        .assert()
        .success();
    let wide_stdout = String::from_utf8(wide.get_output().stdout.clone()).unwrap();
    let wide_widest = wide_stdout
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    assert!(
        narrow_widest <= 82,
        "narrow output should respect COLUMNS=80, widest line was {narrow_widest}"
    );
    assert!(
        wide_widest > narrow_widest + 40,
        "wide output ({wide_widest}) should be substantially wider than narrow ({narrow_widest})"
    );
}

#[test]
fn rules_list_full_shows_complete_notes() {
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["rules", "list", "--full", "--color", "never"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains('…'),
        "--full should never emit an ellipsis: {stdout}"
    );
}

#[test]
fn rules_list_color_always_emits_ansi_escapes() {
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["rules", "list", "--color", "always"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("\x1b["),
        "--color always should emit ANSI escapes: {stdout}"
    );
}

#[test]
fn rules_list_json_parses_as_array() {
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["rules", "list", "--format", "json"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let entries = value.as_array().expect("rules list JSON must be an array");
    assert!(!entries.is_empty(), "rules list must not be empty");
    let entry = &entries[0];
    assert!(entry["rule"].is_string());
    assert!(entry["status"].is_string());
    assert!(entry["spec_section"].is_string());
}

#[test]
fn rules_list_status_filter_narrows_output() {
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["rules", "list", "--format", "json", "--status", "error"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let entries = value.as_array().unwrap();
    assert!(!entries.is_empty());
    for entry in entries {
        assert_eq!(entry["status"].as_str().unwrap(), "Error");
    }
}
