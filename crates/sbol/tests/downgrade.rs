//! Integration tests for the SBOL 3 → SBOL 2 downgrade pipeline.
//!
//! These tests grow with each phase. Phase 1 verifies the skeleton:
//! identity restoration, backport metadata consumption, and graph
//! shape. Phase 2 verifies type and predicate rewrites.

use std::path::PathBuf;

use sbol::{Document, RdfFormat};

fn workspace_fixture(relative: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push(relative);
    path
}

fn has_triple(graph: &sbol::RdfGraph, subject: &str, predicate: &str, object: &str) -> bool {
    graph.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some(subject)
            && t.predicate.as_str() == predicate
            && t.object.as_iri().map(|i| i.as_str()) == Some(object)
    })
}

fn has_literal_triple(
    graph: &sbol::RdfGraph,
    subject: &str,
    predicate: &str,
    object: &str,
) -> bool {
    graph.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some(subject)
            && t.predicate.as_str() == predicate
            && t.object.as_literal().map(|l| l.value()) == Some(object)
    })
}

fn count_triples(graph: &sbol::RdfGraph, subject: &str, predicate: &str, object: &str) -> usize {
    graph
        .triples()
        .iter()
        .filter(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some(subject)
                && t.predicate.as_str() == predicate
                && t.object.as_iri().map(|i| i.as_str()) == Some(object)
        })
        .count()
}

#[test]
fn round_trip_simple_component_definition() {
    let input =
        std::fs::read_to_string(workspace_fixture("tests/fixtures/sbol2/single_cd.ttl")).unwrap();
    let (upgraded, _ureport) = Document::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();

    let (downgraded_graph, dreport) = upgraded.downgrade_to_sbol2().expect("downgrade");
    assert!(
        dreport.is_clean(),
        "downgrade emitted warnings: {:?}",
        dreport.warnings()
    );

    // The downgraded graph should contain an sbol2:ComponentDefinition
    // typed subject and the canonical SBOL 2 persistentIdentity /
    // version triples.
    let triples = downgraded_graph.triples();
    let has_cd_type = triples.iter().any(|t| {
        t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str())
                == Some("http://sbols.org/v2#ComponentDefinition")
    });
    assert!(has_cd_type, "expected sbol2:ComponentDefinition rdf:type");

    let has_persistent_identity = triples
        .iter()
        .any(|t| t.predicate.as_str() == "http://sbols.org/v2#persistentIdentity");
    assert!(has_persistent_identity, "expected sbol2:persistentIdentity");

    let has_version = triples
        .iter()
        .any(|t| t.predicate.as_str() == "http://sbols.org/v2#version");
    assert!(has_version, "expected sbol2:version");
}

#[test]
fn identity_is_versioned_after_downgrade() {
    let input =
        std::fs::read_to_string(workspace_fixture("tests/fixtures/sbol2/single_cd.ttl")).unwrap();
    let (upgraded, _) = Document::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let (graph, _) = upgraded.downgrade_to_sbol2().unwrap();

    // The original fixture has identity `<https://example.org/lab/J23100/1>`
    // (versioned). The upgrade strips `/1`; the downgrade should
    // restore it.
    let triples = graph.triples();
    let found_versioned_subject = triples.iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/J23100/1")
    });
    assert!(
        found_versioned_subject,
        "expected `/1` version suffix restored on top-level identity"
    );
}

#[test]
fn downgrade_emits_well_formed_rdfxml() {
    let input =
        std::fs::read_to_string(workspace_fixture("tests/fixtures/sbol2/single_cd.ttl")).unwrap();
    let (upgraded, _) = Document::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let (graph, _) = upgraded.downgrade_to_sbol2().unwrap();

    // Both Turtle and RDF/XML should serialize cleanly.
    let turtle = graph.write(RdfFormat::Turtle).expect("write turtle");
    assert!(turtle.contains("http://sbols.org/v2#ComponentDefinition"));
    let xml = graph.write(RdfFormat::RdfXml).expect("write rdfxml");
    assert!(xml.contains("ComponentDefinition") || xml.contains("v2#"));
}

#[test]
fn experiment_member_downgrades_to_experimental_data() {
    let ttl = r#"
@prefix sbol: <http://sbols.org/v3#> .

<https://example.org/lab/data> a sbol:ExperimentalData ;
    sbol:displayId "data" ;
    sbol:hasNamespace <https://example.org/lab> .

<https://example.org/lab/exp> a sbol:Collection, sbol:Experiment ;
    sbol:displayId "exp" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:member <https://example.org/lab/data> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");
    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/exp",
            "http://sbols.org/v2#experimentalData",
            "https://example.org/lab/data"
        ),
        "Experiment.member should downgrade to sbol2:experimentalData"
    );
    assert_eq!(
        count_triples(
            &graph,
            "https://example.org/lab/exp",
            "http://sbols.org/v2#member",
            "https://example.org/lab/data"
        ),
        0,
        "Experiment.member should not downgrade to Collection.member"
    );
}

#[test]
fn variable_feature_cardinality_downgrades_to_operator() {
    let ttl = r#"
@prefix sbol: <http://sbols.org/v3#> .

<https://example.org/lab/deriv> a sbol:CombinatorialDerivation ;
    sbol:displayId "deriv" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:template <https://example.org/lab/template> ;
    sbol:strategy sbol:enumerate ;
    sbol:hasVariableFeature <https://example.org/lab/deriv/vc> .

<https://example.org/lab/deriv/vc> a sbol:VariableFeature ;
    sbol:displayId "vc" ;
    sbol:cardinality sbol:one ;
    sbol:variable <https://example.org/lab/template/sub> ;
    sbol:variant <https://example.org/lab/variant> .

<https://example.org/lab/template> a sbol:Component ;
    sbol:displayId "template" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> ;
    sbol:hasFeature <https://example.org/lab/template/sub> .

<https://example.org/lab/template/sub> a sbol:SubComponent ;
    sbol:displayId "sub" ;
    sbol:instanceOf <https://example.org/lab/part> .

<https://example.org/lab/variant> a sbol:Component ;
    sbol:displayId "variant" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> .

<https://example.org/lab/part> a sbol:Component ;
    sbol:displayId "part" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");
    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/deriv/vc",
            "http://sbols.org/v2#operator",
            "http://sbols.org/v2#one"
        ),
        "VariableFeature.cardinality should downgrade to sbol2:operator"
    );
    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/deriv",
            "http://sbols.org/v2#strategy",
            "http://sbols.org/v2#enumerate"
        ),
        "CombinatorialDerivation.strategy values should downgrade to the SBOL 2 namespace"
    );
}

#[test]
fn role_integration_values_downgrade_to_sbol2_namespace() {
    let ttl = r#"
@prefix sbol: <http://sbols.org/v3#> .

<https://example.org/lab/cd> a sbol:Component ;
    sbol:displayId "cd" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> ;
    sbol:hasFeature <https://example.org/lab/cd/sub> .

<https://example.org/lab/cd/sub> a sbol:SubComponent ;
    sbol:displayId "sub" ;
    sbol:instanceOf <https://example.org/lab/part> ;
    sbol:role <https://identifiers.org/SO:0000167> ;
    sbol:roleIntegration sbol:mergeRoles .

<https://example.org/lab/part> a sbol:Component ;
    sbol:displayId "part" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");
    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/cd/sub",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v2#Component"
        ),
        "native structural SubComponents should downgrade to sbol2:Component"
    );
    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/cd/sub",
            "http://sbols.org/v2#roleIntegration",
            "http://sbols.org/v2#mergeRoles"
        ),
        "SubComponent.roleIntegration values should downgrade to the SBOL 2 namespace"
    );
}

