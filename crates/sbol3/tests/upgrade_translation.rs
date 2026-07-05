//! Integration tests for basic SBOL 2 → SBOL 3 type and predicate
//! translation, plus translation edge cases.

mod common;

use common::upgrade::*;

use std::path::Path;

use sbol3::{RdfFormat, SbolTopLevel, UpgradeError, UpgradeOptions, UpgradeWarning};

#[test]
fn single_component_definition_upgrades() {
    let document = upgrade_fixture("single_cd.ttl");
    document
        .check()
        .unwrap_or_else(|report| panic!("validation failed: {report:?}"));

    let component = document
        .components()
        .find(|c| c.identity.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/J23100"))
        .expect("converted Component missing");
    let namespace = component
        .namespace()
        .expect("converted Component is missing hasNamespace");
    assert_eq!(namespace.as_str(), "https://example.org/lab");
}

#[test]
fn namespace_derives_from_display_id_when_persistent_identity_is_missing() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://example.org/lab/cd/1>
    a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:type biopax:Dna .
"#;
    let (document, _report) =
        sbol3::upgrade::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    let component = document
        .components()
        .find(|c| c.identity.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/cd"))
        .expect("converted Component missing");
    assert_eq!(
        component.namespace().map(|iri| iri.as_str()),
        Some("https://example.org/lab"),
        "namespace should be derived from the version-stripped identity plus displayId"
    );
}

#[test]
fn component_with_annotation_keeps_location_sequence_link() {
    let document = upgrade_fixture("cd_with_annotation.ttl");
    document
        .check()
        .unwrap_or_else(|report| panic!("validation failed: {report:?}"));
    assert_eq!(document.sequence_features().count(), 1);
    assert_eq!(document.ranges().count(), 1);
}

#[test]
fn sequence_annotation_with_component_emits_collapse_warning() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://example.org/lab/cd/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna ;
    sbol:sequence <https://example.org/lab/seq/1> ;
    sbol:component <https://example.org/lab/cd/sub/1> ;
    sbol:sequenceAnnotation <https://example.org/lab/cd/ann/1> .

<https://example.org/lab/cd/sub/1>
    a sbol:Component ;
    sbol:persistentIdentity <https://example.org/lab/cd/sub> ;
    sbol:displayId "sub" ;
    sbol:version "1" ;
    sbol:definition <https://example.org/lab/part/1> .

<https://example.org/lab/cd/ann/1>
    a sbol:SequenceAnnotation ;
    sbol:persistentIdentity <https://example.org/lab/cd/ann> ;
    sbol:displayId "ann" ;
    sbol:version "1" ;
    sbol:component <https://example.org/lab/cd/sub/1> ;
    sbol:location <https://example.org/lab/cd/ann/range/1> .

<https://example.org/lab/cd/ann/range/1>
    a sbol:Range ;
    sbol:persistentIdentity <https://example.org/lab/cd/ann/range> ;
    sbol:displayId "range" ;
    sbol:version "1" ;
    sbol:start 1 ;
    sbol:end 4 .

<https://example.org/lab/seq/1>
    a sbol:Sequence ;
    sbol:persistentIdentity <https://example.org/lab/seq> ;
    sbol:displayId "seq" ;
    sbol:version "1" ;
    sbol:elements "acgt" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .

<https://example.org/lab/part/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/part> ;
    sbol:displayId "part" ;
    sbol:version "1" ;
    sbol:type biopax:Dna .
"#;
    let (document, report) =
        sbol3::upgrade::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    document
        .check()
        .unwrap_or_else(|report| panic!("validation failed: {report:?}"));
    assert!(
        document
            .ranges()
            .any(|range| range.identity.as_iri().map(|iri| iri.as_str())
                == Some("https://example.org/lab/cd/sub/range")),
        "collapsed Location should be reparented under the SubComponent using its displayId"
    );
    assert!(
        report.warnings().iter().any(|w| matches!(
            w,
            UpgradeWarning::SequenceAnnotationWithComponent { annotation }
                if annotation == "https://example.org/lab/cd/ann/1"
        )),
        "expected SequenceAnnotationWithComponent warning, got {:?}",
        report.warnings()
    );
}

#[test]
fn experiment_experimental_data_maps_to_member() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .

<https://example.org/lab/exp/1>
    a sbol:Experiment ;
    sbol:persistentIdentity <https://example.org/lab/exp> ;
    sbol:displayId "exp" ;
    sbol:version "1" ;
    sbol:experimentalData <https://example.org/lab/data/1> .

