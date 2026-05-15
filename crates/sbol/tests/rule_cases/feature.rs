//! `sbol3-109xx`, `sbol3-110xx`, `sbol3-111xx`, `sbol3-112xx`,
//! `sbol3-113xx` — ComponentReference containment, LocalSubComponent
//! types/roles, ExternallyDefined types, SequenceFeature overlapping
//! locations, and Location sequence membership rules.

use super::{RuleCase, component_role_type_body, overlapping_location_body};
use sbol::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "ComponentReference inChildOf outside parent Component",
            rule: "sbol3-10901",
            severity: Error,
            body: component_reference_body("in_child_outside_parent"),
        },
        RuleCase {
            name: "nested ComponentReference inChildOf outside referenced Component",
            rule: "sbol3-10902",
            severity: Error,
            body: component_reference_body("nested_in_child_outside_referenced_component"),
        },
        RuleCase {
            name: "ComponentReference refersTo invalid ComponentReference",
            rule: "sbol3-10903",
            severity: Error,
            body: component_reference_body("refers_to_bad_component_reference"),
        },
        RuleCase {
            name: "ComponentReference refersTo Feature outside referenced Component",
            rule: "sbol3-10904",
            severity: Error,
            body: component_reference_body("refers_to_feature_outside_referenced_component"),
        },
        RuleCase {
            name: "LocalSubComponent has multiple Table 2 type values",
            rule: "sbol3-11001",
            severity: Error,
            body: r#":feature a sbol:LocalSubComponent;
    sbol:displayId "feature";
    sbol:type SBO:0000251, SBO:0000250 .
"#,
        },
        RuleCase {
            name: "LocalSubComponent type is known but not a Component type term",
            rule: "sbol3-11002",
            severity: Error,
            body: r#":feature a sbol:LocalSubComponent;
    sbol:displayId "feature";
    sbol:type SBO:0000176 .
"#,
        },
        RuleCase {
            name: "LocalSubComponent type terms conflict",
            rule: "sbol3-11005",
            severity: Error,
            body: r#":feature a sbol:LocalSubComponent;
    sbol:displayId "feature";
    sbol:type SBO:0000251, SBO:0000252 .
"#,
        },
        RuleCase {
            name: "LocalSubComponent type is a known physical-entity SBO term outside Table 2",
            rule: "sbol3-11004",
            severity: Warning,
            body: r#":feature a sbol:LocalSubComponent;
    sbol:displayId "feature";
    sbol:type SBO:0000240 .
"#,
        },
        RuleCase {
            name: "LocalSubComponent has more than one known topology type",
            rule: "sbol3-11007",
            severity: Warning,
            body: r#":feature a sbol:LocalSubComponent;
    sbol:displayId "feature";
    sbol:type SBO:0000251, SO:0000987, SO:0000988 .
"#,
        },
        RuleCase {
            name: "LocalSubComponent has strand type without DNA or RNA type",
            rule: "sbol3-11008",
            severity: Warning,
            body: r#":feature a sbol:LocalSubComponent;
    sbol:displayId "feature";
    sbol:type SBO:0000252, SO:0000984 .
"#,
        },
        RuleCase {
            name: "LocalSubComponent role incompatible with type",
            rule: "sbol3-11009",
            severity: Error,
            body: component_role_type_body("local_subcomponent_incompatible"),
        },
        RuleCase {
            name: "LocalSubComponent SO sequence feature role without DNA or RNA type",
            rule: "sbol3-11011",
            severity: Warning,
            body: component_role_type_body(
                "local_subcomponent_sequence_feature_without_nucleic_acid",
            ),
        },
        RuleCase {
            name: "DNA LocalSubComponent lacks exactly one known SO sequence feature role",
            rule: "sbol3-11012",
            severity: Warning,
            body: component_role_type_body("local_subcomponent_dna_missing_sequence_feature_role"),
        },
        RuleCase {
            name: "LocalSubComponent overlapping locations",
            rule: "sbol3-11013",
            severity: Error,
            body: overlapping_location_body("sbol:LocalSubComponent", "sbol:type SBO:0000251;"),
        },
        RuleCase {
            name: "ExternallyDefined has multiple Table 2 type values",
            rule: "sbol3-11101",
            severity: Error,
            body: r#":feature a sbol:ExternallyDefined;
    sbol:definition <https://identifiers.org/uniprot:P03023>;
    sbol:displayId "feature";
    sbol:type SBO:0000251, SBO:0000250 .
"#,
        },
        RuleCase {
            name: "ExternallyDefined type is known but not a Component type term",
            rule: "sbol3-11102",
            severity: Error,
            body: r#":feature a sbol:ExternallyDefined;
    sbol:definition <https://example.org/external>;
    sbol:displayId "feature";
    sbol:type SBO:0000176 .
"#,
        },
        RuleCase {
            name: "ExternallyDefined type terms conflict",
            rule: "sbol3-11105",
            severity: Error,
            body: r#":feature a sbol:ExternallyDefined;
    sbol:definition <https://example.org/external>;
    sbol:displayId "feature";
    sbol:type SBO:0000251, SBO:0000252 .
"#,
        },
        RuleCase {
            name: "ExternallyDefined type is a known physical-entity SBO term outside Table 2",
            rule: "sbol3-11104",
            severity: Warning,
            body: r#":feature a sbol:ExternallyDefined;
    sbol:definition <https://example.org/external>;
    sbol:displayId "feature";
    sbol:type SBO:0000240 .
"#,
        },
        RuleCase {
            name: "ExternallyDefined has more than one known topology type",
            rule: "sbol3-11107",
            severity: Warning,
            body: r#":feature a sbol:ExternallyDefined;
    sbol:definition <https://example.org/external>;
    sbol:displayId "feature";
    sbol:type SBO:0000251, SO:0000987, SO:0000988 .
"#,
        },
        RuleCase {
            name: "ExternallyDefined has topology type without DNA or RNA type",
            rule: "sbol3-11108",
            severity: Warning,
            body: r#":feature a sbol:ExternallyDefined;
    sbol:definition <https://example.org/external>;
    sbol:displayId "feature";
    sbol:type SBO:0000252, SO:0000988 .
"#,
        },
        RuleCase {
            name: "ExternallyDefined protein definition uses simple chemical resource",
            rule: "sbol3-11109",
            severity: Warning,
            body: r#":feature a sbol:ExternallyDefined;
    sbol:definition CHEBI:35224;
    sbol:displayId "feature";
    sbol:type SBO:0000252 .
"#,
        },
        RuleCase {
            name: "SequenceFeature overlapping locations",
            rule: "sbol3-11201",
            severity: Error,
            body: overlapping_location_body("sbol:SequenceFeature", ""),
        },
        RuleCase {
            name: "invalid Location orientation",
            rule: "sbol3-11301",
            severity: Error,
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:range a sbol:Range;
    sbol:displayId "range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:orientation <https://example.org/bad-orientation>;
    sbol:start "1" .
"#,
        },
        RuleCase {
            name: "Location orientation sbol:none is not in Table 5 or Table 6",
            rule: "sbol3-11301",
            severity: Error,
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:range a sbol:Range;
    sbol:displayId "range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:orientation sbol:none;
    sbol:start "1" .
"#,
        },
        RuleCase {
            name: "Feature Location sequence outside parent Component",
            rule: "sbol3-11302",
            severity: Error,
            body: location_sequence_membership_body("has_location"),
        },
        RuleCase {
            name: "SubComponent sourceLocation sequence outside instanceOf Component",
            rule: "sbol3-11303",
            severity: Error,
            body: location_sequence_membership_body("source_location"),
        },
    ]
}