#[test]
fn structural_interface_feature_downgrades_to_public_access_not_direction() {
    let ttl = r#"
@prefix sbol: <http://sbols.org/v3#> .
@prefix backport: <http://sboltools.org/backport#> .

<https://example.org/lab/cd> a sbol:Component ;
    sbol:displayId "cd" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> ;
    sbol:hasFeature <https://example.org/lab/cd/sub> ;
    sbol:hasInterface <https://example.org/lab/cd/Interface> ;
    backport:sbol2type <http://sbols.org/v2#ComponentDefinition> .

<https://example.org/lab/cd/sub> a sbol:SubComponent ;
    sbol:displayId "sub" ;
    sbol:instanceOf <https://example.org/lab/part> ;
    backport:sbol2type <http://sbols.org/v2#Component> ;
    backport:sbol2_access <http://sbols.org/v2#public> .

<https://example.org/lab/cd/Interface> a sbol:Interface ;
    sbol:displayId "Interface" ;
    sbol:nondirectional <https://example.org/lab/cd/sub> .

<https://example.org/lab/part> a sbol:Component ;
    sbol:displayId "part" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");
    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/cd/sub",
            "http://sbols.org/v2#access",
            "http://sbols.org/v2#public"
        ),
        "structural Interface.nondirectional features should become sbol2:access public"
    );
    assert_eq!(
        graph
            .triples()
            .iter()
            .filter(|t| {
                t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/cd/sub")
                    && t.predicate.as_str() == "http://sbols.org/v2#direction"
            })
            .count(),
        0,
        "SBOL 2 Component outputs must not receive FunctionalComponent direction"
    );
}

#[test]
fn native_component_instances_receive_sbol2_defaults() {
    let ttl = r#"
@prefix sbol: <http://sbols.org/v3#> .

<https://example.org/lab/cd> a sbol:Component ;
    sbol:displayId "cd" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> ;
    sbol:hasFeature <https://example.org/lab/cd/sub> .

<https://example.org/lab/md> a sbol:Component ;
    sbol:displayId "md" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000241> ;
    sbol:hasFeature <https://example.org/lab/md/fc> .

<https://example.org/lab/cd/sub> a sbol:SubComponent ;
    sbol:displayId "sub" ;
    sbol:instanceOf <https://example.org/lab/part> .

<https://example.org/lab/md/fc> a sbol:SubComponent ;
    sbol:displayId "fc" ;
    sbol:instanceOf <https://example.org/lab/part> .

<https://example.org/lab/part> a sbol:Component ;
    sbol:displayId "part" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/cd/sub",
            "http://sbols.org/v2#access",
            "http://sbols.org/v2#private"
        ),
        "native structural SubComponents should receive the SBOL 2 access default"
    );
    assert_eq!(
        graph
            .triples()
            .iter()
            .filter(|t| {
                t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/cd/sub")
                    && t.predicate.as_str() == "http://sbols.org/v2#direction"
            })
            .count(),
        0,
        "structural Components must not receive FunctionalComponent direction"
    );
    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/md/fc",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v2#FunctionalComponent"
        ),
        "native ModuleDefinition features should downgrade to sbol2:FunctionalComponent"
    );
    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/md/fc",
            "http://sbols.org/v2#access",
            "http://sbols.org/v2#private"
        ),
        "native FunctionalComponents should receive the SBOL 2 access default"
    );
    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/md/fc",
            "http://sbols.org/v2#direction",
            "http://sbols.org/v2#none"
        ),
        "native FunctionalComponents should receive the SBOL 2 direction default"
    );
}

#[test]
fn dual_role_interface_direction_lands_on_functional_component_variant() {
    let ttl = r#"
@prefix sbol: <http://sbols.org/v3#> .

<https://example.org/lab/dual> a sbol:Component ;
    sbol:displayId "dual" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> ;
    sbol:hasSequence <https://example.org/lab/dual_seq> ;
    sbol:hasFeature <https://example.org/lab/dual/sub> ;
    sbol:hasInteraction <https://example.org/lab/dual/interaction> ;
    sbol:hasInterface <https://example.org/lab/dual/interface> .

<https://example.org/lab/dual/sub> a sbol:SubComponent ;
    sbol:displayId "sub" ;
    sbol:instanceOf <https://example.org/lab/part> .

<https://example.org/lab/dual/interface> a sbol:Interface ;
    sbol:displayId "interface" ;
    sbol:input <https://example.org/lab/dual/sub> .

<https://example.org/lab/dual/interaction> a sbol:Interaction ;
    sbol:displayId "interaction" ;
    sbol:type <https://identifiers.org/SBO:0000170> .

<https://example.org/lab/dual_seq> a sbol:Sequence ;
    sbol:displayId "dual_seq" ;
    sbol:hasNamespace <https://example.org/lab> .

<https://example.org/lab/part> a sbol:Component ;
    sbol:displayId "part" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/dual/sub_fc",
            "http://sbols.org/v2#direction",
            "http://sbols.org/v2#in"
        ),
        "dual-role Interface.input should target the MD-side FunctionalComponent variant"
    );
    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/dual/sub_fc",
            "http://sbols.org/v2#access",
            "http://sbols.org/v2#public"
        ),
        "Interface-visible FunctionalComponent variant should be public"
    );
    assert_eq!(
        graph
            .triples()
            .iter()
            .filter(|t| {
                t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/dual/sub")
                    && t.predicate.as_str() == "http://sbols.org/v2#direction"
            })
            .count(),
        0,
        "dual-role CD-side Component variant must not receive direction"
    );
}

#[test]
fn downgrade_preserves_non_sbol_type_with_backport_type_hint() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix backport: <http://sboltools.org/backport#> .
@prefix ex: <https://example.org/types#> .

<https://lab/cd> a sbol3:Component, ex:CustomComponent ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "cd" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    backport:sbol2type <http://sbols.org/v2#ComponentDefinition> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");
    let triples = graph.triples();
    let component_definition_type_count = triples
        .iter()
        .filter(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/cd")
                && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
                && t.object.as_iri().map(|i| i.as_str())
                    == Some("http://sbols.org/v2#ComponentDefinition")
        })
        .count();
    assert_eq!(
        component_definition_type_count, 1,
        "backport type should produce exactly one SBOL 2 class assertion"
    );
    let custom_type_preserved = triples.iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/cd")
            && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str())
                == Some("https://example.org/types#CustomComponent")
    });
    assert!(
        custom_type_preserved,
        "non-SBOL rdf:type should survive downgrade alongside backport type"
    );
}

#[test]
fn downgrade_does_not_apply_backport_type_to_unmatched_sbol3_type() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix backport: <http://sboltools.org/backport#> .

<https://lab/cd> a sbol3:Component, sbol3:FutureThing ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "cd" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    backport:sbol2type <http://sbols.org/v2#ComponentDefinition> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, report) = document.downgrade_to_sbol2().expect("downgrade");
    assert!(
        report.warnings().iter().any(|w| matches!(
            w,
            sbol::DowngradeWarning::UnsupportedSbol3Type { subject, sbol3_type }
                if subject == "https://lab/cd" && sbol3_type == "http://sbols.org/v3#FutureThing"
        )),
        "expected unmatched SBOL 3 type warning, got {:?}",
        report.warnings()
    );
    assert_eq!(
        count_triples(
            &graph,
            "https://lab/cd",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v2#ComponentDefinition",
        ),
        1,
        "backport type should apply only to the matching sbol3:Component class"
    );
    assert_eq!(
        count_triples(
            &graph,
            "https://lab/cd",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v3#FutureThing",
        ),
        0,
        "unsupported SBOL 3 class should be warned and dropped, not restored as a duplicate"
    );
}

#[test]
fn native_subcomponent_locations_emit_sequence_annotation_wrapper() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/design> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "design" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/design_seq> ;
    sbol3:hasFeature <https://lab/design/part> .

<https://lab/design/part> a sbol3:SubComponent ;
    sbol3:displayId "part" ;
    sbol3:instanceOf <https://lab/part_def> ;
    sbol3:hasLocation <https://lab/design/part/range> .

<https://lab/design/part/range> a sbol3:Range ;
    sbol3:displayId "range" ;
    sbol3:hasSequence <https://lab/design_seq> ;
    sbol3:start 1 ;
    sbol3:end 4 .

<https://lab/part_def> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "part_def" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .

<https://lab/design_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "design_seq" .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    assert!(
        has_triple(
            &graph,
            "https://lab/design",
            "http://sbols.org/v2#sequenceAnnotation",
            "https://lab/design/part_annotation",
        ),
        "parent CD should point at synthesized SequenceAnnotation"
    );
    assert!(
        has_triple(
            &graph,
            "https://lab/design/part_annotation",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v2#SequenceAnnotation",
        ),
        "native SubComponent location should synthesize an SBOL 2 SequenceAnnotation"
    );
    assert!(
        has_triple(
            &graph,
            "https://lab/design/part_annotation",
            "http://sbols.org/v2#component",
            "https://lab/design/part",
        ),
        "SequenceAnnotation should point at the downgraded Component"
    );
    assert!(
        has_triple(
            &graph,
            "https://lab/design/part_annotation",
            "http://sbols.org/v2#location",
            "https://lab/design/part/range",
        ),
        "SequenceAnnotation should carry the SubComponent Location"
    );
    assert!(
        !has_triple(
            &graph,
            "https://lab/design/part",
            "http://sbols.org/v2#location",
            "https://lab/design/part/range",
        ),
        "sbol2:Component must not receive location directly"
    );
}

