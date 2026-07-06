//! Integration tests for SBOL 2 → SBOL 3 Interface synthesis and
//! MapsTo decomposition.

mod common;

use common::upgrade::*;

use sbol_convert::UpgradeWarning;
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
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/cd/Interface")
                && t.predicate.as_str() == "http://sbols.org/v3#nondirectional"
                && t.object.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/cd/sub")
        }),
        "public SBOL 2 Component should become an Interface.nondirectional feature"
    );
}

#[test]
fn public_none_functional_component_synthesizes_nondirectional_interface() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://example.org/lab/md/1>
    a sbol:ModuleDefinition ;
    sbol:persistentIdentity <https://example.org/lab/md> ;
    sbol:displayId "md" ;
    sbol:version "1" ;
    sbol:functionalComponent <https://example.org/lab/md/fc/1> .

<https://example.org/lab/md/fc/1>
    a sbol:FunctionalComponent ;
    sbol:persistentIdentity <https://example.org/lab/md/fc> ;
    sbol:displayId "fc" ;
    sbol:version "1" ;
    sbol:definition <https://example.org/lab/cd/1> ;
    sbol:access sbol:public ;
    sbol:direction sbol:none .

<https://example.org/lab/cd/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna .
"#;
    let (document, report) =
        sbol_convert::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    assert!(report.is_clean(), "warnings: {:?}", report.warnings());

    let upgraded = document.rdf_graph().triples();
    let has_nondirectional = upgraded.iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/md/Interface")
            && t.predicate.as_str() == "http://sbols.org/v3#nondirectional"
            && t.object.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/md/fc")
    });
    assert!(
        has_nondirectional,
        "public direction=none FunctionalComponent should become Interface.nondirectional"
    );

    let (downgraded, dreport) = sbol_convert::downgrade(&document).expect("downgrade");
    assert!(dreport.is_clean(), "warnings: {:?}", dreport.warnings());
    let has_none = downgraded.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/md/fc/1")
            && t.predicate.as_str() == "http://sbols.org/v2#direction"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v2#none")
    });
    let has_inout = downgraded.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/md/fc/1")
            && t.predicate.as_str() == "http://sbols.org/v2#direction"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v2#inout")
    });
    assert!(has_none, "original sbol2:direction none should be restored");
    assert!(
        !has_inout,
        "restored direction must not be contradicted by Interface-derived inout"
    );
}

#[test]
fn synthesized_interface_avoids_existing_child_iri() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://example.org/lab/md/1>
    a sbol:ModuleDefinition ;
    sbol:persistentIdentity <https://example.org/lab/md> ;
    sbol:displayId "md" ;
    sbol:version "1" ;
    sbol:functionalComponent <https://example.org/lab/md/Interface/1> .

<https://example.org/lab/md/Interface/1>
    a sbol:FunctionalComponent ;
    sbol:persistentIdentity <https://example.org/lab/md/Interface> ;
    sbol:displayId "Interface" ;
    sbol:version "1" ;
    sbol:definition <https://example.org/lab/cd/1> ;
    sbol:access sbol:public ;
    sbol:direction sbol:in .

<https://example.org/lab/cd/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna .
"#;
    let (document, report) =
        sbol_convert::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    assert!(report.is_clean(), "warnings: {:?}", report.warnings());
    let triples = document.rdf_graph().triples();

    let collision_is_interface = triples.iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/md/Interface")
            && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v3#Interface")
    });
    let disambiguated_interface = triples.iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/md/Interface_2")
            && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v3#Interface")
    });
    assert!(
        !collision_is_interface,
        "synthesized Interface reused an existing child IRI"
    );
    assert!(
        disambiguated_interface,
        "synthesized Interface should be allocated at Interface_2"
    );
}

#[test]
fn mapsto_synthesis_avoids_existing_child_iri_and_restores_display_id() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://example.org/lab/md/1>
    a sbol:ModuleDefinition ;
    sbol:persistentIdentity <https://example.org/lab/md> ;
    sbol:displayId "md" ;
    sbol:version "1" ;
    sbol:functionalComponent <https://example.org/lab/md/collision/1> ;
    sbol:functionalComponent <https://example.org/lab/md/carrier/1> .

<https://example.org/lab/md/collision/1>
    a sbol:FunctionalComponent ;
    sbol:persistentIdentity <https://example.org/lab/md/collision> ;
    sbol:displayId "collision" ;
    sbol:version "1" ;
    sbol:definition <https://example.org/lab/cd/1> ;
    sbol:access sbol:public ;
    sbol:direction sbol:none .

<https://example.org/lab/md/carrier/1>
    a sbol:FunctionalComponent ;
    sbol:persistentIdentity <https://example.org/lab/md/carrier> ;
    sbol:displayId "carrier" ;
    sbol:version "1" ;
    sbol:definition <https://example.org/lab/cd/1> ;
    sbol:access sbol:public ;
    sbol:direction sbol:none ;
    sbol:mapsTo <https://example.org/lab/md/carrier/collision/1> .

<https://example.org/lab/md/carrier/collision/1>
    a sbol:MapsTo ;
    sbol:persistentIdentity <https://example.org/lab/md/carrier/collision> ;
    sbol:displayId "collision" ;
    sbol:version "1" ;
    sbol:local <https://example.org/lab/md/collision/1> ;
    sbol:remote <https://example.org/lab/md/collision/1> ;
    sbol:refinement sbol:verifyIdentical .

