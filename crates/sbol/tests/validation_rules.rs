//! Coverage cross-check tests over the per-spec-area `RuleCase`
//! catalog, plus a handful of hand-rolled positive/cluster tests that
//! exercise multi-rule policies (topology completeness, ontology
//! valid-case suppression, namespace IRI validation, OM consistency).
//!
//! The 117 per-rule regression cases live in `tests/rule_cases/`, one
//! file per spec area. See `rule_cases::mod.rs` for the module map and
//! `rule_cases::all_rule_cases` for the concatenated slice consumed
//! here.

use std::collections::BTreeSet;

mod rule_cases;
use rule_cases::{
    PREFIXES, all_positive_cases, all_rule_cases, assert_no_rule, assert_rule, read_case,
    read_positive_case,
};

#[test]
fn implemented_validation_rules_have_regression_cases() {
    let cases = all_rule_cases();
    let mut covered_rules = cases.iter().map(|case| case.rule).collect::<BTreeSet<_>>();
    covered_rules.extend(rule_cases::OPTION_POLICY_RULES.iter().copied());
    covered_rules.extend(rule_cases::EXTERNAL_POLICY_RULES.iter().copied());
    let statuses = sbol::validation_rule_statuses();

    for status in statuses {
        // Statuses with no algorithm don't get a regression case.
        // MachineUncheckable rules either have no algorithm or only a
        // local subset that emits Warnings — those that DO emit are
        // covered by case files in `tests/rule_cases/`; the catalog
        // doesn't enforce a 1:1 mapping for ▲ rules.
        if matches!(
            status.status,
            sbol::RuleStatus::Unimplemented | sbol::RuleStatus::MachineUncheckable
        ) {
            continue;
        }
        assert!(
            covered_rules.contains(status.rule),
            "{} is marked {:?} but has no regression case",
            status.rule,
            status.status
        );
    }

    for rule in covered_rules {
        assert!(
            statuses.iter().any(|status| status.rule == rule),
            "{rule} has a regression case but no validation_rule_statuses entry"
        );
    }
}

#[test]
fn validation_rule_regression_cases_report_expected_rule_ids() {
    for case in all_rule_cases() {
        let document = read_case(case);
        let report = document.validate();
        let issues = match case.severity {
            sbol::Severity::Error => report.errors().collect::<Vec<_>>(),
            sbol::Severity::Warning => report.warnings().collect::<Vec<_>>(),
            _ => panic!("unexpected severity {:?}", case.severity),
        };

        assert!(
            issues.iter().any(|issue| issue.rule == case.rule),
            "{} did not report {} as {:?}; got {:?}",
            case.name,
            case.rule,
            case.severity,
            report.issues()
        );
    }
}

#[test]
fn every_rule_appears_in_exactly_one_coverage_bucket() {
    let document = sbol::Document::read_turtle("").unwrap();
    let report = document.validate();
    let coverage = report.coverage();

    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for rule in &coverage.fully_applied {
        assert!(seen.insert(rule), "{rule} appears in fully_applied twice");
    }
    for partial in &coverage.partially_applied {
        assert!(
            seen.insert(partial.rule),
            "{} appears in multiple coverage buckets",
            partial.rule
        );
    }
    for not_applied in &coverage.not_applied {
        assert!(
            seen.insert(not_applied.rule),
            "{} appears in multiple coverage buckets",
            not_applied.rule
        );
    }

    let catalog_rules: BTreeSet<&str> = sbol::validation_rule_statuses()
        .iter()
        .map(|status| status.rule)
        .collect();
    assert_eq!(
        seen, catalog_rules,
        "every catalog rule must appear in exactly one coverage bucket"
    );
}