#[test]
fn downgrade_merges_interface_input_and_output_as_inout() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/module> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "module" ;
    sbol3:type <https://identifiers.org/SBO:0000241> ;
    sbol3:hasFeature <https://lab/module/fc> ;
    sbol3:hasInterface <https://lab/module/interface> .

<https://lab/module/fc> a sbol3:SubComponent ;
    sbol3:displayId "fc" ;
    sbol3:instanceOf <https://lab/target> .

<https://lab/module/interface> a sbol3:Interface ;
    sbol3:displayId "interface" ;
    sbol3:input <https://lab/module/fc> ;
    sbol3:output <https://lab/module/fc> .

<https://lab/target> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "target" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");
    let has_inout = graph.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/module/fc")
            && t.predicate.as_str() == "http://sbols.org/v2#direction"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v2#inout")
    });
    assert!(
        has_inout,
        "a feature listed as both Interface.input and Interface.output should downgrade to sbol2:inout"
    );
    let module_is_md = graph.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/module")
            && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v2#ModuleDefinition")
    });
    assert!(
        module_is_md,
        "functional-only native Component should downgrade to sbol2:ModuleDefinition"
    );
}

#[test]
fn downgrade_does_not_version_external_annotation_objects_under_top_level() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix backport: <http://sboltools.org/backport#> .
@prefix ex: <https://example.org/annotation#> .

<https://lab/design> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "design" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    backport:sbol2type <http://sbols.org/v2#ComponentDefinition> ;
    backport:sbol2version "1" ;
    ex:taxon <https://lab/design/taxon/511145> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    let taxon_objects: Vec<&str> = graph
        .triples()
        .iter()
        .filter(|t| t.predicate.as_str() == "https://example.org/annotation#taxon")
        .filter_map(|t| t.object.as_iri().map(|i| i.as_str()))
        .collect();
    assert!(
        taxon_objects.contains(&"https://lab/design/taxon/511145"),
        "external annotation object should remain untouched, got {taxon_objects:?}"
    );
    assert!(
        !taxon_objects.contains(&"https://lab/design/taxon/511145/1"),
        "external annotation object was incorrectly versioned"
    );
    let fabricated_metadata = graph.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/design/taxon/511145/1")
            && t.predicate.as_str() == "http://sbols.org/v2#persistentIdentity"
    });
    assert!(
        !fabricated_metadata,
        "downgrade fabricated SBOL 2 identity metadata for an object-only annotation IRI"
    );
}

#[test]
fn downgrade_drops_unsupported_sbol3_subjects() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/design> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "design" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasFeature <https://lab/design/local> .

<https://lab/design/local> a sbol3:LocalSubComponent ;
    sbol3:displayId "local" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, report) = document.downgrade_to_sbol2().expect("downgrade");
    assert!(
        report.warnings().iter().any(|w| matches!(
            w,
            sbol::DowngradeWarning::UnsupportedSbol3Type { subject, sbol3_type }
                if subject == "https://lab/design/local"
                    && sbol3_type == "http://sbols.org/v3#LocalSubComponent"
        )),
        "expected UnsupportedSbol3Type warning, got {:?}",
        report.warnings()
    );
    let emitted_local_subject = graph
        .triples()
        .iter()
        .any(|t| t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/design/local"));
    assert!(
        !emitted_local_subject,
        "unsupported LocalSubComponent subject leaked into SBOL 2 output"
    );
    let emitted_parent_pointer = graph.triples().iter().any(|t| {
        t.predicate.as_str() == "http://sbols.org/v2#component"
            && t.object.as_iri().map(|i| i.as_str()) == Some("https://lab/design/local")
    });
    assert!(
        !emitted_parent_pointer,
        "parent still points at dropped LocalSubComponent"
    );
}

#[test]
fn downgrade_honors_split_dual_role_components_false() {
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
    let mut options = sbol::DowngradeOptions::default();
    options.split_dual_role_components = false;
    let (graph, report) = document
        .downgrade_to_sbol2_with(options)
        .expect("downgrade");

    assert!(
        !report
            .warnings()
            .iter()
            .any(|w| matches!(w, sbol::DowngradeWarning::DualRoleComponent { .. })),
        "split_dual_role_components=false should not emit a dual-role split warning: {:?}",
        report.warnings()
    );
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
        "dual-role Component should collapse to a single MD when splitting is disabled"
    );
    assert!(
        !has_type(
            "https://lab/dual_role_component",
            "http://sbols.org/v2#ComponentDefinition",
        ),
        "split_dual_role_components=false still emitted the synthesized CD half"
    );
}

#[test]
fn downgrade_uses_longest_top_level_prefix_for_child_versions() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix backport: <http://sboltools.org/backport#> .

<https://lab/root> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "root" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    backport:sbol2type <http://sbols.org/v2#ComponentDefinition> ;
    backport:sbol2version "1" .

<https://lab/root/inner> a sbol3:Component ;
    sbol3:hasNamespace <https://lab/root> ;
    sbol3:displayId "inner" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasFeature <https://lab/root/inner/sub> ;
    backport:sbol2type <http://sbols.org/v2#ComponentDefinition> ;
    backport:sbol2version "2" .

<https://lab/root/inner/sub> a sbol3:SubComponent ;
    sbol3:displayId "sub" ;
    sbol3:instanceOf <https://lab/root> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");
    let saw_inner_child_version = graph
        .triples()
        .iter()
        .any(|t| t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/root/inner/sub/2"));
    assert!(
        saw_inner_child_version,
        "child under nested top-level should inherit the longest matching top-level version"
    );
    let saw_outer_child_version = graph
        .triples()
        .iter()
        .any(|t| t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/root/inner/sub/1"));
    assert!(
        !saw_outer_child_version,
        "child under nested top-level inherited the shorter outer top-level version"
    );
}

#[test]
fn downgrade_component_shape_uses_all_rdf_types() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix ex: <https://example.org/types#> .

<https://lab/module> a ex:Custom, sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "module" ;
    sbol3:type <https://identifiers.org/SBO:0000241> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");
    let module_is_md = graph.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/module")
            && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v2#ModuleDefinition")
    });
    assert!(
        module_is_md,
        "custom rdf:type must not hide the SBOL Component class during shape classification"
    );
}

#[test]
fn downgrade_mapsto_discovery_uses_all_rdf_types() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix backport: <http://sboltools.org/backport#> .
@prefix ex: <https://example.org/types#> .

<https://lab/module> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "module" ;
    sbol3:type <https://identifiers.org/SBO:0000241> ;
    sbol3:hasFeature <https://lab/module/carrier> ;
    sbol3:hasFeature <https://lab/module/local> ;
    sbol3:hasConstraint <https://lab/module/map_constraint> .

<https://lab/module/carrier> a sbol3:SubComponent ;
    sbol3:displayId "carrier" ;
    sbol3:instanceOf <https://lab/target> .

<https://lab/module/local> a sbol3:SubComponent ;
    sbol3:displayId "local" ;
    sbol3:instanceOf <https://lab/target> .

<https://lab/module/map> a sbol3:ComponentReference .
<https://lab/module/map> a ex:CustomReference ;
    sbol3:displayId "map" ;
    sbol3:inChildOf <https://lab/module/carrier> ;
    sbol3:refersTo <https://lab/module/local> ;
    backport:mapsToRefinement <http://sbols.org/v2#verifyIdentical> .

<https://lab/module/map_constraint> a sbol3:Constraint .
<https://lab/module/map_constraint> a ex:CustomConstraint ;
    sbol3:subject <https://lab/module/local> ;
    sbol3:object <https://lab/module/map> ;
    sbol3:restriction sbol3:verifyIdentical .

