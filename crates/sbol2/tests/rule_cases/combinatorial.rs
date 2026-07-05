//! CombinatorialDerivation and VariableComponent semantic rules (129xx, 130xx).

use super::common::{PositiveCase, RuleCase};
use sbol2::validation::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "CombinatorialDerivation strategy is not a Table 15 URI",
            rule: "sbol2-12902",
            severity: Error,
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:strategy <http://example.org/custom-strategy> ;
    sbol:template <http://ex/t> .
"#,
        },
        RuleCase {
            name: "enumerate CombinatorialDerivation with an unbounded operator",
            rule: "sbol2-12903",
            severity: Error,
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:strategy <http://sbols.org/v2#enumerate> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc> .
<http://ex/cda/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/cda/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#oneOrMore> .
"#,
        },
        RuleCase {
            name: "CombinatorialDerivation template does not resolve",
            rule: "sbol2-12905",
            severity: Error,
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/missing-template> .
"#,
        },
        RuleCase {
            name: "two VariableComponents share a variable",
            rule: "sbol2-12907",
            severity: Error,
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc1>, <http://ex/cda/vc2> .
<http://ex/cda/vc1> a sbol:VariableComponent ;
    sbol:displayId "vc1" ;
    sbol:persistentIdentity <http://ex/cda/vc1> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variant <http://ex/v> .
<http://ex/cda/vc2> a sbol:VariableComponent ;
    sbol:displayId "vc2" ;
    sbol:persistentIdentity <http://ex/cda/vc2> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variant <http://ex/v> .
"#,
        },
        RuleCase {
            name: "CombinatorialDerivation template has no Components",
            rule: "sbol2-12909",
            severity: Warning,
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
        RuleCase {
            name: "derived ComponentDefinition types differ from its template",
            rule: "sbol2-12910",
            severity: Warning,
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
<http://ex/derived> a sbol:ComponentDefinition ;
    sbol:displayId "derived" ;
    sbol:persistentIdentity <http://ex/derived> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#Protein> ;
    prov:wasDerivedFrom <http://ex/cda> .
"#,
        },
        RuleCase {
            name: "derived ComponentDefinition roles differ from its template",
            rule: "sbol2-12911",
            severity: Warning,
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:role <http://identifiers.org/so/SO:0000167> .
<http://ex/derived> a sbol:ComponentDefinition ;
    sbol:displayId "derived" ;
    sbol:persistentIdentity <http://ex/derived> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:role <http://identifiers.org/so/SO:0000316> ;
    prov:wasDerivedFrom <http://ex/cda> .
"#,
        },
        RuleCase {
            name: "Collection member derives from a CombinatorialDerivation the Collection omits",
            rule: "sbol2-12912",
            severity: Warning,
            body: r#"<http://ex/coll> a sbol:Collection ;
    sbol:displayId "coll" ;
    sbol:persistentIdentity <http://ex/coll> ;
    sbol:member <http://ex/m> .
<http://ex/m> a sbol:ComponentDefinition ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    prov:wasDerivedFrom <http://ex/cda> .
<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> .
"#,
        },
        RuleCase {
            name: "Collection derives from a CombinatorialDerivation a member omits",
            rule: "sbol2-12913",
            severity: Warning,
            body: r#"<http://ex/coll> a sbol:Collection ;
    sbol:displayId "coll" ;
    sbol:persistentIdentity <http://ex/coll> ;
    sbol:member <http://ex/m> ;
    prov:wasDerivedFrom <http://ex/cda> .
<http://ex/m> a sbol:ComponentDefinition ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> .
"#,
        },
        RuleCase {
            name: "VariableComponent operator is not a Table 16 URI",
            rule: "sbol2-13003",
            severity: Error,
            body: r#"<http://ex/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://example.org/custom-operator> .
"#,
        },
        RuleCase {
            name: "VariableComponent offers no variant source",
            rule: "sbol2-13006",
            severity: Warning,
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc> .
<http://ex/cda/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/cda/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> .
"#,
        },
        RuleCase {
            name: "VariableComponent variant does not resolve",
            rule: "sbol2-13008",
            severity: Error,
            body: r#"<http://ex/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variant <http://ex/missing-variant> .
"#,
        },
        RuleCase {
            name: "VariableComponent variantCollection is not a Collection",
            rule: "sbol2-13010",
            severity: Error,
            body: r#"<http://ex/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variantCollection <http://ex/not-collection> .
<http://ex/not-collection> a sbol:ComponentDefinition ;
    sbol:displayId "nc" ;
    sbol:persistentIdentity <http://ex/not-collection> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
        RuleCase {
            name: "VariableComponent variantDerivation does not resolve",
            rule: "sbol2-13012",
            severity: Error,
            body: r#"<http://ex/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variantDerivation <http://ex/missing-derivation> .
"#,
        },
        RuleCase {
            name: "VariableComponent variantDerivation is not a CombinatorialDerivation",
            rule: "sbol2-13014",
            severity: Error,
            body: r#"<http://ex/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variantDerivation <http://ex/not-derivation> .
<http://ex/not-derivation> a sbol:ComponentDefinition ;
    sbol:displayId "nd" ;
    sbol:persistentIdentity <http://ex/not-derivation> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
        RuleCase {
            name: "CombinatorialDerivation cycle through variantDerivations",
            rule: "sbol2-13015",
            severity: Error,
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc> .
<http://ex/cda/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/cda/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variantDerivation <http://ex/cdb> .
<http://ex/cdb> a sbol:CombinatorialDerivation ;
    sbol:displayId "cdb" ;
    sbol:persistentIdentity <http://ex/cdb> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cdb/vc> .
<http://ex/cdb/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/cdb/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variantDerivation <http://ex/cda> .
"#,
        },
        RuleCase {
            name: "derived Component roles differ from its template Component",
            rule: "sbol2-13018",
            severity: Warning,
            body: DERIVED_COMPONENT_ROLE_MISMATCH,
        },
        RuleCase {
            name: "zeroOrOne VariableComponent realized more than once",
            rule: "sbol2-13019",
            severity: Warning,
            body: realization("http://sbols.org/v2#zeroOrOne", 2),
        },
        RuleCase {
            name: "one VariableComponent realized zero times",
            rule: "sbol2-13020",
            severity: Warning,
            body: realization("http://sbols.org/v2#one", 0),
        },
        RuleCase {
            name: "oneOrMore VariableComponent realized zero times",
            rule: "sbol2-13021",
            severity: Warning,
            body: realization("http://sbols.org/v2#oneOrMore", 0),
        },
        RuleCase {
            name: "non-replaced template Component not realized exactly once",
            rule: "sbol2-13022",
            severity: Warning,
            body: NON_REPLACED_UNREALIZED,
        },
    ]
}

/// A CombinatorialDerivation with a VariableComponent of `operator` over
/// template Component `t/c`, and a derived ComponentDefinition realizing it
/// `count` times.
fn realization(operator: &str, count: usize) -> &'static str {
    match (operator, count) {
        ("http://sbols.org/v2#zeroOrOne", 2) => REALIZE_ZEROORONE_TWICE,
        ("http://sbols.org/v2#one", 0) => REALIZE_ONE_ZERO,
        ("http://sbols.org/v2#oneOrMore", 0) => REALIZE_ONEORMORE_ZERO,
        _ => unreachable!(),
    }
}

