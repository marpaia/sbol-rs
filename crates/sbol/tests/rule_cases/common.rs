//! `sbol3-101xx`, `sbol3-102xx`, `sbol3-103xx` — TopLevel identity,
//! displayId, derivation, and namespace rules.

use super::{PositiveCase, RuleCase};
use sbol::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "blank node TopLevel identity",
            rule: "sbol3-10101",
            severity: Error,
            body: r#"[] a sbol:Component;
    sbol:displayId "blank_component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        RuleCase {
            name: "TopLevel URL displayId mismatch",
            rule: "sbol3-10102",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "other";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        RuleCase {
            name: "TopLevel URL prefixes another TopLevel URL",
            rule: "sbol3-10103",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/child> a sbol:Component;
    sbol:displayId "child";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        RuleCase {
            name: "child URL displayId mismatch",
            rule: "sbol3-10104",
            severity: Error,
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/not_feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/not_feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition .
"#,
        },
        RuleCase {
            name: "unknown SBOL property",
            rule: "sbol3-10105",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:notAProperty "x";
    sbol:type SBO:0000251 .
"#,
        },
        RuleCase {
            name: "disjoint SBOL rdf types",
            rule: "sbol3-10106",
            severity: Error,
            body: r#":thing a sbol:Component, sbol:Sequence;
    sbol:displayId "thing";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        RuleCase {
            name: "multiple concrete inherited SBOL rdf types",
            rule: "sbol3-10107",
            severity: Warning,
            body: r#":experiment a sbol:Experiment, sbol:Collection;
    sbol:displayId "experiment";
    sbol:hasNamespace <https://example.org> .
"#,
        },
        RuleCase {
            name: "SBOL property without SBOL rdf type",
            rule: "sbol3-10108",
            severity: Warning,
            body: r#":thing sbol:displayId "thing" .
"#,
        },
        RuleCase {
            name: "SBOL property not allowed for class",
            rule: "sbol3-10109",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:elements "ATGC";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        RuleCase {
            name: "missing required property",
            rule: "sbol3-10110",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org> .
"#,
        },
        RuleCase {
            name: "wrong value kind",
            rule: "sbol3-10111",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type "not a URI" .
"#,
        },
        RuleCase {
            name: "typed integer literal on String property",
            rule: "sbol3-10111",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId 42;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        RuleCase {
            name: "malformed integer on Range.start",
            rule: "sbol3-10111",
            severity: Error,
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <https://example.org/component/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<https://example.org/component/feature> a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <https://example.org/component/feature/range> .
<https://example.org/component/feature/range> a sbol:Range;
    sbol:displayId "range";
    sbol:start "not-a-number";
    sbol:end "10";
    sbol:hasSequence :sequence .
"#,
        },
        RuleCase {
            name: "non-numeric Attachment.size literal",
            rule: "sbol3-10111",
            severity: Error,
            body: r#":attachment a sbol:Attachment;
    sbol:displayId "attachment";
    sbol:hasNamespace <https://example.org>;
    sbol:size "twelve";
    sbol:source <https://example.org/data.bin> .
"#,
        },
        RuleCase {
            name: "malformed dateTime on prov:Activity end time",
            rule: "sbol3-10111",
            severity: Error,
            body: r#":activity a prov:Activity;
    sbol:displayId "activity";
    sbol:hasNamespace <https://example.org>;
    prov:endedAtTime "yesterday" .
"#,
        },
        RuleCase {
            name: "missing local child reference",
            rule: "sbol3-10112",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature :missing_feature;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        RuleCase {
            name: "reference target has wrong class",
            rule: "sbol3-10113",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature :sequence;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:hasNamespace <https://example.org> .
"#,
        },
        RuleCase {
            name: "invalid displayId",
            rule: "sbol3-10201",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "bad id";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        RuleCase {
            name: "self derivation",
            rule: "sbol3-10202",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251;
    prov:wasDerivedFrom :component .
"#,
        },
        RuleCase {
            name: "derivation cycle",
            rule: "sbol3-10203",
            severity: Error,
            body: r#":component_a a sbol:Component;
    sbol:displayId "component_a";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251;
    prov:wasDerivedFrom :component_b .
:component_b a sbol:Component;
    sbol:displayId "component_b";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251;
    prov:wasDerivedFrom :component_a .
"#,
        },
        RuleCase {
            name: "wasGeneratedBy provenance cycle",
            rule: "sbol3-10204",
            severity: Error,
            body: r#":component_a a sbol:Component;
    sbol:displayId "component_a";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251;
    prov:wasGeneratedBy :activity_a .
:activity_a a prov:Activity;
    sbol:displayId "activity_a";
    sbol:hasNamespace <https://example.org>;
    prov:qualifiedUsage <https://example.org/activity_a/usage_a> .
<https://example.org/activity_a/usage_a> a prov:Usage;
    sbol:displayId "usage_a";
    prov:entity :component_b .
:component_b a sbol:Component;
    sbol:displayId "component_b";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251;
    prov:wasGeneratedBy :activity_b .
:activity_b a prov:Activity;
    sbol:displayId "activity_b";
    sbol:hasNamespace <https://example.org>;
    prov:qualifiedUsage <https://example.org/activity_b/usage_b> .
<https://example.org/activity_b/usage_b> a prov:Usage;
    sbol:displayId "usage_b";
    prov:entity :component_a .
"#,
        },
        RuleCase {
            name: "TopLevel namespace mismatch",
            rule: "sbol3-10301",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://other.example.org>;
    sbol:type SBO:0000251 .
"#,
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "compliant TopLevel URL with namespace + displayId",
            rule: "sbol3-10102",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "compliant child URL under parent",
            rule: "sbol3-10104",
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition .
"#,
        },
        PositiveCase {
            name: "displayId starting with underscore is valid",
            rule: "sbol3-10201",
            body: r#":_id a sbol:Component;
    sbol:displayId "_id";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "displayId with mixed case and digits is valid",
            rule: "sbol3-10201",
            body: r#":Component_v1 a sbol:Component;
    sbol:displayId "Component_v1";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "TopLevel namespace matches URL prefix",
            rule: "sbol3-10301",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "URN namespace tolerated for legacy URN-identified subject",
            rule: "sbol3-10301",
            body: r#"<urn:uuid:fb2eb360-351f-4cac-b7a2-decc65464f6a> a sbol:Component;
    sbol:hasNamespace <urn:uuid:1d856b96-89d5-4d63-9683-a90caf03ea80>;
    sbol:type SBO:0000252 .
"#,
        },
        PositiveCase {
            name: "compliant TopLevel with named identity IRI",
            rule: "sbol3-10101",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "compliant sibling TopLevels not nested under one another",
            rule: "sbol3-10103",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:child a sbol:Component;
    sbol:displayId "child";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "compliant TopLevel with only known SBOL properties",
            rule: "sbol3-10105",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "compliant TopLevel with a single SBOL rdf type",
            rule: "sbol3-10106",
            body: r#":thing a sbol:Component;
    sbol:displayId "thing";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "compliant TopLevel with a single concrete inherited type",
            rule: "sbol3-10107",
            body: r#":experiment a sbol:Experiment;
    sbol:displayId "experiment";
    sbol:hasNamespace <https://example.org> .
"#,
        },
        PositiveCase {
            name: "compliant SBOL property carried by an SBOL rdf type",
            rule: "sbol3-10108",
            body: r#":thing a sbol:Component;
    sbol:displayId "thing";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "compliant Component without Sequence-only properties",
            rule: "sbol3-10109",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "compliant Component with all required properties present",
            rule: "sbol3-10110",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "compliant URI-valued type property",
            rule: "sbol3-10111",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "compliant local child reference that resolves",
            rule: "sbol3-10112",
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition .
"#,
        },
        PositiveCase {
            name: "compliant reference target of the expected class",
            rule: "sbol3-10113",
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition .
"#,
        },
        PositiveCase {
            name: "external TopLevel reference not flagged in offline validation",
            rule: "sbol3-10114",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf <https://other.example.org/external_definition> .
"#,
        },
        PositiveCase {
            name: "compliant TopLevel derived from a distinct entity",
            rule: "sbol3-10202",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251;
    prov:wasDerivedFrom :ancestor .
:ancestor a sbol:Component;
    sbol:displayId "ancestor";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "compliant acyclic derivation chain",
            rule: "sbol3-10203",
            body: r#":component_a a sbol:Component;
    sbol:displayId "component_a";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251;
    prov:wasDerivedFrom :component_b .
:component_b a sbol:Component;
    sbol:displayId "component_b";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "compliant acyclic wasGeneratedBy provenance",
            rule: "sbol3-10204",
            body: r#":component_a a sbol:Component;
    sbol:displayId "component_a";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251;
    prov:wasGeneratedBy :activity_a .
:activity_a a prov:Activity;
    sbol:displayId "activity_a";
    sbol:hasNamespace <https://example.org>;
    prov:qualifiedUsage <https://example.org/activity_a/usage_a> .
<https://example.org/activity_a/usage_a> a prov:Usage;
    sbol:displayId "usage_a";
    prov:entity :component_b .
:component_b a sbol:Component;
    sbol:displayId "component_b";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
    ]
}
