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
