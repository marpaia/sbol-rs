//! Identity, compliant-URI, and derivation-cycle rules (10101, 102xx, 103xx).

use super::common::{PositiveCase, RuleCase};
use sbol2::validation::Severity::Error;

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "document names no SBOL 2 namespace term",
            rule: "sbol2-10101",
            severity: Error,
            body: r#"<http://ex/x> a <http://example.org/Foo> ;
    <http://example.org/p> "v" .
"#,
        },
        RuleCase {
            name: "blank-node Identified object has no URI identity",
            rule: "sbol2-10201",
            severity: Error,
            body: r#"[] a sbol:Sequence ;
    sbol:displayId "s" ;
    sbol:elements "A" ;
    sbol:encoding <http://sbols.org/v2#IUPACDNA> .
"#,
        },
        RuleCase {
            name: "compliant object without a displayId",
            rule: "sbol2-10215",
            severity: Error,
            body: r#"<http://ex/s> a sbol:Sequence ;
    sbol:persistentIdentity <http://ex/s> ;
    sbol:elements "A" ;
    sbol:encoding <http://sbols.org/v2#IUPACDNA> .
"#,
        },
        RuleCase {
            name: "compliant TopLevel persistentIdentity not ending in displayId",
            rule: "sbol2-10216",
            severity: Error,
            body: r#"<http://ex/s1> a sbol:Sequence ;
    sbol:displayId "s1" ;
    sbol:persistentIdentity <http://ex/mismatch> ;
    sbol:elements "A" ;
    sbol:encoding <http://sbols.org/v2#IUPACDNA> .
"#,
        },
        RuleCase {
            name: "compliant child persistentIdentity not extending its parent",
            rule: "sbol2-10217",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/c> .
<http://ex/cd/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/wrong/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/other> .
"#,
        },
        RuleCase {
            name: "compliant identity that is not its persistentIdentity",
            rule: "sbol2-10218",
            severity: Error,
            body: r#"<http://ex/a> a sbol:Collection ;
    sbol:displayId "a" ;
    sbol:persistentIdentity <http://ex/b> .
"#,
        },
        RuleCase {
            name: "compliant child version disagreeing with its parent",
            rule: "sbol2-10219",
            severity: Error,
            body: r#"<http://ex/cd/1> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:version "1" ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/c/1> .
<http://ex/cd/c/1> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/cd/c> ;
    sbol:version "2" ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/other> .
"#,
        },
        RuleCase {
            name: "two objects share a persistentIdentity but differ in class",
            rule: "sbol2-10220",
            severity: Error,
            body: r#"<http://ex/a> a sbol:Collection ;
    sbol:displayId "a" ;
    sbol:persistentIdentity <http://ex/shared> .
<http://ex/b> a sbol:Sequence ;
    sbol:displayId "b" ;
    sbol:persistentIdentity <http://ex/shared> ;
    sbol:elements "A" ;
    sbol:encoding <http://sbols.org/v2#IUPACDNA> .
"#,
        },
        RuleCase {
            name: "object with two SBOL rdfType properties",
            rule: "sbol2-10228",
            severity: Error,
            body: r#"<http://ex/x> a sbol:Collection, sbol:Sequence ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "TopLevel deriving from itself",
            rule: "sbol2-10303",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    prov:wasDerivedFrom <http://ex/cd> .
"#,
        },
        RuleCase {
            name: "TopLevel derivation cycle",
            rule: "sbol2-10304",
            severity: Error,
            body: r#"<http://ex/a> a sbol:ComponentDefinition ;
    sbol:displayId "a" ;
    sbol:persistentIdentity <http://ex/a> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    prov:wasDerivedFrom <http://ex/b> .
<http://ex/b> a sbol:ComponentDefinition ;
    sbol:displayId "b" ;
    sbol:persistentIdentity <http://ex/b> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    prov:wasDerivedFrom <http://ex/a> .
"#,
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "document using an SBOL 2 class and property",
            rule: "sbol2-10101",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
        PositiveCase {
            name: "Identified object with a URI identity",
            rule: "sbol2-10201",
            body: r#"<http://ex/s> a sbol:Sequence ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/s> ;
    sbol:elements "A" ;
    sbol:encoding <http://sbols.org/v2#IUPACDNA> .
"#,
        },
        PositiveCase {
            name: "compliant object carrying a displayId",
            rule: "sbol2-10215",
            body: r#"<http://ex/s> a sbol:Sequence ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/s> ;
    sbol:elements "A" ;
    sbol:encoding <http://sbols.org/v2#IUPACDNA> .
"#,
        },
        PositiveCase {
            name: "compliant TopLevel persistentIdentity ending in displayId",
            rule: "sbol2-10216",
            body: r#"<http://ex/foo/s1> a sbol:Sequence ;
    sbol:displayId "s1" ;
    sbol:persistentIdentity <http://ex/foo/s1> ;
    sbol:elements "A" ;
    sbol:encoding <http://sbols.org/v2#IUPACDNA> .
"#,
        },
        PositiveCase {
            name: "compliant child persistentIdentity extending its parent",
            rule: "sbol2-10217",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/c> .
<http://ex/cd/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/cd/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/other> .
"#,
        },
        PositiveCase {
            name: "compliant identity equal to persistentIdentity",
            rule: "sbol2-10218",
            body: r#"<http://ex/a> a sbol:Collection ;
    sbol:displayId "a" ;
    sbol:persistentIdentity <http://ex/a> .
"#,
        },
        PositiveCase {
            name: "compliant child version matching its parent",
            rule: "sbol2-10219",
            body: r#"<http://ex/cd/1> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:version "1" ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/c/1> .
<http://ex/cd/c/1> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/cd/c> ;
    sbol:version "1" ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/other> .
"#,
        },
        PositiveCase {
            name: "objects with the same persistentIdentity share a class",
            rule: "sbol2-10220",
            body: r#"<http://ex/a> a sbol:Collection ;
    sbol:displayId "a" ;
    sbol:persistentIdentity <http://ex/a> .
<http://ex/b> a sbol:Collection ;
    sbol:displayId "b" ;
    sbol:persistentIdentity <http://ex/b> .
"#,
        },
        PositiveCase {
            name: "object with a single SBOL rdfType",
            rule: "sbol2-10228",
            body: r#"<http://ex/x> a sbol:Collection ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "TopLevel deriving from a distinct object",
            rule: "sbol2-10303",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    prov:wasDerivedFrom <http://ex/ancestor> .
"#,
        },
        PositiveCase {
            name: "acyclic TopLevel derivation chain",
            rule: "sbol2-10304",
            body: r#"<http://ex/a> a sbol:ComponentDefinition ;
    sbol:displayId "a" ;
    sbol:persistentIdentity <http://ex/a> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    prov:wasDerivedFrom <http://ex/b> .
<http://ex/b> a sbol:ComponentDefinition ;
    sbol:displayId "b" ;
    sbol:persistentIdentity <http://ex/b> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
    ]
}
