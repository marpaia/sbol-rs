//! `sbol3-118xx`, `sbol3-119xx`, `sbol3-120xx` — Interaction types,
//! Participation roles, and Interface input/output/nondirectional
//! containment.

use super::{PositiveCase, RuleCase};
use sbol::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "Interaction type is known but not an Interaction type term",
            rule: "sbol3-11801",
            severity: Error,
            body: interaction_body("type_not_interaction"),
        },
        RuleCase {
            name: "Interaction type terms conflict",
            rule: "sbol3-11802",
            severity: Error,
            body: interaction_body("conflicting_types"),
        },
        RuleCase {
            name: "Interaction has more than one known Table 11 type",
            rule: "sbol3-11803",
            severity: Warning,
            body: interaction_body("multiple_table_11_types"),
        },
        RuleCase {
            name: "Interaction participation role is not cross-listed",
            rule: "sbol3-11804",
            severity: Warning,
            body: interaction_body("participation_role_not_cross_listed"),
        },
        RuleCase {
            name: "Participation missing participant kind",
            rule: "sbol3-11901",
            severity: Error,
            body: participation_body(""),
        },
        RuleCase {
            name: "Participation participant outside component",
            rule: "sbol3-11902",
            severity: Error,
            body: participation_body("sbol:participant :outside_feature;"),
        },
        RuleCase {
            name: "Participation higherOrderParticipant outside component",
            rule: "sbol3-11903",
            severity: Error,
            body: participation_body("sbol:higherOrderParticipant :outside_interaction;"),
        },
        RuleCase {
            name: "Participation role is known but not a Participation role term",
            rule: "sbol3-11904",
            severity: Error,
            body: interaction_body("participation_role_not_participation"),
        },
        RuleCase {
            name: "Participation role terms conflict",
            rule: "sbol3-11905",
            severity: Error,
            body: interaction_body("participation_conflicting_roles"),
        },
        RuleCase {
            name: "Participation has more than one known Table 12 role",
            rule: "sbol3-11906",
            severity: Warning,
            body: interaction_body("participation_multiple_table_12_roles"),
        },
        RuleCase {
            name: "Interface input outside component",
            rule: "sbol3-12001",
            severity: Error,
            body: interface_body("sbol:input"),
        },
        RuleCase {
            name: "Interface output outside component",
            rule: "sbol3-12002",
            severity: Error,
            body: interface_body("sbol:output"),
        },
        RuleCase {
            name: "Interface nondirectional outside component",
            rule: "sbol3-12003",
            severity: Error,
            body: interface_body("sbol:nondirectional"),
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "Interaction with exactly one known Table 11 type",
            rule: "sbol3-11803",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasInteraction <component/interaction>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:type SBO:0000176 .
"#,
        },
        PositiveCase {
            name: "Participation role cross-listed with the Interaction type",
            rule: "sbol3-11804",
            body: r#":definition a sbol:Component;
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
    sbol:instanceOf :definition .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000176 .
<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:participant <component/feature>;
    sbol:role SBO:0000010 .
"#,
        },
        PositiveCase {
            name: "Participation with a participant kind",
            rule: "sbol3-11901",
            body: r#":definition a sbol:Component;
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
    sbol:instanceOf :definition .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000176 .
<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:participant <component/feature>;
    sbol:role SBO:0000010 .
"#,
        },
        PositiveCase {
            name: "Participation participant is a Feature of the containing Component",
            rule: "sbol3-11902",
            body: r#":definition a sbol:Component;
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
    sbol:instanceOf :definition .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000176 .
<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:participant <component/feature>;
    sbol:role SBO:0000010 .
"#,
        },
        PositiveCase {
            name: "Participation higherOrderParticipant is an Interaction of the containing Component",
            rule: "sbol3-11903",
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasInteraction <component/interaction>, <component/inner>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition .
<component/inner> a sbol:Interaction;
    sbol:displayId "inner";
    sbol:type SBO:0000176 .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000176 .
<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:higherOrderParticipant <component/inner>;
    sbol:role SBO:0000010 .
"#,
        },
        PositiveCase {
            name: "Participation with exactly one known Table 12 role",
            rule: "sbol3-11906",
            body: r#":definition a sbol:Component;
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
    sbol:instanceOf :definition .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000176 .
<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:participant <component/feature>;
    sbol:role SBO:0000010 .
"#,
        },
        PositiveCase {
            name: "Interface input is a Feature of the containing Component",
            rule: "sbol3-12001",
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasInterface <component/interface>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition .
<component/interface> a sbol:Interface;
    sbol:displayId "interface";
    sbol:input <component/feature> .
"#,
        },
        PositiveCase {
            name: "Interface output is a Feature of the containing Component",
            rule: "sbol3-12002",
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasInterface <component/interface>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition .
<component/interface> a sbol:Interface;
    sbol:displayId "interface";
    sbol:output <component/feature> .
"#,
        },
        PositiveCase {
            name: "Interface nondirectional is a Feature of the containing Component",
            rule: "sbol3-12003",
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasInterface <component/interface>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:SubComponent;
    sbol:displayId "feature";
    sbol:instanceOf :definition .
<component/interface> a sbol:Interface;
    sbol:displayId "interface";
    sbol:nondirectional <component/feature> .
"#,
        },
    ]
}

