//! `sbol3-121xx`, `sbol3-122xx` — CombinatorialDerivation and
//! VariableFeature rules: strategy/cardinality vocabularies, template
//! semantics, derived-feature consistency, variant resolution, and
//! variantDerivation cycles.

use super::RuleCase;
use sbol::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "invalid CombinatorialDerivation strategy",
            rule: "sbol3-12101",
            severity: Error,
            body: derivation_body("sbol:strategy <https://example.org/bad-strategy>;", ""),
        },
        RuleCase {
            name: "enumerate strategy with unbounded cardinality",
            rule: "sbol3-12102",
            severity: Error,
            body: derivation_body(
                "sbol:strategy sbol:enumerate;",
                "sbol:cardinality sbol:oneOrMore;",
            ),
        },
        RuleCase {
            name: "duplicate VariableFeature variable",
            rule: "sbol3-12103",
            severity: Error,
            body: duplicate_variable_feature_body(),
        },
        RuleCase {
            name: "CombinatorialDerivation template has no features",
            rule: "sbol3-12104",
            severity: Warning,
            body: derivation_semantic_body("template_empty"),
        },
        RuleCase {
            name: "derived Feature does not derive from template Feature",
            rule: "sbol3-12105",
            severity: Warning,
            body: derivation_semantic_body("feature_not_derived"),
        },
        RuleCase {
            name: "derived Collection member does not derive from derivation",
            rule: "sbol3-12106",
            severity: Warning,
            body: derivation_semantic_body("collection_member_not_derived"),
        },
        RuleCase {
            name: "derived Component missing template type",
            rule: "sbol3-12107",
            severity: Warning,
            body: derivation_semantic_body("component_missing_type"),
        },
        RuleCase {
            name: "derived Component missing template role",
            rule: "sbol3-12108",
            severity: Warning,
            body: derivation_semantic_body("component_missing_role"),
        },
        RuleCase {
            name: "derived static Feature properties differ",
            rule: "sbol3-12109",
            severity: Error,
            body: derivation_semantic_body("static_feature_mismatch"),
        },
        RuleCase {
            name: "static template Feature missing derived Feature",
            rule: "sbol3-12110",
            severity: Warning,
            body: derivation_semantic_body("static_feature_missing"),
        },
        RuleCase {
            name: "variable Feature cardinality not satisfied",
            rule: "sbol3-12111",
            severity: Warning,
            body: derivation_semantic_body("variable_cardinality"),
        },
        RuleCase {
            name: "derived variable SubComponent uses disallowed variant",
            rule: "sbol3-12112",
            severity: Error,
            body: derivation_semantic_body("disallowed_variant"),
        },
        RuleCase {
            name: "derived Features violate template Constraint",
            rule: "sbol3-12113",
            severity: Error,
            body: derivation_semantic_body("derived_constraint"),
        },
        RuleCase {
            name: "derived variable Feature missing template role",
            rule: "sbol3-12114",
            severity: Warning,
            body: derivation_semantic_body("variable_role"),
        },
        RuleCase {
            name: "derived variable Feature referent missing template type",
            rule: "sbol3-12115",
            severity: Warning,
            body: derivation_semantic_body("type_referent"),
        },
        RuleCase {
            name: "invalid VariableFeature cardinality",
            rule: "sbol3-12201",
            severity: Error,
            body: derivation_body(
                "",
                "sbol:cardinality <https://example.org/bad-cardinality>;",
            ),
        },
        RuleCase {
            name: "VariableFeature variable outside template",
            rule: "sbol3-12202",
            severity: Error,
            body: variable_outside_template_body(),
        },
        RuleCase {
            name: "variantDerivation cycle",
            rule: "sbol3-12204",
            severity: Error,
            body: variant_derivation_cycle_body(),
        },
        RuleCase {
            name: "VariableFeature variantCollection contains non-Component member",
            rule: "sbol3-12203",
            severity: Error,
            body: variant_collection_member_body(),
        },
    ]
}