const REALIZE_ZEROORONE_TWICE: &str = r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc> .
<http://ex/cda/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/cda/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#zeroOrOne> ;
    sbol:variant <http://ex/v> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/t/c> .
<http://ex/t/c> a sbol:Component ;
    sbol:displayId "tc" ;
    sbol:persistentIdentity <http://ex/t/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> .
<http://ex/derived> a sbol:ComponentDefinition ;
    sbol:displayId "derived" ;
    sbol:persistentIdentity <http://ex/derived> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/derived/c1>, <http://ex/derived/c2> ;
    prov:wasDerivedFrom <http://ex/cda> .
<http://ex/derived/c1> a sbol:Component ;
    sbol:displayId "c1" ;
    sbol:persistentIdentity <http://ex/derived/c1> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/v> ;
    prov:wasDerivedFrom <http://ex/t/c> .
<http://ex/derived/c2> a sbol:Component ;
    sbol:displayId "c2" ;
    sbol:persistentIdentity <http://ex/derived/c2> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/v> ;
    prov:wasDerivedFrom <http://ex/t/c> .
"#;

const REALIZE_ONE_ZERO: &str = r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc> .
<http://ex/cda/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/cda/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variant <http://ex/v> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/t/c> .
<http://ex/t/c> a sbol:Component ;
    sbol:displayId "tc" ;
    sbol:persistentIdentity <http://ex/t/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> .
