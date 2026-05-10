//! Tests for typed iterators and parent → child traversal on `Document`.

use sbol::constants::{
    CARDINALITY_ONE, RESTRICTION_PRECEDES, SBO_DNA, SBO_GENETIC_PRODUCTION, SBO_PRODUCT,
    SO_PROMOTER,
};
use sbol::prelude::SbolIdentified;
use sbol::{
    Collection, Component, Document, FeatureRef, Interaction, LocationRef, Participation,
    SbolObject, SequenceFeature, SubComponent,
};

fn build_doc() -> (Document, /* component identity */ sbol::Resource) {
    // Component → has feature (SubComponent) → references another Component
    // Component → has sequence feature → has range location
    // Component → has interaction → has participation
    // Component → has constraint
    let other = Component::new("https://example.org/lab", "other", [SBO_DNA.clone()]).unwrap();
    let sequence = sbol::Sequence::new("https://example.org/lab", "seq").unwrap();

    let mut owner = Component::builder("https://example.org/lab", "owner")
        .unwrap()
        .types([SBO_DNA.clone()])
        .build()
        .unwrap();

    let sub = SubComponent::new(&owner.identity, "sub", other.identity.clone()).unwrap();
    owner.features.push(sub.identity.clone());

    // SequenceFeature requires locations; build the Range first so we can wire it in.
    let sf_identity = sbol::Resource::iri("https://example.org/lab/owner/sf1");
    let range = sbol::Range::builder(&sf_identity, "r")
        .unwrap()
        .start(1)
        .end(100)
        .sequence(sequence.identity.clone())
        .build()
        .unwrap();
    let sf = SequenceFeature::builder(&owner.identity, "sf1")
        .unwrap()
        .roles([SO_PROMOTER.clone()])
        .add_location(range.identity.clone())
        .build()
        .unwrap();
    owner.features.push(sf.identity.clone());

    let interaction =
        Interaction::new(&owner.identity, "i1", [SBO_GENETIC_PRODUCTION.clone()]).unwrap();
    let participation =
        Participation::new(&interaction.identity, "p1", [SBO_PRODUCT.clone()]).unwrap();
    let mut interaction = interaction;
    interaction
        .participations
        .push(participation.identity.clone());
    owner.interactions.push(interaction.identity.clone());

    let constraint = sbol::Constraint::new(
        &owner.identity,
        "k1",
        sub.identity.clone(),
        sf.identity.clone(),
        RESTRICTION_PRECEDES.clone(),
    )
    .unwrap();
    owner.constraints.push(constraint.identity.clone());

    let variable_feature = sbol::VariableFeature::new(
        &sbol::Resource::iri("https://example.org/lab/cd"),
        "vf",
        CARDINALITY_ONE.clone(),
        sub.identity.clone(),
    )
    .unwrap();

    let owner_identity = owner.identity.clone();

    let objects = vec![
        SbolObject::Component(owner),
        SbolObject::Component(other),
        SbolObject::Sequence(sequence),
        SbolObject::SubComponent(sub),
        SbolObject::SequenceFeature(sf),
        SbolObject::Range(range),
        SbolObject::Interaction(interaction),
        SbolObject::Participation(participation),
        SbolObject::Constraint(constraint),
        SbolObject::VariableFeature(variable_feature),
    ];

    let document = Document::from_objects(objects).unwrap();
    (document, owner_identity)
}

#[test]
fn find_by_display_id_returns_typed_object() {
    let (doc, _) = build_doc();
    let typed = &doc; // alias for readability; methods are on Document
    let owner = typed
        .find_by_display_id("https://example.org/lab", "owner")
        .expect("owner must be findable");
    assert!(matches!(owner, SbolObject::Component(_)));
}

#[test]
fn find_by_display_id_returns_none_for_missing() {
    let (doc, _) = build_doc();
    let typed = &doc; // alias for readability; methods are on Document
    assert!(
        typed
            .find_by_display_id("https://example.org/lab", "nonexistent")
            .is_none()
    );
}