fn derivation_body(
    derivation_extra: &'static str,
    variable_feature_cardinality: &'static str,
) -> &'static str {
    match (derivation_extra, variable_feature_cardinality) {
        ("sbol:strategy <https://example.org/bad-strategy>;", "") => {
            r#":variant a sbol:Component;
    sbol:displayId "variant";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :variant .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:strategy <https://example.org/bad-strategy>;
    sbol:template :template .
"#
        }
        ("sbol:strategy sbol:enumerate;", "sbol:cardinality sbol:oneOrMore;") => {
            r#":variant a sbol:Component;
    sbol:displayId "variant";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :variant .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:hasVariableFeature <derivation/variable_feature>;
    sbol:strategy sbol:enumerate;
    sbol:template :template .
<derivation/variable_feature> a sbol:VariableFeature;
    sbol:cardinality sbol:oneOrMore;
    sbol:displayId "variable_feature";
    sbol:variable <template/feature>;
    sbol:variant :variant .
"#
        }
        ("", "sbol:cardinality <https://example.org/bad-cardinality>;") => {
            r#":variant a sbol:Component;
    sbol:displayId "variant";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :variant .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:hasVariableFeature <derivation/variable_feature>;
    sbol:template :template .
<derivation/variable_feature> a sbol:VariableFeature;
    sbol:cardinality <https://example.org/bad-cardinality>;
    sbol:displayId "variable_feature";
    sbol:variable <template/feature>;
    sbol:variant :variant .
"#
        }
        _ => unreachable!(),
    }
}

fn duplicate_variable_feature_body() -> &'static str {
    r#":variant a sbol:Component;
    sbol:displayId "variant";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :variant .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:hasVariableFeature <derivation/variable_feature_a>, <derivation/variable_feature_b>;
    sbol:template :template .
<derivation/variable_feature_a> a sbol:VariableFeature;
    sbol:cardinality sbol:one;
    sbol:displayId "variable_feature_a";
    sbol:variable <template/feature>;
    sbol:variant :variant .
<derivation/variable_feature_b> a sbol:VariableFeature;
    sbol:cardinality sbol:one;
    sbol:displayId "variable_feature_b";
    sbol:variable <template/feature>;
    sbol:variant :variant .
	"#
}