<http://ex/derived> a sbol:ComponentDefinition ;
    sbol:displayId "derived" ;
    sbol:persistentIdentity <http://ex/derived> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    prov:wasDerivedFrom <http://ex/cda> .
"#;

const REALIZE_ONEORMORE_ZERO: &str = r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc> .
<http://ex/cda/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/cda/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#oneOrMore> ;
    sbol:variant <http://ex/v> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/t/c> .
<http://ex/t/c> a sbol:Component ;
    sbol:displayId "tc" ;
    sbol:persistentIdentity <http://ex/t/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> .
<http://ex/derived> a sbol:ComponentDefinition ;
    sbol:displayId "derived" ;
    sbol:persistentIdentity <http://ex/derived> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    prov:wasDerivedFrom <http://ex/cda> .
"#;

const DERIVED_COMPONENT_ROLE_MISMATCH: &str = r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/t/c> .
<http://ex/t/c> a sbol:Component ;
    sbol:displayId "tc" ;
    sbol:persistentIdentity <http://ex/t/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    sbol:roleIntegration <http://sbols.org/v2#mergeRoles> .
<http://ex/derived> a sbol:ComponentDefinition ;
    sbol:displayId "derived" ;
    sbol:persistentIdentity <http://ex/derived> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/derived/c> ;
    prov:wasDerivedFrom <http://ex/cda> .
<http://ex/derived/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/derived/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> ;
    sbol:role <http://identifiers.org/so/SO:0000316> ;
    sbol:roleIntegration <http://sbols.org/v2#mergeRoles> ;
    prov:wasDerivedFrom <http://ex/t/c> .
"#;

const NON_REPLACED_UNREALIZED: &str = r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc> .
<http://ex/cda/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/cda/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variant <http://ex/v> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/t/c>, <http://ex/t/c2> .
<http://ex/t/c> a sbol:Component ;
    sbol:displayId "tc" ;
    sbol:persistentIdentity <http://ex/t/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> .
<http://ex/t/c2> a sbol:Component ;
    sbol:displayId "tc2" ;
    sbol:persistentIdentity <http://ex/t/c2> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> .
<http://ex/derived> a sbol:ComponentDefinition ;
    sbol:displayId "derived" ;
    sbol:persistentIdentity <http://ex/derived> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/derived/c> ;
    prov:wasDerivedFrom <http://ex/cda> .
<http://ex/derived/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/derived/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/v> ;
    prov:wasDerivedFrom <http://ex/t/c> .
"#;

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "CombinatorialDerivation strategy is a Table 15 URI",
            rule: "sbol2-12902",
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:strategy <http://sbols.org/v2#enumerate> ;
    sbol:template <http://ex/t> .