<https://lab/target> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "target" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, report) = document.downgrade_to_sbol2().expect("downgrade");
    assert!(
        !report.warnings().iter().any(|w| matches!(
            w,
            sbol::DowngradeWarning::OrphanComponentReference { .. }
                | sbol::DowngradeWarning::UnresolvableConstraintToMapsTo { .. }
        )),
        "MapsTo pair should be recognized despite extension rdf:types: {:?}",
        report.warnings()
    );
    let has_maps_to = graph.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/module/carrier")
            && t.predicate.as_str() == "http://sbols.org/v2#mapsTo"
            && t.object.as_iri().map(|i| i.as_str()) == Some("https://lab/module/carrier/map")
    });
    assert!(
        has_maps_to,
        "custom rdf:type must not hide ComponentReference/Constraint classes during MapsTo discovery"
    );
}

/// The killer test: take an SBOL 2 fixture, upgrade to SBOL 3,
/// downgrade back to SBOL 2, and diff against the original. Anything
/// that doesn't survive the round trip is either a bug in one
/// direction or an intentional documented divergence.
fn round_trip_diff(fixture: &str) -> (Vec<String>, Vec<String>) {
    let input = std::fs::read_to_string(workspace_fixture(fixture)).unwrap();
    let original_graph = sbol::RdfGraph::parse(&input, RdfFormat::Turtle).expect("parse original");
    let original: Vec<String> = canonicalize(&original_graph);

    let (upgraded, _ureport) =
        Document::upgrade_from_sbol2(&input, RdfFormat::Turtle).expect("upgrade");
    let (downgraded_graph, _dreport) = upgraded.downgrade_to_sbol2().expect("downgrade");
    let downgraded: Vec<String> = canonicalize(&downgraded_graph);

    let only_in_original: Vec<String> = original
        .iter()
        .filter(|t| !downgraded.contains(t))
        .cloned()
        .collect();
    let only_in_downgraded: Vec<String> = downgraded
        .iter()
        .filter(|t| !original.contains(t))
        .cloned()
        .collect();
    (only_in_original, only_in_downgraded)
}

fn canonicalize(graph: &sbol::RdfGraph) -> Vec<String> {
    let mut lines: Vec<String> = graph
        .normalized_triples()
        .iter()
        .map(sbol::upgrade::canonical_nt_line)
        .collect();
    lines.sort();
    lines.dedup();
    lines
}

#[test]
fn round_trip_single_cd() {
    // The fixture encodes `biopax:Dna` (bare, not `DnaRegion`). The
    // BioPAX collapse on upgrade is reversed via `backport:biopaxType`,
    // so the round-trip is lossless without filtering.
    let (lost, gained) = round_trip_diff("tests/fixtures/sbol2/single_cd.ttl");
    assert!(
        lost.is_empty() && gained.is_empty(),
        "round-trip drift on single_cd.ttl\nonly in original:\n{}\nonly in downgrade:\n{}",
        lost.join("\n"),
        gained.join("\n")
    );
}

#[test]
fn round_trip_cd_with_subparts() {
    let (lost, gained) = round_trip_diff("tests/fixtures/sbol2/cd_with_subparts.ttl");
    assert!(
        lost.is_empty() && gained.is_empty(),
        "round-trip drift on cd_with_subparts.ttl\nonly in original:\n{}\nonly in downgrade:\n{}",
        lost.join("\n"),
        gained.join("\n")
    );
}

#[test]
fn round_trip_md_simple_structurally() {
    // md_simple has Interactions and Participations — Phase 3 should
    // restore them with the correct MD predicates. Check the
    // structural shape (counts of each rdf:type) rather than exact
    // triple equality, since the Phase 3 reconstruction may still
    // have ordering / annotation differences.
    let input =
        std::fs::read_to_string(workspace_fixture("tests/fixtures/sbol2/md_simple.ttl")).unwrap();
    let original = sbol::RdfGraph::parse(&input, RdfFormat::Turtle).unwrap();
    let (upgraded, _) = Document::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let (downgraded, _) = upgraded.downgrade_to_sbol2().unwrap();

    let count_types = |g: &sbol::RdfGraph| -> std::collections::BTreeMap<String, usize> {
        let mut out = std::collections::BTreeMap::new();
        for t in g.triples() {
            if t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
                && let Some(iri) = t.object.as_iri()
            {
                *out.entry(iri.as_str().to_owned()).or_insert(0) += 1;
            }
        }
        out
    };
    let original_types = count_types(&original);
    let downgraded_types = count_types(&downgraded);
    for (type_iri, expected_count) in &original_types {
        let actual = downgraded_types.get(type_iri).copied().unwrap_or(0);
        assert!(
            actual >= *expected_count,
            "round-trip lost some {type_iri}: original {expected_count}, downgrade {actual}"
        );
    }
}

#[test]
fn round_trip_cd_with_annotation_preserves_range_count() {
    // SequenceAnnotation reconstruction: the upgrade collapses
    // SA-with-component into SubComponent.hasLocation. Phase 3
    // restoration reverses this. For now the Range count should
    // survive even if the SA wrapper hierarchy changes shape.
    let input = std::fs::read_to_string(workspace_fixture(
        "tests/fixtures/sbol2/cd_with_annotation.ttl",
    ))
    .unwrap();
    let original = sbol::RdfGraph::parse(&input, RdfFormat::Turtle).unwrap();
    let (upgraded, _) = Document::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let (downgraded, _) = upgraded.downgrade_to_sbol2().unwrap();

    let count_ranges = |g: &sbol::RdfGraph| -> usize {
        g.triples()
            .iter()
            .filter(|t| {
                t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
                    && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v2#Range")
            })
            .count()
    };
    assert_eq!(
        count_ranges(&downgraded),
        count_ranges(&original),
        "Range count drift in round-trip"
    );
}