#[test]
fn every_appendix_b_triangle_rule_is_marked_machine_uncheckable() {
    // Source of truth: Appendix B of the SBOL 3.1.0 spec, where the ▲
    // symbol marks a rule as weak-REQUIRED but not machine-checkable.
    // The validator surfaces these via coverage rather than diagnostics.
    let expected: BTreeSet<&str> = [
        "sbol3-10502",
        "sbol3-10504",
        "sbol3-10602",
        "sbol3-10603",
        "sbol3-10605",
        "sbol3-10606",
        "sbol3-10609",
        "sbol3-10610",
        "sbol3-10611",
        "sbol3-10614",
        "sbol3-10615",
        "sbol3-10701",
        "sbol3-11002",
        "sbol3-11003",
        "sbol3-11005",
        "sbol3-11006",
        "sbol3-11009",
        "sbol3-11010",
        "sbol3-11102",
        "sbol3-11103",
        "sbol3-11105",
        "sbol3-11106",
        "sbol3-11109",
        "sbol3-11801",
        "sbol3-11802",
        "sbol3-11904",
        "sbol3-11905",
        "sbol3-12301",
        "sbol3-12302",
        "sbol3-12303",
        "sbol3-12501",
        "sbol3-12502",
        "sbol3-12503",
        "sbol3-12505",
        "sbol3-12506",
        "sbol3-12801",
        "sbol3-12802",
        "sbol3-12804",
        "sbol3-12805",
        "sbol3-12806",
    ]
    .into_iter()
    .collect();

    let actual: BTreeSet<&str> = sbol::validation_rule_statuses()
        .iter()
        .filter(|status| status.is_machine_uncheckable())
        .map(|status| status.rule)
        .collect();

    assert_eq!(
        actual, expected,
        "rules.toml `MachineUncheckable` set drifted from spec Appendix B ▲ rules"
    );
}

#[test]
fn deferred_machine_uncheckable_rules_report_machine_uncheckable_reason() {
    use sbol::NotAppliedReason;
    let document = sbol::Document::read_turtle("").unwrap();
    let report = document.validate();
    // sbol3-12303 (Faithful Built) is ▲ + Deferred → MachineUncheckable.
    let entry = report
        .coverage()
        .not_applied
        .iter()
        .find(|entry| entry.rule == "sbol3-12303")
        .expect("sbol3-12303 must appear in not_applied");
    assert!(matches!(entry.reason, NotAppliedReason::MachineUncheckable));
}

#[test]
fn override_unknown_rule_id_is_rejected() {
    let err = sbol::ValidationOptions::default()
        .deny("sbol3-99999")
        .expect_err("unknown rule id should be rejected");
    assert_eq!(err.rule, "sbol3-99999");
}

#[test]
fn allow_suppresses_issue_for_overridden_rule() {
    let document = sbol::Document::read_turtle(
        r#"
PREFIX sbol: <http://sbols.org/v3#>

<https://example.org/lab/c> a sbol:Component;
    sbol:displayId "1bad" ;
    sbol:hasNamespace <https://example.org/lab> .
"#,
    )
    .unwrap();

    let report = document.validate();
    assert!(report.errors().any(|issue| issue.rule == "sbol3-10201"));

    let options = sbol::ValidationOptions::default()
        .allow("sbol3-10201")
        .unwrap();
    let report = document.validate_with(options);
    assert!(
        report
            .issues()
            .iter()
            .all(|issue| issue.rule != "sbol3-10201")
    );
}

#[test]
fn deny_upgrades_warning_to_error() {
    let document = sbol::Document::read_turtle(
        r#"
PREFIX sbol: <http://sbols.org/v3#>
PREFIX SBO: <https://identifiers.org/SBO:>

<https://example.org/lab/c> a sbol:Component, sbol:Collection ;
    sbol:displayId "c" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type SBO:0000251 .
"#,
    )
    .unwrap();
    let report = document.validate();
    let warning_seen = report.warnings().any(|issue| issue.rule == "sbol3-10107");
    assert!(warning_seen, "sbol3-10107 should be a warning by default");

    let options = sbol::ValidationOptions::default()
        .deny("sbol3-10107")
        .unwrap();
    let report = document.validate_with(options);
    assert!(
        report.errors().any(|issue| issue.rule == "sbol3-10107"),
        "deny() should upgrade the warning to an error"
    );
}

