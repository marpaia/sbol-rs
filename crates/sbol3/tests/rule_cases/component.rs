//! `sbol3-106xx`, `sbol3-107xx`, `sbol3-108xx` — Component types/roles,
//! Component-Sequence compatibility, SubComponent containment and
//! cycles, SubComponent location length.

use super::{PositiveCase, RuleCase, component_role_type_body, overlapping_location_body};
use sbol3::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "Component has multiple Table 2 type values",
            rule: "sbol3-10601",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251, SBO:0000250 .
"#,
        },
        RuleCase {
            name: "Component type is known but not a Component type term",
            rule: "sbol3-10602",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000176 .
"#,
        },
        RuleCase {
            name: "Component type terms conflict",
            rule: "sbol3-10605",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251, SBO:0000252 .
"#,
        },
        RuleCase {
            name: "Component type is a known SBO term outside the physical entity branch",
            rule: "sbol3-10604",
            severity: Warning,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251, SBO:0000231 .
"#,
        },
        RuleCase {
            name: "Component has more than one known topology type",
            rule: "sbol3-10607",
            severity: Warning,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251, SO:0000987, SO:0000988 .
"#,
        },
        RuleCase {
            name: "Component has topology type without DNA or RNA type",
            rule: "sbol3-10608",
            severity: Warning,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000252, SO:0000987 .
"#,
        },
        RuleCase {
            name: "Component role incompatible with type",
            rule: "sbol3-10609",
            severity: Error,
            body: component_role_type_body("component_incompatible"),
        },
        RuleCase {
            name: "Component role is known but not a Component role term",
            rule: "sbol3-10610",
            severity: Error,
            body: component_role_type_body("component_role_not_component"),
        },
        RuleCase {
            name: "Component SO sequence feature role without DNA or RNA type",
            rule: "sbol3-10612",
            severity: Warning,
            body: component_role_type_body("component_sequence_feature_without_nucleic_acid"),
        },
        RuleCase {
            name: "DNA Component lacks exactly one known SO sequence feature role",
            rule: "sbol3-10613",
            severity: Warning,
            body: component_role_type_body("component_dna_missing_sequence_feature_role"),
        },
        RuleCase {
            name: "Component type conflicts with local Sequence encoding",
            rule: "sbol3-10614",
            severity: Error,
            body: component_sequence_type_mismatch_body(),
        },
        RuleCase {
            name: "Component has conflicting local Sequence encodings",
            rule: "sbol3-10615",
            severity: Error,
            body: component_sequence_conflicting_encoding_body(),
        },
        RuleCase {
            name: "Component lacks required compatible Sequence encoding",
            rule: "sbol3-10616",
            severity: Error,
            body: component_sequence_type_mismatch_body(),
        },
        RuleCase {
            name: "Component has same-encoding Sequences with different elements",
            rule: "sbol3-10617",
            severity: Warning,
            body: component_sequence_same_encoding_different_elements_body(),
        },
        RuleCase {
            name: "Feature role is known but not a Feature role term",
            rule: "sbol3-10701",
            severity: Error,
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:feature a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition;
    sbol:role SBO:0000176;
    sbol:roleIntegration sbol:mergeRoles .
"#,
        },
        RuleCase {
            name: "invalid Feature orientation",
            rule: "sbol3-10702",
            severity: Error,
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:feature a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition;
    sbol:orientation <https://example.org/bad-orientation> .
"#,
        },
        RuleCase {
            name: "invalid SubComponent roleIntegration",
            rule: "sbol3-10801",
            severity: Error,
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:feature a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition;
    sbol:roleIntegration <https://example.org/bad-role-integration> .
"#,
        },
        RuleCase {
            name: "SubComponent role without roleIntegration",
            rule: "sbol3-10802",
            severity: Error,
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:feature a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition;
    sbol:role SO:0000167 .
"#,
        },
        RuleCase {
            name: "SubComponent instanceOf containing Component",
            rule: "sbol3-10803",
            severity: Error,
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/subcomponent>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/subcomponent> a sbol:SubComponent;
    sbol:displayId "subcomponent";
    sbol:instanceOf :component .
"#,
        },
        RuleCase {
            name: "Component instance cycle",
            rule: "sbol3-10804",
            severity: Error,
            body: r#":component_a a sbol:Component;
    sbol:displayId "component_a";
    sbol:hasFeature <component_a/component_b>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component_a/component_b> a sbol:SubComponent;
    sbol:displayId "component_b";
    sbol:instanceOf :component_b .
:component_b a sbol:Component;
    sbol:displayId "component_b";
    sbol:hasFeature <component_b/component_a>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component_b/component_a> a sbol:SubComponent;
    sbol:displayId "component_a";
    sbol:instanceOf :component_a .
"#,
        },
        RuleCase {
            name: "SubComponent overlapping locations",
            rule: "sbol3-10805",
            severity: Error,
            body: overlapping_location_body("sbol:SubComponent", "sbol:instanceOf :definition;"),
        },
        RuleCase {
            name: "SubComponent location length differs from sourceLocation length",
            rule: "sbol3-10806",
            severity: Error,
            body: subcomponent_location_length_body("source_mismatch"),
        },
        RuleCase {
            name: "SubComponent locations do not cover instanceOf sequence",
            rule: "sbol3-10807",
            severity: Error,
            body: subcomponent_location_length_body("instance_sequence_mismatch"),
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "Component with single Table 2 type",
            rule: "sbol3-10601",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "DNA Component with circular topology",
            rule: "sbol3-10607",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251, SO:0000988 .
"#,
        },
        PositiveCase {
            name: "DNA Component with linear topology",
            rule: "sbol3-10608",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251, SO:0000987 .
"#,
        },
        PositiveCase {
            name: "Component DNA type matches DNA Sequence encoding",
            rule: "sbol3-10614",
            body: r#":sequence a sbol:Sequence;
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
        },
        PositiveCase {
            name: "SubComponent referencing distinct Component is acyclic",
            rule: "sbol3-10804",
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/subcomponent>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/subcomponent> a sbol:SubComponent;
    sbol:displayId "subcomponent";
    sbol:instanceOf :definition .
"#,
        },
        PositiveCase {
            name: "Protein Component with chemical role from GO",
            rule: "sbol3-10609",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:role GO:0001216;
    sbol:type SBO:0000252 .
"#,
        },
        PositiveCase {
            name: "Component type from the SBO physical entity branch",
            rule: "sbol3-10604",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "DNA Component carrying an SO sequence feature role",
            rule: "sbol3-10612",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:role SO:0000167;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "DNA Component with exactly one SO sequence feature role",
            rule: "sbol3-10613",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:role SO:0000167;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "DNA Component with compatible DNA Sequence encoding",
            rule: "sbol3-10616",
            body: r#":dna_sequence a sbol:Sequence;
    sbol:displayId "dna_sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :dna_sequence;
    sbol:type SBO:0000251 .
"#,
        },
        PositiveCase {
            name: "Component with same-encoding Sequences sharing elements",
            rule: "sbol3-10617",
            body: r#":sequence_a a sbol:Sequence;
    sbol:displayId "sequence_a";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:sequence_b a sbol:Sequence;
    sbol:displayId "sequence_b";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :sequence_a, :sequence_b;
    sbol:type SBO:0000241 .
"#,
        },
        PositiveCase {
            name: "Feature with a supported orientation",
            rule: "sbol3-10702",
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:feature a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition;
    sbol:orientation sbol:inline .
"#,
        },
        PositiveCase {
            name: "SubComponent with a supported roleIntegration",
            rule: "sbol3-10801",
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:feature a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition;
    sbol:roleIntegration sbol:mergeRoles .
"#,
        },
        PositiveCase {
            name: "SubComponent role accompanied by roleIntegration",
            rule: "sbol3-10802",
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:feature a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition;
    sbol:role SO:0000167;
    sbol:roleIntegration sbol:mergeRoles .
"#,
        },
        PositiveCase {
            name: "SubComponent instanceOf a distinct Component",
            rule: "sbol3-10803",
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/subcomponent>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/subcomponent> a sbol:SubComponent;
    sbol:displayId "subcomponent";
    sbol:instanceOf :definition .
"#,
        },
        PositiveCase {
            name: "SubComponent with non-overlapping locations",
            rule: "sbol3-10805",
            body: r#":definition a sbol:Component;
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
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "1" .
<component/feature/range2> a sbol:Range;
    sbol:displayId "range2";
    sbol:end "4";
    sbol:hasSequence :sequence;
    sbol:start "3" .
"#,
        },
        PositiveCase {
            name: "SubComponent location length matches sourceLocation length",
            rule: "sbol3-10806",
            body: r#":definition a sbol:Component;
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
    sbol:hasLocation <component/feature/range>;
    sbol:instanceOf :definition;
    sbol:sourceLocation <component/feature/source_range> .
<component/feature/range> a sbol:Range;
    sbol:displayId "range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "1" .
<component/feature/source_range> a sbol:Range;
    sbol:displayId "source_range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "1" .
"#,
        },
        PositiveCase {
            name: "SubComponent locations cover its instanceOf sequence",
            rule: "sbol3-10807",
            body: r#":definition_sequence a sbol:Sequence;
    sbol:displayId "definition_sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :definition_sequence;
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
    sbol:hasLocation <component/feature/range>;
    sbol:instanceOf :definition .
<component/feature/range> a sbol:Range;
    sbol:displayId "range";
    sbol:end "4";
    sbol:hasSequence :sequence;
    sbol:start "1" .
"#,
        },
    ]
}