/// Round-trips a real iGEM design via SynBioHub. Exercises the SA
/// reconstruction path — every Range originates from an
/// SA-with-component collapse, and the downgrade must rebuild the
/// SequenceAnnotation wrapper so the re-upgrade re-collapses to the
/// same triple set.
#[test]
fn round_trip_real_synbiohub_sa_reconstruction() {
    let input = std::fs::read_to_string(workspace_fixture(
        "tests/fixtures/sbol2/real/synbiohub/BBa_F2620.xml",
    ))
    .unwrap();
    let (upgraded, _) = Document::upgrade_from_sbol2(&input, RdfFormat::RdfXml).expect("upgrade");
    let original = canonicalize(upgraded.rdf_graph());
    let (downgraded, _) = upgraded.downgrade_to_sbol2().expect("downgrade");
    let turtle = downgraded.write(RdfFormat::Turtle).expect("write turtle");
    let (reupgraded, _) =
        Document::upgrade_from_sbol2(&turtle, RdfFormat::Turtle).expect("re-upgrade");
    let after = canonicalize(reupgraded.rdf_graph());
    let lost: Vec<&String> = original.iter().filter(|t| !after.contains(t)).collect();
    let gained: Vec<&String> = after.iter().filter(|t| !original.contains(t)).collect();
    assert!(
        lost.is_empty() && gained.is_empty(),
        "BBa_F2620 round-trip drifted: lost {}, gained {}\nFirst lost:\n{}\nFirst gained:\n{}",
        lost.len(),
        gained.len(),
        lost.iter()
            .take(3)
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        gained
            .iter()
            .take(3)
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

/// Round-trips the CRISPR repression model, exercising MapsTo
/// decomposition (ComponentReference + Constraint pairs) AND Interface
/// decomposition (FunctionalComponent directions folded into
/// `nondirectional` lists). Both must reverse cleanly for the SBOL 2
/// triple set to survive a full round trip.
#[test]
fn round_trip_real_repression_model_mapsto_and_interface() {
    let input = std::fs::read_to_string(workspace_fixture(
        "tests/fixtures/sbol2/real/RepressionModel.xml",
    ))
    .unwrap();
    let (upgraded, _) = Document::upgrade_from_sbol2(&input, RdfFormat::RdfXml).expect("upgrade");
    let original = canonicalize(upgraded.rdf_graph());
    let (downgraded, dreport) = upgraded.downgrade_to_sbol2().expect("downgrade");
    assert!(
        dreport.counts().maps_to_reconstructed >= 5,
        "expected at least 5 MapsTo reconstructions, got {}",
        dreport.counts().maps_to_reconstructed,
    );
    let turtle = downgraded.write(RdfFormat::Turtle).expect("write turtle");
    let (reupgraded, _) =
        Document::upgrade_from_sbol2(&turtle, RdfFormat::Turtle).expect("re-upgrade");
    let after = canonicalize(reupgraded.rdf_graph());
    let lost: Vec<&String> = original.iter().filter(|t| !after.contains(t)).collect();
    let gained: Vec<&String> = after.iter().filter(|t| !original.contains(t)).collect();
    assert!(
        lost.is_empty() && gained.is_empty(),
        "RepressionModel round-trip drifted: lost {}, gained {}\nFirst lost:\n{}\nFirst gained:\n{}",
        lost.len(),
        gained.len(),
        lost.iter()
            .take(3)
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        gained
            .iter()
            .take(3)
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

/// Native SBOL 3 dual-role Component: carries both structural
/// (`sbol3:hasSequence`, biopax type) AND functional
/// (`sbol3:hasInteraction`) data, so it splits on downgrade into a CD
/// holding the structural triples and an MD holding the functional
/// triples, plus a synthesized linking FunctionalComponent.
#[test]
fn dual_role_component_splits_into_cd_and_md() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .
@prefix sbo: <https://identifiers.org/SBO:> .

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
    let (graph, report) = document.downgrade_to_sbol2().expect("downgrade");

    assert_eq!(
        report.counts().components_split_into_both,
        1,
        "expected one dual-role split, got counts={:?}",
        report.counts()
    );
    assert!(
        report
            .warnings()
            .iter()
            .any(|w| matches!(w, sbol::DowngradeWarning::DualRoleComponent { .. })),
        "expected a DualRoleComponent warning, got {:?}",
        report.warnings()
    );

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
            "https://lab/dual_role_component",
            "http://sbols.org/v2#ComponentDefinition",
        ),
        "expected sbol2:ComponentDefinition on the _component half"
    );
    assert!(
        has_type(
            "https://lab/dual_role",
            "http://sbols.org/v2#ModuleDefinition"
        ),
        "expected sbol2:ModuleDefinition on the bare IRI (heuristic: interactions present)"
    );
    assert!(
        has_type(
            "https://lab/dual_role/dual_role",
            "http://sbols.org/v2#FunctionalComponent",
        ),
        "expected synthesized linking FunctionalComponent"
    );

    // The CD carries the structural triples; the MD carries the
    // functional ones; the linking FC carries the SplitComponentComposition
    // marker so a future re-upgrade can detect the split origin.
    let cd_has = |predicate: &str| {
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/dual_role_component")
                && t.predicate.as_str() == predicate
        })
    };
    let md_has = |predicate: &str| {
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/dual_role")
                && t.predicate.as_str() == predicate
        })
    };
    assert!(
        cd_has("http://sbols.org/v2#sequence"),
        "CD missing sbol2:sequence"
    );
    assert!(cd_has("http://sbols.org/v2#type"), "CD missing sbol2:type");
    assert!(
        md_has("http://sbols.org/v2#interaction"),
        "MD missing sbol2:interaction"
    );
    assert!(
        md_has("http://sbols.org/v2#functionalComponent"),
        "MD missing the linking FunctionalComponent"
    );
    let has_split_marker = triples.iter().any(|t| {
        t.predicate.as_str() == "http://sboltools.org/backport#type"
            && t.object.as_iri().map(|i| i.as_str())
                == Some("http://sboltools.org/backport#SplitComponentComposition")
    });
    assert!(
        has_split_marker,
        "expected SplitComponentComposition marker on linking FC"
    );

    // Both halves carry backport:sbol3identity pointing at the original
    // SBOL 3 Component IRI so the inverse direction can re-merge.
    let sbol3id_count = triples
        .iter()
        .filter(|t| {
            t.predicate.as_str() == "http://sboltools.org/backport#sbol3identity"
                && t.object.as_iri().map(|i| i.as_str()) == Some("https://lab/dual_role")
        })
        .count();
    assert_eq!(
        sbol3id_count, 2,
        "expected 2 backport:sbol3identity stamps (one per half)"
    );
}

#[test]
fn split_subjects_preserve_extension_types_and_archive_unknown_sbol3_predicates() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix ex: <https://example.org/types#> .

<https://lab/dual_role> a sbol3:Component, ex:CustomDual ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/dual_role_seq> ;
    sbol3:hasFeature <https://lab/dual_role/sc> ;
    sbol3:hasInteraction <https://lab/dual_role/inter> ;
    sbol3:futureThing <https://example.org/value> .

<https://lab/dual_role/sc> a sbol3:SubComponent, ex:CustomSub ;
    sbol3:displayId "sc" ;
    sbol3:instanceOf <https://lab/target> ;
    sbol3:futureSub <https://example.org/subvalue> .

<https://lab/target> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "target" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .

<https://lab/dual_role/inter> a sbol3:Interaction ;
    sbol3:displayId "inter" ;
    sbol3:type <https://identifiers.org/SBO:0000170> .

<https://lab/dual_role_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role_seq" .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, report) = document.downgrade_to_sbol2().expect("downgrade");
    let dual_warnings = report
        .warnings()
        .iter()
        .filter(|w| matches!(w, sbol::DowngradeWarning::DualRoleComponent { .. }))
        .count();
    assert_eq!(report.counts().components_split_into_both, 1);
    assert_eq!(dual_warnings, 1, "dual-role warning should not duplicate");

    assert_eq!(
        count_triples(
            &graph,
            "https://lab/dual_role_component",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v2#ComponentDefinition",
        ),
        1,
        "CD split type should be emitted once"
    );
    assert_eq!(
        count_triples(
            &graph,
            "https://lab/dual_role",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v2#ModuleDefinition",
        ),
        1,
        "MD split type should be emitted once"
    );
    assert_eq!(
        count_triples(
            &graph,
            "https://lab/dual_role",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "https://example.org/types#CustomDual",
        ),
        1,
        "dual-role Component extension rdf:type should survive on the bare half"
    );
    assert_eq!(
        count_triples(
            &graph,
            "https://lab/dual_role/sc",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v2#Component",
        ),
        1,
        "SubComponent Component split type should be emitted once"
    );
    assert_eq!(
        count_triples(
            &graph,
            "https://lab/dual_role/sc_fc",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v2#FunctionalComponent",
        ),
        1,
        "SubComponent FunctionalComponent split type should be emitted once"
    );
    assert_eq!(
        count_triples(
            &graph,
            "https://lab/dual_role/sc",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "https://example.org/types#CustomSub",
        ),
        1,
        "split SubComponent extension rdf:type should survive on the bare variant"
    );
    assert!(
        has_triple(
            &graph,
            "https://lab/dual_role",
            "http://sboltools.org/backport#sbol3_futureThing",
            "https://example.org/value",
        ),
        "unknown SBOL 3 predicate on dual-role Component should be archived"
    );
    assert!(
        has_triple(
            &graph,
            "https://lab/dual_role/sc",
            "http://sboltools.org/backport#sbol3_futureSub",
            "https://example.org/subvalue",
        ),
        "unknown SBOL 3 predicate on split SubComponent should be archived"
    );
}

/// SubComponents under a dual-role parent triple into three SBOL 2
/// variants: an `sbol2:Component` under the CD half, an
/// `sbol2:FunctionalComponent` under the MD half, and (when the target
/// is an MD-shaped Component) an `sbol2:Module` under the MD half.
/// Each variant gets its own `sbol2:definition` plus identified
/// properties so all three are valid SBOL 2 objects.
#[test]
fn dual_role_subcomponent_triples_into_three_variants() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/dual_role> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/seq> ;
    sbol3:hasFeature <https://lab/dual_role/sc> ;
    sbol3:hasInteraction <https://lab/dual_role/inter> .

<https://lab/dual_role/sc> a sbol3:SubComponent ;
    sbol3:displayId "sc" ;
    sbol3:instanceOf <https://lab/target> .

<https://lab/target> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "target" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .

<https://lab/dual_role/inter> a sbol3:Interaction ;
    sbol3:displayId "inter" ;
    sbol3:type <https://identifiers.org/SBO:0000170> .

