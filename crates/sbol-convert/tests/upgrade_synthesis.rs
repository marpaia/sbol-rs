//! Integration tests for SBOL 2 → SBOL 3 Interface synthesis and
//! MapsTo decomposition.

mod common;

use sbol3::RdfFormat;

#[test]
fn public_component_in_component_definition_enters_interface() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://example.org/lab/cd/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna ;
    sbol:component <https://example.org/lab/cd/sub/1> .

<https://example.org/lab/cd/sub/1>
    a sbol:Component ;
    sbol:persistentIdentity <https://example.org/lab/cd/sub> ;
    sbol:displayId "sub" ;
    sbol:version "1" ;
    sbol:definition <https://example.org/lab/part/1> ;
    sbol:access <http://sbols.org/v2#public> .

<https://example.org/lab/part/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/part> ;
    sbol:displayId "part" ;
    sbol:version "1" ;
    sbol:type biopax:Dna .
"#;
    let (document, report) =
        sbol_convert::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    assert!(
        report.is_clean(),
        "unexpected warnings: {:?}",
        report.warnings()
    );
    document
        .check()
        .unwrap_or_else(|report| panic!("validation failed: {report:?}"));
    let triples = document.rdf_graph().triples();
    assert!(
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str())
                == Some("https://example.org/lab/1/cd/Interface1")
                && t.predicate.as_str() == "http://sbols.org/v3#nondirectional"
                && t.object.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/1/cd/sub")
        }),
        "public SBOL 2 Component should become an Interface.nondirectional feature"
    );
}