#[test]
fn external_mode_provided_promotes_resolver_blocked_rules_to_fully_applied() {
    let document = sbol::Document::read_turtle("").unwrap();

    let report = document.validate();
    let resolver_partial: Vec<_> = report
        .coverage()
        .partially_applied
        .iter()
        .filter(|partial| partial.blocker == sbol::Blocker::Resolver)
        .map(|partial| partial.rule)
        .collect();
    assert!(
        !resolver_partial.is_empty(),
        "expected at least one Resolver-blocked Partial rule by default"
    );

    let context = sbol::ValidationContext::new()
        .with_external_mode(sbol::ExternalValidationMode::ProvidedOnly);
    let report = document.validate_with_context(context);
    for rule in &resolver_partial {
        assert!(
            report.coverage().fully_applied.contains(rule),
            "{rule} should be fully_applied when external_mode is ProvidedOnly"
        );
    }
}

#[test]
fn hash_algorithm_policy_strict_rejects_unknown_algorithm() {
    let turtle = r#"
PREFIX sbol: <http://sbols.org/v3#>

<https://example.org/lab/a> a sbol:Attachment ;
    sbol:displayId "a" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:source <https://example.org/data.txt> ;
    sbol:hash "deadbeef" ;
    sbol:hashAlgorithm "md5" .
"#;
    let document = sbol::Document::read_turtle(turtle).unwrap();

    // Conservative (default): md5 has registry-style shape → no error.
    let report = document.validate();
    assert!(
        report.errors().all(|issue| issue.rule != "sbol3-12806"),
        "Conservative policy must accept md5"
    );

    // Strict: md5 is not in the known-algorithm allowlist → error.
    let mut options = sbol::ValidationOptions::default();
    options.policy.hash_algorithm_registry = sbol::HashAlgorithmRegistry::Strict;
    let report = document.validate_with(options);
    assert!(
        report.errors().any(|issue| issue.rule == "sbol3-12806"),
        "Strict policy must reject md5"
    );

    // Lenient: skip the check entirely.
    let mut options = sbol::ValidationOptions::default();
    options.policy.hash_algorithm_registry = sbol::HashAlgorithmRegistry::Lenient;
    let report = document.validate_with(options);
    assert!(
        report
            .issues()
            .iter()
            .all(|issue| issue.rule != "sbol3-12806"),
        "Lenient policy must skip sbol3-12806"
    );
}

#[test]
fn every_policy_blocked_rule_has_an_adr_file() {
    use std::path::PathBuf;
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let policies_dir = manifest_dir.join("../../docs/policies");
    for status in sbol::validation_rule_statuses() {
        if status.blocker == Some(sbol::Blocker::Policy) {
            let path = policies_dir.join(format!("{}.md", status.rule));
            assert!(
                path.exists(),
                "{}: blocker = Policy requires {} (build invariant)",
                status.rule,
                path.display()
            );
        }
    }
}

#[test]
fn applied_options_round_trips_overrides() {
    let document = sbol::Document::read_turtle("").unwrap();
    let options = sbol::ValidationOptions::default()
        .allow("sbol3-10101")
        .unwrap()
        .deny("sbol3-10107")
        .unwrap()
        .with_severity_floor(sbol::Severity::Warning);
    let report = document.validate_with(options);
    let summary = report.options_summary();
    assert_eq!(summary.overridden_rules.len(), 2);
    assert_eq!(summary.severity_floor, Some(sbol::Severity::Warning));
    assert!(summary.severity_ceiling.is_none());
}

