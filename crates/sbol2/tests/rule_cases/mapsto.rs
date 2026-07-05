//! MapsTo containment and access rules (10803, 10804, 10807).

use super::common::{PositiveCase, RuleCase};
use sbol2::validation::Severity::Error;

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "MapsTo local is not a Component of the same ComponentDefinition",
            rule: "sbol2-10803",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/c> .
<http://ex/cd/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/cd/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:mapsTo <http://ex/cd/c/m> .
<http://ex/cd/c/m> a sbol:MapsTo ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/cd/c/m> ;
    sbol:refinement <http://sbols.org/v2#useLocal> ;
    sbol:local <http://ex/not-a-local> ;
    sbol:remote <http://ex/d/r> .
"#,
        },
        RuleCase {
            name: "MapsTo local is not a FunctionalComponent of the same ModuleDefinition",
            rule: "sbol2-10804",
            severity: Error,
            body: r#"<http://ex/md> a sbol:ModuleDefinition ;
    sbol:displayId "md" ;
    sbol:persistentIdentity <http://ex/md> ;
    sbol:module <http://ex/md/mod> .
<http://ex/md/mod> a sbol:Module ;
    sbol:displayId "mod" ;
    sbol:persistentIdentity <http://ex/md/mod> ;
    sbol:definition <http://ex/other-md> ;
    sbol:mapsTo <http://ex/md/mod/m> .
<http://ex/md/mod/m> a sbol:MapsTo ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/md/mod/m> ;
    sbol:refinement <http://sbols.org/v2#useLocal> ;
    sbol:local <http://ex/not-a-local> ;
    sbol:remote <http://ex/other-md/r> .
"#,
        },
        RuleCase {
            name: "MapsTo remote refers to a ComponentInstance without public access",
            rule: "sbol2-10807",
            severity: Error,
            body: r#"<http://ex/m> a sbol:MapsTo ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    sbol:refinement <http://sbols.org/v2#useRemote> ;
    sbol:local <http://ex/local> ;
    sbol:remote <http://ex/remote> .
<http://ex/remote> a sbol:Component ;
    sbol:displayId "remote" ;
    sbol:persistentIdentity <http://ex/remote> ;
    sbol:access <http://sbols.org/v2#private> ;
    sbol:definition <http://ex/d> .
"#,
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "MapsTo local is a Component of the same ComponentDefinition",
            rule: "sbol2-10803",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/c>, <http://ex/cd/local> .
<http://ex/cd/local> a sbol:Component ;
    sbol:displayId "local" ;
    sbol:persistentIdentity <http://ex/cd/local> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> .
<http://ex/cd/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/cd/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:mapsTo <http://ex/cd/c/m> .
<http://ex/cd/c/m> a sbol:MapsTo ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/cd/c/m> ;
    sbol:refinement <http://sbols.org/v2#useLocal> ;
    sbol:local <http://ex/cd/local> ;
    sbol:remote <http://ex/d/r> .
"#,
        },
        PositiveCase {
            name: "MapsTo local is a FunctionalComponent of the same ModuleDefinition",
            rule: "sbol2-10804",
            body: r#"<http://ex/md> a sbol:ModuleDefinition ;
    sbol:displayId "md" ;
    sbol:persistentIdentity <http://ex/md> ;
    sbol:functionalComponent <http://ex/md/local> ;
    sbol:module <http://ex/md/mod> .
<http://ex/md/local> a sbol:FunctionalComponent ;
    sbol:displayId "local" ;
    sbol:persistentIdentity <http://ex/md/local> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:direction <http://sbols.org/v2#inout> ;
    sbol:definition <http://ex/d> .
<http://ex/md/mod> a sbol:Module ;
    sbol:displayId "mod" ;
    sbol:persistentIdentity <http://ex/md/mod> ;
    sbol:definition <http://ex/other-md> ;
    sbol:mapsTo <http://ex/md/mod/m> .
<http://ex/md/mod/m> a sbol:MapsTo ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/md/mod/m> ;
    sbol:refinement <http://sbols.org/v2#useLocal> ;
    sbol:local <http://ex/md/local> ;
    sbol:remote <http://ex/other-md/r> .
"#,
        },
        PositiveCase {
            name: "MapsTo remote refers to a ComponentInstance with public access",
            rule: "sbol2-10807",
            body: r#"<http://ex/m> a sbol:MapsTo ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    sbol:refinement <http://sbols.org/v2#useRemote> ;
    sbol:local <http://ex/local> ;
    sbol:remote <http://ex/remote> .
<http://ex/remote> a sbol:Component ;
    sbol:displayId "remote" ;
    sbol:persistentIdentity <http://ex/remote> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> .
"#,
        },
    ]
}
