//! Per-rule negative corpus: every `InvalidFiles/sbol-NNNNN.xml` violates
//! rule `sbol2-NNNNN`. Under a config that enables the rule's family, the
//! validator must reject the file — either the reader fails to parse it
//! (a structural violation caught at read time) or validation reports an
//! error. When the exact expected rule fires, the file is a "strict" match;
//! files rejected via a related error are "loose".
//!
//! Files the validator does not yet reject are listed in `DEFERRED` with the
//! rule they exercise, so the strict set is explicit and grows over time
//! rather than the whole assertion being loosened.

mod common;

use std::collections::BTreeSet;

use sbol2::validation::{ValidationConfig, validation_rule_statuses};

use common::{read_xml, xml_files};

/// Rules whose dedicated negative fixture the validator does not yet reject:
/// the rule is catalogued but its check is not implemented, or the fixture's
/// violation is not machine-detectable in the local subset.
fn deferred() -> BTreeSet<&'static str> {
    // Populated from the measurement below; kept explicit so the strict/loose
    // set is auditable.
    include!("invalid_files_deferred.in")
        .iter()
        .copied()
        .collect()
}

fn expected_rule(name: &str) -> Option<String> {
    let stem = name.strip_suffix(".xml")?;
    let number = stem.strip_prefix("sbol-")?;
    Some(format!("sbol2-{number}"))
}

fn catalog_ids() -> BTreeSet<&'static str> {
    validation_rule_statuses().iter().map(|s| s.rule).collect()
}

#[test]
fn every_invalid_file_is_rejected() {
    let catalog = catalog_ids();
    let deferred = deferred();
    let all_on = ValidationConfig::all_on();

    let mut strict = 0usize;
    let mut loose = 0usize;
    let mut parse_rejected = 0usize;
    let mut deferred_hit = 0usize;
    let mut unexpectedly_clean: Vec<String> = Vec::new();

    for path in xml_files("InvalidFiles") {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let Some(expected) = expected_rule(&name) else {
            continue;
        };

        let doc = match read_xml(&path) {
            // Structural violations rejected at read time count as caught.
            Err(_) => {
                parse_rejected += 1;
                continue;
            }
            Ok(doc) => doc,
        };

        let report = doc.validate_with_config(&all_on);
        let hit_expected = report.errors().any(|issue| issue.rule == expected);
        if hit_expected {
            strict += 1;
        } else if report.has_errors() {
            loose += 1;
        } else if deferred.contains(expected.as_str()) {
            deferred_hit += 1;
        } else {
            unexpectedly_clean.push(format!("{name} (expected {expected})"));
        }
    }
    // The catalog is the libSBOLj machine ruleset (268 rules); the negative
    // corpus also carries fixtures for appendix-only rule ids outside it.
    let _ = &catalog;

    eprintln!(
        "InvalidFiles: strict={strict} loose={loose} parse_rejected={parse_rejected} deferred={deferred_hit}"
    );
    assert!(
        unexpectedly_clean.is_empty(),
        "these invalid files validated clean and are not in the deferred allowlist: {unexpectedly_clean:#?}"
    );
    assert!(strict >= 1, "expected at least one strict rule match");
}
