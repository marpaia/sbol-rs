//! Model, ModuleDefinition, and Module semantic rules (115xx, 116xx, 117xx).

use super::common::{PositiveCase, RuleCase};
use sbol2::validation::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "Model language is not an EDAM term",
            rule: "sbol2-11507",
            severity: Warning,
            body: r#"<http://ex/model> a sbol:Model ;
    sbol:displayId "model" ;
    sbol:persistentIdentity <http://ex/model> ;
    sbol:source <http://ex/model.xml> ;
    sbol:language <http://example.org/not-edam> ;
    sbol:framework <http://identifiers.org/biomodels.sbo/SBO:0000062> .
"#,
        },
        RuleCase {
            name: "Model framework is not in the SBO modeling-framework branch",
            rule: "sbol2-11511",
            severity: Warning,
            body: r#"<http://ex/model> a sbol:Model ;
    sbol:displayId "model" ;
    sbol:persistentIdentity <http://ex/model> ;
    sbol:source <http://ex/model.xml> ;
    sbol:language <http://identifiers.org/edam:format_2585> ;
    sbol:framework <http://example.org/not-sbo> .
"#,
        },
        RuleCase {
            name: "ModuleDefinition model reference does not resolve",
            rule: "sbol2-11608",
            severity: Error,
            body: r#"<http://ex/md> a sbol:ModuleDefinition ;
    sbol:displayId "md" ;
    sbol:persistentIdentity <http://ex/md> ;
    sbol:model <http://ex/missing-model> .
"#,
        },
        RuleCase {
            name: "two useRemote MapsTos of a ModuleDefinition share a local",
            rule: "sbol2-11609",
            severity: Error,
            body: r#"<http://ex/md> a sbol:ModuleDefinition ;
    sbol:displayId "md" ;
    sbol:persistentIdentity <http://ex/md> ;
    sbol:functionalComponent <http://ex/md/local> ;
    sbol:module <http://ex/md/m1>, <http://ex/md/m2> .
<http://ex/md/local> a sbol:FunctionalComponent ;
    sbol:displayId "local" ;
    sbol:persistentIdentity <http://ex/md/local> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:direction <http://sbols.org/v2#inout> ;
    sbol:definition <http://ex/d> .
<http://ex/md/m1> a sbol:Module ;
    sbol:displayId "m1" ;
    sbol:persistentIdentity <http://ex/md/m1> ;
    sbol:definition <http://ex/other> ;
    sbol:mapsTo <http://ex/md/m1/mt> .
<http://ex/md/m1/mt> a sbol:MapsTo ;
    sbol:displayId "mt" ;
    sbol:persistentIdentity <http://ex/md/m1/mt> ;
    sbol:refinement <http://sbols.org/v2#useRemote> ;
    sbol:local <http://ex/md/local> ;
    sbol:remote <http://ex/other/r> .
<http://ex/md/m2> a sbol:Module ;
    sbol:displayId "m2" ;
    sbol:persistentIdentity <http://ex/md/m2> ;
    sbol:definition <http://ex/other> ;
    sbol:mapsTo <http://ex/md/m2/mt> .
<http://ex/md/m2/mt> a sbol:MapsTo ;
    sbol:displayId "mt" ;
    sbol:persistentIdentity <http://ex/md/m2/mt> ;
    sbol:refinement <http://sbols.org/v2#useRemote> ;
    sbol:local <http://ex/md/local> ;
    sbol:remote <http://ex/other/r> .
"#,
        },
        RuleCase {
            name: "Module definition does not resolve to a ModuleDefinition",
            rule: "sbol2-11703",
            severity: Error,
            body: r#"<http://ex/m> a sbol:Module ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    sbol:definition <http://ex/missing-md> .
"#,
        },
        RuleCase {
            name: "Module definition refers to its own containing ModuleDefinition",
            rule: "sbol2-11704",
            severity: Error,
            body: r#"<http://ex/md> a sbol:ModuleDefinition ;
    sbol:displayId "md" ;
    sbol:persistentIdentity <http://ex/md> ;
    sbol:module <http://ex/md/m> .
<http://ex/md/m> a sbol:Module ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/md/m> ;
    sbol:definition <http://ex/md> .
"#,
        },
        RuleCase {
            name: "ModuleDefinition cycle through Module definitions",
            rule: "sbol2-11705",
            severity: Error,
            body: r#"<http://ex/a> a sbol:ModuleDefinition ;
    sbol:displayId "a" ;
    sbol:persistentIdentity <http://ex/a> ;
    sbol:module <http://ex/a/m> .
<http://ex/a/m> a sbol:Module ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/a/m> ;
    sbol:definition <http://ex/b> .
<http://ex/b> a sbol:ModuleDefinition ;
    sbol:displayId "b" ;
    sbol:persistentIdentity <http://ex/b> ;
    sbol:module <http://ex/b/m> .
<http://ex/b/m> a sbol:Module ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/b/m> ;
    sbol:definition <http://ex/a> .
"#,
        },
        RuleCase {
            name: "Module measure does not resolve to an om:Measure",
            rule: "sbol2-11707",
            severity: Error,
            body: r#"<http://ex/m> a sbol:Module ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    sbol:definition <http://ex/md> ;
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
            name: "Model language is an EDAM term",
            rule: "sbol2-11507",
            body: r#"<http://ex/model> a sbol:Model ;
    sbol:displayId "model" ;
    sbol:persistentIdentity <http://ex/model> ;
    sbol:source <http://ex/model.xml> ;
    sbol:language <http://identifiers.org/edam:format_2585> ;
    sbol:framework <http://identifiers.org/biomodels.sbo/SBO:0000062> .
"#,
        },
        PositiveCase {
            name: "Model framework is in the SBO modeling-framework branch",
            rule: "sbol2-11511",
            body: r#"<http://ex/model> a sbol:Model ;
    sbol:displayId "model" ;
    sbol:persistentIdentity <http://ex/model> ;
    sbol:source <http://ex/model.xml> ;
    sbol:language <http://identifiers.org/edam:format_2585> ;
    sbol:framework <http://identifiers.org/biomodels.sbo/SBO:0000062> .
"#,
        },
        PositiveCase {
            name: "ModuleDefinition model reference resolves to a Model",
            rule: "sbol2-11608",
            body: r#"<http://ex/md> a sbol:ModuleDefinition ;
    sbol:displayId "md" ;
    sbol:persistentIdentity <http://ex/md> ;
    sbol:model <http://ex/model> .
<http://ex/model> a sbol:Model ;
    sbol:displayId "model" ;
    sbol:persistentIdentity <http://ex/model> ;
    sbol:source <http://ex/model.xml> ;
    sbol:language <http://identifiers.org/edam:format_2585> ;
    sbol:framework <http://identifiers.org/biomodels.sbo/SBO:0000062> .
"#,
        },
        PositiveCase {
            name: "useRemote MapsTos of a ModuleDefinition use distinct locals",
            rule: "sbol2-11609",
            body: r#"<http://ex/md> a sbol:ModuleDefinition ;
    sbol:displayId "md" ;
    sbol:persistentIdentity <http://ex/md> ;
    sbol:functionalComponent <http://ex/md/l1>, <http://ex/md/l2> ;
    sbol:module <http://ex/md/m1>, <http://ex/md/m2> .
<http://ex/md/l1> a sbol:FunctionalComponent ;
    sbol:displayId "l1" ;
    sbol:persistentIdentity <http://ex/md/l1> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:direction <http://sbols.org/v2#inout> ;
    sbol:definition <http://ex/d> .
<http://ex/md/l2> a sbol:FunctionalComponent ;
    sbol:displayId "l2" ;
    sbol:persistentIdentity <http://ex/md/l2> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:direction <http://sbols.org/v2#inout> ;
    sbol:definition <http://ex/d> .
<http://ex/md/m1> a sbol:Module ;
    sbol:displayId "m1" ;
    sbol:persistentIdentity <http://ex/md/m1> ;
    sbol:definition <http://ex/other> ;
    sbol:mapsTo <http://ex/md/m1/mt> .
<http://ex/md/m1/mt> a sbol:MapsTo ;
    sbol:displayId "mt" ;
    sbol:persistentIdentity <http://ex/md/m1/mt> ;
    sbol:refinement <http://sbols.org/v2#useRemote> ;
    sbol:local <http://ex/md/l1> ;
    sbol:remote <http://ex/other/r> .
<http://ex/md/m2> a sbol:Module ;
    sbol:displayId "m2" ;
    sbol:persistentIdentity <http://ex/md/m2> ;
    sbol:definition <http://ex/other> ;
    sbol:mapsTo <http://ex/md/m2/mt> .
<http://ex/md/m2/mt> a sbol:MapsTo ;
    sbol:displayId "mt" ;
    sbol:persistentIdentity <http://ex/md/m2/mt> ;
    sbol:refinement <http://sbols.org/v2#useRemote> ;
    sbol:local <http://ex/md/l2> ;
    sbol:remote <http://ex/other/r> .
"#,
        },
        PositiveCase {
            name: "Module definition resolves to a ModuleDefinition",
            rule: "sbol2-11703",
            body: r#"<http://ex/m> a sbol:Module ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    sbol:definition <http://ex/md> .
<http://ex/md> a sbol:ModuleDefinition ;
    sbol:displayId "md" ;
    sbol:persistentIdentity <http://ex/md> .
"#,
        },
        PositiveCase {
            name: "Module definition refers to a distinct ModuleDefinition",
            rule: "sbol2-11704",
            body: r#"<http://ex/md> a sbol:ModuleDefinition ;
    sbol:displayId "md" ;
    sbol:persistentIdentity <http://ex/md> ;
    sbol:module <http://ex/md/m> .
<http://ex/md/m> a sbol:Module ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/md/m> ;
    sbol:definition <http://ex/other> .
"#,
        },
        PositiveCase {
            name: "acyclic ModuleDefinition-Module hierarchy",
            rule: "sbol2-11705",
            body: r#"<http://ex/a> a sbol:ModuleDefinition ;
    sbol:displayId "a" ;
    sbol:persistentIdentity <http://ex/a> ;
    sbol:module <http://ex/a/m> .
<http://ex/a/m> a sbol:Module ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/a/m> ;
    sbol:definition <http://ex/b> .
<http://ex/b> a sbol:ModuleDefinition ;
    sbol:displayId "b" ;
    sbol:persistentIdentity <http://ex/b> .
"#,
        },
        PositiveCase {
            name: "Module measure resolves to an om:Measure",
            rule: "sbol2-11707",
            body: r#"<http://ex/m> a sbol:Module ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    sbol:definition <http://ex/md> ;
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