fn component_sequence_type_mismatch_body() -> &'static str {
    r#":protein_sequence a sbol:Sequence;
    sbol:displayId "protein_sequence";
    sbol:elements "MSTN";
    sbol:encoding EDAM:format_1208;
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :protein_sequence;
    sbol:type SBO:0000251 .
"#
}

fn component_sequence_conflicting_encoding_body() -> &'static str {
    r#":dna_sequence a sbol:Sequence;
    sbol:displayId "dna_sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:protein_sequence a sbol:Sequence;
    sbol:displayId "protein_sequence";
    sbol:elements "MSTN";
    sbol:encoding EDAM:format_1208;
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :dna_sequence, :protein_sequence;
    sbol:type SBO:0000241 .
"#
}

fn component_sequence_same_encoding_different_elements_body() -> &'static str {
    r#":sequence_a a sbol:Sequence;
    sbol:displayId "sequence_a";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:sequence_b a sbol:Sequence;
    sbol:displayId "sequence_b";
    sbol:elements "ATGA";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :sequence_a, :sequence_b;
    sbol:type SBO:0000241 .
"#
}

fn subcomponent_location_length_body(kind: &'static str) -> &'static str {
    match kind {
        "source_mismatch" => {
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
    sbol:hasLocation <component/feature/range>;
    sbol:instanceOf :definition;
    sbol:sourceLocation <component/feature/source_range> .
<component/feature/range> a sbol:Range;
    sbol:displayId "range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "1" .
<component/feature/source_range> a sbol:Range;
    sbol:displayId "source_range";
    sbol:end "3";
    sbol:hasSequence :sequence;
    sbol:start "1" .
"#
        }
        "instance_sequence_mismatch" => {
            r#":definition_sequence a sbol:Sequence;
    sbol:displayId "definition_sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :definition_sequence;
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
    sbol:hasLocation <component/feature/range>;
    sbol:instanceOf :definition .
<component/feature/range> a sbol:Range;
    sbol:displayId "range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "1" .
"#
        }
        _ => unreachable!(),
    }
}