<https://lab/seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "seq" .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    let triples = graph.triples();
    let has_typed_subject = |iri: &str, ty: &str| {
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some(iri)
                && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
                && t.object.as_iri().map(|i| i.as_str()) == Some(ty)
        })
    };

    // The target is CD-only (just type+namespace), so its SubComponent
    // doesn't get a Module variant. Component (C) keeps the bare IRI;
    // FunctionalComponent gets the `_fc` suffix.
    assert!(
        has_typed_subject("https://lab/dual_role/sc", "http://sbols.org/v2#Component"),
        "expected sbol2:Component on bare SubComponent IRI"
    );
    assert!(
        has_typed_subject(
            "https://lab/dual_role/sc_fc",
            "http://sbols.org/v2#FunctionalComponent",
        ),
        "expected sbol2:FunctionalComponent on `_fc` variant"
    );

    // Each variant carries its own definition pointing at the target's
    // CD half.
    let target_cd = "https://lab/target";
    let count_definition = |subject: &str| {
        triples
            .iter()
            .filter(|t| {
                t.subject.as_iri().map(|i| i.as_str()) == Some(subject)
                    && t.predicate.as_str() == "http://sbols.org/v2#definition"
                    && t.object.as_iri().map(|i| i.as_str()) == Some(target_cd)
            })
            .count()
    };
    assert_eq!(
        count_definition("https://lab/dual_role/sc"),
        1,
        "C variant missing sbol2:definition → target"
    );
    assert_eq!(
        count_definition("https://lab/dual_role/sc_fc"),
        1,
        "FC variant missing sbol2:definition → target"
    );
}

/// A Collection that lists a dual-role Component as a member must
/// reference BOTH halves of the split in the SBOL 2 output. Otherwise
/// the SBOL 2 Collection only sees the structural OR functional view,
/// losing data.
#[test]
fn collection_membership_duplicates_for_dual_role_split() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/collection> a sbol3:Collection ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "collection" ;
    sbol3:member <https://lab/dual_role> .

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
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    let members: Vec<&str> = graph
        .triples()
        .iter()
        .filter(|t| {
            t.predicate.as_str() == "http://sbols.org/v2#member"
                && t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/collection")
        })
        .filter_map(|t| t.object.as_iri().map(|i| i.as_str()))
        .collect();
    assert!(
        members.contains(&"https://lab/dual_role"),
        "Collection missing MD-half member, got {members:?}"
    );
    assert!(
        members.contains(&"https://lab/dual_role_component"),
        "Collection missing CD-half member, got {members:?}"
    );
}

#[test]
fn empty_default_version_is_rejected() {
    let input =
        std::fs::read_to_string(workspace_fixture("tests/fixtures/sbol2/single_cd.ttl")).unwrap();
    let (upgraded, _) = Document::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let mut options = sbol::DowngradeOptions::default();
    options.default_version = Some(String::new());
    let err = upgraded.downgrade_to_sbol2_with(options).unwrap_err();
    assert!(matches!(
        err,
        sbol::DowngradeError::InvalidDefaultVersion(_)
    ));
}

#[test]
fn default_version_some_synthesizes_for_unversioned_sources() {
    // `ModuleDefinitionOutput.xml` ships without `sbol2:version` on any
    // top-level. With `default_version = Some("1")` the downgrade must
    // synthesize that version on every owned subject and emit the
    // matching warning. With the default `None`, no synthesis occurs.
    let path = workspace_fixture("tests/fixtures/sbol2/real/ModuleDefinitionOutput.xml");
    let input = std::fs::read_to_string(&path).unwrap();
    let (upgraded, _) = Document::upgrade_from_sbol2(&input, RdfFormat::RdfXml).unwrap();

    // Default `None`: no version triples synthesized.
    let (default_graph, default_report) = upgraded.downgrade_to_sbol2().unwrap();
    assert!(
        default_report
            .warnings()
            .iter()
            .all(|w| !matches!(w, sbol::DowngradeWarning::SynthesizedVersion { .. })),
        "default `None` must not emit SynthesizedVersion warnings"
    );
    let default_versioned = default_graph
        .triples()
        .iter()
        .any(|t| t.predicate.as_str() == "http://sbols.org/v2#version");
    assert!(
        !default_versioned,
        "default `None` must not emit any sbol2:version triples for an unversioned source"
    );

    // Opt-in `Some("1")`: synthesizes version.
    let mut options = sbol::DowngradeOptions::default();
    options.default_version = Some("1".to_string());
    let (opt_in_graph, opt_in_report) = upgraded.downgrade_to_sbol2_with(options).unwrap();
    let synthesized = opt_in_report
        .warnings()
        .iter()
        .filter(|w| matches!(w, sbol::DowngradeWarning::SynthesizedVersion { .. }))
        .count();
    assert!(
        synthesized > 0,
        "opt-in `Some(\"1\")` must emit at least one SynthesizedVersion warning"
    );
    let any_versioned = opt_in_graph
        .triples()
        .iter()
        .any(|t| t.predicate.as_str() == "http://sbols.org/v2#version");
    assert!(
        any_versioned,
        "opt-in `Some(\"1\")` must emit sbol2:version triples"
    );
}

#[test]
fn native_attachment_properties_downgrade_to_sbol2_surface() {
    let ttl = r#"
@prefix sbol: <http://sbols.org/v3#> .

<https://example.org/lab/cd> a sbol:Component ;
    sbol:displayId "cd" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> ;
    sbol:hasAttachment <https://example.org/lab/att> .

<https://example.org/lab/att> a sbol:Attachment ;
    sbol:displayId "att" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:source <https://files.example/att.txt> ;
    sbol:hash "abcdef" ;
    sbol:hashAlgorithm "sha3-256" .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/cd",
            "http://sbols.org/v2#attachment",
            "https://example.org/lab/att"
        ),
        "native sbol3:hasAttachment should downgrade to sbol2:attachment"
    );
    assert!(
        has_literal_triple(
            &graph,
            "https://example.org/lab/att",
            "http://sbols.org/v2#hashAlgorithm",
            "sha3-256"
        ),
        "native sbol3:hashAlgorithm should downgrade to sbol2:hashAlgorithm"
    );
}

#[test]
fn functional_nondirectional_interface_downgrades_to_direction_none() {
    let ttl = r#"
@prefix sbol: <http://sbols.org/v3#> .

<https://lab/module> a sbol:Component ;
    sbol:hasNamespace <https://lab> ;
    sbol:displayId "module" ;
    sbol:type <https://identifiers.org/SBO:0000241> ;
    sbol:hasFeature <https://lab/module/fc> ;
    sbol:hasInterface <https://lab/module/interface> .

<https://lab/module/fc> a sbol:SubComponent ;
    sbol:displayId "fc" ;
    sbol:instanceOf <https://lab/target> .

<https://lab/module/interface> a sbol:Interface ;
    sbol:displayId "interface" ;
    sbol:nondirectional <https://lab/module/fc> .

<https://lab/target> a sbol:Component ;
    sbol:hasNamespace <https://lab> ;
    sbol:displayId "target" ;
    sbol:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    assert!(
        has_triple(
            &graph,
            "https://lab/module/fc",
            "http://sbols.org/v2#direction",
            "http://sbols.org/v2#none"
        ),
        "Interface.nondirectional should downgrade to sbol2:direction none"
    );
    assert!(
        !has_triple(
            &graph,
            "https://lab/module/fc",
            "http://sbols.org/v2#direction",
            "http://sbols.org/v2#inout"
        ),
        "Interface.nondirectional must not be misrepresented as inout"
    );
}

#[test]
fn collapsed_sequence_annotation_metadata_round_trips() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix so: <https://identifiers.org/SO:> .
@prefix ex: <https://example.org/custom#> .

<https://example.org/lab/cd/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna ;
    sbol:sequence <https://example.org/lab/seq/1> ;
    sbol:component <https://example.org/lab/cd/sub/1> ;
    sbol:sequenceAnnotation <https://example.org/lab/cd/ann/1> .

<https://example.org/lab/seq/1>
    a sbol:Sequence ;
    sbol:persistentIdentity <https://example.org/lab/seq> ;
    sbol:displayId "seq" ;
    sbol:version "1" ;
    sbol:elements "ATGC" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .

<https://example.org/lab/part/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/part> ;
    sbol:displayId "part" ;
    sbol:version "1" ;
    sbol:type biopax:Dna .