#[test]
fn check_complete_errors_when_any_rule_is_partially_applied() {
    let document = sbol::Document::read_turtle(
        r#"
PREFIX sbol: <http://sbols.org/v3#>
PREFIX SBO: <https://identifiers.org/SBO:>

<https://example.org/lab/c> a sbol:Component;
    sbol:displayId "c";
    sbol:hasNamespace <https://example.org/lab>;
    sbol:type SBO:0000251 .
"#,
    )
    .unwrap();

    assert!(
        document.check().is_ok(),
        "document has no errors so check() should be Ok"
    );
    assert!(
        document.check_complete().is_err(),
        "check_complete() should be Err while any catalog rule remains Partial"
    );
}

#[test]
fn positive_rule_cases_do_not_report_their_rule() {
    for case in all_positive_cases() {
        let document = read_positive_case(case);
        let report = document.validate();
        assert!(
            report.issues().iter().all(|issue| issue.rule != case.rule),
            "positive case `{}` unexpectedly reported {}: {:?}",
            case.name,
            case.rule,
            report.issues()
        );
    }
}

#[test]
fn table_sequence_component_valid_cases_do_not_emit_new_errors() {
    let cases = [
        r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :sequence;
    sbol:type SBO:0000251 .
"#,
        r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "custom";
    sbol:encoding <https://example.org/custom-encoding>;
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :sequence;
    sbol:type SBO:0000251 .
"#,
    ];

    for body in cases {
        let document = sbol::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
        let report = document.validate();

        assert!(
            report.is_valid(),
            "valid local Table 1/Table 2 case produced errors: {:?}",
            report.issues()
        );
        assert_no_rule(&report, "sbol3-10502");
        assert_no_rule(&report, "sbol3-10504");
        assert_no_rule(&report, "sbol3-10505");
        assert_no_rule(&report, "sbol3-10602");
        assert_no_rule(&report, "sbol3-10605");
        assert_no_rule(&report, "sbol3-10614");
        assert_no_rule(&report, "sbol3-10615");
        assert_no_rule(&report, "sbol3-10616");
        assert_no_rule(&report, "sbol3-10617");
    }
}

#[test]
fn ontology_known_type_modifiers_and_unknown_custom_terms_remain_valid() {
    let body = r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251, SO:0000987, SO:0000985, <https://example.org/custom-type> .
"#;

    let document = sbol::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
    let report = document.validate();

    assert!(
        report.is_valid(),
        "known modifiers and unknown custom terms produced errors: {:?}",
        report.issues()
    );
    assert_no_rule(&report, "sbol3-10602");
    assert_no_rule(&report, "sbol3-10605");
    assert_no_rule(&report, "sbol3-10607");
    assert_no_rule(&report, "sbol3-10608");
}

#[test]
fn ontology_known_feature_interaction_and_participation_terms_remain_valid() {
    let body = r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasInteraction <component/interaction>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition;
    sbol:role SO:0000167;
    sbol:roleIntegration sbol:mergeRoles .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000176 .
<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:participant <component/feature>;
    sbol:role SBO:0000010 .
"#;

    let document = sbol::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
    let report = document.validate();

    assert!(
        report.is_valid(),
        "known ontology role/type terms produced errors: {:?}",
        report.issues()
    );
    for rule in [
        "sbol3-10701",
        "sbol3-11801",
        "sbol3-11802",
        "sbol3-11803",
        "sbol3-11804",
        "sbol3-11904",
        "sbol3-11905",
        "sbol3-11906",
    ] {
        assert_no_rule(&report, rule);
    }
}

