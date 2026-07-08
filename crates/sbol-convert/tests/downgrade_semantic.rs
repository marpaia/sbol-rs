//! Integration tests for SBOL 3 → SBOL 2 type, predicate, and value mapping.

mod common;

use common::downgrade::*;

use sbol3::{Document, RdfFormat};

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
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");
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
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");
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
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");
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
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");
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
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");

    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/cd/sub",
            "http://sbols.org/v2#access",
            "http://sbols.org/v2#public"
        ),
        "ComponentDefinition subcomponents receive access public"
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
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");
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
    let (graph, report) = sbol_convert::downgrade(&document).expect("downgrade");
    assert!(
        report.warnings().iter().any(|w| matches!(
            w,
            sbol_convert::DowngradeWarning::UnsupportedSbol3Type { subject, sbol3_type }
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
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");

    assert!(
        has_triple(
            &graph,
            "https://lab/design",
            "http://sbols.org/v2#sequenceAnnotation",
            "https://lab/design/part_range",
        ),
        "parent CD should point at synthesized SequenceAnnotation"
    );
    assert!(
        has_triple(
            &graph,
            "https://lab/design/part_range",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v2#SequenceAnnotation",
        ),
        "native SubComponent location should synthesize an SBOL 2 SequenceAnnotation"
    );
    assert!(
        has_triple(
            &graph,
            "https://lab/design/part_range",
            "http://sbols.org/v2#component",
            "https://lab/design/part",
        ),
        "SequenceAnnotation should point at the downgraded Component"
    );
    assert!(
        has_triple(
            &graph,
            "https://lab/design/part_range",
            "http://sbols.org/v2#location",
            "https://lab/design/part_range/range",
        ),
        "SequenceAnnotation should carry the Location nested under it"
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
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");
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
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");

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
    let (graph, report) = sbol_convert::downgrade(&document).expect("downgrade");
    assert!(
        report.warnings().iter().any(|w| matches!(
            w,
            sbol_convert::DowngradeWarning::UnsupportedSbol3Type { subject, sbol3_type }
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
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");
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
    let (graph, report) = sbol_convert::downgrade(&document).expect("downgrade");
    assert!(
        !report.warnings().iter().any(|w| matches!(
            w,
            sbol_convert::DowngradeWarning::OrphanComponentReference { .. }
                | sbol_convert::DowngradeWarning::UnresolvableConstraintToMapsTo { .. }
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
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");

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
fn functional_nondirectional_interface_downgrades_to_direction_inout() {
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
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");

    assert!(
        has_triple(
            &graph,
            "https://lab/module/fc",
            "http://sbols.org/v2#direction",
            "http://sbols.org/v2#inout"
        ),
        "Interface.nondirectional should downgrade to sbol2:direction inout"
    );
    assert!(
        !has_triple(
            &graph,
            "https://lab/module/fc",
            "http://sbols.org/v2#direction",
            "http://sbols.org/v2#none"
        ),
        "Interface.nondirectional maps to inout, not none"
    );
}
