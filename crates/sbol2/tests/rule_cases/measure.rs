//! Measure ontology and unit-reference rules (13505, 13506).

use super::common::{PositiveCase, RuleCase};
use sbol2::validation::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "Measure with types but no systems-description-parameter SBO type",
            rule: "sbol2-13505",
            severity: Warning,
            body: r#"<http://ex/m> a om:Measure ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    om:hasNumericalValue 1.0 ;
    om:hasUnit <http://ex/unit> ;
    sbol:type <http://example.org/not-sbo> .
"#,
        },
        RuleCase {
            name: "Measure hasUnit refers to a non-Unit object",
            rule: "sbol2-13506",
            severity: Error,
            body: r#"<http://ex/m> a om:Measure ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    om:hasNumericalValue 1.0 ;
    om:hasUnit <http://ex/notunit> .
<http://ex/notunit> a sbol:Collection ;
    sbol:displayId "notunit" ;
    sbol:persistentIdentity <http://ex/notunit> .
"#,
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "Measure carries a systems-description-parameter SBO type",
            rule: "sbol2-13505",
            body: r#"<http://ex/m> a om:Measure ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    om:hasNumericalValue 1.0 ;
    om:hasUnit <http://ex/unit> ;
    sbol:type <http://identifiers.org/biomodels.sbo/SBO:0000545> .
"#,
        },
        PositiveCase {
            name: "Measure hasUnit refers to an om:Unit",
            rule: "sbol2-13506",
            body: r#"<http://ex/m> a om:Measure ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    om:hasNumericalValue 1.0 ;
    om:hasUnit <http://ex/unit> .
<http://ex/unit> a om:Unit ;
    sbol:displayId "unit" ;
    sbol:persistentIdentity <http://ex/unit> .
"#,
        },
    ]
}