#[test]
fn find_by_display_id_resolves_compliant_url_with_nonempty_local_path() {
    // SBOL 3.1.0 §5.1: compliant URLs are `[namespace]/[local]/[displayId]`
    // where the `local` fragment is an optional path. The lookup must
    // succeed regardless of whether `local` is present.
    let turtle = r#"
PREFIX sbol: <http://sbols.org/v3#>
PREFIX SBO: <https://identifiers.org/SBO:>

<https://example.org/lab/library/2024/widget> a sbol:Component ;
    sbol:displayId "widget" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type SBO:0000251 .
"#;
    let document = sbol::Document::read_turtle(turtle).unwrap();
    let widget = document
        .find_by_display_id("https://example.org/lab", "widget")
        .expect("widget must be findable despite the `library/2024/` local path");
    assert_eq!(
        widget.identity().to_string(),
        "https://example.org/lab/library/2024/widget"
    );
}

#[test]
fn namespaces_returns_distinct_top_level_namespaces() {
    let (doc, _) = build_doc();
    let typed = &doc; // alias for readability; methods are on Document
    let namespaces: Vec<&str> = typed.namespaces().map(|i| i.as_str()).collect();
    assert_eq!(namespaces, vec!["https://example.org/lab"]);
}

#[test]
fn component_features_iterates_typed_children() {
    let (doc, _) = build_doc();
    let typed = &doc; // alias for readability; methods are on Document
    let owner = typed
        .components()
        .find(|c| c.identified.display_id.as_deref() == Some("owner"))
        .unwrap();
    let features: Vec<_> = owner.features(typed).collect();
    assert_eq!(features.len(), 2);
    let kinds: Vec<&str> = features
        .iter()
        .map(|f| match f {
            FeatureRef::SubComponent(_) => "sub",
            FeatureRef::SequenceFeature(_) => "sf",
            _ => "other",
        })
        .collect();
    assert!(kinds.contains(&"sub"));
    assert!(kinds.contains(&"sf"));
}

#[test]
fn component_constraints_resolves_typed_children() {
    let (doc, _) = build_doc();
    let typed = &doc; // alias for readability; methods are on Document
    let owner = typed
        .components()
        .find(|c| c.identified.display_id.as_deref() == Some("owner"))
        .unwrap();
    let constraints: Vec<_> = owner.constraints(typed).collect();
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0].identified.display_id.as_deref(), Some("k1"));
}

#[test]
fn interaction_participations_resolves_typed_children() {
    let (doc, _) = build_doc();
    let typed = &doc; // alias for readability; methods are on Document
    let owner = typed
        .components()
        .find(|c| c.identified.display_id.as_deref() == Some("owner"))
        .unwrap();
    let interaction = owner.interactions(typed).next().unwrap();
    let participations: Vec<_> = interaction.participations(typed).collect();
    assert_eq!(participations.len(), 1);
}

#[test]
fn sequence_feature_locations_resolves_typed_children() {
    let (doc, _) = build_doc();
    let typed = &doc; // alias for readability; methods are on Document
    let owner = typed
        .components()
        .find(|c| c.identified.display_id.as_deref() == Some("owner"))
        .unwrap();
    let sf = owner
        .features(typed)
        .filter_map(|f| match f {
            FeatureRef::SequenceFeature(sf) => Some(sf),
            _ => None,
        })
        .next()
        .unwrap();
    let locations: Vec<_> = sf.locations(typed).collect();
    assert_eq!(locations.len(), 1);
    assert!(matches!(locations[0], LocationRef::Range(_)));
}

#[test]
fn parent_identity_returns_none_for_top_level_objects() {
    let (doc, owner_identity) = build_doc();
    let owner = doc.resolve(&owner_identity).expect("owner present");
    assert!(owner.parent_identity().is_none());
}

#[test]
fn parent_identity_returns_parent_for_child_objects() {
    let (doc, _) = build_doc();
    let sub = doc
        .sub_components()
        .next()
        .expect("subcomponent in document");
    let parent = SbolObject::SubComponent(sub.clone())
        .parent_identity()
        .expect("subcomponent has parent identity");
    assert_eq!(parent.to_string(), "https://example.org/lab/owner");
    assert!(doc.resolve(&parent).is_some());
}

