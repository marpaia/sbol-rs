//! Integration tests for the SBOL 2 → SBOL 3 upgrade pipeline.
//!
//! Each fixture in `tests/fixtures/sbol2/` is parsed, upgraded, and validated
//! through the public `Document` API. The validator must accept the output
//! without errors; warnings are tolerated since SBOL 2 content does not
//! always satisfy every SBOL 3 best-practice rule.

use std::path::{Path, PathBuf};

use sbol::{Document, RdfFormat, SbolTopLevel, UpgradeError, UpgradeOptions, UpgradeWarning};

fn workspace_fixture(relative: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("tests/fixtures/sbol2");
    path.push(relative);
    path
}

fn upgrade_fixture(name: &str) -> Document {
    let path = workspace_fixture(name);
    let input = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    let (document, report) = Document::upgrade_from_sbol2(&input, RdfFormat::Turtle)
        .unwrap_or_else(|err| panic!("upgrade {}: {err}", path.display()));
    assert!(
        report.is_clean(),
        "{}: unexpected warnings: {:?}",
        name,
        report.warnings()
    );
    document
}

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
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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
    match Document::upgrade_from_sbol2(sbol3_input, RdfFormat::Turtle) {
        Err(UpgradeError::NotSbol2) => {}
        other => panic!("expected NotSbol2 error, got {other:?}"),
    }
}