#[test]
fn ontology_component_role_type_valid_cases_do_not_emit_role_warnings_or_errors() {
    let body = r#":dna_component a sbol:Component;
    sbol:displayId "dna_component";
    sbol:hasNamespace <https://example.org>;
    sbol:role SO:0000167;
    sbol:type SBO:0000251 .
:rna_component a sbol:Component;
    sbol:displayId "rna_component";
    sbol:hasNamespace <https://example.org>;
    sbol:role SO:0000139;
    sbol:type SBO:0000250 .
:protein_component a sbol:Component;
    sbol:displayId "protein_component";
    sbol:hasNamespace <https://example.org>;
    sbol:role GO:0001216;
    sbol:type SBO:0000252 .
:chemical_component a sbol:Component;
    sbol:displayId "chemical_component";
    sbol:hasNamespace <https://example.org>;
    sbol:role CHEBI:35224;
    sbol:type SBO:0000247 .
:dna_feature a sbol:LocalSubComponent;
    sbol:displayId "dna_feature";
    sbol:role SO:0000139;
    sbol:type SBO:0000251 .
:rna_feature a sbol:LocalSubComponent;
    sbol:displayId "rna_feature";
    sbol:role SO:0000167;
    sbol:type SBO:0000250 .
:protein_feature a sbol:LocalSubComponent;
    sbol:displayId "protein_feature";
    sbol:role GO:0003700;
    sbol:type SBO:0000252 .
"#;

    let document = sbol::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
    let report = document.validate();

    assert!(
        report.is_valid(),
        "known role/type compatible cases produced errors: {:?}",
        report.issues()
    );
    for rule in [
        "sbol3-10609",
        "sbol3-10610",
        "sbol3-10612",
        "sbol3-10613",
        "sbol3-11009",
        "sbol3-11011",
        "sbol3-11012",
    ] {
        assert_no_rule(&report, rule);
    }
}

#[test]
fn ontology_generated_branch_role_type_incompatibilities_are_reported() {
    let body = r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:role GO:0001216;
    sbol:type SBO:0000251 .
:feature a sbol:LocalSubComponent;
    sbol:displayId "feature";
    sbol:role CHEBI:35224;
    sbol:type SBO:0000252 .
"#;

    let document = sbol::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
    let report = document.validate();

    assert_rule(&report, "sbol3-10609");
    assert_rule(&report, "sbol3-11009");
}

#[test]
fn ontology_custom_roles_and_external_definition_resources_remain_undecided_when_unknown() {
    let body = r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:role <https://example.org/custom-role>;
    sbol:type SBO:0000251 .
:feature a sbol:ExternallyDefined;
    sbol:definition <https://example.org/external-protein>;
    sbol:displayId "feature";
    sbol:type SBO:0000252 .
"#;

    let document = sbol::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
    let report = document.validate();

    for rule in [
        "sbol3-10609",
        "sbol3-10610",
        "sbol3-10612",
        "sbol3-10613",
        "sbol3-11109",
    ] {
        assert_no_rule(&report, rule);
    }
}

#[test]
fn externally_defined_recommended_resources_do_not_emit_11109() {
    let body = r#":protein a sbol:ExternallyDefined;
    sbol:definition <https://identifiers.org/uniprot:P03023>;
    sbol:displayId "protein";
    sbol:type SBO:0000252 .
:chebi a sbol:ExternallyDefined;
    sbol:definition CHEBI:35224;
    sbol:displayId "chebi";
    sbol:type SBO:0000247 .
:pubchem a sbol:ExternallyDefined;
    sbol:definition <https://identifiers.org/pubchem.compound:1234>;
    sbol:displayId "pubchem";
    sbol:type SBO:0000247 .
"#;

    let document = sbol::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
    let report = document.validate();

    assert_no_rule(&report, "sbol3-11109");
}

