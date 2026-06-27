//! `sbol3-117xx` — Constraint containment, restriction vocabulary,
//! orientation and sequence relation checks.

use super::{PositiveCase, RuleCase};
use sbol::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "Constraint subject outside component",
            rule: "sbol3-11701",
            severity: Error,
            body: constraint_body(":outside_feature", "<component/inside_feature>"),
        },
        RuleCase {
            name: "Constraint object outside component",
            rule: "sbol3-11702",
            severity: Error,
            body: constraint_body("<component/inside_feature>", ":outside_feature"),
        },
        RuleCase {
            name: "Constraint subject and object identical",
            rule: "sbol3-11703",
            severity: Error,
            body: constraint_body("<component/inside_feature>", "<component/inside_feature>"),
        },
        RuleCase {
            name: "Constraint restriction outside recommended tables",
            rule: "sbol3-11704",
            severity: Warning,
            body: constraint_unknown_restriction_body(),
        },
        RuleCase {
            name: "Constraint orientation relation contradicted",
            rule: "sbol3-11705",
            severity: Error,
            body: constraint_orientation_relation_body(),
        },
        RuleCase {
            name: "Constraint sequential relation contradicted by locations",
            rule: "sbol3-11706",
            severity: Error,
            body: constraint_sequence_relation_body(),
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "subject and object in same component is valid",
            rule: "sbol3-11701",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/a>, <component/b>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/a> a sbol:LocalSubComponent;
    sbol:displayId "a";
    sbol:type SBO:0000251 .
<component/b> a sbol:LocalSubComponent;
    sbol:displayId "b";
    sbol:type SBO:0000251 .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/b>;
    sbol:restriction sbol:precedes;
    sbol:subject <component/a> .
"#,
        },
        PositiveCase {
            name: "distinct subject and object",
            rule: "sbol3-11703",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/a>, <component/b>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/a> a sbol:LocalSubComponent;
    sbol:displayId "a";
    sbol:type SBO:0000251 .
<component/b> a sbol:LocalSubComponent;
    sbol:displayId "b";
    sbol:type SBO:0000251 .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/b>;
    sbol:restriction sbol:precedes;
    sbol:subject <component/a> .
"#,
        },
        PositiveCase {
            name: "sameOrientationAs with inline-inline pair",
            rule: "sbol3-11705",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/a>, <component/b>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/a> a sbol:LocalSubComponent;
    sbol:displayId "a";
    sbol:orientation sbol:inline;
    sbol:type SBO:0000251 .
<component/b> a sbol:LocalSubComponent;
    sbol:displayId "b";
    sbol:orientation sbol:inline;
    sbol:type SBO:0000251 .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/b>;
    sbol:restriction sbol:sameOrientationAs;
    sbol:subject <component/a> .
"#,
        },
        PositiveCase {
            name: "oppositeOrientationAs with inline-reverseComplement pair",
            rule: "sbol3-11705",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/a>, <component/b>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/a> a sbol:LocalSubComponent;
    sbol:displayId "a";
    sbol:orientation sbol:inline;
    sbol:type SBO:0000251 .
<component/b> a sbol:LocalSubComponent;
    sbol:displayId "b";
    sbol:orientation sbol:reverseComplement;
    sbol:type SBO:0000251 .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/b>;
    sbol:restriction sbol:oppositeOrientationAs;
    sbol:subject <component/a> .
"#,
        },
        PositiveCase {
            name: "Table 8 restriction vocabulary is recognized",
            rule: "sbol3-11704",
            body: r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/a>, <component/b>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/a> a sbol:LocalSubComponent;
    sbol:displayId "a";
    sbol:type SBO:0000251 .
<component/b> a sbol:LocalSubComponent;
    sbol:displayId "b";
    sbol:type SBO:0000251 .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/b>;
    sbol:restriction sbol:precedes;
    sbol:subject <component/a> .
"#,
        },
        PositiveCase {
            name: "precedes restriction consistent with range positions",
            rule: "sbol3-11706",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGCATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/subject>, <component/object>;
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :sequence;
    sbol:type SBO:0000251 .
<component/subject> a sbol:SequenceFeature;
    sbol:displayId "subject";
    sbol:hasLocation <component/subject/range> .
<component/subject/range> a sbol:Range;
    sbol:displayId "range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "1" .
<component/object> a sbol:SequenceFeature;
    sbol:displayId "object";
    sbol:hasLocation <component/object/range> .
<component/object/range> a sbol:Range;
    sbol:displayId "range";
    sbol:end "4";
    sbol:hasSequence :sequence;
    sbol:start "3" .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/object>;
    sbol:restriction sbol:precedes;
    sbol:subject <component/subject> .
"#,
        },
        PositiveCase {
            name: "constraint object contained in component is valid",
            rule: "sbol3-11702",
            body: r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/subject_feature>, <component/object_feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/subject_feature> a sbol:SubComponent;
    sbol:displayId "subject_feature";
    sbol:instanceOf :definition .
<component/object_feature> a sbol:SubComponent;
    sbol:displayId "object_feature";
    sbol:instanceOf :definition .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/object_feature>;
    sbol:restriction sbol:precedes;
    sbol:subject <component/subject_feature> .
"#,
        },
    ]
}