fn derivation_semantic_body(kind: &'static str) -> &'static str {
    match kind {
        "template_empty" => {
            r#":template a sbol:Component;
    sbol:displayId "template";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:template :template .
"#
        }
        "feature_not_derived" => {
            r#":template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/static>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/static> a sbol:LocalSubComponent;
    sbol:displayId "static";
    sbol:type SBO:0000251 .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:template :template .
:derived a sbol:Component;
    sbol:displayId "derived";
    sbol:hasFeature <derived/unmapped>;
    sbol:hasNamespace <https://example.org>;
    prov:wasDerivedFrom :derivation;
    sbol:type SBO:0000251 .
<derived/unmapped> a sbol:LocalSubComponent;
    sbol:displayId "unmapped";
    sbol:type SBO:0000251 .
"#
        }
        "collection_member_not_derived" => {
            r#":template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/static>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/static> a sbol:LocalSubComponent;
    sbol:displayId "static";
    sbol:type SBO:0000251 .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:template :template .
:collection a sbol:Collection;
    sbol:displayId "collection";
    sbol:hasNamespace <https://example.org>;
    sbol:member :member;
    prov:wasDerivedFrom :derivation .
:member a sbol:Component;
    sbol:displayId "member";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#
        }
        "component_missing_type" => {
            r#":template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/static>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/static> a sbol:LocalSubComponent;
    sbol:displayId "static";
    sbol:type SBO:0000251 .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:template :template .
:derived a sbol:Component;
    sbol:displayId "derived";
    sbol:hasNamespace <https://example.org>;
    prov:wasDerivedFrom :derivation;
    sbol:type SBO:0000252 .
"#
        }
        "component_missing_role" => {
            r#":template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/static>;
    sbol:hasNamespace <https://example.org>;
    sbol:role SO:0000704;
    sbol:type SBO:0000251 .
<template/static> a sbol:LocalSubComponent;
    sbol:displayId "static";
    sbol:type SBO:0000251 .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:template :template .
:derived a sbol:Component;
    sbol:displayId "derived";
    sbol:hasNamespace <https://example.org>;
    sbol:role SO:0000316;
    prov:wasDerivedFrom :derivation;
    sbol:type SBO:0000251 .
"#
        }
        "static_feature_mismatch" => {
            r#":template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/static>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/static> a sbol:LocalSubComponent;
    sbol:displayId "static";
    sbol:role SO:0000704;
    sbol:type SBO:0000251 .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:template :template .
:derived a sbol:Component;
    sbol:displayId "derived";
    sbol:hasFeature <derived/static>;
    sbol:hasNamespace <https://example.org>;
    prov:wasDerivedFrom :derivation;
    sbol:type SBO:0000251 .
<derived/static> a sbol:LocalSubComponent;
    sbol:displayId "static";
    sbol:role SO:0000316;
    prov:wasDerivedFrom <template/static>;
    sbol:type SBO:0000251 .
"#
        }
        "static_feature_missing" => {
            r#":template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/static>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/static> a sbol:LocalSubComponent;
    sbol:displayId "static";
    sbol:type SBO:0000251 .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:template :template .
:derived a sbol:Component;
    sbol:displayId "derived";
    sbol:hasNamespace <https://example.org>;
    prov:wasDerivedFrom :derivation;
    sbol:type SBO:0000251 .
"#
        }
        "variable_cardinality" => {
            r#":template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/slot>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/slot> a sbol:LocalSubComponent;
    sbol:displayId "slot";
    sbol:type SBO:0000251 .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:hasVariableFeature <derivation/variable_feature>;
    sbol:template :template .
<derivation/variable_feature> a sbol:VariableFeature;
    sbol:cardinality sbol:one;
    sbol:displayId "variable_feature";
    sbol:variable <template/slot> .
:derived a sbol:Component;
    sbol:displayId "derived";
    sbol:hasNamespace <https://example.org>;
    prov:wasDerivedFrom :derivation;
    sbol:type SBO:0000251 .
"#
        }
        "disallowed_variant" => {
            r#":allowed_variant a sbol:Component;
    sbol:displayId "allowed_variant";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:bad_variant a sbol:Component;
    sbol:displayId "bad_variant";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/slot>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/slot> a sbol:SubComponent;
    sbol:displayId "slot";
    sbol:instanceOf :allowed_variant .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:hasVariableFeature <derivation/variable_feature>;
    sbol:template :template .
<derivation/variable_feature> a sbol:VariableFeature;
    sbol:cardinality sbol:one;
    sbol:displayId "variable_feature";
    sbol:variable <template/slot>;
    sbol:variant :allowed_variant .
:derived a sbol:Component;
    sbol:displayId "derived";
    sbol:hasFeature <derived/slot>;
    sbol:hasNamespace <https://example.org>;
    prov:wasDerivedFrom :derivation;
    sbol:type SBO:0000251 .
<derived/slot> a sbol:SubComponent;
    sbol:displayId "slot";
    sbol:instanceOf :bad_variant;
    prov:wasDerivedFrom <template/slot> .
"#
        }
        "derived_constraint" => {
            r#":template a sbol:Component;
    sbol:displayId "template";
    sbol:hasConstraint <template/constraint>;
    sbol:hasFeature <template/a>, <template/b>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/a> a sbol:LocalSubComponent;
    sbol:displayId "a";
    sbol:orientation sbol:inline;
    sbol:type SBO:0000251 .
<template/b> a sbol:LocalSubComponent;
    sbol:displayId "b";
    sbol:orientation sbol:inline;
    sbol:type SBO:0000251 .
<template/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <template/b>;
    sbol:restriction sbol:sameOrientationAs;
    sbol:subject <template/a> .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:hasVariableFeature <derivation/variable_a>, <derivation/variable_b>;
    sbol:template :template .
<derivation/variable_a> a sbol:VariableFeature;
    sbol:cardinality sbol:one;
    sbol:displayId "variable_a";
    sbol:variable <template/a> .
<derivation/variable_b> a sbol:VariableFeature;
    sbol:cardinality sbol:one;
    sbol:displayId "variable_b";
    sbol:variable <template/b> .
:derived a sbol:Component;
    sbol:displayId "derived";
    sbol:hasFeature <derived/a>, <derived/b>;
    sbol:hasNamespace <https://example.org>;
    prov:wasDerivedFrom :derivation;
    sbol:type SBO:0000251 .
<derived/a> a sbol:LocalSubComponent;
    sbol:displayId "a";
    sbol:orientation sbol:inline;
    prov:wasDerivedFrom <template/a>;
    sbol:type SBO:0000251 .
<derived/b> a sbol:LocalSubComponent;
    sbol:displayId "b";
    sbol:orientation sbol:reverseComplement;
    prov:wasDerivedFrom <template/b>;
    sbol:type SBO:0000251 .
"#
        }
        "variable_role" => {
            r#":template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/slot>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/slot> a sbol:LocalSubComponent;
    sbol:displayId "slot";
    sbol:role SO:0000704;
    sbol:type SBO:0000251 .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:hasVariableFeature <derivation/variable_feature>;
    sbol:template :template .
<derivation/variable_feature> a sbol:VariableFeature;
    sbol:cardinality sbol:one;
    sbol:displayId "variable_feature";
    sbol:variable <template/slot> .
:derived a sbol:Component;
    sbol:displayId "derived";
    sbol:hasFeature <derived/slot>;
    sbol:hasNamespace <https://example.org>;
    prov:wasDerivedFrom :derivation;
    sbol:type SBO:0000251 .
<derived/slot> a sbol:LocalSubComponent;
    sbol:displayId "slot";
    prov:wasDerivedFrom <template/slot>;
    sbol:type SBO:0000251 .
"#
        }
        "type_referent" => {
            r#":template_variant a sbol:Component;
    sbol:displayId "template_variant";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:derived_variant a sbol:Component;
    sbol:displayId "derived_variant";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000252 .
:template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/slot>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/slot> a sbol:SubComponent;
    sbol:displayId "slot";
    sbol:instanceOf :template_variant .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:hasVariableFeature <derivation/variable_feature>;
    sbol:template :template .
<derivation/variable_feature> a sbol:VariableFeature;
    sbol:cardinality sbol:one;
    sbol:displayId "variable_feature";
    sbol:variable <template/slot>;
    sbol:variant :derived_variant .
:derived a sbol:Component;
    sbol:displayId "derived";
    sbol:hasFeature <derived/slot>;
    sbol:hasNamespace <https://example.org>;
    prov:wasDerivedFrom :derivation;
    sbol:type SBO:0000251 .
<derived/slot> a sbol:SubComponent;
    sbol:displayId "slot";
    sbol:instanceOf :derived_variant;
    prov:wasDerivedFrom <template/slot> .
"#
        }
        _ => unreachable!(),
    }
}

fn variable_outside_template_body() -> &'static str {
    r#":variant a sbol:Component;
    sbol:displayId "variant";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:outside_feature a sbol:SubComponent;
    sbol:displayId "outside_feature";
    sbol:instanceOf :variant .
:template a sbol:Component;
    sbol:displayId "template";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:hasVariableFeature <derivation/variable_feature>;
    sbol:template :template .
<derivation/variable_feature> a sbol:VariableFeature;
    sbol:cardinality sbol:one;
    sbol:displayId "variable_feature";
    sbol:variable :outside_feature;
    sbol:variant :variant .
"#
}

