//! `sbol3-134xx`, `sbol3-135xx`, `sbol3-142xx` — OM Measure type
//! recommendations and OM Unit/Prefix label/comment consistency
//! recommendations.

use super::{PositiveCase, RuleCase};
use sbol::Severity::Warning;

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "om:Measure type is a known SBO term outside the Systems Description Parameter branch",
            rule: "sbol3-13401",
            severity: Warning,
            body: r#":measure a om:Measure;
    sbol:displayId "measure";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251;
    om:hasNumericalValue 1.0;
    om:hasUnit <http://www.ontology-of-units-of-measure.org/resource/om-2/metre> .
"#,
        },
        RuleCase {
            name: "om:Unit name differs from label",
            rule: "sbol3-13501",
            severity: Warning,
            body: om_unit_string_mismatch_body("name"),
        },
        RuleCase {
            name: "om:Unit description differs from comment",
            rule: "sbol3-13502",
            severity: Warning,
            body: om_unit_string_mismatch_body("description"),
        },
        RuleCase {
            name: "om:Prefix name differs from label",
            rule: "sbol3-14201",
            severity: Warning,
            body: om_prefix_string_mismatch_body("name"),
        },
        RuleCase {
            name: "om:Prefix description differs from comment",
            rule: "sbol3-14202",
            severity: Warning,
            body: om_prefix_string_mismatch_body("description"),
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "om:Measure type is a Systems Description Parameter term",
            rule: "sbol3-13401",
            body: r#":measure a om:Measure;
    sbol:displayId "measure";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000545;
    om:hasNumericalValue 1.0;
    om:hasUnit <http://www.ontology-of-units-of-measure.org/resource/om-2/metre> .
"#,
        },
        PositiveCase {
            name: "om:Unit name matches label",
            rule: "sbol3-13501",
            body: r#":meter a om:Unit;
    sbol:displayId "meter";
    sbol:hasNamespace <https://example.org>;
    sbol:name "metre";
    om:label "metre";
    om:symbol "m" .
"#,
        },
        PositiveCase {
            name: "om:Unit description matches comment",
            rule: "sbol3-13502",
            body: r#":meter a om:Unit;
    sbol:description "unit of length";
    sbol:displayId "meter";
    sbol:hasNamespace <https://example.org>;
    om:comment "unit of length";
    om:label "meter";
    om:symbol "m" .
"#,
        },
        PositiveCase {
            name: "om:Prefix name matches label",
            rule: "sbol3-14201",
            body: r#":kilo a om:Prefix;
    sbol:displayId "kilo";
    sbol:hasNamespace <https://example.org>;
    sbol:name "kilo";
    om:hasFactor "1000";
    om:label "kilo";
    om:symbol "k" .
"#,
        },
        PositiveCase {
            name: "om:Prefix description matches comment",
            rule: "sbol3-14202",
            body: r#":kilo a om:Prefix;
    sbol:description "thousand";
    sbol:displayId "kilo";
    sbol:hasNamespace <https://example.org>;
    om:comment "thousand";
    om:hasFactor "1000";
    om:label "kilo";
    om:symbol "k" .
"#,
        },
    ]
}

fn om_unit_string_mismatch_body(kind: &'static str) -> &'static str {
    match kind {
        "name" => {
            r#":meter a om:Unit;
    sbol:displayId "meter";
    sbol:hasNamespace <https://example.org>;
    sbol:name "meter";
    om:label "metre";
    om:symbol "m" .
"#
        }
        "description" => {
            r#":meter a om:Unit;
    sbol:description "unit of length";
    sbol:displayId "meter";
    sbol:hasNamespace <https://example.org>;
    om:comment "length unit";
    om:label "meter";
    om:symbol "m" .
"#
        }
        _ => unreachable!(),
    }
}

fn om_prefix_string_mismatch_body(kind: &'static str) -> &'static str {
    match kind {
        "name" => {
            r#":kilo a om:Prefix;
    sbol:displayId "kilo";
    sbol:hasNamespace <https://example.org>;
    sbol:name "kilo";
    om:hasFactor "1000";
    om:label "k";
    om:symbol "k" .
"#
        }
        "description" => {
            r#":kilo a om:Prefix;
    sbol:description "thousand";
    sbol:displayId "kilo";
    sbol:hasNamespace <https://example.org>;
    om:comment "factor of 1000";
    om:hasFactor "1000";
    om:label "kilo";
    om:symbol "k" .
"#
        }
        _ => unreachable!(),
    }
}
