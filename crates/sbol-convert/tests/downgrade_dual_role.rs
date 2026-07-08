//! SBOL 3 → SBOL 2 single-target Component routing.
//!
//! Each SBOL 3 `Component` maps to exactly one SBOL 2 class: a
//! `ModuleDefinition` when it carries functional signals (interactions, the
//! FunctionalEntity type, or a Module-derived subcomponent), otherwise a
//! `ComponentDefinition`. A Component is never split across both classes.

mod common;

use sbol3::{Document, RdfFormat};

#[test]
fn component_with_structural_and_functional_signals_routes_to_module_definition() {
    // A Component carrying BOTH structural (`hasSequence`, a biopax-derived
    // type) and functional (`hasInteraction`) data downgrades to a single
    // ModuleDefinition — functional signals decide the class, and no
    // ComponentDefinition half is synthesized.
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/dual_role> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/dual_role_seq> ;
    sbol3:hasInteraction <https://lab/dual_role/some_interaction> .

<https://lab/dual_role/some_interaction> a sbol3:Interaction ;
    sbol3:displayId "some_interaction" ;
    sbol3:type <https://identifiers.org/SBO:0000170> .

<https://lab/dual_role_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role_seq" .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");

    let triples = graph.triples();
    let has_type = |subject: &str, ty: &str| {
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some(subject)
                && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
                && t.object.as_iri().map(|i| i.as_str()) == Some(ty)
        })
    };
    assert!(
        has_type(
            "https://lab/dual_role",
            "http://sbols.org/v2#ModuleDefinition"
        ),
        "a Component with functional signals collapses to a single ModuleDefinition"
    );
    assert!(
        !has_type(
            "https://lab/dual_role_component",
            "http://sbols.org/v2#ComponentDefinition",
        ),
        "no synthesized ComponentDefinition half is emitted"
    );
}
