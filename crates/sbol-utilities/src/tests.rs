use crate::prelude::*;
use sbol3::constants::{
    RESTRICTION_MEETS, SBO_DNA, SO_CDS, SO_ENGINEERED_REGION, SO_PROMOTER, SO_RBS, SO_TERMINATOR,
};
use sbol3::design::Design;
use sbol3::prelude::SbolIdentified;

const NS: &str = "https://github.com/DRAGGON-Lab";

#[test]
fn promoter_creates_component_and_sequence() {
    let mut d = Design::new(NS).unwrap();
    d.promoter("pLac", "caatacg")
        .description("Promoter for LacI repressible expression")
        .add();
    let doc = d.finish().unwrap();

    assert_eq!(doc.sequences().count(), 1);
    let plac = doc
        .components()
        .find(|c| c.display_id() == Some("pLac"))
        .unwrap();
    assert_eq!(plac.types, vec![SBO_DNA]);
    assert_eq!(plac.roles, vec![SO_PROMOTER]);
    assert_eq!(plac.sequences.len(), 1);
    assert_eq!(
        plac.description(),
        Some("Promoter for LacI repressible expression")
    );
}

#[test]
fn engineered_region_orders_parts_and_copies_roles() {
    let mut d = Design::new(NS).unwrap();
    let plac = d.promoter("pLac", "caatacg").add();
    let b0034 = d.rbs("B0034", "ttgaac").add();
    let tetr = d.cds("tetR", "atggtg").add();
    let b0015 = d.terminator("B0015", "GTCCat").add();

    d.engineered_region("pLac_tu", [plac, b0034, tetr, b0015])
        .description("Transcriptional unit")
        .add();

    let doc = d.finish().unwrap();

    let region = doc
        .components()
        .find(|c| c.display_id() == Some("pLac_tu"))
        .unwrap();
    assert_eq!(region.roles, vec![SO_ENGINEERED_REGION]);
    assert_eq!(region.features.len(), 4);
    assert_eq!(region.constraints.len(), 3); // 4 parts -> 3 meets

    // Each sub-component carries the copied SO role of its part.
    let roles_present: Vec<_> = doc
        .sub_components()
        .flat_map(|s| s.feature.roles.clone())
        .collect();
    for role in [SO_PROMOTER, SO_RBS, SO_CDS, SO_TERMINATOR] {
        assert!(roles_present.contains(&role), "missing role {role:?}");
    }

    // Every constraint is a `meets`.
    assert!(
        doc.constraints()
            .all(|c| c.restriction.as_ref() == Some(&RESTRICTION_MEETS))
    );

    assert!(doc.check().is_ok());
}

#[test]
fn engineered_region_places_detached_feature_as_configured() {
    let mut d = Design::new(NS).unwrap();
    let plac = d.promoter("pLac", "caatacg").add();
    let tetr = d.cds("tetR", "atggtg").add();

    // A detached feature instancing the promoter but carrying a CDS role of
    // its own; the region keeps the feature's config rather than copying the
    // instantiated component's roles.
    let custom = d
        .detached_sub_component("custom")
        .instance_of(plac)
        .role(SO_CDS)
        .add();

    let parts: [Part; 3] = [plac.into(), custom.into(), tetr.into()];
    d.engineered_region("tu", parts).add();

    let doc = d.finish().unwrap();

    let region = doc
        .components()
        .find(|c| c.display_id() == Some("tu"))
        .unwrap();
    assert_eq!(region.features.len(), 3);
    assert_eq!(region.constraints.len(), 2); // 3 features -> 2 meets

    let custom_sub = doc
        .sub_components()
        .find(|s| s.display_id() == Some("custom"))
        .unwrap();
    assert_eq!(custom_sub.feature.roles, vec![SO_CDS]);
    assert!(custom_sub.instance_of.is_some());

    assert!(doc.check().is_ok());
}

#[test]
fn part_without_roles_is_reported() {
    let mut d = Design::new(NS).unwrap();
    // A bare component with no role — the DNAplotlib precondition should fire.
    let bare = d.component("bare").dna().add();
    d.engineered_region("region", [bare]).add();

    let err = d.finish().expect_err("should report missing roles");
    assert!(err.problems.iter().any(|p| matches!(
        p,
        sbol3::design::DesignProblem::Custom(msg) if msg.contains("no roles")
    )));
}