<https://example.org/lab/data/1>
    a sbol:ExperimentalData ;
    sbol:persistentIdentity <https://example.org/lab/data> ;
    sbol:displayId "data" ;
    sbol:version "1" .
"#;
    let (document, report) =
        sbol3::upgrade::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    assert!(
        report.is_clean(),
        "unexpected warnings: {:?}",
        report.warnings()
    );
    let triples = document.rdf_graph().triples();
    assert!(
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/exp")
                && t.predicate.as_str() == "http://sbols.org/v3#member"
                && t.object.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/data")
        }),
        "sbol2:experimentalData should upgrade to sbol3:member"
    );
    assert!(
        triples.iter().all(|t| {
            t.predicate.as_str() != "http://sboltools.org/backport#sbol2_experimentalData"
        }),
        "recognized experimentalData predicate should not be archived as unknown SBOL 2"
    );
}

#[test]
fn variable_component_operator_maps_to_cardinality() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://example.org/lab/deriv/1>
    a sbol:CombinatorialDerivation ;
    sbol:persistentIdentity <https://example.org/lab/deriv> ;
    sbol:displayId "deriv" ;
    sbol:version "1" ;
    sbol:template <https://example.org/lab/template/1> ;
    sbol:strategy <http://sbols.org/v2#enumerate> ;
    sbol:variableComponent <https://example.org/lab/deriv/vc/1> .

<https://example.org/lab/deriv/vc/1>
    a sbol:VariableComponent ;
    sbol:persistentIdentity <https://example.org/lab/deriv/vc> ;
    sbol:displayId "vc" ;
    sbol:version "1" ;
    sbol:operator <http://sbols.org/v2#one> ;
    sbol:variable <https://example.org/lab/template/sub/1> ;
    sbol:variant <https://example.org/lab/variant/1> .

<https://example.org/lab/template/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/template> ;
    sbol:displayId "template" ;
    sbol:version "1" ;
    sbol:type biopax:Dna ;
    sbol:component <https://example.org/lab/template/sub/1> .

<https://example.org/lab/template/sub/1>
    a sbol:Component ;
    sbol:persistentIdentity <https://example.org/lab/template/sub> ;
    sbol:displayId "sub" ;
    sbol:version "1" ;
    sbol:definition <https://example.org/lab/part/1> .

<https://example.org/lab/variant/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/variant> ;
    sbol:displayId "variant" ;
    sbol:version "1" ;
    sbol:type biopax:Dna .

<https://example.org/lab/part/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/part> ;
    sbol:displayId "part" ;
    sbol:version "1" ;
    sbol:type biopax:Dna .
"#;
    let (document, report) =
        sbol3::upgrade::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/deriv/vc")
                && t.predicate.as_str() == "http://sbols.org/v3#cardinality"
                && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v3#one")
        }),
        "sbol2:operator should upgrade to sbol3:cardinality"
    );
    assert!(
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/deriv")
                && t.predicate.as_str() == "http://sbols.org/v3#strategy"
                && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v3#enumerate")
        }),
        "sbol2 strategy values should move into the SBOL 3 namespace"
    );
    assert!(
        triples
            .iter()
            .all(|t| t.predicate.as_str() != "http://sboltools.org/backport#sbol2_operator"),
        "recognized operator predicate should not be archived as unknown SBOL 2"
    );
}

#[test]
fn role_integration_values_move_to_sbol3_namespace() {
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
    sbol:role <https://identifiers.org/SO:0000167> ;
    sbol:roleIntegration <http://sbols.org/v2#mergeRoles> .

<https://example.org/lab/part/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/part> ;
    sbol:displayId "part" ;
    sbol:version "1" ;
    sbol:type biopax:Dna .
"#;
    let (document, report) =
        sbol3::upgrade::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    assert!(
        report.is_clean(),
        "unexpected warnings: {:?}",
        report.warnings()
    );
    document
        .check()
        .unwrap_or_else(|report| panic!("validation failed: {report:?}"));
    assert!(
        document.rdf_graph().triples().iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/cd/sub")
                && t.predicate.as_str() == "http://sbols.org/v3#roleIntegration"
                && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v3#mergeRoles")
        }),
        "sbol2 roleIntegration values should move into the SBOL 3 namespace"
    );
}

#[test]
fn component_with_subparts_emits_sub_components() {
    let document = upgrade_fixture("cd_with_subparts.ttl");
    document
        .check()
        .unwrap_or_else(|report| panic!("validation failed: {report:?}"));
    let parent = document
        .components()
        .find(|c| {
            c.identity.as_iri().map(|i| i.as_str())
                == Some("https://example.org/lab/expression_cassette")
        })
        .expect("parent Component missing");
    let _ = parent.identity.as_iri().unwrap();
    assert_eq!(document.sub_components().count(), 2);
}