fn component_reference_body(kind: &'static str) -> &'static str {
    match kind {
        "in_child_outside_parent" => {
            r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasFeature <definition/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<definition/feature> a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <definition/feature/location> .
<definition/feature/location> a sbol:EntireSequence;
    sbol:displayId "location";
    sbol:hasSequence :sequence .
:sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/reference>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:outside_subcomponent a sbol:SubComponent;
    sbol:displayId "outside_subcomponent";
    sbol:instanceOf :definition .
<component/reference> a sbol:ComponentReference;
    sbol:displayId "reference";
    sbol:inChildOf :outside_subcomponent;
    sbol:refersTo <definition/feature> .
"#
        }
        "nested_in_child_outside_referenced_component" => {
            r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasFeature <definition/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<definition/feature> a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <definition/feature/location> .
<definition/feature/location> a sbol:EntireSequence;
    sbol:displayId "location";
    sbol:hasSequence :sequence .
:sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/subcomponent>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/subcomponent> a sbol:SubComponent;
    sbol:displayId "subcomponent";
    sbol:instanceOf :definition .
<component/reference> a sbol:ComponentReference;
    sbol:displayId "reference";
    sbol:inChildOf <component/subcomponent>;
    sbol:refersTo <definition/feature> .
:outside_subcomponent a sbol:SubComponent;
    sbol:displayId "outside_subcomponent";
    sbol:instanceOf :definition .
<component/reference/child> a sbol:ComponentReference;
    sbol:displayId "child";
    sbol:inChildOf :outside_subcomponent;
    sbol:refersTo <definition/feature> .
"#
        }
        "refers_to_bad_component_reference" => {
            r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasFeature <definition/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<definition/feature> a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <definition/feature/location> .
<definition/feature/location> a sbol:EntireSequence;
    sbol:displayId "location";
    sbol:hasSequence :sequence .
:sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/subcomponent>, <component/reference>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/subcomponent> a sbol:SubComponent;
    sbol:displayId "subcomponent";
    sbol:instanceOf :definition .
<component/reference> a sbol:ComponentReference;
    sbol:displayId "reference";
    sbol:inChildOf <component/subcomponent>;
    sbol:refersTo :outside_reference .
:outside_reference a sbol:ComponentReference;
    sbol:displayId "outside_reference";
    sbol:inChildOf <component/subcomponent>;
    sbol:refersTo <definition/feature> .
"#
        }
        "refers_to_feature_outside_referenced_component" => {
            r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasFeature <definition/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<definition/feature> a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <definition/feature/location> .
<definition/feature/location> a sbol:EntireSequence;
    sbol:displayId "location";
    sbol:hasSequence :sequence .
:sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/subcomponent>, <component/reference>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/subcomponent> a sbol:SubComponent;
    sbol:displayId "subcomponent";
    sbol:instanceOf :definition .
:outside_feature a sbol:SequenceFeature;
    sbol:displayId "outside_feature";
    sbol:hasLocation <outside_feature/location> .
<outside_feature/location> a sbol:EntireSequence;
    sbol:displayId "location";
    sbol:hasSequence :sequence .
<component/reference> a sbol:ComponentReference;
    sbol:displayId "reference";
    sbol:inChildOf <component/subcomponent>;
    sbol:refersTo :outside_feature .
"#
        }
        _ => unreachable!(),
    }
}

fn location_sequence_membership_body(kind: &'static str) -> &'static str {
    match kind {
        "has_location" => {
            r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <component/feature/range> .
<component/feature/range> a sbol:Range;
    sbol:displayId "range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "1" .
"#
        }
        "source_location" => {
            r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:definition a sbol:Component;
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
    sbol:instanceOf :definition;
    sbol:sourceLocation <component/feature/source_range> .
<component/feature/source_range> a sbol:Range;
    sbol:displayId "source_range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "1" .
"#
        }
        _ => unreachable!(),
    }
}