"#,
        },
        PositiveCase {
            name: "enumerate CombinatorialDerivation with a bounded operator",
            rule: "sbol2-12903",
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:strategy <http://sbols.org/v2#enumerate> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc> .
<http://ex/cda/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/cda/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variant <http://ex/v> .
"#,
        },
        PositiveCase {
            name: "CombinatorialDerivation template resolves to a ComponentDefinition",
            rule: "sbol2-12905",
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/t/c> .
<http://ex/t/c> a sbol:Component ;
    sbol:displayId "tc" ;
    sbol:persistentIdentity <http://ex/t/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> .
"#,
        },
        PositiveCase {
            name: "VariableComponents use distinct variables",
            rule: "sbol2-12907",
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc1>, <http://ex/cda/vc2> .
<http://ex/cda/vc1> a sbol:VariableComponent ;
    sbol:displayId "vc1" ;
    sbol:persistentIdentity <http://ex/cda/vc1> ;
    sbol:variable <http://ex/t/c1> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variant <http://ex/v> .
<http://ex/cda/vc2> a sbol:VariableComponent ;
    sbol:displayId "vc2" ;
    sbol:persistentIdentity <http://ex/cda/vc2> ;
    sbol:variable <http://ex/t/c2> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variant <http://ex/v> .
"#,
        },
        PositiveCase {
            name: "CombinatorialDerivation template has a Component",
            rule: "sbol2-12909",
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/t/c> .
<http://ex/t/c> a sbol:Component ;
    sbol:displayId "tc" ;
    sbol:persistentIdentity <http://ex/t/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> .
"#,
        },
        PositiveCase {
            name: "derived ComponentDefinition keeps its template types",
            rule: "sbol2-12910",
            body: DERIVED_MATCHES_TEMPLATE,
        },
        PositiveCase {
            name: "derived ComponentDefinition keeps its template roles",
            rule: "sbol2-12911",
            body: DERIVED_MATCHES_TEMPLATE,
        },
        PositiveCase {
            name: "Collection records the CombinatorialDerivation its member derives from",
            rule: "sbol2-12912",
            body: COLLECTION_CONSISTENT,
        },
        PositiveCase {
            name: "Collection members record the CombinatorialDerivation the Collection derives from",
            rule: "sbol2-12913",
            body: COLLECTION_CONSISTENT,
        },
        PositiveCase {
            name: "VariableComponent operator is a Table 16 URI",
            rule: "sbol2-13003",
            body: r#"<http://ex/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> .
"#,
        },
        PositiveCase {
            name: "VariableComponent offers a variant source",
            rule: "sbol2-13006",
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc> .
<http://ex/cda/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/cda/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variant <http://ex/v> .
"#,
        },
        PositiveCase {
            name: "VariableComponent variant resolves to a ComponentDefinition",
            rule: "sbol2-13008",
            body: r#"<http://ex/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variant <http://ex/v> .
<http://ex/v> a sbol:ComponentDefinition ;
    sbol:displayId "v" ;
    sbol:persistentIdentity <http://ex/v> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
        PositiveCase {
            name: "VariableComponent variantCollection is a Collection",
            rule: "sbol2-13010",
            body: r#"<http://ex/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variantCollection <http://ex/coll> .
<http://ex/coll> a sbol:Collection ;
    sbol:displayId "coll" ;
    sbol:persistentIdentity <http://ex/coll> ;
    sbol:member <http://ex/v> .
"#,
        },
        PositiveCase {
            name: "VariableComponent variantDerivation resolves to a CombinatorialDerivation",
            rule: "sbol2-13012",
            body: r#"<http://ex/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variantDerivation <http://ex/cdb> .
<http://ex/cdb> a sbol:CombinatorialDerivation ;
    sbol:displayId "cdb" ;
    sbol:persistentIdentity <http://ex/cdb> ;
    sbol:template <http://ex/t> .
"#,
        },
        PositiveCase {
            name: "VariableComponent variantDerivation is a CombinatorialDerivation",
            rule: "sbol2-13014",
            body: r#"<http://ex/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variantDerivation <http://ex/cdb> .
<http://ex/cdb> a sbol:CombinatorialDerivation ;
    sbol:displayId "cdb" ;
    sbol:persistentIdentity <http://ex/cdb> ;
    sbol:template <http://ex/t> .
"#,
        },
        PositiveCase {
            name: "acyclic CombinatorialDerivation variantDerivation chain",
            rule: "sbol2-13015",
            body: r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc> .
<http://ex/cda/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/cda/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variantDerivation <http://ex/cdb> .
<http://ex/cdb> a sbol:CombinatorialDerivation ;
    sbol:displayId "cdb" ;
    sbol:persistentIdentity <http://ex/cdb> ;
    sbol:template <http://ex/t> .
"#,
        },
        PositiveCase {
            name: "derived Component keeps its template Component roles",
            rule: "sbol2-13018",
            body: DERIVED_COMPONENT_ROLE_MATCH,
        },
        PositiveCase {
            name: "zeroOrOne VariableComponent realized once",
            rule: "sbol2-13019",
            body: REALIZE_ONE_ONCE,
        },
        PositiveCase {
            name: "one VariableComponent realized once",
            rule: "sbol2-13020",
            body: REALIZE_ONE_ONCE,
        },
        PositiveCase {
            name: "oneOrMore VariableComponent realized once",
            rule: "sbol2-13021",
            body: REALIZE_ONE_ONCE,
        },
        PositiveCase {
            name: "non-replaced template Component realized once",
            rule: "sbol2-13022",
            body: NON_REPLACED_REALIZED,
        },
    ]
}

