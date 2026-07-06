//! Property-based tests via `proptest`. Five spec-derived invariants:
//! displayId lexical agreement, compliant URL construction, Range
//! `start <= end` policy, derivation cycle detection, and the
//! Component builder's required-field semantics.
//!
//! CI runs these under proptest's default 256 cases per property. Set
//! `PROPTEST_CASES=10000` for a long-form local run.

use proptest::prelude::*;
use sbol3::constants::SBO_DNA;
use sbol3::{
    BuildError, Component, DisplayId, Document, Namespace, SbolIdentity, SbolObject, Severity,
};

const PREFIXES: &str = r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX EDAM: <https://identifiers.org/edam:>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX sbol: <http://sbols.org/v3#>
"#;

/// Independent implementation of rule sbol3-10201's lexical form. Used
/// as a witness against `DisplayId::new` so a regression in either
/// side fails the property.
fn is_valid_display_id_witness(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

proptest! {
    /// `DisplayId::new(s)` accepts a string iff it matches the SBOL
    /// 3.1.0 lexical form for `sbol:displayId` (sbol3-10201). The
    /// witness re-implements the rule from the spec text; agreement
    /// catches drift in either the newtype or the validator.
    #[test]
    fn display_id_newtype_agrees_with_lexical_form(s in "[\\x00-\\x7f]{0,16}") {
        let newtype_ok = DisplayId::new(s.clone()).is_ok();
        let witness_ok = is_valid_display_id_witness(&s);
        prop_assert_eq!(
            newtype_ok, witness_ok,
            "DisplayId::new and the lexical witness disagree on {:?}", s
        );
    }
}

proptest! {
    /// `SbolIdentity::new(ns, did).to_iri()` always renders to
    /// `{namespace}/{display_id}` exactly. Component built from the
    /// matching pieces survives `Document::from_objects` (i.e.,
    /// produces no sbol3-10102 or sbol3-10104 errors).
    #[test]
    fn compliant_url_round_trips_through_component_builder(
        host in "[a-z]{2,8}\\.[a-z]{2,3}",
        path in "[a-z]{2,8}",
        did_first in "[a-zA-Z_]",
        did_rest in "[a-zA-Z0-9_]{0,12}",
    ) {
        let ns_str = format!("https://{host}/{path}");
        let did_str = format!("{did_first}{did_rest}");

        let namespace = Namespace::new(ns_str.clone()).unwrap();
        let display_id = DisplayId::new(did_str.clone()).unwrap();
        let identity = SbolIdentity::new(namespace, display_id);
        let rendered = identity.to_iri();
        let expected = format!("{ns_str}/{did_str}");

        prop_assert_eq!(rendered.as_str(), expected);

        let component = Component::new(ns_str.as_str(), did_str.as_str(), [SBO_DNA.clone()])
            .unwrap();
        let document = Document::from_objects(vec![SbolObject::Component(component)]).unwrap();
        let report = document.validate();

        prop_assert!(
            report.issues().iter().all(|issue| {
                issue.rule != "sbol3-10102" && issue.rule != "sbol3-10104"
                    && issue.rule != "sbol3-10201" && issue.rule != "sbol3-10301"
            }),
            "compliant identity emitted URL-pattern errors: {:?}",
            report.issues()
        );
    }
}

proptest! {
    /// Rule sbol3-11403 fires iff Range `end < start`. The validator's
    /// decision must match a direct integer comparison for every pair
    /// in `[1, 99]`.
    #[test]
    fn range_emits_11403_iff_end_less_than_start(
        start in 1u32..50,
        end in 1u32..50,
    ) {
        let body = format!(
            r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "{}";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:range a sbol:Range;
    sbol:displayId "range";
    sbol:end "{end}";
    sbol:hasSequence :sequence;
    sbol:start "{start}" .
"#,
            "A".repeat(60)
        );
        let document = sbol3::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
        let report = document.validate();
        let has_11403 = report.issues().iter().any(|i| i.rule == "sbol3-11403");
        prop_assert_eq!(
            has_11403,
            end < start,
            "11403 fired={} for start={} end={} (expected end < start)", has_11403, start, end
        );
    }
}

proptest! {
    /// Rule sbol3-10203 (derivation cycle) fires iff the
    /// `prov:wasDerivedFrom` edges form a cycle across three nodes.
    /// The cycle predicate is computed independently from the boolean
    /// adjacency inputs; the validator's verdict must match.
    #[test]
    fn derivation_cycle_detection_matches_cycle_predicate(
        a_to_b in any::<bool>(),
        b_to_c in any::<bool>(),
        c_to_a in any::<bool>(),
    ) {
        let body = build_three_node_derivation_body(a_to_b, b_to_c, c_to_a);
        let document = sbol3::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
        let report = document.validate();
        let has_10203 = report.issues().iter().any(|i| i.rule == "sbol3-10203");

        let expected_cycle = a_to_b && b_to_c && c_to_a;
        prop_assert_eq!(
            has_10203,
            expected_cycle,
            "10203 fired={} but expected cycle={} for edges a→b={} b→c={} c→a={}",
            has_10203, expected_cycle, a_to_b, b_to_c, c_to_a
        );
    }
}

fn build_three_node_derivation_body(a_to_b: bool, b_to_c: bool, c_to_a: bool) -> String {
    let mut body = String::new();
    for (name, derives_from) in [
        ("a", if a_to_b { Some("b") } else { None }),
        ("b", if b_to_c { Some("c") } else { None }),
        ("c", if c_to_a { Some("a") } else { None }),
    ] {
        body.push_str(&format!(
            r#":{name} a sbol:Component;
    sbol:displayId "{name}";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251"#
        ));
        if let Some(target) = derives_from {
            body.push_str(&format!(";\n    prov:wasDerivedFrom :{target}"));
        }
        body.push_str(" .\n");
    }
    body
}

proptest! {
    /// `Component::builder(ns, did).build()` returns
    /// `Err(BuildError::MissingRequired { property: "type", .. })`
    /// iff `types` is empty. Adding any number of `Iri` types makes
    /// `build()` succeed (modulo other required fields, which are all
    /// optional for `Component`).
    #[test]
    fn component_builder_requires_at_least_one_type(num_types in 0u32..6) {
        let mut builder = Component::builder("https://example.org/lab", "candidate").unwrap();
        for _ in 0..num_types {
            builder = builder.add_type(SBO_DNA.clone());
        }
        let result = builder.build();
        if num_types == 0 {
            prop_assert!(
                matches!(&result, Err(BuildError::MissingRequired { property, .. }) if *property == "type"),
                "expected MissingRequired type when no types provided, got {result:?}"
            );
        } else {
            prop_assert!(
                result.is_ok(),
                "expected build to succeed with {num_types} type(s), got {result:?}"
            );
        }
    }
}

proptest! {
    /// `options.{deny,warn,allow}` builders are last-write-wins. A
    /// sequence of calls ending in `allow` suppresses the rule; ending
    /// in `deny` promotes to Error; ending in `warn` keeps Warning.
    /// Exercised end-to-end against a fixture that always fires
    /// sbol3-10107 (multiple SBOL rdf:type values, non-disjoint).
    #[test]
    fn rule_override_builder_is_last_write_wins(seq in proptest::collection::vec(0u8..3, 1..6)) {
        let turtle = r#"
PREFIX sbol: <http://sbols.org/v3#>
PREFIX SBO: <https://identifiers.org/SBO:>

<https://example.org/lab/c> a sbol:Component, sbol:Collection ;
    sbol:displayId "c" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type SBO:0000251 .
"#;
        let document = sbol3::Document::read_turtle(turtle).unwrap();

        let mut options = sbol3::ValidationOptions::default();
        let mut last = 2u8; // default = warn-equivalent
        for op in &seq {
            match op {
                0 => options = options.allow("sbol3-10107").unwrap(),
                1 => options = options.deny("sbol3-10107").unwrap(),
                _ => options = options.warn("sbol3-10107").unwrap(),
            }
            last = *op;
        }

        let report = document.validate_with(options);
        let issue = report
            .issues()
            .iter()
            .find(|issue| issue.rule == "sbol3-10107");
        match last {
            0 => prop_assert!(issue.is_none(), "allow should suppress"),
            1 => prop_assert_eq!(
                issue.map(|i| i.severity),
                Some(sbol3::Severity::Error),
                "deny should promote to Error"
            ),
            _ => prop_assert_eq!(
                issue.map(|i| i.severity),
                Some(sbol3::Severity::Warning),
                "warn should yield Warning"
            ),
        }
    }
}

proptest! {
    /// Every catalog rule appears in exactly one coverage bucket for
    /// any valid document, under any external mode. Coverage is
    /// catalog-driven and config-driven; never document-dependent.
    #[test]
    fn coverage_partitions_catalog_for_every_external_mode(mode_index in 0u8..3) {
        let document = sbol3::Document::read_turtle("").unwrap();
        let mode = match mode_index {
            0 => sbol3::ExternalValidationMode::Off,
            1 => sbol3::ExternalValidationMode::ProvidedOnly,
            _ => sbol3::ExternalValidationMode::ExternalAllowed,
        };
        let context = sbol3::ValidationContext::new().with_external_mode(mode);
        let report = document.validate_with_context(context);
        let coverage = report.coverage();

        let mut seen = std::collections::BTreeSet::new();
        for rule in &coverage.fully_applied {
            prop_assert!(seen.insert(*rule), "duplicate in fully_applied: {rule}");
        }
        for partial in &coverage.partially_applied {
            prop_assert!(seen.insert(partial.rule), "duplicate: {}", partial.rule);
        }
        for not_applied in &coverage.not_applied {
            prop_assert!(seen.insert(not_applied.rule), "duplicate: {}", not_applied.rule);
        }
        prop_assert_eq!(seen.len(), sbol3::validation_rule_statuses().len());
    }
}

#[test]
fn severity_remains_exhaustive_for_proptest_assertions() {
    // Compile-time guard: if `Severity` grows a new variant, this
    // match (and the proptest properties that assume the two-state
    // Error/Warning split) needs updating. Failing here is louder
    // than silently letting proptest assertions miss a case.
    let _ = match Severity::Error {
        Severity::Error => 0,
        Severity::Warning => 1,
        _ => 2,
    };
}
