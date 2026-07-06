//! Per-spec-area validation rule regression cases. Each sub-module
//! exposes a `cases()` function returning a `Vec<RuleCase>`; the
//! top-level `all_rule_cases()` concatenates them for the coverage
//! cross-check tests in `tests/validation_rules.rs`.
//!
//! Coverage is split + selective: every non-deferred rule has at
//! least one negative case, and high-value clusters (sequence,
//! location, constraint, common, component) also expose positives
//! via `positives()`. Full positive/boundary/insufficient-info matrix
//! expansion is gated on the rule catalog becoming a first-class
//! artifact.

pub mod attachment;
pub mod combinatorial;
pub mod common;
pub mod component;
pub mod constraint;
pub mod feature;
pub mod interaction;
pub mod location;
pub mod om;
pub mod sequence;
pub mod workflow;

pub const PREFIXES: &str = r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX EDAM: <https://identifiers.org/edam:>
PREFIX CHEBI: <https://identifiers.org/CHEBI:>
PREFIX GO: <https://identifiers.org/GO:>
PREFIX om: <http://www.ontology-of-units-of-measure.org/resource/om-2/>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX SO: <https://identifiers.org/SO:>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX sbol: <http://sbols.org/v3#>
"#;

/// Rules whose coverage comes from a dedicated test outside `cases()` —
/// they exercise opt-in validation policy (topology completeness).
pub const OPTION_POLICY_RULES: &[&str] = &["sbol3-10606", "sbol3-11006", "sbol3-11106"];

/// Rules whose coverage comes from a dedicated test outside `cases()` —
/// they require external resolver-backed validation.
pub const EXTERNAL_POLICY_RULES: &[&str] = &["sbol3-10114", "sbol3-12501", "sbol3-12801"];

/// Negative case: a Turtle body that MUST report `rule` at
/// `severity`. The legacy 117-entry catalog is built from these.
#[derive(Clone, Copy)]
pub struct RuleCase {
    pub name: &'static str,
    pub rule: &'static str,
    pub severity: sbol3::Severity,
    pub body: &'static str,
}

/// Positive case: a Turtle body that MUST NOT report `rule`. The body
/// may still trigger other rules (we only assert the named one is
/// absent), but it should be locally valid for the rule under test.
/// Currently shipped for the high-value clusters (sequence, location,
/// constraint, common, component); per-rule positive coverage for
/// every non-deferred rule is gated on the rule catalog becoming a
/// first-class artifact.
#[derive(Clone, Copy)]
pub struct PositiveCase {
    pub name: &'static str,
    pub rule: &'static str,
    pub body: &'static str,
}

pub fn all_rule_cases() -> Vec<RuleCase> {
    let mut cases = Vec::new();
    cases.extend(common::cases());
    cases.extend(sequence::cases());
    cases.extend(component::cases());
    cases.extend(feature::cases());
    cases.extend(location::cases());
    cases.extend(constraint::cases());
    cases.extend(interaction::cases());
    cases.extend(combinatorial::cases());
    cases.extend(attachment::cases());
    cases.extend(om::cases());
    cases.extend(workflow::cases());
    cases
}

pub fn all_positive_cases() -> Vec<PositiveCase> {
    let mut cases = Vec::new();
    cases.extend(common::positives());
    cases.extend(sequence::positives());
    cases.extend(component::positives());
    cases.extend(feature::positives());
    cases.extend(location::positives());
    cases.extend(constraint::positives());
    cases.extend(interaction::positives());
    cases.extend(combinatorial::positives());
    cases.extend(attachment::positives());
    cases.extend(om::positives());
    cases.extend(workflow::positives());
    cases
}

pub fn read_case(case: RuleCase) -> sbol3::Document {
    let turtle = format!("{PREFIXES}\n{}", case.body);
    sbol3::Document::read_turtle(&turtle)
        .unwrap_or_else(|error| panic!("{} did not parse: {error}", case.name))
}

pub fn read_positive_case(case: PositiveCase) -> sbol3::Document {
    let turtle = format!("{PREFIXES}\n{}", case.body);
    sbol3::Document::read_turtle(&turtle)
        .unwrap_or_else(|error| panic!("{} did not parse: {error}", case.name))
}