fn variant_derivation_cycle_body() -> &'static str {
    r#":variant a sbol:Component;
    sbol:displayId "variant";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :variant .
:derivation_a a sbol:CombinatorialDerivation;
    sbol:displayId "derivation_a";
    sbol:hasNamespace <https://example.org>;
    sbol:hasVariableFeature <derivation_a/variable_feature>;
    sbol:template :template .
<derivation_a/variable_feature> a sbol:VariableFeature;
    sbol:cardinality sbol:one;
    sbol:displayId "variable_feature";
    sbol:variable <template/feature>;
    sbol:variantDerivation :derivation_b .
:derivation_b a sbol:CombinatorialDerivation;
    sbol:displayId "derivation_b";
    sbol:hasNamespace <https://example.org>;
    sbol:hasVariableFeature <derivation_b/variable_feature>;
    sbol:template :template .
<derivation_b/variable_feature> a sbol:VariableFeature;
    sbol:cardinality sbol:one;
    sbol:displayId "variable_feature";
    sbol:variable <template/feature>;
    sbol:variantDerivation :derivation_a .
"#
}

fn variant_collection_member_body() -> &'static str {
    r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:hasNamespace <https://example.org> .
:collection a sbol:Collection;
    sbol:displayId "collection";
    sbol:hasNamespace <https://example.org>;
    sbol:member :sequence .
:variant a sbol:Component;
    sbol:displayId "variant";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:template a sbol:Component;
    sbol:displayId "template";
    sbol:hasFeature <template/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<template/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :variant .
:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:hasVariableFeature <derivation/variable_feature>;
    sbol:template :template .
<derivation/variable_feature> a sbol:VariableFeature;
    sbol:cardinality sbol:one;
    sbol:displayId "variable_feature";
    sbol:variable <template/feature>;
    sbol:variantCollection :collection .
"#
}