#[test]
fn upgrade_options_default_preserves_backport_triples() {
    let path = workspace_fixture("single_cd.ttl");
    let input = std::fs::read_to_string(&path).unwrap();
    let (document, _report) =
        Document::upgrade_from_sbol2(&input, RdfFormat::Turtle).expect("upgrade");
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
        Document::upgrade_from_sbol2_with(&input, RdfFormat::Turtle, options).expect("upgrade");
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
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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

    let (downgraded, dreport) = document.downgrade_to_sbol2().expect("downgrade");
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
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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

    let (downgraded, dreport) = document.downgrade_to_sbol2().expect("downgrade");
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
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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

    let (downgraded, _dreport) = document.downgrade_to_sbol2().expect("downgrade");
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
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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

    let (downgraded, _dreport) = document.downgrade_to_sbol2().expect("downgrade");
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

fn real_fixture(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("tests/fixtures/sbol2/real");
    path.push(name);
    path
}

fn upgrade_real(name: &str) -> (Document, sbol::UpgradeReport) {
    let path = real_fixture(name);
    let input = std::fs::read_to_string(&path).unwrap();
    Document::upgrade_from_sbol2(&input, RdfFormat::RdfXml).expect("real fixture upgrade")
}

#[test]
fn real_implementation_example_validates_clean() {
    let (document, report) = upgrade_real("implementation_example.xml");
    document
        .check()
        .unwrap_or_else(|r| panic!("validation failed: {r:?}"));
    assert!(report.is_clean(), "warnings: {:?}", report.warnings());
}

#[test]
fn real_module_definition_with_mapsto_decomposes_cleanly() {
    // This fixture has known semantic issues in the source (SubComponents
    // without location coverage), so we don't expect a clean validate. The
    // assertion is structural: MapsTo → ComponentReference + Constraint,
    // direction → Interface.
    let (document, _report) = upgrade_real("ModuleDefinitionOutput.xml");
    assert!(
        document.component_references().count() >= 1,
        "MapsTo decomposition should emit a ComponentReference"
    );
    assert!(
        document.constraints().count() >= 1,
        "MapsTo decomposition should emit a Constraint"
    );
    assert!(
        document.interfaces().count() >= 1,
        "FC directions should synthesize an Interface"
    );
}

#[test]
fn real_sequence_constraint_uses_sbol3_restriction_uris() {
    let (document, _report) = upgrade_real("SequenceConstraintOutput.xml");
    document
        .check()
        .unwrap_or_else(|r| panic!("validation failed: {r:?}"));
    assert!(document.constraints().count() >= 1);
}

#[test]
fn real_sa_with_component_collapses_locations_onto_subcomponent() {
    let (document, _report) = upgrade_real("BBa_I0462.xml");
    document
        .check()
        .unwrap_or_else(|r| panic!("validation failed: {r:?}"));
    // The four SAs in the source each reference a SubComponent. After
    // collapse, the document should have those SubComponents but no
    // standalone SequenceFeatures (in this fixture every SA has a component).
    assert_eq!(document.sub_components().count(), 3);
    assert_eq!(document.sequence_features().count(), 0);
}

#[test]
fn location_without_inferable_sequence_emits_warning() {
    let (_document, report) = upgrade_real("CD_SA_Range_Example.xml");
    let has_location_warning = report.warnings().iter().any(|w| {
        matches!(
            w,
            UpgradeWarning::LocationWithoutSequence {
                sequence_count: 0,
                ..
            }
        )
    });
    assert!(
        has_location_warning,
        "expected LocationWithoutSequence warning, got: {:?}",
        report.warnings()
    );
}

#[test]
fn upgrade_report_tallies_per_construct_counts() {
    let (_document, report) = upgrade_real("ModuleDefinitionOutput.xml");
    let counts = report.counts();
    assert!(counts.module_definitions >= 1, "expected ≥1 MD conversion");
    assert!(
        counts.sub_components >= 1,
        "expected ≥1 SubComponent conversion"
    );
    assert!(
        counts.mapstos_decomposed >= 1,
        "expected ≥1 MapsTo decomposition"
    );
    assert!(
        counts.interfaces_synthesized >= 1,
        "expected ≥1 Interface synthesis"
    );
}

/// SAs that reference a `sbol2:component` get collapsed into the
/// referenced SubComponent's locations. The collapse must be counted —
/// the counter previously stayed at zero because of a control-flow bug
/// in `handle_triple`.
#[test]
fn upgrade_counts_collapsed_sequence_annotations() {
    let (_document, report) = upgrade_real("BBa_I0462.xml");
    let counts = report.counts();
    assert!(
        counts.sequence_annotations_collapsed >= 1,
        "expected ≥1 SA-with-component collapse, got {counts:?}"
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
        Document::upgrade_from_sbol2(&input, RdfFormat::Turtle).expect("upgrade");
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
    let (downgraded, _dreport) = document.downgrade_to_sbol2().expect("downgrade");
    let restored_merge = downgraded.triples().iter().any(|t| {
        t.predicate.as_str() == "http://sbols.org/v2#refinement"
            && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v2#merge")
    });
    assert!(
        restored_merge,
        "downgrade must restore the original sbol2:merge refinement from the backport hint",
    );
}

#[test]
fn unknown_sbol2_predicates_survive_as_backport_triples() {
    // `sbol2:access` has no SBOL 3 home but should be preserved under the
    // backport namespace so it isn't silently lost.
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .
@prefix so: <https://identifiers.org/SO:> .

<https://example.org/lab/cd/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna ;
    sbol:role so:0000167 ;
    sbol:access <http://sbols.org/v2#public> .
"#;
    let (document, _report) =
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    let backport_access = document
        .rdf_graph()
        .triples()
        .iter()
        .find(|t| t.predicate.as_str() == "http://sboltools.org/backport#sbol2_access");
    assert!(
        backport_access.is_some(),
        "sbol2:access should be preserved under backport namespace"
    );
}

#[test]
fn upgrade_does_not_strip_numeric_external_annotation_iris() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .
@prefix so: <https://identifiers.org/SO:> .
@prefix ex: <https://example.org/annotation#> .

<https://example.org/lab/cd/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna ;
    sbol:role so:0000167 ;
    ex:taxon <https://example.org/taxon/511145> .
"#;
    let (document, _report) =
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    let taxon_object = document.rdf_graph().triples().iter().find(|t| {
        t.predicate.as_str() == "https://example.org/annotation#taxon"
            && t.object.as_iri().map(|i| i.as_str()) == Some("https://example.org/taxon/511145")
    });
    assert!(
        taxon_object.is_some(),
        "external annotation IRI ending in digits should not be treated as an SBOL version"
    );
    let stripped_taxon_object = document.rdf_graph().triples().iter().find(|t| {
        t.predicate.as_str() == "https://example.org/annotation#taxon"
            && t.object.as_iri().map(|i| i.as_str()) == Some("https://example.org/taxon")
    });
    assert!(
        stripped_taxon_object.is_none(),
        "upgrade stripped a numeric external annotation IRI"
    );
}

#[test]
fn path_helper_works_for_turtle_extension() {
    let _ = Path::new(""); // keep import live
    let path = workspace_fixture("single_cd.ttl");
    let (document, _report) = Document::upgrade_from_sbol2_path(&path).expect("upgrade by path");
    assert_eq!(document.components().count(), 1);
}

#[test]
fn path_helper_accepts_sbol2_rdfxml_xml_extension() {
    let path = workspace_fixture("real/implementation_example.xml");
    let (document, _report) = Document::upgrade_from_sbol2_path(&path).expect("upgrade .xml path");
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
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
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

/// Two distinct SBOL 2 subjects whose canonical SBOL 3 forms coincide
/// (e.g. `<lab/foo/1>` version-strips to `<lab/foo>` and is then
/// indistinguishable from a separately-typed `<lab/foo>` subject in
/// the same document) silently merge into one chimeric SBOL 3 subject.
/// The input is non-conformant SBOL 2, but the merge is otherwise
/// invisible; the upgrade now surfaces the collision via
/// [`UpgradeWarning::IdentityCollision`] so callers can audit.
#[test]
fn colliding_canonical_identities_emit_warning() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://lab/foo/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://lab/foo> ;
    sbol:displayId "foo" ;
    sbol:version "1" ;
    sbol:type biopax:DnaRegion .

<https://lab/foo>
    a sbol:Sequence ;
    sbol:displayId "foo" ;
    sbol:elements "ACGT" .
"#;
    let (_document, report) =
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    let collision = report.warnings().iter().find_map(|w| match w {
        UpgradeWarning::IdentityCollision { canonical, sources } => {
            Some((canonical.clone(), sources.clone()))
        }
        _ => None,
    });
    let (canonical, sources) = collision.expect("expected IdentityCollision warning");
    assert_eq!(canonical, "https://lab/foo");
    assert_eq!(
        sources,
        vec![
            "https://lab/foo".to_string(),
            "https://lab/foo/1".to_string(),
        ]
    );
}

/// Two SBOL 2 SequenceAnnotations that both point at the same
/// SubComponent (`sbol2:component`) and carry Locations with the same
/// `displayId` used to collide on upgrade: the SA-with-component
/// collapse rewrote both Locations to `{SubComponent}/{loc_did}`, and
/// the second silently overwrote the first in the identity rewrite
/// map. The upgrade now routes Location rewrites through the shared
/// `used_iris` pool — collisions get a `_N` disambiguation tail, and
/// the emitted `sbol3:displayId` is overridden to match the new IRI's
/// last segment so sbol3-10204 still holds.
#[test]
fn sa_collapse_locations_with_same_display_id_round_trip_distinctly() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://lab/parent/1> a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://lab/parent> ;
    sbol:displayId "parent" ;
    sbol:version "1" ;
    sbol:type biopax:DnaRegion ;
    sbol:component <https://lab/parent/sub/1> ;
    sbol:sequenceAnnotation <https://lab/parent/sa1/1> ;
    sbol:sequenceAnnotation <https://lab/parent/sa2/1> .

<https://lab/parent/sub/1> a sbol:Component ;
    sbol:persistentIdentity <https://lab/parent/sub> ;
    sbol:displayId "sub" ;
    sbol:version "1" ;
    sbol:definition <https://lab/widget/1> ;
    sbol:access sbol:public .

<https://lab/parent/sa1/1> a sbol:SequenceAnnotation ;
    sbol:persistentIdentity <https://lab/parent/sa1> ;
    sbol:displayId "sa1" ;
    sbol:version "1" ;
    sbol:component <https://lab/parent/sub/1> ;
    sbol:location <https://lab/parent/sa1/range1/1> .

<https://lab/parent/sa1/range1/1> a sbol:Range ;
    sbol:persistentIdentity <https://lab/parent/sa1/range1> ;
    sbol:displayId "range1" ;
    sbol:version "1" ;
    sbol:start 1 ;
    sbol:end 10 .

<https://lab/parent/sa2/1> a sbol:SequenceAnnotation ;
    sbol:persistentIdentity <https://lab/parent/sa2> ;
    sbol:displayId "sa2" ;
    sbol:version "1" ;
    sbol:component <https://lab/parent/sub/1> ;
    sbol:location <https://lab/parent/sa2/range1/1> .

<https://lab/parent/sa2/range1/1> a sbol:Range ;
    sbol:persistentIdentity <https://lab/parent/sa2/range1> ;
    sbol:displayId "range1" ;
    sbol:version "1" ;
    sbol:start 50 ;
    sbol:end 60 .

<https://lab/widget/1> a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://lab/widget> ;
    sbol:displayId "widget" ;
    sbol:version "1" ;
    sbol:type biopax:DnaRegion .
"#;
    let (document, _report) =
        Document::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");

    // Both source Ranges must survive as distinct subjects in the SBOL 3
    // output (rather than one merging into the other).
    let range_iris: std::collections::HashSet<String> = document
        .rdf_graph()
        .triples()
        .iter()
        .filter(|t| {
            t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
                && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v3#Range")
        })
        .filter_map(|t| t.subject.as_iri().map(|i| i.as_str().to_owned()))
        .collect();
    assert_eq!(
        range_iris.len(),
        2,
        "two source Ranges must round-trip as two distinct SBOL 3 subjects, got {range_iris:?}"
    );

    // The canonical-IRI Range keeps its source coordinates; the
    // disambiguated Range carries the OTHER source's coordinates.
    let coord = |iri: &str, pred: &str| -> Option<String> {
        document.rdf_graph().triples().iter().find_map(|t| {
            (t.subject.as_iri().map(|i| i.as_str()) == Some(iri) && t.predicate.as_str() == pred)
                .then(|| t.object.as_literal().map(|l| l.value().to_owned()))
                .flatten()
        })
    };
    let starts: std::collections::BTreeSet<String> = range_iris
        .iter()
        .filter_map(|iri| coord(iri, "http://sbols.org/v3#start"))
        .collect();
    assert!(
        starts.contains("1") && starts.contains("50"),
        "both source start values must be preserved across the two Ranges, got {starts:?}"
    );

    // The disambiguated Range's displayId must match its IRI's last
    // segment — sbol3-10204 compliance check.
    let disambig_iri = "https://lab/parent/sub/range1_2";
    assert_eq!(
        coord(disambig_iri, "http://sbols.org/v3#displayId").as_deref(),
        Some("range1_2"),
        "disambiguated Location's displayId must match its IRI's last segment"
    );
}
