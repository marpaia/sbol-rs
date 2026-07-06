//! Round-trip tests for the hand-written JSON v1 emitter in
//! `crates/sbol/src/validation/output.rs`. The core sbol crate does not
//! depend on serde_json; these tests pull it in as a `[dev-dependencies]`
//! deserializer so the emitter's output is verified to be valid JSON
//! with every documented field present.

use serde_json::Value;

#[test]
fn json_emits_schema_version_and_spec_version() {
    let document = sbol3::Document::read_turtle("").unwrap();
    let report = document.validate();
    let json = sbol3::to_json(&report);
    let value: Value = serde_json::from_str(&json).expect("emitted JSON must parse");

    assert_eq!(value["schema_version"].as_u64().unwrap(), 1);
    assert_eq!(value["spec_version"].as_str().unwrap(), "3.1.0");
    assert_eq!(
        value["$schema"].as_str().unwrap(),
        "https://sbolstandard.org/sbol-rs/validation-report/v1.json"
    );
}

#[test]
fn json_round_trips_issue_fields() {
    let document = sbol3::Document::read_turtle(
        r#"
PREFIX sbol: <http://sbols.org/v3#>

<https://example.org/lab/c> a sbol:Component;
    sbol:displayId "1bad" ;
    sbol:hasNamespace <https://example.org/lab> .
"#,
    )
    .unwrap();

    let report = document.validate();
    let json = sbol3::to_json(&report);
    let value: Value = serde_json::from_str(&json).expect("emitted JSON must parse");

    let issues = value["issues"].as_array().expect("issues array");
    let issue = issues
        .iter()
        .find(|issue| issue["rule"].as_str() == Some("sbol3-10201"))
        .expect("rule sbol3-10201 should appear");

    assert_eq!(issue["severity"].as_str().unwrap(), "Error");
    assert_eq!(issue["rule"].as_str().unwrap(), "sbol3-10201");
    assert!(issue["subject"].as_str().is_some());
    assert!(issue.get("property").is_some());
    assert!(issue.get("message").is_some());
    assert!(issue.get("hint").is_some());
}

#[test]
fn json_round_trips_coverage_buckets() {
    let document = sbol3::Document::read_turtle("").unwrap();
    let report = document.validate();
    let json = sbol3::to_json(&report);
    let value: Value = serde_json::from_str(&json).unwrap();

    let coverage = &value["coverage"];
    let fully = coverage["fully_applied"].as_array().unwrap();
    let partial = coverage["partially_applied"].as_array().unwrap();
    let not_applied = coverage["not_applied"].as_array().unwrap();

    let total = fully.len() + partial.len() + not_applied.len();
    let catalog = sbol3::validation_rule_statuses().len();
    assert_eq!(
        total, catalog,
        "every catalog rule appears in exactly one coverage bucket"
    );
}

#[test]
fn json_round_trips_options_summary_and_overrides() {
    let document = sbol3::Document::read_turtle("").unwrap();
    let options = sbol3::ValidationOptions::default()
        .allow("sbol3-10101")
        .unwrap()
        .deny("sbol3-10107")
        .unwrap()
        .with_severity_floor(sbol3::Severity::Warning);
    let report = document.validate_with(options);
    let json = sbol3::to_json(&report);
    let value: Value = serde_json::from_str(&json).unwrap();

    let options_summary = &value["applied_options"];
    assert_eq!(
        options_summary["severity_floor"].as_str().unwrap(),
        "Warning"
    );
    assert!(options_summary["severity_ceiling"].is_null());
    let overridden = options_summary["overridden_rules"].as_array().unwrap();
    assert_eq!(overridden.len(), 2);
    assert!(
        overridden
            .iter()
            .any(|entry| entry["rule"].as_str() == Some("sbol3-10101")
                && entry["override"].as_str() == Some("Suppress"))
    );
    assert!(overridden.iter().any(|entry| {
        entry["rule"].as_str() == Some("sbol3-10107")
            && entry["override"]["Severity"].as_str() == Some("Error")
    }));
}
