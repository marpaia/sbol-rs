use super::*;
use sbol3::constants::{CARDINALITY_ONE, SBO_DNA, SO_CDS, SO_PROMOTER, STRATEGY_ENUMERATE};
use sbol3::{
    CombinatorialDerivation, Component, Document, Resource, SbolIdentified, SbolObject,
    SubComponent, VariableFeature,
};

const NS: &str = "https://github.com/DRAGGON-Lab";

fn dna_component(display_id: &str, role: sbol3::Iri) -> Component {
    Component::builder(NS, display_id)
        .unwrap()
        .types([SBO_DNA])
        .add_component_role(role)
        .build()
        .unwrap()
}

fn derivation_iri(document: &Document, display_id: &str) -> Resource {
    document
        .combinatorial_derivations()
        .find(|cd| cd.display_id() == Some(display_id))
        .expect("derivation present")
        .identity
        .clone()
}

#[test]
fn expands_two_variables_into_full_cartesian_product() {
    let gfp = dna_component("gfp", SO_CDS);
    let rfp = dna_component("rfp", SO_CDS);
    let ptac = dna_component("pTac", SO_PROMOTER);
    let pbad = dna_component("pBad", SO_PROMOTER);

    // Template with two placeholder slots, each varied by one VariableFeature.
    let template = dna_component("cassette", SO_CDS);
    let promoter_slot = SubComponent::builder(&template.identity, "promoter_slot")
        .unwrap()
        .instance_of(ptac.identity.clone())
        .build()
        .unwrap();
    let cds_slot = SubComponent::builder(&template.identity, "cds_slot")
        .unwrap()
        .instance_of(gfp.identity.clone())
        .build()
        .unwrap();
    let mut template = template;
    template.features = vec![promoter_slot.identity.clone(), cds_slot.identity.clone()];

    let cd = CombinatorialDerivation::builder(NS, "cassette_variants")
        .unwrap()
        .template(template.identity.clone())
        .strategy(STRATEGY_ENUMERATE)
        .build()
        .unwrap();
    let promoter_var = VariableFeature::builder(&cd.identity, "promoter_var")
        .unwrap()
        .cardinality(CARDINALITY_ONE)
        .variable(promoter_slot.identity.clone())
        .variants([ptac.identity.clone(), pbad.identity.clone()])
        .build()
        .unwrap();
    let cds_var = VariableFeature::builder(&cd.identity, "cds_var")
        .unwrap()
        .cardinality(CARDINALITY_ONE)
        .variable(cds_slot.identity.clone())
        .variants([gfp.identity.clone(), rfp.identity.clone()])
        .build()
        .unwrap();
    let mut cd = cd;
    cd.variable_features = vec![promoter_var.identity.clone(), cds_var.identity.clone()];

    let objects = vec![
        SbolObject::Component(gfp),
        SbolObject::Component(rfp),
        SbolObject::Component(ptac),
        SbolObject::Component(pbad),
        SbolObject::Component(template),
        SbolObject::SubComponent(promoter_slot),
        SbolObject::SubComponent(cds_slot),
        SbolObject::CombinatorialDerivation(cd),
        SbolObject::VariableFeature(promoter_var),
        SbolObject::VariableFeature(cds_var),
    ];
    let doc = Document::from_objects(objects).unwrap();

    let expanded = expand_derivations(&doc).unwrap();

    let members: Vec<Resource> = expanded
        .collections()
        .find(|collection| collection.display_id() == Some("cassette_variants_derivatives"))
        .expect("derivatives collection present")
        .members
        .clone();
    assert_eq!(members.len(), 4, "2 x 2 variants = 4 derived components");

    let derived: Vec<_> = expanded
        .components()
        .filter(|component| members.iter().any(|member| member == &component.identity))
        .collect();
    assert_eq!(derived.len(), 4);
    for component in &derived {
        assert_eq!(
            component.features.len(),
            2,
            "each derived component keeps both slots"
        );
    }

    // Five authored components plus four derived ones.
    assert_eq!(expanded.components().count(), 9);
    assert!(
        expanded.check().is_ok(),
        "expanded document should validate"
    );
}

#[test]
fn library_derivation_collects_variants_directly() {
    let j23100 = dna_component("j23100", SO_PROMOTER);
    let j23101 = dna_component("j23101", SO_PROMOTER);

    // One variable over a simple, single-feature template: a library.
    let template = dna_component("promoter_library_template", SO_PROMOTER);
    let slot = SubComponent::builder(&template.identity, "slot")
        .unwrap()
        .instance_of(j23100.identity.clone())
        .build()
        .unwrap();
    let mut template = template;
    template.features = vec![slot.identity.clone()];

    let cd = CombinatorialDerivation::builder(NS, "promoter_library")
        .unwrap()
        .template(template.identity.clone())
        .build()
        .unwrap();
    let var = VariableFeature::builder(&cd.identity, "var")
        .unwrap()
        .cardinality(CARDINALITY_ONE)
        .variable(slot.identity.clone())
        .variants([j23100.identity.clone(), j23101.identity.clone()])
        .build()
        .unwrap();
    let mut cd = cd;
    cd.variable_features = vec![var.identity.clone()];

    let objects = vec![
        SbolObject::Component(j23100),
        SbolObject::Component(j23101),
        SbolObject::Component(template),
        SbolObject::SubComponent(slot),
        SbolObject::CombinatorialDerivation(cd),
        SbolObject::VariableFeature(var),
    ];
    let doc = Document::from_objects(objects).unwrap();

    let expanded = expand_derivation(&doc, &derivation_iri(&doc, "promoter_library")).unwrap();

    let collection = expanded
        .collections()
        .find(|collection| collection.display_id() == Some("promoter_library_collection"))
        .expect("library collection present");
    assert_eq!(collection.members.len(), 2);
    // A library lists the variant components themselves; no clones are minted.
    assert_eq!(expanded.components().count(), 3);
    assert!(expanded.check().is_ok());
}

#[test]
fn missing_derivation_is_reported() {
    let doc =
        Document::from_objects(vec![SbolObject::Component(dna_component("x", SO_CDS))]).unwrap();
    let ghost = Resource::iri(format!("{NS}/nope"));
    let err = expand_derivation(&doc, &ghost).expect_err("unknown derivation should be reported");
    assert!(matches!(err, ExpandError::DerivationNotFound(_)));
}
