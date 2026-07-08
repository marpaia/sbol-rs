use super::*;
use crate::constants::{EDAM_IUPAC_DNA, SBO_DNA, SO_ENGINEERED_REGION, SO_PROMOTER, SO_RBS};
use crate::prelude::*;

const NS: &str = "https://example.org/lab";

#[test]
fn component_references_its_sequence_and_role() {
    let mut d = Design::new(NS).unwrap();
    let seq = d.sequence("pLac_seq").elements("caatacg").dna().add();
    let _plac = d
        .component("pLac")
        .dna()
        .role(SO_PROMOTER)
        .sequence(seq)
        .description("a promoter")
        .add();
    let doc = d.finish().unwrap();

    let plac = doc
        .components()
        .find(|c| c.display_id() == Some("pLac"))
        .expect("component present");
    assert_eq!(plac.types, vec![SBO_DNA]);
    assert_eq!(plac.roles, vec![SO_PROMOTER]);
    assert_eq!(plac.sequences.len(), 1);
    assert_eq!(
        plac.sequences[0].as_iri().unwrap().as_str(),
        "https://example.org/lab/pLac_seq"
    );
    assert_eq!(doc.sequences().count(), 1);
}

#[test]
fn sub_components_are_meets_ordered_under_parent() {
    let mut d = Design::new(NS).unwrap();
    let plac = d.component("pLac").dna().role(SO_PROMOTER).add();
    let b0034 = d.component("B0034").dna().role(SO_RBS).add();

    let tu = d
        .component("pLac_tu")
        .dna()
        .role(SO_ENGINEERED_REGION)
        .add();
    let sub_plac = d
        .sub_component(tu, "pLac_sub")
        .instance_of(plac)
        .role(SO_PROMOTER)
        .add();
    let sub_b0034 = d
        .sub_component(tu, "B0034_sub")
        .instance_of(b0034)
        .role(SO_RBS)
        .add();
    d.meets(tu, sub_plac, sub_b0034);

    let doc = d.finish().unwrap();

    let region = doc
        .components()
        .find(|c| c.display_id() == Some("pLac_tu"))
        .expect("region present");
    assert_eq!(region.features.len(), 2);
    assert_eq!(region.constraints.len(), 1);

    // Child identities are minted under the parent.
    assert!(region.features.iter().all(|f| {
        f.as_iri()
            .unwrap()
            .as_str()
            .starts_with("https://example.org/lab/pLac_tu/")
    }));

    let sub = doc
        .sub_components()
        .find(|s| s.display_id() == Some("pLac_sub"))
        .expect("sub present");
    assert_eq!(
        sub.instance_of.as_ref().unwrap().as_iri().unwrap().as_str(),
        "https://example.org/lab/pLac"
    );
    assert_eq!(sub.feature.roles, vec![SO_PROMOTER]);

    let constraint = doc.constraints().next().expect("constraint present");
    assert_eq!(constraint.restriction.as_ref(), Some(&RESTRICTION_MEETS));
    assert_eq!(
        constraint
            .subject
            .as_ref()
            .unwrap()
            .as_iri()
            .unwrap()
            .as_str(),
        "https://example.org/lab/pLac_tu/pLac_sub"
    );
    assert_eq!(
        constraint
            .constrained_object
            .as_ref()
            .unwrap()
            .as_iri()
            .unwrap()
            .as_str(),
        "https://example.org/lab/pLac_tu/B0034_sub"
    );
}

#[test]
fn detached_feature_is_placed_under_parent() {
    let mut d = Design::new(NS).unwrap();
    let plac = d.component("pLac").dna().role(SO_PROMOTER).add();

    // Build the feature before its parent exists, then place it.
    let detached = d
        .detached_sub_component("pLac_sub")
        .instance_of(plac)
        .role(SO_PROMOTER)
        .add();

    let tu = d
        .component("pLac_tu")
        .dna()
        .role(SO_ENGINEERED_REGION)
        .add();
    let placed = d.place_feature(tu, detached);
    let b0034 = d.component("B0034").dna().role(SO_RBS).add();
    let sub_b0034 = d
        .sub_component(tu, "B0034_sub")
        .instance_of(b0034)
        .role(SO_RBS)
        .add();
    d.meets(tu, placed, sub_b0034);

    let doc = d.finish().unwrap();

    let region = doc
        .components()
        .find(|c| c.display_id() == Some("pLac_tu"))
        .expect("region present");
    assert_eq!(region.features.len(), 2);

    // The placed feature is minted under the region it was placed in.
    let sub = doc
        .sub_components()
        .find(|s| s.display_id() == Some("pLac_sub"))
        .expect("placed feature present");
    assert_eq!(
        sub.instance_of.as_ref().unwrap().as_iri().unwrap().as_str(),
        "https://example.org/lab/pLac"
    );
    assert!(
        sub.identity
            .as_iri()
            .unwrap()
            .as_str()
            .starts_with("https://example.org/lab/pLac_tu/")
    );
    assert_eq!(sub.feature.roles, vec![SO_PROMOTER]);
    assert!(doc.check().is_ok());
}

