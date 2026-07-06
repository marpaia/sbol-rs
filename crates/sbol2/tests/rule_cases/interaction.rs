//! Interaction and Participation semantic rules (119xx, 120xx).

use super::common::{PositiveCase, RuleCase};
use sbol2::validation::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "Interaction without an occurring-entity SBO type",
            rule: "sbol2-11905",
            severity: Warning,
            body: r#"<http://ex/i> a sbol:Interaction ;
    sbol:displayId "i" ;
    sbol:persistentIdentity <http://ex/i> ;
    sbol:type <http://identifiers.org/so/SO:0000167> .
"#,
        },
        RuleCase {
            name: "inhibition Interaction with an incompatible participant role",
            rule: "sbol2-11907",
            severity: Warning,
            body: r#"<http://ex/i> a sbol:Interaction ;
    sbol:displayId "i" ;
    sbol:persistentIdentity <http://ex/i> ;
    sbol:type <http://identifiers.org/biomodels.sbo/SBO:0000169> ;
    sbol:participation <http://ex/i/p> .
<http://ex/i/p> a sbol:Participation ;
    sbol:displayId "p" ;
    sbol:persistentIdentity <http://ex/i/p> ;
    sbol:role <http://identifiers.org/biomodels.sbo/SBO:0000011> ;
    sbol:participant <http://ex/i/fc> .
"#,
        },
        RuleCase {
            name: "Interaction measure does not resolve to an om:Measure",
            rule: "sbol2-11908",
            severity: Error,
            body: r#"<http://ex/i> a sbol:Interaction ;
    sbol:displayId "i" ;
    sbol:persistentIdentity <http://ex/i> ;
    sbol:type <http://identifiers.org/biomodels.sbo/SBO:0000169> ;
    sbol:measure <http://ex/not-measure> .
<http://ex/not-measure> a sbol:Collection ;
    sbol:displayId "nm" ;
    sbol:persistentIdentity <http://ex/not-measure> .
"#,
        },
        RuleCase {
            name: "Participation participant is not a FunctionalComponent of the ModuleDefinition",
            rule: "sbol2-12003",
            severity: Error,
            body: r#"<http://ex/md> a sbol:ModuleDefinition ;
    sbol:displayId "md" ;
    sbol:persistentIdentity <http://ex/md> ;
    sbol:interaction <http://ex/md/i> .
<http://ex/md/i> a sbol:Interaction ;
    sbol:displayId "i" ;
    sbol:persistentIdentity <http://ex/md/i> ;
    sbol:type <http://identifiers.org/biomodels.sbo/SBO:0000169> ;
    sbol:participation <http://ex/md/i/p> .
<http://ex/md/i/p> a sbol:Participation ;
    sbol:displayId "p" ;
    sbol:persistentIdentity <http://ex/md/i/p> ;
    sbol:role <http://identifiers.org/biomodels.sbo/SBO:0000020> ;
    sbol:participant <http://ex/ghost-fc> .
"#,
        },
        RuleCase {
            name: "Participation without a participant-role SBO term",
            rule: "sbol2-12007",
            severity: Warning,
            body: r#"<http://ex/p> a sbol:Participation ;
    sbol:displayId "p" ;
    sbol:persistentIdentity <http://ex/p> ;
    sbol:role <http://example.org/not-sbo> ;
    sbol:participant <http://ex/fc> .
"#,
        },
        RuleCase {
            name: "Participation measure does not resolve to an om:Measure",
            rule: "sbol2-12008",
            severity: Error,
            body: r#"<http://ex/p> a sbol:Participation ;
    sbol:displayId "p" ;
    sbol:persistentIdentity <http://ex/p> ;
    sbol:role <http://identifiers.org/biomodels.sbo/SBO:0000010> ;
    sbol:participant <http://ex/fc> ;
    sbol:measure <http://ex/not-measure> .
<http://ex/not-measure> a sbol:Collection ;
    sbol:displayId "nm" ;
    sbol:persistentIdentity <http://ex/not-measure> .
"#,
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "Interaction with a single occurring-entity SBO type",
            rule: "sbol2-11905",
            body: r#"<http://ex/i> a sbol:Interaction ;
    sbol:displayId "i" ;
    sbol:persistentIdentity <http://ex/i> ;
    sbol:type <http://identifiers.org/biomodels.sbo/SBO:0000169> .
"#,
        },
        PositiveCase {
            name: "inhibition Interaction with a compatible participant role",
            rule: "sbol2-11907",
            body: r#"<http://ex/i> a sbol:Interaction ;
    sbol:displayId "i" ;
    sbol:persistentIdentity <http://ex/i> ;
    sbol:type <http://identifiers.org/biomodels.sbo/SBO:0000169> ;
    sbol:participation <http://ex/i/p> .
<http://ex/i/p> a sbol:Participation ;
    sbol:displayId "p" ;
    sbol:persistentIdentity <http://ex/i/p> ;
    sbol:role <http://identifiers.org/biomodels.sbo/SBO:0000020> ;
    sbol:participant <http://ex/i/fc> .
"#,
        },
        PositiveCase {
            name: "Interaction measure resolves to an om:Measure",
            rule: "sbol2-11908",
            body: r#"<http://ex/i> a sbol:Interaction ;
    sbol:displayId "i" ;
    sbol:persistentIdentity <http://ex/i> ;
    sbol:type <http://identifiers.org/biomodels.sbo/SBO:0000169> ;
    sbol:measure <http://ex/measure> .
<http://ex/measure> a om:Measure ;
    sbol:displayId "measure" ;
    sbol:persistentIdentity <http://ex/measure> ;
    om:hasNumericalValue 1.0 ;
    om:hasUnit <http://ex/unit> .
"#,
        },
        PositiveCase {
            name: "Participation participant is a FunctionalComponent of the ModuleDefinition",
            rule: "sbol2-12003",
            body: r#"<http://ex/md> a sbol:ModuleDefinition ;
    sbol:displayId "md" ;
    sbol:persistentIdentity <http://ex/md> ;
    sbol:functionalComponent <http://ex/md/fc> ;
    sbol:interaction <http://ex/md/i> .
<http://ex/md/fc> a sbol:FunctionalComponent ;
    sbol:displayId "fc" ;
    sbol:persistentIdentity <http://ex/md/fc> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:direction <http://sbols.org/v2#inout> ;
    sbol:definition <http://ex/d> .
<http://ex/md/i> a sbol:Interaction ;
    sbol:displayId "i" ;
    sbol:persistentIdentity <http://ex/md/i> ;
    sbol:type <http://identifiers.org/biomodels.sbo/SBO:0000169> ;
    sbol:participation <http://ex/md/i/p> .
<http://ex/md/i/p> a sbol:Participation ;
    sbol:displayId "p" ;
    sbol:persistentIdentity <http://ex/md/i/p> ;
    sbol:role <http://identifiers.org/biomodels.sbo/SBO:0000020> ;
    sbol:participant <http://ex/md/fc> .
"#,
        },
        PositiveCase {
            name: "Participation with a single participant-role SBO term",
            rule: "sbol2-12007",
            body: r#"<http://ex/p> a sbol:Participation ;
    sbol:displayId "p" ;
    sbol:persistentIdentity <http://ex/p> ;
    sbol:role <http://identifiers.org/biomodels.sbo/SBO:0000010> ;
    sbol:participant <http://ex/fc> .
"#,
        },
        PositiveCase {
            name: "Participation measure resolves to an om:Measure",
            rule: "sbol2-12008",
            body: r#"<http://ex/p> a sbol:Participation ;
    sbol:displayId "p" ;
    sbol:persistentIdentity <http://ex/p> ;
    sbol:role <http://identifiers.org/biomodels.sbo/SBO:0000010> ;
    sbol:participant <http://ex/fc> ;
    sbol:measure <http://ex/measure> .
<http://ex/measure> a om:Measure ;
    sbol:displayId "measure" ;
    sbol:persistentIdentity <http://ex/measure> ;
    om:hasNumericalValue 1.0 ;
    om:hasUnit <http://ex/unit> .
"#,
        },
    ]
}