#[test]
fn document_resolve_and_namespaces_expose_public_resolver_surface() {
    let (doc, owner_identity) = build_doc();
    assert!(doc.resolve(&owner_identity).is_some());
    let namespaces: Vec<&str> = doc.namespaces().map(|i| i.as_str()).collect();
    assert_eq!(namespaces, vec!["https://example.org/lab"]);
}

#[test]
fn sub_component_parent_component_returns_owner() {
    let (doc, owner_identity) = build_doc();
    let sub = doc
        .sub_components()
        .find(|s| s.display_id() == Some("sub"))
        .expect("sub feature must exist");
    let parent = sub
        .parent_component(&doc)
        .expect("sub belongs to a component");
    assert_eq!(parent.identity, owner_identity);
}

#[test]
fn sequence_feature_parent_component_returns_owner() {
    let (doc, owner_identity) = build_doc();
    let sf = doc
        .sequence_features()
        .find(|s| s.display_id() == Some("sf1"))
        .expect("sequence feature must exist");
    let parent = sf
        .parent_component(&doc)
        .expect("sequence feature belongs to a component");
    assert_eq!(parent.identity, owner_identity);
}

#[test]
fn participation_parent_interaction_returns_owner() {
    let (doc, _) = build_doc();
    let participation = doc
        .participations()
        .next()
        .expect("participation must exist");
    let interaction = participation
        .parent_interaction(&doc)
        .expect("participation belongs to an interaction");
    assert!(
        interaction.participations.contains(&participation.identity),
        "interaction must list the participation among its children"
    );
}

#[test]
fn component_parent_collections_lists_each_membership() {
    // The build_doc() fixture has no Collections — set up a small graph
    // with one Component included in two distinct Collections.
    let component =
        Component::new("https://example.org/lab", "shared_part", [SBO_DNA.clone()]).unwrap();
    let mut collection_a = Collection::new("https://example.org/lab", "collection_a").unwrap();
    let mut collection_b = Collection::new("https://example.org/lab", "collection_b").unwrap();
    collection_a.members.push(component.identity.clone());
    collection_b.members.push(component.identity.clone());

    let doc = Document::from_objects(vec![
        SbolObject::Component(component.clone()),
        SbolObject::Collection(collection_a),
        SbolObject::Collection(collection_b),
    ])
    .unwrap();

    let resolved = doc
        .components()
        .find(|c| c.display_id() == Some("shared_part"))
        .unwrap();
    let collections = resolved.parent_collections(&doc);
    assert_eq!(collections.len(), 2);
    let names: Vec<_> = collections.iter().filter_map(|c| c.display_id()).collect();
    assert!(names.contains(&"collection_a"));
    assert!(names.contains(&"collection_b"));
}

#[test]
fn parent_lookup_returns_none_when_owner_is_absent() {
    // A free-floating SubComponent whose owning Component is not in the
    // document yields None — the helpers do not invent ownership.
    let owner = Component::new("https://example.org/lab", "owner", [SBO_DNA.clone()]).unwrap();
    let sub = SubComponent::new(&owner.identity, "orphan_sub", owner.identity.clone()).unwrap();

    let doc = Document::from_objects(vec![SbolObject::SubComponent(sub.clone())]).unwrap();
    let resolved = doc
        .sub_components()
        .find(|s| s.display_id() == Some("orphan_sub"))
        .unwrap();
    assert!(resolved.parent_component(&doc).is_none());
}

#[test]
fn unresolved_references_are_silently_skipped() {
    // Build an Interaction that references a non-existent participation.
    let mut interaction = Interaction::new(
        &sbol::Resource::iri("https://example.org/lab/c"),
        "i1",
        [SBO_GENETIC_PRODUCTION.clone()],
    )
    .unwrap();
    interaction
        .participations
        .push(sbol::Resource::iri("https://example.org/missing"));

    let doc = Document::from_objects(vec![SbolObject::Interaction(interaction.clone())]).unwrap();
    let typed = &doc; // alias for readability; methods are on Document
    let i = typed.interactions().next().unwrap();
    let participations: Vec<_> = i.participations(typed).collect();
    assert!(participations.is_empty());
}