fn interaction_body(kind: &'static str) -> &'static str {
    match kind {
        "type_not_interaction" => {
            r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasInteraction <component/interaction>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:type SBO:0000251 .
"#
        }
        "conflicting_types" => {
            r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasInteraction <component/interaction>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:type SBO:0000169, SBO:0000170 .
"#
        }
        "multiple_table_11_types" => {
            r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasInteraction <component/interaction>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:type SBO:0000176, SBO:0000177 .
"#
        }
        "participation_role_not_cross_listed" => {
            r#":definition a sbol:Component;
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
    sbol:instanceOf :definition .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000169 .
<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:participant <component/feature>;
    sbol:role SBO:0000459 .
"#
        }
        "participation_role_not_participation" => {
            r#":definition a sbol:Component;
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
    sbol:instanceOf :definition .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000176 .
<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:participant <component/feature>;
    sbol:role SO:0000167 .
"#
        }
        "participation_conflicting_roles" => {
            r#":definition a sbol:Component;
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
    sbol:instanceOf :definition .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000168 .
<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:participant <component/feature>;
    sbol:role SBO:0000020, SBO:0000459 .
"#
        }
        "participation_multiple_table_12_roles" => {
            r#":definition a sbol:Component;
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
    sbol:instanceOf :definition .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000176 .
<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:participant <component/feature>;
    sbol:role SBO:0000010, SBO:0000011 .
"#
        }
        _ => unreachable!(),
    }
}

fn participation_body(participant_property: &'static str) -> &'static str {
    match participant_property {
        "" => {
            r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasInteraction <component/interaction>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000176 .
<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:role SBO:0000000 .
"#
        }
        "sbol:participant :outside_feature;" => {
            r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:outside_feature a sbol:SubComponent;
    sbol:displayId "outside_feature";
    sbol:instanceOf :definition .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasInteraction <component/interaction>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000176 .
<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:participant :outside_feature;
    sbol:role SBO:0000000 .
"#
        }
        "sbol:higherOrderParticipant :outside_interaction;" => {
            r#":outside_interaction a sbol:Interaction;
    sbol:displayId "outside_interaction";
    sbol:type SBO:0000176 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasInteraction <component/interaction>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000176 .
<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:higherOrderParticipant :outside_interaction;
    sbol:role SBO:0000000 .
"#
        }
        _ => unreachable!(),
    }
}

fn interface_body(predicate: &'static str) -> &'static str {
    match predicate {
        "sbol:input" => {
            r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:outside_feature a sbol:SubComponent;
    sbol:displayId "outside_feature";
    sbol:instanceOf :definition .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasInterface <component/interface>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/interface> a sbol:Interface;
    sbol:displayId "interface";
    sbol:input :outside_feature .
"#
        }
        "sbol:output" => {
            r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:outside_feature a sbol:SubComponent;
    sbol:displayId "outside_feature";
    sbol:instanceOf :definition .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasInterface <component/interface>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/interface> a sbol:Interface;
    sbol:displayId "interface";
    sbol:output :outside_feature .
"#
        }
        "sbol:nondirectional" => {
            r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:outside_feature a sbol:SubComponent;
    sbol:displayId "outside_feature";
    sbol:instanceOf :definition .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasInterface <component/interface>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/interface> a sbol:Interface;
    sbol:displayId "interface";
    sbol:nondirectional :outside_feature .
"#
        }
        _ => unreachable!(),
    }
}