#[test]
fn module_definition_becomes_component_with_synthetic_type() {
    let document = upgrade_fixture("md_simple.ttl");
    document
        .check()
        .unwrap_or_else(|report| panic!("validation failed: {report:?}"));
    assert!(
        document.interactions().count() >= 1,
        "interactions should survive MD upgrade"
    );
    assert!(
        document.participations().count() >= 1,
        "participations should survive MD upgrade"
    );
}

#[test]
fn urn_style_identity_derives_namespace_from_persistent_identity() {
    let document = upgrade_fixture("urn_design.ttl");
    document
        .check()
        .unwrap_or_else(|report| panic!("validation failed: {report:?}"));
    let component = document
        .components()
        .find(|c| c.identity.as_iri().map(|i| i.as_str()) == Some("urn:sbol:design:promoter"))
        .expect("URN-form Component missing");
    let namespace = component
        .namespace()
        .expect("URN Component should derive hasNamespace via persistentIdentity strip");
    assert_eq!(namespace.as_str(), "urn:sbol:design");
}

#[test]
fn collection_upgrades() {
    let document = upgrade_fixture("collection.ttl");
    document
        .check()
        .unwrap_or_else(|report| panic!("validation failed: {report:?}"));
    assert_eq!(document.collections().count(), 1);
}

#[test]
fn sbol3_input_rejected_with_not_sbol2() {
    let sbol3_input = r#"
@prefix sbol: <http://sbols.org/v3#> .
<https://example.org/lab/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> .
"#;
    match sbol3::upgrade::upgrade_from_sbol2(sbol3_input, RdfFormat::Turtle) {
        Err(UpgradeError::NotSbol2) => {}
        other => panic!("expected NotSbol2 error, got {other:?}"),
    }
}

#[test]
fn upgrade_options_default_preserves_backport_triples() {
    let path = workspace_fixture("single_cd.ttl");
    let input = std::fs::read_to_string(&path).unwrap();
    let (document, _report) =
        sbol3::upgrade::upgrade_from_sbol2(&input, RdfFormat::Turtle).expect("upgrade");
    let backport_triples: Vec<_> = document
        .rdf_graph()
        .triples()
        .iter()
        .filter(|t| {
            t.predicate
                .as_str()
                .starts_with("http://sboltools.org/backport#")
        })
        .collect();
    assert!(
        !backport_triples.is_empty(),
        "default options should emit backport triples"
    );
}

#[test]
fn upgrade_options_without_backport_drops_archived_triples() {
    let path = workspace_fixture("single_cd.ttl");
    let input = std::fs::read_to_string(&path).unwrap();
    let mut options = UpgradeOptions::default();
    options.preserve_backport = false;
    let (document, _report) =
        sbol3::upgrade::upgrade_from_sbol2_with(&input, RdfFormat::Turtle, options).expect("upgrade");
    let backport_triples: Vec<_> = document
        .rdf_graph()
        .triples()
        .iter()
        .filter(|t| {
            t.predicate
                .as_str()
                .starts_with("http://sboltools.org/backport#")
        })
        .collect();
    assert!(
        backport_triples.is_empty(),
        "preserve_backport=false should drop archived triples"
    );
}

#[test]
fn unknown_sbol2_type_is_archived_for_downgrade() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .

<https://example.org/lab/future/1>
    a sbol:FutureThing ;
    sbol:persistentIdentity <https://example.org/lab/future> ;
    sbol:displayId "future" ;
    sbol:version "1" .
"#;
    let (document, report) =
        sbol3::upgrade::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    assert!(
        report.warnings().iter().any(|w| matches!(
            w,
            UpgradeWarning::UnknownSbol2Type { subject, sbol2_type }
                if subject == "https://example.org/lab/future/1"
                    && sbol2_type == "http://sbols.org/v2#FutureThing"
        )),
        "expected UnknownSbol2Type warning, got {:?}",
        report.warnings()
    );
    let archived_type = document.rdf_graph().triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/future")
            && t.predicate.as_str() == "http://sboltools.org/backport#sbol2type"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v2#FutureThing")
    });
    assert!(
        archived_type,
        "unknown SBOL 2 rdf:type should be archived under backport:sbol2type"
    );

    let (downgraded, _dreport) = sbol3::downgrade::downgrade(&document).expect("downgrade");
    let restored_type = downgraded.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/future/1")
            && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v2#FutureThing")
    });
    assert!(
        restored_type,
        "downgrade should restore archived unknown SBOL 2 rdf:type"
    );
}