<https://example.org/lab/cd/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna .
"#;
    let (document, report) =
        sbol_convert::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    assert!(report.is_clean(), "warnings: {:?}", report.warnings());
    let triples = document.rdf_graph().triples();
    let collision_is_cref = triples.iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/md/collision")
            && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str())
                == Some("http://sbols.org/v3#ComponentReference")
    });
    let disambiguated_cref = triples.iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/md/collision_2")
            && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str())
                == Some("http://sbols.org/v3#ComponentReference")
    });
    let display_id_hint = triples.iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/md/collision_2")
            && t.predicate.as_str() == "http://sboltools.org/backport#mapsToDisplayId"
            && t.object.as_literal().map(|l| l.value()) == Some("collision")
    });
    assert!(
        !collision_is_cref,
        "MapsTo ComponentReference reused an existing child IRI"
    );
    assert!(
        disambiguated_cref,
        "MapsTo ComponentReference should be allocated at collision_2"
    );
    assert!(
        display_id_hint,
        "renamed MapsTo ComponentReference should preserve original displayId"
    );

    let (downgraded, dreport) = sbol_convert::downgrade(&document).expect("downgrade");
    assert!(dreport.is_clean(), "warnings: {:?}", dreport.warnings());
    let restored_display_id = downgraded.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str())
            == Some("https://example.org/lab/md/carrier/collision/1")
            && t.predicate.as_str() == "http://sbols.org/v2#displayId"
            && t.object.as_literal().map(|l| l.value()) == Some("collision")
    });
    assert!(
        restored_display_id,
        "downgrade should reconstruct the original MapsTo displayId"
    );
}

#[test]
fn synthesized_interface_avoids_mapsto_component_reference_iri() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://example.org/lab/md/1>
    a sbol:ModuleDefinition ;
    sbol:persistentIdentity <https://example.org/lab/md> ;
    sbol:displayId "md" ;
    sbol:version "1" ;
    sbol:functionalComponent <https://example.org/lab/md/local/1> ;
    sbol:functionalComponent <https://example.org/lab/md/carrier/1> .

<https://example.org/lab/md/local/1>
    a sbol:FunctionalComponent ;
    sbol:persistentIdentity <https://example.org/lab/md/local> ;
    sbol:displayId "local" ;
    sbol:version "1" ;
    sbol:definition <https://example.org/lab/cd/1> ;
    sbol:access sbol:public ;
    sbol:direction sbol:none .

<https://example.org/lab/md/carrier/1>
    a sbol:FunctionalComponent ;
    sbol:persistentIdentity <https://example.org/lab/md/carrier> ;
    sbol:displayId "carrier" ;
    sbol:version "1" ;
    sbol:definition <https://example.org/lab/cd/1> ;
    sbol:access sbol:public ;
    sbol:direction sbol:in ;
    sbol:mapsTo <https://example.org/lab/md/carrier/Interface/1> .

<https://example.org/lab/md/carrier/Interface/1>
    a sbol:MapsTo ;
    sbol:persistentIdentity <https://example.org/lab/md/carrier/Interface> ;
    sbol:displayId "Interface" ;
    sbol:version "1" ;
    sbol:local <https://example.org/lab/md/local/1> ;
    sbol:remote <https://example.org/lab/md/local/1> ;
    sbol:refinement sbol:verifyIdentical .

<https://example.org/lab/cd/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna .
"#;
    let (document, report) =
        sbol_convert::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    assert!(report.is_clean(), "warnings: {:?}", report.warnings());
    let triples = document.rdf_graph().triples();

    let mapsto_cref_at_interface = triples.iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/md/Interface")
            && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str())
                == Some("http://sbols.org/v3#ComponentReference")
    });
    let synthesized_interface_disambiguated = triples.iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/md/Interface_2")
            && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v3#Interface")
    });
    assert!(
        mapsto_cref_at_interface,
        "MapsTo ComponentReference should claim the base Interface displayId"
    );
    assert!(
        synthesized_interface_disambiguated,
        "later Interface synthesis must avoid the earlier MapsTo ComponentReference IRI"
    );
}

#[test]
fn mapsto_merge_refinement_round_trips_as_use_remote() {
    // SBOL 3.1.0 §10.2 directs converters to treat `sbol2:merge` as
    // `sbol2:useRemote`. The upgrade therefore emits a `replaces`
    // restriction (with the CRef in subject position) and preserves the
    // original `merge` IRI under `backport:mapsToRefinement` so a
    // downgrade restores the exact source refinement.
    let path = workspace_fixture("mapsto_merge.ttl");
    let input = std::fs::read_to_string(&path).unwrap();
    let (document, report) =
        sbol_convert::upgrade_from_sbol2(&input, RdfFormat::Turtle).expect("upgrade");
    assert!(
        !report
            .warnings()
            .iter()
            .any(|w| matches!(w, UpgradeWarning::UnsupportedRefinement { .. })),
        "merge should map cleanly to replaces+useRemote per SBOL 3.1.0 §10.2, got: {:?}",
        report.warnings()
    );
    let preserved_merge = document.rdf_graph().triples().iter().any(|t| {
        t.predicate.as_str() == "http://sboltools.org/backport#mapsToRefinement"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v2#merge")
    });
    assert!(
        preserved_merge,
        "upgrade must preserve the original `sbol2:merge` refinement under backport:mapsToRefinement",
    );

    // Round-trip back to SBOL 2 and confirm the merge refinement is
    // restored verbatim, not silently coerced to useRemote.
    let (downgraded, _dreport) = sbol_convert::downgrade(&document).expect("downgrade");
    let restored_merge = downgraded.triples().iter().any(|t| {
        t.predicate.as_str() == "http://sbols.org/v2#refinement"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v2#merge")
    });
    assert!(
        restored_merge,
        "downgrade must restore the original sbol2:merge refinement from the backport hint",
    );
}