#[test]
fn placing_a_non_detached_feature_is_reported() {
    let mut d = Design::new(NS).unwrap();
    let host = d.component("host").dna().role(SO_ENGINEERED_REGION).add();
    let inner = d.component("inner").dna().role(SO_PROMOTER).add();
    // A normally-built (already parented) feature is not placeable.
    let built = d
        .sub_component(host, "inner_sub")
        .instance_of(inner)
        .role(SO_PROMOTER)
        .add();
    let other = d.component("other").dna().role(SO_ENGINEERED_REGION).add();
    d.place_feature(other, built);

    let err = d
        .finish()
        .expect_err("placing a built feature should be reported");
    assert!(err.problems.iter().any(|p| matches!(
        p,
        DesignProblem::Custom(msg) if msg.contains("detached feature")
    )));
}

#[test]
fn from_document_round_trips_and_allows_extension() {
    // Build a small document with the arena.
    let mut d = Design::new(NS).unwrap();
    let plac = d.component("pLac").dna().role(SO_PROMOTER).add();
    let _tu = d
        .component("pLac_tu")
        .dna()
        .role(SO_ENGINEERED_REGION)
        .add();
    let _ = plac;
    let original = d.finish().unwrap();
    let original_count = original.typed_objects().len();

    // Re-import it: every object should survive a finish() unchanged.
    let reimported = Design::from_document(&original).unwrap().finish().unwrap();
    assert_eq!(reimported.typed_objects().len(), original_count);
    assert!(
        reimported
            .components()
            .any(|c| c.display_id() == Some("pLac"))
    );
    assert!(reimported.check().is_ok());

    // Import again and add a child under an existing component by identity.
    let region_iri = original
        .components()
        .find(|c| c.display_id() == Some("pLac_tu"))
        .unwrap()
        .identity
        .clone();
    let plac_iri = original
        .components()
        .find(|c| c.display_id() == Some("pLac"))
        .unwrap()
        .identity
        .clone();

    let mut design = Design::from_document(&original).unwrap();
    let region = design
        .component_id(&region_iri)
        .expect("imported region handle");
    let plac = design
        .component_id(&plac_iri)
        .expect("imported part handle");
    design
        .sub_component(region, "pLac_sub")
        .instance_of(plac)
        .role(SO_PROMOTER)
        .add();
    let extended = design.finish().unwrap();

    assert_eq!(extended.typed_objects().len(), original_count + 1);
    let region = extended
        .components()
        .find(|c| c.display_id() == Some("pLac_tu"))
        .unwrap();
    assert_eq!(region.features.len(), 1);
    assert!(extended.check().is_ok());
}

#[test]
fn from_document_requires_a_namespace() {
    // A document assembled with no namespaced top-level cannot be imported.
    let empty = Document::from_objects(Vec::new()).unwrap();
    match Design::from_document(&empty) {
        Err(err) => assert!(matches!(
            err.problems.as_slice(),
            [DesignProblem::Custom(_)]
        )),
        Ok(_) => panic!("expected import to fail without a namespace"),
    }
}

#[test]
fn invalid_display_id_is_reported_at_finish() {
    let mut d = Design::new(NS).unwrap();
    d.component("has a space").dna().add();
    let err = d.finish().expect_err("should fail");
    assert_eq!(err.problems.len(), 1);
    match &err.problems[0] {
        DesignProblem::Build { display_id, .. } => assert_eq!(display_id, "has a space"),
        other => panic!("unexpected problem: {other:?}"),
    }
}

#[test]
fn multiple_errors_aggregate() {
    let mut d = Design::new(NS).unwrap();
    d.component("bad one").dna().add();
    d.sequence("bad two").add();
    let err = d.finish().expect_err("should fail");
    assert_eq!(err.problems.len(), 2);
}

#[test]
fn round_trip_equals_hand_built() {
    let mut d = Design::new(NS).unwrap();
    let seq = d.sequence("pLac_seq").elements("caatacg").dna().add();
    d.component("pLac")
        .dna()
        .role(SO_PROMOTER)
        .sequence(seq)
        .add();
    let from_arena = d.finish().unwrap();

    let sequence = Sequence::builder(NS, "pLac_seq")
        .unwrap()
        .elements("caatacg")
        .encoding(EDAM_IUPAC_DNA)
        .build()
        .unwrap();
    let component = Component::builder(NS, "pLac")
        .unwrap()
        .types([SBO_DNA])
        .add_component_role(SO_PROMOTER)
        .add_sequence(sequence.identity.clone())
        .build()
        .unwrap();
    let hand_built = Document::from_objects(vec![
        SbolObject::Sequence(sequence),
        SbolObject::Component(component),
    ])
    .unwrap();

    assert!(from_arena.diff(&hand_built).is_empty());
}

#[test]
fn well_formed_design_has_no_validation_errors() {
    let mut d = Design::new(NS).unwrap();
    let seq = d.sequence("pLac_seq").elements("caatacg").dna().add();
    d.component("pLac")
        .dna()
        .role(SO_PROMOTER)
        .sequence(seq)
        .add();
    let doc = d.finish().unwrap();
    assert!(doc.check().is_ok(), "expected no validation errors");
}

#[test]
fn sanitize_display_id_enforces_lexical_rules() {
    assert_eq!(sanitize_display_id("pLac"), "pLac");
    assert_eq!(sanitize_display_id("has a space"), "has_a_space");
    assert_eq!(sanitize_display_id("B0015-double"), "B0015_double");
    assert_eq!(sanitize_display_id("123start"), "_123start");
    assert_eq!(sanitize_display_id(""), "_");
}