fn constraint_body(subject: &'static str, object: &'static str) -> &'static str {
    match (subject, object) {
        (":outside_feature", "<component/inside_feature>") => {
            r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/inside_feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/inside_feature> a sbol:SubComponent;
    sbol:displayId "inside_feature";
    sbol:instanceOf :definition .
:outside_feature a sbol:SubComponent;
    sbol:displayId "outside_feature";
    sbol:instanceOf :definition .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/inside_feature>;
    sbol:restriction sbol:precedes;
    sbol:subject :outside_feature .
"#
        }
        ("<component/inside_feature>", ":outside_feature") => {
            r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/inside_feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/inside_feature> a sbol:SubComponent;
    sbol:displayId "inside_feature";
    sbol:instanceOf :definition .
:outside_feature a sbol:SubComponent;
    sbol:displayId "outside_feature";
    sbol:instanceOf :definition .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object :outside_feature;
    sbol:restriction sbol:precedes;
    sbol:subject <component/inside_feature> .
"#
        }
        ("<component/inside_feature>", "<component/inside_feature>") => {
            r#":definition a sbol:Component;
    sbol:displayId "definition";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/inside_feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/inside_feature> a sbol:SubComponent;
    sbol:displayId "inside_feature";
    sbol:instanceOf :definition .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/inside_feature>;
    sbol:restriction sbol:precedes;
    sbol:subject <component/inside_feature> .
"#
        }
        _ => unreachable!(),
    }
}

fn constraint_unknown_restriction_body() -> &'static str {
    r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/subject>, <component/object>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/subject> a sbol:SequenceFeature;
    sbol:displayId "subject" .
<component/object> a sbol:SequenceFeature;
    sbol:displayId "object" .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/object>;
    sbol:restriction <https://example.org/customRestriction>;
    sbol:subject <component/subject> .
"#
}

fn constraint_orientation_relation_body() -> &'static str {
    r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/subject>, <component/object>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/subject> a sbol:LocalSubComponent;
    sbol:displayId "subject";
    sbol:orientation sbol:inline;
    sbol:type SBO:0000251 .
<component/object> a sbol:LocalSubComponent;
    sbol:displayId "object";
    sbol:orientation sbol:reverseComplement;
    sbol:type SBO:0000251 .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/object>;
    sbol:restriction sbol:sameOrientationAs;
    sbol:subject <component/subject> .
"#
}

fn constraint_sequence_relation_body() -> &'static str {
    r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGCATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/subject>, <component/object>;
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :sequence;
    sbol:type SBO:0000251 .
<component/subject> a sbol:SequenceFeature;
    sbol:displayId "subject";
    sbol:hasLocation <component/subject/range> .
<component/subject/range> a sbol:Range;
    sbol:displayId "range";
    sbol:end "4";
    sbol:hasSequence :sequence;
    sbol:start "3" .
<component/object> a sbol:SequenceFeature;
    sbol:displayId "object";
    sbol:hasLocation <component/object/range> .
<component/object/range> a sbol:Range;
    sbol:displayId "range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "1" .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/object>;
    sbol:restriction sbol:precedes;
    sbol:subject <component/subject> .
"#
}
