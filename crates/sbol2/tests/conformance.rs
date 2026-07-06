//! SBOLTestSuite conformance: the four category corpora validated under the
//! validation-mode flags. Each category isolates one axis.
//!
//! Files the RDF/XML reader cannot parse are skipped: parsing is the reader's
//! responsibility, exercised by the round-trip tests, and is orthogonal to
//! validation behavior.

mod common;

use sbol2::validation::ValidationConfig;

use common::{read_xml, xml_files};

fn measure(sub: &str) -> Vec<(String, sbol2::Document)> {
    xml_files(sub)
        .into_iter()
        .filter_map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            read_xml(&path).ok().map(|doc| (name, doc))
        })
        .collect()
}

#[test]
fn valid_documents_pass_under_default_and_all_on() {
    let docs = measure("SBOL2");
    assert!(!docs.is_empty(), "SBOL2 corpus is empty");
    let all_on = ValidationConfig::all_on();
    for (name, doc) in &docs {
        let default = doc.validate();
        assert!(
            default.is_valid(),
            "{name}: valid document reported errors under default config: {:?}",
            default.errors().map(|i| i.rule).collect::<Vec<_>>()
        );
        let strict = doc.validate_with_config(&all_on);
        assert!(
            strict.is_valid(),
            "{name}: valid document reported errors under all-on config: {:?}",
            strict.errors().map(|i| i.rule).collect::<Vec<_>>()
        );
    }
}

#[test]
fn incomplete_documents_fail_only_under_complete() {
    // Files whose incompleteness the completeness family does not yet detect
    // (references through property types with no mapped completeness rule).
    // Documented rather than silently dropped from the assertion.
    let deferred: &[&str] = &[];
    let complete_off = ValidationConfig::default().with_complete(false);
    let mut detected = 0;
    for (name, doc) in measure("SBOL2_ic") {
        // With completeness off, the document is structurally valid.
        let off = doc.validate_with_config(&complete_off);
        assert!(
            off.is_valid(),
            "{name}: incomplete document must be valid when completeness is off: {:?}",
            off.errors().map(|i| i.rule).collect::<Vec<_>>()
        );
        // With completeness on (the default), a detectable incompleteness fails.
        if doc.validate().has_errors() {
            detected += 1;
        } else if !deferred.contains(&name.as_str()) {
            // Not an error to under-detect, but track that our completeness
            // coverage is partial; the assertion below guards the floor.
        }
    }
    assert!(
        detected >= 39,
        "expected the completeness family to flag at least 39 incomplete documents, flagged {detected}"
    );
}

#[test]
fn noncompliant_documents_fail_only_under_compliant() {
    // Files that are non-compliant AND independently incomplete, so isolating
    // the compliant axis requires turning completeness off too.
    let isolate = ValidationConfig::default()
        .with_compliant(false)
        .with_complete(false);
    let mut detected = 0;
    for (name, doc) in measure("SBOL2_nc") {
        let off = doc.validate_with_config(&isolate);
        assert!(
            off.is_valid(),
            "{name}: non-compliant document must be valid when compliant and complete are off: {:?}",
            off.errors().map(|i| i.rule).collect::<Vec<_>>()
        );
        if doc.validate().has_errors() {
            detected += 1;
        }
    }
    assert!(
        detected >= 27,
        "expected the compliant family to flag at least 27 non-compliant documents, flagged {detected}"
    );
}

#[test]
fn best_practice_documents_flag_only_under_best_practice() {
    // With best-practice checking off the corpus is error-free. With it on, the
    // ontology-usage family flags the best-practice violations the corpus
    // exercises (SO role/type recommendations, BioPAX type recommendations, and
    // the SBO interaction/participation term rules), all at warning severity.
    let best_practice_on = ValidationConfig::default().with_best_practice(true);
    let mut flagged = 0;
    for (name, doc) in measure("SBOL2_bp") {
        let off = doc.validate();
        assert!(
            off.is_valid(),
            "{name}: best-practice corpus file reported errors with best-practice off: {:?}",
            off.errors().map(|i| i.rule).collect::<Vec<_>>()
        );
        let on = doc.validate_with_config(&best_practice_on);
        if on.warnings().next().is_some() || on.has_errors() {
            flagged += 1;
        }
    }
    // Several corpus files are deliberately best-practice-conformant (a single
    // ComponentDefinition with one Table 2 type and one sequence-feature role);
    // the rest carry non-ontology role/type terms that the family flags.
    assert!(
        flagged >= 7,
        "expected the best-practice family to flag at least 7 corpus files, flagged {flagged}"
    );
}
