//! Build-time code generator that turns a versioned SBOL validation
//! `rules.toml` into Rust rule-catalog source.
//!
//! The `sbol2` and `sbol3` crates each call [`generate`] from their `build.rs`,
//! pointing at their own `rules.toml`. This keeps one generator, one set of
//! catalog invariants, and one emitted shape across both data-model versions.
//!
//! Cold-compile cost on a fresh `target/` dir is roughly 30s because of the
//! `toml`+`serde` dep stack. Subsequent builds reuse the cached deps; a caller
//! re-runs the script only when its `rules.toml`, `build.rs`, or policies
//! directory changes (the caller emits the `cargo:rerun-if-changed` directives).
//!
//! Outputs into `out_dir`:
//!   - `rule_catalog.rs` — `VALIDATION_RULE_STATUSES` slice literal, sorted by
//!     rule id for diff stability.
//!   - `rule_spec_meta.rs` — the `VALIDATION_RULE_SPEC_*` constants sourced from
//!     the TOML `[meta]` block.
//!
//! Failures (TOML parse error, unknown enum variant, missing required field,
//! missing policy ADR) panic with a message that names the offending rule id
//! and field, so a build failure points straight at the bad entry.

use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    meta: Meta,
    #[serde(default, rename = "rule")]
    rules: Vec<RawRule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Meta {
    spec_version: String,
    spec_path: String,
    spec_canonical_url: String,
    spec_pdf_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    id: String,
    status: String,
    normative_severity: String,
    spec_section: String,
    note: String,
    #[serde(default)]
    blocker: Option<String>,
    #[serde(default)]
    validator_function: Option<String>,
    /// Optional per-rule coverage tag. When set, overrides the
    /// default inferred from `status`/`blocker`.
    #[serde(default)]
    coverage_kind: Option<String>,
}

/// Generates `rule_catalog.rs` and `rule_spec_meta.rs` into `out_dir` from the
/// `rules.toml` at `rules_toml`.
///
/// When `policies_dir` is `Some`, every rule whose blocker is `Policy` must
/// have a matching `<rule-id>.md` ADR file in that directory; a missing ADR
/// panics. Pass `None` to skip the check (for example when building from a
/// packaged crates.io tarball, where the workspace policies directory is
/// absent).
///
/// Panics on any catalog error, naming the offending rule and field.
pub fn generate(rules_toml: &Path, out_dir: &Path, policies_dir: Option<&Path>) {
    let text = match fs::read_to_string(rules_toml) {
        Ok(t) => t,
        Err(err) => panic!("failed to read {}: {err}", rules_toml.display()),
    };
    let catalog: Catalog = match toml::from_str(&text) {
        Ok(c) => c,
        Err(err) => panic!("failed to parse {}: {err}", rules_toml.display()),
    };

    let mut rules = catalog.rules;
    rules.sort_by(|a, b| a.id.cmp(&b.id));
    for rule in &rules {
        validate_status(&rule.id, &rule.status);
        validate_severity(&rule.id, &rule.normative_severity);
        validate_blocker(&rule.id, &rule.status, rule.blocker.as_deref());
        validate_coverage_kind(&rule.id, rule.coverage_kind.as_deref());
        if let Some(policies_dir) = policies_dir {
            if rule.blocker.as_deref() == Some("Policy") {
                let adr_path = policies_dir.join(format!("{}.md", rule.id));
                if !adr_path.exists() {
                    panic!(
                        "rule {}: blocker = \"Policy\" requires {}/{}.md (not found at {})",
                        rule.id,
                        policies_dir.display(),
                        rule.id,
                        adr_path.display()
                    );
                }
            }
        }
    }

    write_rule_catalog(&out_dir.join("rule_catalog.rs"), &rules);
    write_spec_meta(&out_dir.join("rule_spec_meta.rs"), &catalog.meta);
}

const VALID_STATUSES: &[&str] = &[
    "Error",
    "Warning",
    "Configurable",
    "MachineUncheckable",
    "Unimplemented",
];

fn validate_status(rule_id: &str, status: &str) -> &'static str {
    for valid in VALID_STATUSES {
        if status == *valid {
            return valid;
        }
    }
    panic!("rule {rule_id}: invalid status `{status}` (expected one of {VALID_STATUSES:?})");
}

fn validate_severity(rule_id: &str, severity: &str) {
    match severity {
        "MUST" | "SHOULD" | "MAY" => {}
        other => panic!(
            "rule {rule_id}: invalid normative_severity `{other}` (expected MUST, SHOULD, or MAY)"
        ),
    }
}