<https://example.org/lab/cd/sub/1>
    a sbol:Component ;
    sbol:persistentIdentity <https://example.org/lab/cd/sub> ;
    sbol:displayId "sub" ;
    sbol:version "1" ;
    sbol:definition <https://example.org/lab/part/1> ;
    sbol:access sbol:public .

<https://example.org/lab/cd/ann/1>
    a sbol:SequenceAnnotation, ex:SpecialAnnotation ;
    sbol:persistentIdentity <https://example.org/lab/cd/ann> ;
    sbol:displayId "ann" ;
    sbol:version "1" ;
    sbol:component <https://example.org/lab/cd/sub/1> ;
    sbol:location <https://example.org/lab/cd/ann/range/1> ;
    sbol:role so:0000167 ;
    dcterms:title "annotation title" ;
    prov:wasDerivedFrom <https://example.org/source/ann> .

<https://example.org/lab/cd/ann/range/1>
    a sbol:Range ;
    sbol:persistentIdentity <https://example.org/lab/cd/ann/range> ;
    sbol:displayId "range" ;
    sbol:version "1" ;
    sbol:start 1 ;
    sbol:end 4 ;
    sbol:sequence <https://example.org/lab/seq/1> .
"#;
    let (upgraded, _ureport) =
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    let (downgraded, _dreport) = upgraded.downgrade_to_sbol2().expect("downgrade");
    let sa = "https://example.org/lab/cd/ann/1";

    assert!(
        has_triple(
            &downgraded,
            sa,
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "https://example.org/custom#SpecialAnnotation"
        ),
        "extra rdf:type on a collapsed SequenceAnnotation should round-trip"
    );
    assert!(
        has_triple(
            &downgraded,
            sa,
            "http://sbols.org/v2#persistentIdentity",
            "https://example.org/lab/cd/ann"
        ),
        "SA persistentIdentity should be restored from preserved metadata"
    );
    assert!(
        has_literal_triple(&downgraded, sa, "http://sbols.org/v2#version", "1"),
        "SA version should be restored from preserved metadata"
    );
    assert!(
        has_triple(
            &downgraded,
            sa,
            "http://sbols.org/v2#role",
            "https://identifiers.org/SO:0000167"
        ),
        "SA role should be restored from preserved metadata"
    );
    assert!(
        has_literal_triple(
            &downgraded,
            sa,
            "http://purl.org/dc/terms/title",
            "annotation title"
        ),
        "SA Dublin Core metadata should be restored from preserved metadata"
    );
    assert!(
        has_triple(
            &downgraded,
            sa,
            "http://www.w3.org/ns/prov#wasDerivedFrom",
            "https://example.org/source/ann"
        ),
        "SA provenance should be restored from preserved metadata"
    );
}

/// Native SBOL 3 dual-role Components whose displayId matches a
/// SubComponent's displayId previously emitted the synthesized linking
/// FunctionalComponent at the SAME IRI as the SubComponent — two
/// contradictory rdf:types on one subject, rejected by any compliant
/// SBOL 2 reader. The downgrade now allocates the next available
/// `displayId_N` and propagates it to the FC's displayId so the IRI's
/// last segment matches.
#[test]
fn dual_role_linking_fc_avoids_subcomponent_iri_collision() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/widget> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "widget" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/widget_seq> ;
    sbol3:hasInteraction <https://lab/widget/inhibition> ;
    sbol3:hasFeature <https://lab/widget/widget> .

<https://lab/widget/widget> a sbol3:SubComponent ;
    sbol3:displayId "widget" ;
    sbol3:instanceOf <https://lab/inner> .

<https://lab/widget/inhibition> a sbol3:Interaction ;
    sbol3:displayId "inhibition" ;
    sbol3:type <https://identifiers.org/SBO:0000169> .

<https://lab/widget_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "widget_seq" ;
    sbol3:encoding <https://identifiers.org/edam:format_1207> ;
    sbol3:elements "ACGT" .

<https://lab/inner> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "inner" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    let collision_iri = "https://lab/widget/widget";
    let types_at = |iri: &str| -> Vec<String> {
        graph
            .triples()
            .iter()
            .filter(|t| {
                t.subject.as_iri().map(|i| i.as_str()) == Some(iri)
                    && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            })
            .filter_map(|t| t.object.as_iri().map(|i| i.as_str().to_owned()))
            .collect()
    };
    let types = types_at(collision_iri);
    assert!(
        !(types
            .iter()
            .any(|t| t == "http://sbols.org/v2#FunctionalComponent")
            && types.iter().any(|t| t == "http://sbols.org/v2#Component")),
        "linking FC and SubComponent must not share an IRI, got types at {collision_iri}: {types:?}"
    );

    // The disambiguated linking FC should live at `widget_2`.
    let disambig_iri = "https://lab/widget/widget_2";
    assert!(
        types_at(disambig_iri)
            .iter()
            .any(|t| t == "http://sbols.org/v2#FunctionalComponent"),
        "expected synthesized linking FC at the next-available IRI `{disambig_iri}`"
    );

    // The disambiguated displayId must match the new IRI's last segment
    // so the output passes SBOL 2 compliance (sbol-12302).
    let did = graph.triples().iter().find_map(|t| {
        (t.subject.as_iri().map(|i| i.as_str()) == Some(disambig_iri)
            && t.predicate.as_str() == "http://sbols.org/v2#displayId")
            .then(|| t.object.as_literal().map(|l| l.value().to_owned()))
            .flatten()
    });
    assert_eq!(
        did.as_deref(),
        Some("widget_2"),
        "linking FC displayId must equal its IRI's last segment"
    );
}

/// SBOL 2 sbol-12302 requires `displayId` to equal the last path segment
/// of `persistentIdentity`. When the downgrade triple-splits a
/// SubComponent under a dual-role parent into `_c` / `_fc` / `_m`
/// variants, each variant's displayId must carry the same suffix.
#[test]
fn dual_role_subcomponent_variant_display_ids_match_iri_suffix() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/dual> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/dual_seq> ;
    sbol3:hasInteraction <https://lab/dual/inh> ;
    sbol3:hasFeature <https://lab/dual/inner_sc> .

<https://lab/dual/inner_sc> a sbol3:SubComponent ;
    sbol3:displayId "inner_sc" ;
    sbol3:instanceOf <https://lab/target> .

<https://lab/dual/inh> a sbol3:Interaction ;
    sbol3:displayId "inh" ;
    sbol3:type <https://identifiers.org/SBO:0000169> .

<https://lab/dual_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_seq" ;
    sbol3:encoding <https://identifiers.org/edam:format_1207> ;
    sbol3:elements "ACGT" .

<https://lab/target> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "target" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");
    let display_id_at = |iri: &str| -> Option<String> {
        graph.triples().iter().find_map(|t| {
            (t.subject.as_iri().map(|i| i.as_str()) == Some(iri)
                && t.predicate.as_str() == "http://sbols.org/v2#displayId")
                .then(|| t.object.as_literal().map(|l| l.value().to_owned()))
                .flatten()
        })
    };
    assert_eq!(
        display_id_at("https://lab/dual/inner_sc").as_deref(),
        Some("inner_sc"),
        "Component variant keeps the bare displayId"
    );
    assert_eq!(
        display_id_at("https://lab/dual/inner_sc_fc").as_deref(),
        Some("inner_sc_fc"),
        "FunctionalComponent variant displayId must carry the `_fc` suffix to match its IRI"
    );
}

/// A ComponentDefinition carrying both `biopax:Dna` and
/// `biopax:DnaRegion` — two variants that collapse to the *same* SBO
/// term (`SBO:0000251`) on upgrade — must round-trip distinctly: one
/// `sbol2:type biopax:Dna` triple and one `sbol2:type biopax:DnaRegion`
/// triple in the SBOL 2 output. Previously the downgrade picked the
/// first iterator-order variant for every `sbol3:type SBO:0000251`
/// triple, dropping the other variant.
#[test]
fn biopax_dna_plus_dna_region_round_trip_distinctly() {
    let sbol2 = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://lab/cd/1> a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna ;
    sbol:type biopax:DnaRegion .
"#;
    let (document, _r) = Document::upgrade_from_sbol2(sbol2, RdfFormat::Turtle).expect("upgrade");
    let (graph, _r) = document.downgrade_to_sbol2().expect("downgrade");

    let biopax_types: std::collections::HashSet<String> = graph
        .triples()
        .iter()
        .filter(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/cd/1")
                && t.predicate.as_str() == "http://sbols.org/v2#type"
        })
        .filter_map(|t| t.object.as_iri().map(|i| i.as_str().to_owned()))
        .collect();
    assert!(
        biopax_types.contains("http://www.biopax.org/release/biopax-level3.owl#Dna"),
        "biopax:Dna lost on round-trip — got {biopax_types:?}"
    );
    assert!(
        biopax_types.contains("http://www.biopax.org/release/biopax-level3.owl#DnaRegion"),
        "biopax:DnaRegion lost on round-trip — got {biopax_types:?}"
    );
}