const DERIVED_MATCHES_TEMPLATE: &str = r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    sbol:component <http://ex/t/c> .
<http://ex/t/c> a sbol:Component ;
    sbol:displayId "tc" ;
    sbol:persistentIdentity <http://ex/t/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> .
<http://ex/derived> a sbol:ComponentDefinition ;
    sbol:displayId "derived" ;
    sbol:persistentIdentity <http://ex/derived> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    prov:wasDerivedFrom <http://ex/cda> .
"#;

const COLLECTION_CONSISTENT: &str = r#"<http://ex/coll> a sbol:Collection ;
    sbol:displayId "coll" ;
    sbol:persistentIdentity <http://ex/coll> ;
    sbol:member <http://ex/m> ;
    prov:wasDerivedFrom <http://ex/cda> .
<http://ex/m> a sbol:ComponentDefinition ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    prov:wasDerivedFrom <http://ex/cda> .
<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> .
"#;

const DERIVED_COMPONENT_ROLE_MATCH: &str = r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/t/c> .
<http://ex/t/c> a sbol:Component ;
    sbol:displayId "tc" ;
    sbol:persistentIdentity <http://ex/t/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    sbol:roleIntegration <http://sbols.org/v2#mergeRoles> .
<http://ex/derived> a sbol:ComponentDefinition ;
    sbol:displayId "derived" ;
    sbol:persistentIdentity <http://ex/derived> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/derived/c> ;
    prov:wasDerivedFrom <http://ex/cda> .
<http://ex/derived/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/derived/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    sbol:roleIntegration <http://sbols.org/v2#mergeRoles> ;
    prov:wasDerivedFrom <http://ex/t/c> .
"#;

const REALIZE_ONE_ONCE: &str = r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc> .
<http://ex/cda/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/cda/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variant <http://ex/v> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/t/c> .
<http://ex/t/c> a sbol:Component ;
    sbol:displayId "tc" ;
    sbol:persistentIdentity <http://ex/t/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> .
<http://ex/derived> a sbol:ComponentDefinition ;
    sbol:displayId "derived" ;
    sbol:persistentIdentity <http://ex/derived> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/derived/c> ;
    prov:wasDerivedFrom <http://ex/cda> .
<http://ex/derived/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/derived/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/v> ;
    prov:wasDerivedFrom <http://ex/t/c> .
"#;

const NON_REPLACED_REALIZED: &str = r#"<http://ex/cda> a sbol:CombinatorialDerivation ;
    sbol:displayId "cda" ;
    sbol:persistentIdentity <http://ex/cda> ;
    sbol:template <http://ex/t> ;
    sbol:variableComponent <http://ex/cda/vc> .
<http://ex/cda/vc> a sbol:VariableComponent ;
    sbol:displayId "vc" ;
    sbol:persistentIdentity <http://ex/cda/vc> ;
    sbol:variable <http://ex/t/c> ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variant <http://ex/v> .
<http://ex/t> a sbol:ComponentDefinition ;
    sbol:displayId "t" ;
    sbol:persistentIdentity <http://ex/t> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/t/c>, <http://ex/t/c2> .
<http://ex/t/c> a sbol:Component ;
    sbol:displayId "tc" ;
    sbol:persistentIdentity <http://ex/t/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> .
<http://ex/t/c2> a sbol:Component ;
    sbol:displayId "tc2" ;
    sbol:persistentIdentity <http://ex/t/c2> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> .
<http://ex/derived> a sbol:ComponentDefinition ;
    sbol:displayId "derived" ;
    sbol:persistentIdentity <http://ex/derived> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/derived/c>, <http://ex/derived/c2> ;
    prov:wasDerivedFrom <http://ex/cda> .
<http://ex/derived/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/derived/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/v> ;
    prov:wasDerivedFrom <http://ex/t/c> .
<http://ex/derived/c2> a sbol:Component ;
    sbol:displayId "c2" ;
    sbol:persistentIdentity <http://ex/derived/c2> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/tcd> ;
    prov:wasDerivedFrom <http://ex/t/c2> .
"#;
