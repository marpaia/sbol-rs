//! Integration tests validating SBOL 2 → SBOL 3 upgrade against
//! real-world fixtures, plus backport-metadata preservation and
//! identity-collision auditing.

mod common;

use common::upgrade::*;

use sbol::{Document, RdfFormat, UpgradeWarning};

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
