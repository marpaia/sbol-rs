use crate::prelude::*;
use sbol3::Iri;
use sbol3::constants::{
    EDAM_IUPAC_PROTEIN, EDAM_IUPAC_RNA, SBO_DNA, SBO_FUNCTIONAL_ENTITY, SBO_PROTEIN, SBO_RNA,
    SO_GENE, SO_MRNA, SO_OPERATOR,
};
use sbol3::design::Design;
use sbol3::prelude::SbolIdentified;

const NS: &str = "https://github.com/DRAGGON-Lab";

#[test]
fn gene_creates_dna_component_and_sequence() {
    let mut d = Design::new(NS).unwrap();
    d.gene("araC", "atggtgaaacag").add();
    let doc = d.finish().unwrap();

    assert_eq!(doc.sequences().count(), 1);
    let c = doc
        .components()
        .find(|c| c.display_id() == Some("araC"))
        .unwrap();
    assert_eq!(c.types, vec![SBO_DNA]);
    assert_eq!(c.roles, vec![SO_GENE]);
    assert_eq!(c.sequences.len(), 1);
    assert!(doc.check().is_ok());
}

#[test]
fn operator_creates_dna_component_and_sequence() {
    let mut d = Design::new(NS).unwrap();
    d.operator("operator1", "aattgtgagc").add();
    let doc = d.finish().unwrap();

    let c = doc
        .components()
        .find(|c| c.display_id() == Some("operator1"))
        .unwrap();
    assert_eq!(c.types, vec![SBO_DNA]);
    assert_eq!(c.roles, vec![SO_OPERATOR]);
    assert!(doc.check().is_ok());
}

#[test]
fn mrna_creates_rna_component_with_rna_sequence() {
    let mut d = Design::new(NS).unwrap();
    d.mrna("gfp_mrna", "auggugagcaag").add();
    let doc = d.finish().unwrap();

    let c = doc
        .components()
        .find(|c| c.display_id() == Some("gfp_mrna"))
        .unwrap();
    assert_eq!(c.types, vec![SBO_RNA]);
    assert_eq!(c.roles, vec![SO_MRNA]);

    let seq = doc.sequences().next().unwrap();
    assert_eq!(seq.encoding, Some(EDAM_IUPAC_RNA));
    assert!(doc.check().is_ok());
}

#[test]
fn transcription_factor_creates_protein_component_with_sequence() {
    let mut d = Design::new(NS).unwrap();
    d.transcription_factor("tetR_tf", "MARLNRESVI").add();
    let doc = d.finish().unwrap();

    let c = doc
        .components()
        .find(|c| c.display_id() == Some("tetR_tf"))
        .unwrap();
    assert_eq!(c.types, vec![SBO_PROTEIN]);
    assert_eq!(
        c.roles,
        vec![Iri::from_static("https://identifiers.org/SO:0003700")]
    );

    let seq = doc.sequences().next().unwrap();
    assert_eq!(seq.encoding, Some(EDAM_IUPAC_PROTEIN));
    assert!(
        doc.check().is_ok(),
        "TF_CHECK_FAILED: {:?}",
        doc.check().err()
    );
}

#[test]
fn functional_component_has_type_but_no_sequence() {
    let mut d = Design::new(NS).unwrap();
    d.functional_component("LacI")
        .description("LacI tetramer")
        .add();
    let doc = d.finish().unwrap();

    assert_eq!(doc.sequences().count(), 0);
    let c = doc
        .components()
        .find(|c| c.display_id() == Some("LacI"))
        .unwrap();
    assert_eq!(c.types, vec![SBO_FUNCTIONAL_ENTITY]);
    assert!(c.roles.is_empty());
    assert!(c.sequences.is_empty());
    assert_eq!(c.description(), Some("LacI tetramer"));
    assert!(doc.check().is_ok());
}