#[test]
fn topology_completeness_policy_is_opt_in() {
    let body = r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/local>, <component/external>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/local> a sbol:LocalSubComponent;
    sbol:displayId "local";
    sbol:type SBO:0000251 .
<component/external> a sbol:ExternallyDefined;
    sbol:definition <https://identifiers.org/uniprot:P03023>;
    sbol:displayId "external";
    sbol:type SBO:0000251 .
"#;

    let document = sbol::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
    let default_report = document.validate();
    assert_no_rule(&default_report, "sbol3-10606");
    assert_no_rule(&default_report, "sbol3-11006");
    assert_no_rule(&default_report, "sbol3-11106");

    let mut options = sbol::ValidationOptions::default();
    options.topology_completeness = sbol::TopologyCompleteness::RequireKnownForNucleicAcids;
    let complete_report = document.validate_with(options);
    assert_rule(&complete_report, "sbol3-10606");
    assert_rule(&complete_report, "sbol3-11006");
    assert_rule(&complete_report, "sbol3-11106");
}

#[test]
fn explicit_topology_satisfies_topology_complete_validation() {
    let body = r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251, SO:0000987 .
"#;

    let document = sbol::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
    let mut options = sbol::ValidationOptions::default();
    options.topology_completeness = sbol::TopologyCompleteness::RequireKnownForNucleicAcids;
    let report = document.validate_with(options);

    assert_no_rule(&report, "sbol3-10606");
}

#[test]
fn top_level_namespace_must_be_namespace_iri() {
    for namespace in [
        r#"<mailto:namespace@example.org>"#,
        r#""https://example.org""#,
        r#"[]"#,
    ] {
        let body = format!(
            r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace {namespace};
    sbol:type SBO:0000251 .
"#
        );
        let document = sbol::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
        assert_rule(&document.validate(), "sbol3-10111");
    }
}

#[test]
fn urn_namespace_remains_valid_for_urn_identity_fixtures() {
    let body = r#"<urn:uuid:fb2eb360-351f-4cac-b7a2-decc65464f6a> a sbol:Component;
    sbol:hasNamespace <urn:uuid:1d856b96-89d5-4d63-9683-a90caf03ea80>;
    sbol:type SBO:0000252 .
"#;

    let document = sbol::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
    assert_no_rule(&document.validate(), "sbol3-10111");
}

#[test]
fn direct_orientation_constraints_valid_cases_do_not_emit_11705() {
    let body = r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/same_constraint>, <component/opposite_constraint>;
    sbol:hasFeature <component/subject>, <component/object_inline>, <component/object_reverse>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/subject> a sbol:LocalSubComponent;
    sbol:displayId "subject";
    sbol:orientation sbol:inline;
    sbol:type SBO:0000251 .
<component/object_inline> a sbol:LocalSubComponent;
    sbol:displayId "object_inline";
    sbol:orientation sbol:inline;
    sbol:type SBO:0000251 .
<component/object_reverse> a sbol:LocalSubComponent;
    sbol:displayId "object_reverse";
    sbol:orientation sbol:reverseComplement;
    sbol:type SBO:0000251 .
<component/same_constraint> a sbol:Constraint;
    sbol:displayId "same_constraint";
    sbol:object <component/object_inline>;
    sbol:restriction sbol:sameOrientationAs;
    sbol:subject <component/subject> .
<component/opposite_constraint> a sbol:Constraint;
    sbol:displayId "opposite_constraint";
    sbol:object <component/object_reverse>;
    sbol:restriction sbol:oppositeOrientationAs;
    sbol:subject <component/subject> .
"#;

    let document = sbol::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
    let report = document.validate();

    assert_no_rule(&report, "sbol3-11705");
}

#[test]
fn matching_om_unit_and_prefix_strings_do_not_emit_consistency_warnings() {
    let body = r#":meter a om:Unit;
    sbol:description "unit of length";
    sbol:displayId "meter";
    sbol:hasNamespace <https://example.org>;
    sbol:name "meter";
    om:comment "unit of length";
    om:label "meter";
    om:symbol "m" .
:kilo a om:Prefix;
    sbol:description "thousand";
    sbol:displayId "kilo";
    sbol:hasNamespace <https://example.org>;
    sbol:name "kilo";
    om:comment "thousand";
    om:hasFactor "1000";
    om:label "kilo";
    om:symbol "k" .
"#;

    let document = sbol::Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap();
    let report = document.validate();

    assert_no_rule(&report, "sbol3-13501");
    assert_no_rule(&report, "sbol3-13502");
    assert_no_rule(&report, "sbol3-14201");
    assert_no_rule(&report, "sbol3-14202");
}