/// A dual-role Component whose synthesized CD half lands on a
/// separately-named Component's IRI used to silently merge the two
/// (both `sbol2:ComponentDefinition` rdf:types on a single IRI plus
/// chimeric structural triples). The downgrade now routes the
/// synthesized half through the suffix allocator, so a collision picks
/// up a `_2` tail instead of overwriting the sibling Component.
#[test]
fn dual_role_cd_half_avoids_separately_named_component_iri() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/X> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "X" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/X_seq> ;
    sbol3:hasInteraction <https://lab/X/inter> .

<https://lab/X/inter> a sbol3:Interaction ;
    sbol3:displayId "inter" ;
    sbol3:type <https://identifiers.org/SBO:0000169> .

<https://lab/X_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "X_seq" ;
    sbol3:elements "ACGT" .

<https://lab/X_component> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "X_component" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _r) = document.downgrade_to_sbol2().expect("downgrade");

    let types_at = |iri: &str| -> Vec<String> {
        graph
            .triples()
            .iter()
            .filter(|t| {
                t.subject.as_iri().map(|i| i.as_str()) == Some(iri)
                    && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            })
            .filter_map(|t| t.object.as_iri().map(|i| i.as_str().to_owned()))
            .collect()
    };

    let at_collision = types_at("https://lab/X_component");
    assert_eq!(
        at_collision.len(),
        1,
        "the separately-named Component should keep its IRI to itself, got {at_collision:?}"
    );

    assert!(
        types_at("https://lab/X_component_2")
            .iter()
            .any(|t| t == "http://sbols.org/v2#ComponentDefinition"),
        "X's CD half should disambiguate to `_component_2` when `_component` is taken"
    );

    // And the displayId of the disambiguated CD half must match its
    // IRI's last segment (SBOL 2 sbol-12302 compliance).
    let did = graph.triples().iter().find_map(|t| {
        (t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/X_component_2")
            && t.predicate.as_str() == "http://sbols.org/v2#displayId")
            .then(|| t.object.as_literal().map(|l| l.value().to_owned()))
            .flatten()
    });
    assert_eq!(
        did.as_deref(),
        Some("X_component_2"),
        "displayId of the disambiguated half must equal its IRI's last segment"
    );
}

/// Two SubComponents under a dual-role parent named `foo` and `foo_fc`
/// previously produced a collision: when `foo` triple-splits, its FC
/// variant lands at `parent/foo_fc` — the same IRI as the sibling
/// SubComponent named `foo_fc`. The downgrade now allocates the FC
/// variant via `next_available_child_iri` against a shared `used` set,
/// so the variant gets bumped to `foo_fc_2`.
#[test]
fn dual_role_subcomponent_variant_avoids_sibling_iri_collision() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/parent> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "parent" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/parent_seq> ;
    sbol3:hasInteraction <https://lab/parent/inter> ;
    sbol3:hasFeature <https://lab/parent/foo> ;
    sbol3:hasFeature <https://lab/parent/foo_fc> .

<https://lab/parent/foo> a sbol3:SubComponent ;
    sbol3:displayId "foo" ;
    sbol3:instanceOf <https://lab/target> .

<https://lab/parent/foo_fc> a sbol3:SubComponent ;
    sbol3:displayId "foo_fc" ;
    sbol3:instanceOf <https://lab/target> .

<https://lab/parent/inter> a sbol3:Interaction ;
    sbol3:displayId "inter" ;
    sbol3:type <https://identifiers.org/SBO:0000169> .

<https://lab/parent_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "parent_seq" ;
    sbol3:elements "ACGT" .

<https://lab/target> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "target" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _r) = document.downgrade_to_sbol2().expect("downgrade");
    let types_at = |iri: &str| -> Vec<String> {
        graph
            .triples()
            .iter()
            .filter(|t| {
                t.subject.as_iri().map(|i| i.as_str()) == Some(iri)
                    && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            })
            .filter_map(|t| t.object.as_iri().map(|i| i.as_str().to_owned()))
            .collect()
    };
    let collision_types = types_at("https://lab/parent/foo_fc");
    let has_sc = collision_types
        .iter()
        .any(|t| t == "http://sbols.org/v2#Component");
    let has_fc = collision_types
        .iter()
        .any(|t| t == "http://sbols.org/v2#FunctionalComponent");
    assert!(
        !(has_sc && has_fc),
        "sibling SubComponent named like another's `_fc` variant must not share an IRI, \
         got types at https://lab/parent/foo_fc: {collision_types:?}"
    );
    assert!(
        types_at("https://lab/parent/foo_fc_2")
            .iter()
            .any(|t| t == "http://sbols.org/v2#FunctionalComponent"),
        "FC variant of `foo` should be allocated at `foo_fc_2`"
    );
}

/// A ComponentDefinition that carried multiple distinct BioPAX types
/// (e.g. both `Dna` and `DnaRegion`) used to collapse to one preserved
/// hint, losing the other variant on round-trip. The preserved set is
/// now multi-valued and the downgrade picks the variant whose SBO
/// target matches each `sbol3:type` triple.
#[test]
fn multi_biopax_types_round_trip_distinctly() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix backport: <http://sboltools.org/backport#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://lab/mixed> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "mixed" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:type <https://identifiers.org/SBO:0000252> ;
    backport:biopaxType biopax:Dna ;
    backport:biopaxType biopax:Protein ;
    backport:sbol2type <http://sbols.org/v2#ComponentDefinition> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    let types_emitted: std::collections::HashSet<String> = graph
        .triples()
        .iter()
        .filter(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/mixed")
                && t.predicate.as_str() == "http://sbols.org/v2#type"
        })
        .filter_map(|t| t.object.as_iri().map(|i| i.as_str().to_owned()))
        .collect();
    assert!(
        types_emitted.contains("http://www.biopax.org/release/biopax-level3.owl#Dna"),
        "expected biopax:Dna to be restored (preserved variant for SBO:0000251), got {types_emitted:?}"
    );
    assert!(
        types_emitted.contains("http://www.biopax.org/release/biopax-level3.owl#Protein"),
        "expected biopax:Protein to be restored (preserved variant for SBO:0000252), got {types_emitted:?}"
    );
}

/// Two distinct SBOL 3 subjects whose `iri_rewrites` rewrite to the
/// same SBOL 2 versioned IRI silently merge into one chimeric
/// SBOL 2 subject. The input is technically non-conformant (the
/// implied SBOL 2 versioned identities should be unique), but the
/// merge is otherwise invisible; the downgrade now emits
/// `DowngradeWarning::IdentityCollision` so callers can audit.
/// Symmetric with `UpgradeWarning::IdentityCollision` on the inverse
/// direction.
#[test]
fn colliding_iri_rewrites_emit_warning() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix backport: <http://sboltools.org/backport#> .

<https://lab/foo> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "foo" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    backport:sbol2type <http://sbols.org/v2#ComponentDefinition> ;
    backport:sbol2version "1/1" .

<https://lab/foo/1> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "foo" ;
    sbol3:type <https://identifiers.org/SBO:0000252> ;
    backport:sbol2type <http://sbols.org/v2#ComponentDefinition> ;
    backport:sbol2version "1" .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (_graph, report) = document.downgrade_to_sbol2().expect("downgrade");
    let collision = report.warnings().iter().find_map(|w| match w {
        sbol::DowngradeWarning::IdentityCollision { canonical, sources } => {
            Some((canonical.clone(), sources.clone()))
        }
        _ => None,
    });
    let (canonical, sources) = collision.expect("expected IdentityCollision warning");
    assert_eq!(canonical, "https://lab/foo/1/1");
    assert_eq!(
        sources,
        vec![
            "https://lab/foo".to_string(),
            "https://lab/foo/1".to_string(),
        ]
    );
}