fn validate_coverage_kind(rule_id: &str, coverage_kind: Option<&str>) {
    if let Some(value) = coverage_kind {
        match value {
            "OntologyKnownTermsOnly"
            | "LocalReferencesOnly"
            | "LexicalShapeOnly"
            | "PolicyDefaultUndecided" => {}
            other => panic!(
                "rule {rule_id}: invalid coverage_kind `{other}` (expected \
                 OntologyKnownTermsOnly, LocalReferencesOnly, LexicalShapeOnly, \
                 or PolicyDefaultUndecided)"
            ),
        }
    }
}

fn validate_blocker(rule_id: &str, status: &str, blocker: Option<&str>) {
    // `Error` and `Warning` are unconditional algorithms — no blocker.
    // Every other status carries a blocker that names the configuration
    // axis (Configurable), the spec context (MachineUncheckable), or
    // what's needed to implement (Unimplemented).
    let needs_blocker = !matches!(status, "Error" | "Warning");
    match (needs_blocker, blocker) {
        (true, None) => {
            panic!("rule {rule_id}: status `{status}` requires a `blocker = \"...\"` entry")
        }
        (false, Some(b)) => panic!(
            "rule {rule_id}: status `{status}` must not have a `blocker` entry (found `{b}`)"
        ),
        (true, Some(b)) => match b {
            "Ontology" | "Resolver" | "StrictDatatype" | "Policy" | "External" => {}
            other => panic!(
                "rule {rule_id}: invalid blocker `{other}` (expected Ontology, Resolver, \
                 StrictDatatype, Policy, or External)"
            ),
        },
        (false, None) => {}
    }
}

fn write_rule_catalog(path: &Path, rules: &[RawRule]) {
    use std::fmt::Write;
    let mut buf = String::new();
    buf.push_str(
        "// Generated by sbol-rulegen from rules.toml. Do not edit by hand.\n\
         const VALIDATION_RULE_STATUSES: &[ValidationRuleStatus] = &[\n",
    );
    for rule in rules {
        let status = validate_status(&rule.id, &rule.status);
        let severity = match rule.normative_severity.as_str() {
            "MUST" => "Must",
            "SHOULD" => "Should",
            "MAY" => "May",
            other => panic!("rule {}: invalid severity `{other}`", rule.id),
        };
        writeln!(buf, "    ValidationRuleStatus::new(").unwrap();
        writeln!(buf, "        {},", rust_string_literal(&rule.id)).unwrap();
        writeln!(buf, "        RuleStatus::{status},").unwrap();
        writeln!(buf, "        NormativeSeverity::{severity},").unwrap();
        writeln!(buf, "        {},", rust_string_literal(&rule.spec_section)).unwrap();
        writeln!(buf, "        {},", rust_string_literal(&rule.note)).unwrap();
        match rule.blocker.as_deref() {
            Some(b) => writeln!(buf, "        Some(super::Blocker::{b}),").unwrap(),
            None => writeln!(buf, "        None,").unwrap(),
        }
        match rule.validator_function.as_deref() {
            Some(fn_name) => {
                writeln!(buf, "        Some({}),", rust_string_literal(fn_name)).unwrap()
            }
            None => writeln!(buf, "        None,").unwrap(),
        }
        match rule.coverage_kind.as_deref() {
            Some(kind) => writeln!(buf, "        Some(super::CoverageKind::{kind}),").unwrap(),
            None => writeln!(buf, "        None,").unwrap(),
        }
        writeln!(buf, "    ),").unwrap();
    }
    buf.push_str("];\n");
    fs::write(path, buf).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
}

fn write_spec_meta(path: &Path, meta: &Meta) {
    use std::fmt::Write;
    let mut buf = String::new();
    buf.push_str("// Generated by sbol-rulegen from rules.toml. Do not edit by hand.\n");
    writeln!(
        buf,
        "pub const VALIDATION_RULE_SPEC_VERSION: &str = {};",
        rust_string_literal(&meta.spec_version)
    )
    .unwrap();
    writeln!(
        buf,
        "pub const VALIDATION_RULE_SPEC_PATH: &str = {};",
        rust_string_literal(&meta.spec_path)
    )
    .unwrap();
    writeln!(
        buf,
        "pub const VALIDATION_RULE_SPEC_CANONICAL_URL: &str = {};",
        rust_string_literal(&meta.spec_canonical_url)
    )
    .unwrap();
    writeln!(
        buf,
        "pub const VALIDATION_RULE_SPEC_PDF_SHA256: &str = {};",
        rust_string_literal(&meta.spec_pdf_sha256)
    )
    .unwrap();
    fs::write(path, buf).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
}

fn rust_string_literal(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write;
                write!(out, "\\u{{{:x}}}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