pub fn assert_rule(report: &sbol3::ValidationReport, rule: &str) {
    assert!(
        report.issues().iter().any(|issue| issue.rule == rule),
        "{rule} was not reported; got {:?}",
        report.issues()
    );
}

pub fn assert_no_rule(report: &sbol3::ValidationReport, rule: &str) {
    assert!(
        report.issues().iter().all(|issue| issue.rule != rule),
        "{rule} was reported unexpectedly: {:?}",
        report.issues()
    );
}

/// Cross-cluster body generator: Component/LocalSubComponent role-type
/// compatibility. Used by both `component::cases()` (10609-10613) and
/// `feature::cases()` (11009-11012).
pub fn component_role_type_body(kind: &'static str) -> &'static str {
    match kind {
        "component_incompatible" => {
            r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:role SO:0000167;
    sbol:type SBO:0000252 .
"#
        }
        "component_role_not_component" => {
            r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:role SBO:0000176;
    sbol:type SBO:0000241 .
"#
        }
        "component_sequence_feature_without_nucleic_acid" => {
            r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:role SO:0000167;
    sbol:type SBO:0000252 .
"#
        }
        "component_dna_missing_sequence_feature_role" => {
            r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#
        }
        "local_subcomponent_incompatible" => {
            r#":feature a sbol:LocalSubComponent;
    sbol:displayId "feature";
    sbol:role SO:0000167;
    sbol:type SBO:0000252 .
"#
        }
        "local_subcomponent_sequence_feature_without_nucleic_acid" => {
            r#":feature a sbol:LocalSubComponent;
    sbol:displayId "feature";
    sbol:role SO:0000167;
    sbol:type SBO:0000252 .
"#
        }
        "local_subcomponent_dna_missing_sequence_feature_role" => {
            r#":feature a sbol:LocalSubComponent;
    sbol:displayId "feature";
    sbol:type SBO:0000251 .
"#
        }
        _ => unreachable!(),
    }
}

/// Cross-cluster body generator: overlapping-location regression
/// scaffolding for `SubComponent`, `LocalSubComponent`, and
/// `SequenceFeature`. Used by `component::cases()` (10805),
/// `feature::cases()` (11013, 11201).
pub fn overlapping_location_body(
    feature_class: &'static str,
    extra_feature_property: &'static str,
) -> &'static str {
    match (feature_class, extra_feature_property) {
        ("sbol:SubComponent", "sbol:instanceOf :definition;") => {
            r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:hasLocation <component/feature/range1>, <component/feature/range2>;
    sbol:instanceOf :definition .
<component/feature/range1> a sbol:Range;
    sbol:displayId "range1";
    sbol:end "3";
    sbol:hasSequence :sequence;
    sbol:start "1" .
<component/feature/range2> a sbol:Range;
    sbol:displayId "range2";
    sbol:end "4";
    sbol:hasSequence :sequence;
    sbol:start "2" .
"#
        }
        ("sbol:LocalSubComponent", "sbol:type SBO:0000251;") => {
            r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:feature a sbol:LocalSubComponent;
    sbol:displayId "feature";
    sbol:hasLocation <feature/range1>, <feature/range2>;
    sbol:type SBO:0000251 .
<feature/range1> a sbol:Range;
    sbol:displayId "range1";
    sbol:end "3";
    sbol:hasSequence :sequence;
    sbol:start "1" .
<feature/range2> a sbol:Range;
    sbol:displayId "range2";
    sbol:end "4";
    sbol:hasSequence :sequence;
    sbol:start "2" .
"#
        }
        ("sbol:SequenceFeature", "") => {
            r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:feature a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <feature/range1>, <feature/range2> .
<feature/range1> a sbol:Range;
    sbol:displayId "range1";
    sbol:end "3";
    sbol:hasSequence :sequence;
    sbol:start "1" .
<feature/range2> a sbol:Range;
    sbol:displayId "range2";
    sbol:end "4";
    sbol:hasSequence :sequence;
    sbol:start "2" .
"#
        }
        _ => unreachable!(),
    }
}