#[test]
fn unknown_sbol2_extension_type_does_not_mask_known_type() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://example.org/lab/cd/1>
    a sbol:ComponentDefinition ;
    a sbol:FutureThing ;
    sbol:persistentIdentity <https://example.org/lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna .
"#;
    let (document, report) =
        sbol3::upgrade::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    assert!(
        report.warnings().iter().any(|w| matches!(
            w,
            UpgradeWarning::UnknownSbol2Type { subject, sbol2_type }
                if subject == "https://example.org/lab/cd/1"
                    && sbol2_type == "http://sbols.org/v2#FutureThing"
        )),
        "expected UnknownSbol2Type warning, got {:?}",
        report.warnings()
    );
    let component = document
        .components()
        .find(|c| c.identity.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/cd"))
        .expect("known ComponentDefinition type should still produce a Component");
    assert_eq!(
        component.namespace().map(|iri| iri.as_str()),
        Some("https://example.org/lab"),
        "unknown extension type should not mask top-level namespace synthesis"
    );
    let backport_type_count = document
        .rdf_graph()
        .triples()
        .iter()
        .filter(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/cd")
                && t.predicate.as_str() == "http://sboltools.org/backport#sbol2type"
        })
        .count();
    assert_eq!(
        backport_type_count, 1,
        "only the recognized SBOL 2 class should become the authoritative backport type"
    );

    let (downgraded, _dreport) = sbol3::downgrade::downgrade(&document).expect("downgrade");
    let restored_component_definition = downgraded.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/cd/1")
            && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str())
                == Some("http://sbols.org/v2#ComponentDefinition")
    });
    let restored_future_thing = downgraded.triples().iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/cd/1")
            && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v2#FutureThing")
    });
    assert!(
        restored_component_definition,
        "known SBOL 2 class should remain authoritative on downgrade"
    );
    assert!(
        !restored_future_thing,
        "unknown extension class must not override the known SBOL 2 class on downgrade"
    );
}

#[test]
fn path_helper_works_for_turtle_extension() {
    let _ = Path::new(""); // keep import live
    let path = workspace_fixture("single_cd.ttl");
    let (document, _report) = sbol3::upgrade::upgrade_from_sbol2_path(&path).expect("upgrade by path");
    assert_eq!(document.components().count(), 1);
}

#[test]
fn path_helper_accepts_sbol2_rdfxml_xml_extension() {
    let path = workspace_fixture("real/implementation_example.xml");
    let (document, _report) = sbol3::upgrade::upgrade_from_sbol2_path(&path).expect("upgrade .xml path");
    assert!(
        document.implementations().count() >= 1 || document.components().count() >= 1,
        "expected .xml SBOL 2 fixture to upgrade into typed SBOL 3 objects"
    );
}

#[test]
fn attachment_properties_upgrade_to_native_sbol3() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://example.org/lab/cd/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna ;
    sbol:attachment <https://example.org/lab/att/1> .

<https://example.org/lab/att/1>
    a sbol:Attachment ;
    sbol:persistentIdentity <https://example.org/lab/att> ;
    sbol:displayId "att" ;
    sbol:version "1" ;
    sbol:source <https://files.example/att.txt> ;
    sbol:hash "abcdef" ;
    sbol:hashAlgorithm "sha3-256" .
"#;
    let (document, report) =
        sbol3::upgrade::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    assert!(report.is_clean(), "warnings: {:?}", report.warnings());
    let triples = document.rdf_graph().triples();

    assert!(
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/cd")
                && t.predicate.as_str() == "http://sbols.org/v3#hasAttachment"
                && t.object.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/att")
        }),
        "sbol2:attachment should upgrade to native sbol3:hasAttachment with identity rewriting"
    );
    assert!(
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/att")
                && t.predicate.as_str() == "http://sbols.org/v3#hashAlgorithm"
                && t.object.as_literal().map(|l| l.value()) == Some("sha3-256")
        }),
        "sbol2:hashAlgorithm should upgrade to native sbol3:hashAlgorithm"
    );
    assert!(
        triples.iter().all(|t| {
            t.predicate.as_str() != "http://sboltools.org/backport#sbol2_attachment"
                && t.predicate.as_str() != "http://sboltools.org/backport#sbol2_hashAlgorithm"
        }),
        "native attachment properties must not be archived as unknown backport predicates"
    );
}
