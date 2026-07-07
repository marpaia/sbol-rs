//! Acceptance test for the toggle-switch workflow: it must build with no IRI
//! or `SbolObject` handling, validate without errors, and produce
//! `meets`-ordered sub-components with roles copied from their parts.

use sbol_utilities::prelude::*;
use sbol3::constants::{RESTRICTION_MEETS, SO_ENGINEERED_REGION};
use sbol3::design::Design;
use sbol3::prelude::{RdfFormat, SbolIdentified};

const NS: &str = "https://github.com/DRAGGON-Lab";

fn build_toggle_switch() -> Design {
    let mut d = Design::new(NS).unwrap();

    let plac = d.promoter("pLac", "caatacg").add();
    let b0034 = d.rbs("B0034", "ttgaac").add();
    let tetr = d.cds("tetR", "atggtg").add();
    let b0015 = d.terminator("B0015", "GTCCat").add();
    d.engineered_region("pLac_tu", [plac, b0034, tetr, b0015])
        .description("produces TetR")
        .add();

    let ptet = d.promoter("pTet", "tccctat").add();
    let b0064 = d.rbs("B0064", "AAAGAG").add();
    let laci = d.cds("lacI", "atggtg").add();
    let bd14 = d.rbs("BD14", "gggccc").add();
    let gfp = d.cds("gfp", "atggca").add();
    let l3s2p21 = d.terminator("L3S2P21", "CTCGGT").add();
    d.engineered_region("pTet_tu", [ptet, b0064, laci, bd14, gfp, l3s2p21])
        .description("produces GFP")
        .add();

    d
}

#[test]
fn toggle_switch_builds_and_validates() {
    let doc = build_toggle_switch().finish().expect("well-formed design");

    // 10 parts + 2 regions = 12 components; 10 part sequences.
    assert_eq!(doc.components().count(), 12);
    assert_eq!(doc.sequences().count(), 10);

    let regions: Vec<_> = doc
        .components()
        .filter(|c| c.roles.contains(&SO_ENGINEERED_REGION))
        .collect();
    assert_eq!(regions.len(), 2);

    let plac_tu = regions
        .iter()
        .find(|c| c.display_id() == Some("pLac_tu"))
        .unwrap();
    assert_eq!(plac_tu.features.len(), 4);
    assert_eq!(plac_tu.constraints.len(), 3);

    let ptet_tu = regions
        .iter()
        .find(|c| c.display_id() == Some("pTet_tu"))
        .unwrap();
    assert_eq!(ptet_tu.features.len(), 6);
    assert_eq!(ptet_tu.constraints.len(), 5);

    // Every sub-component carries copied roles, and every constraint is `meets`.
    assert!(doc.sub_components().all(|s| !s.feature.roles.is_empty()));
    assert!(
        doc.constraints()
            .all(|c| c.restriction.as_ref() == Some(&RESTRICTION_MEETS))
    );

    assert!(
        doc.check().is_ok(),
        "toggle switch should have no validation errors"
    );
}

#[test]
fn toggle_switch_serializes_to_ntriples() {
    let doc = build_toggle_switch().finish().unwrap();
    let nt = doc.write(RdfFormat::NTriples).expect("serializes");
    assert!(nt.contains("/pLac_tu"));
    assert!(nt.contains("http://sbols.org/v3#meets"));
}