#[test]
fn every_non_implemented_rule_has_blocker() {
    let mut violations = Vec::new();
    for status in sbol::validation_rule_statuses() {
        // Error and Warning are unconditional; every other status carries
        // a blocker that names the axis (Configurable), the spec context
        // (MachineUncheckable), or the missing work (Unimplemented).
        let needs_blocker = !matches!(
            status.status,
            sbol::RuleStatus::Error | sbol::RuleStatus::Warning
        );
        match (needs_blocker, status.blocker) {
            (true, None) => violations.push(format!(
                "{}: status {:?} but blocker is None",
                status.rule, status.status
            )),
            (false, Some(b)) => violations.push(format!(
                "{}: status {:?} but blocker is Some({:?})",
                status.rule, status.status, b
            )),
            _ => {}
        }
    }
    assert!(
        violations.is_empty(),
        "blocker invariant violated:\n  {}",
        violations.join("\n  ")
    );
}

#[test]
fn implemented_rule_severity_matches_status() {
    let mut violations = Vec::new();
    for status in sbol::validation_rule_statuses() {
        let expected = match status.status {
            sbol::RuleStatus::Error => Some(sbol::NormativeSeverity::Must),
            sbol::RuleStatus::Warning => Some(sbol::NormativeSeverity::Should),
            _ => None,
        };
        if let Some(want) = expected
            && status.normative_severity != want
        {
            violations.push(format!(
                "{}: status {:?} should pair with severity {:?}, found {:?}",
                status.rule, status.status, want, status.normative_severity
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "implemented-rule severity mismatch:\n  {}",
        violations.join("\n  ")
    );
}

#[test]
fn implemented_rule_ids_appear_in_validator_function() {
    use std::fs;
    use std::path::Path;

    let rules_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/validation/rules");
    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    let rules_files: Vec<(String, String)> = fs::read_dir(&rules_dir)
        .expect("validation/rules dir exists")
        .filter_map(Result::ok)
        .filter(|e| {
            let name = e.file_name();
            let n = name.to_string_lossy().to_string();
            n.ends_with(".rs") && n != "mod.rs"
        })
        .map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let body = fs::read_to_string(e.path()).expect("read validation/rules file");
            (name, body)
        })
        .collect();

    // Pre-scan every *.rs under src/ for rule-id literals so a
    // data-table dispatch (e.g. sbol3-10110 in spec.rs) still satisfies
    // the "rule id reachable from code" check.
    let mut src_files: Vec<String> = Vec::new();
    fn collect_rs(dir: &Path, out: &mut Vec<String>) {
        for entry in fs::read_dir(dir).expect("walk src").flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(fs::read_to_string(&path).expect("read rs"));
            }
        }
    }
    collect_rs(&src_root, &mut src_files);

    let mut violations = Vec::new();
    for status in sbol::validation_rule_statuses() {
        let Some(fn_name) = status.validator_function else {
            continue;
        };
        let fn_signature = format!("fn {fn_name}(");
        let has_fn = rules_files
            .iter()
            .any(|(_, body)| body.contains(&fn_signature));
        if !has_fn {
            violations.push(format!(
                "{}: validator_function `{fn_name}` not found under crates/sbol/src/validation/rules/",
                status.rule
            ));
        }
        let literal = format!("\"{}\"", status.rule);
        let has_id = src_files.iter().any(|body| body.contains(&literal));
        if !has_id {
            violations.push(format!(
                "{}: rule-id literal `{literal}` not found anywhere under crates/sbol/src/",
                status.rule
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "validator-function/rule-id drift:\n  {}",
        violations.join("\n  ")
    );
}
