//! End-to-end CLI tests: flag surface, exit codes, output formats.

use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

const TTL_VALID: &str = r#"
PREFIX sbol: <http://sbols.org/v3#>
PREFIX SBO: <https://identifiers.org/SBO:>

<https://example.org/lab/c> a sbol:Component;
    sbol:displayId "c";
    sbol:hasNamespace <https://example.org/lab>;
    sbol:type SBO:0000251 .
"#;

const TTL_INVALID: &str = r#"
PREFIX sbol: <http://sbols.org/v3#>

<https://example.org/lab/c> a sbol:Component;
    sbol:displayId "1bad" ;
    sbol:hasNamespace <https://example.org/lab> .
"#;

fn write_fixture(dir: &TempDir, name: &str, contents: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn version_flag_reports_package_version() {
    let out = Command::cargo_bin("sbol")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    assert!(
        text.starts_with(&format!("sbol {}", env!("CARGO_PKG_VERSION"))),
        "unexpected --version output: {text:?}"
    );
}

#[test]
fn exit_zero_on_clean_validation() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "ok.ttl", TTL_VALID);
    Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn exit_one_on_error_severity_issue() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "bad.ttl", TTL_INVALID);
    Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .code(1);
}

#[test]
fn exit_two_on_io_error() {
    Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", "/nonexistent/path/to/document.ttl"])
        .assert()
        .code(2);
}

#[test]
fn exit_two_on_unknown_rule_id_in_override() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "ok.ttl", TTL_VALID);
    Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", path.to_str().unwrap(), "--allow", "sbol3-99999"])
        .assert()
        .code(2);
}

#[test]
fn allow_flag_suppresses_diagnostic_in_output() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "bad.ttl", TTL_INVALID);
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "validate",
            path.to_str().unwrap(),
            "--allow",
            "sbol3-10110",
            "--format",
            "json",
        ])
        .assert();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let issues = value["issues"].as_array().unwrap();
    assert!(
        issues
            .iter()
            .all(|issue| issue["rule"].as_str() != Some("sbol3-10110")),
        "sbol3-10110 should not appear: {stdout}"
    );
}

#[test]
fn json_output_parses_as_json() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "ok.ttl", TTL_VALID);
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", path.to_str().unwrap(), "--format", "json"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let trimmed = stdout.trim();
    let value: serde_json::Value = serde_json::from_str(trimmed)
        .unwrap_or_else(|err| panic!("CLI JSON output failed to parse: {err}\n{trimmed}"));
    assert_eq!(value["schema_version"].as_u64().unwrap(), 1);
}

#[test]
fn show_coverage_summary_appears_in_text_output() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "ok.ttl", TTL_VALID);
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", path.to_str().unwrap(), "--show-coverage"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("coverage:"),
        "coverage summary missing: {stdout}"
    );
}

#[test]
fn treat_partial_as_errors_yields_exit_code_three_on_clean_document() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "ok.ttl", TTL_VALID);
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "validate",
            path.to_str().unwrap(),
            "--treat-partial-as-errors",
        ])
        .assert()
        .code(3);
}

#[test]
fn external_mode_allowed_without_cache_dir_errors() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "ok.ttl", TTL_VALID);
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "validate",
            path.to_str().unwrap(),
            "--external-mode",
            "allowed",
        ])
        .assert()
        .code(2);
}

#[test]
fn external_mode_provided_succeeds_with_resolve_documents() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "ok.ttl", TTL_VALID);
    let docs_dir = dir.path().join("docs");
    std::fs::create_dir(&docs_dir).unwrap();
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "validate",
            path.to_str().unwrap(),
            "--external-mode",
            "provided",
            "--resolve-documents",
            docs_dir.to_str().unwrap(),
        ])
        .assert()
        .success();
}

#[test]
fn unknown_ontology_name_does_not_suggest_installing_it() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "ok.ttl", TTL_VALID);
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "validate",
            path.to_str().unwrap(),
            "--ontology",
            "definitely-not-an-ontology",
        ])
        .assert()
        .code(2);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("unknown ontology extension"),
        "expected unknown-ontology error, got: {stderr}"
    );
    assert!(
        !stderr.contains("install definitely-not-an-ontology"),
        "should not advise installing a name the CLI does not know: {stderr}"
    );
}

#[test]
fn text_output_prefixes_path_on_every_issue_line() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "bad.ttl", TTL_INVALID);
    let assertion = Command::cargo_bin("sbol")
        .unwrap()
        .args(["validate", path.to_str().unwrap()])
        .assert()
        .code(1);
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let path_str = path.display().to_string();
    let issue_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("error[") || line.contains("warning["))
        .collect();
    assert!(
        !issue_lines.is_empty(),
        "expected at least one issue line: {stdout}"
    );
    for line in &issue_lines {
        assert!(
            line.starts_with(&path_str),
            "issue line missing path prefix: {line}"
        );
    }
}

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

#[test]
fn output_file_receives_json() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "ok.ttl", TTL_VALID);
    let out_path = dir.path().join("report.json");
    Command::cargo_bin("sbol")
        .unwrap()
        .args([
            "validate",
            path.to_str().unwrap(),
            "--format",
            "json",
            "--output",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let contents = fs::read_to_string(&out_path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert!(value["coverage"].is_object());
}
