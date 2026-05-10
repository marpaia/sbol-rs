//! Integration tests for the public reference-resolution API
//! (`ObjectGraph`, `SubComponent::definition`, `ComponentReference::target` /
//! `trace`, `CombinatorialDerivation::variants`).

use sbol::constants::{CARDINALITY_ONE, SBO_DNA};
use sbol::prelude::*;

const NS: &str = "https://example.org/lab";

/// Builds a Component named `display_id` in the standard test namespace.
fn build_component(display_id: &str) -> Component {
    Component::new(NS, display_id, [SBO_DNA.clone()]).unwrap()
}

/// Wraps a typed SbolObject collection into a `Document`.
fn doc(objects: Vec<SbolObject>) -> Document {
    Document::from_objects(objects).unwrap()
}

// ---------- SubComponent::definition ----------

#[test]
fn subcomponent_definition_resolves_in_single_document() {
    let target = build_component("part");
    let mut owner = build_component("owner");
    let sub = SubComponent::new(&owner.identity, "sub", target.identity.clone()).unwrap();
    owner.features.push(sub.identity.clone());

    let document = doc(vec![
        SbolObject::Component(owner),
        SbolObject::Component(target.clone()),
        SbolObject::SubComponent(sub.clone()),
    ]);

    let definition = sub.definition(&document).expect("definition resolves");
    assert_eq!(definition.identity, target.identity);
}

#[test]
fn subcomponent_definition_resolves_across_document_set() {
    let target = build_component("part");
    let mut owner = build_component("owner");
    let sub = SubComponent::new(&owner.identity, "sub", target.identity.clone()).unwrap();
    owner.features.push(sub.identity.clone());

    let design = doc(vec![
        SbolObject::Component(owner),
        SbolObject::SubComponent(sub.clone()),
    ]);
    let parts = doc(vec![SbolObject::Component(target.clone())]);
    let scope = DocumentSet::from_documents([&design, &parts]).unwrap();

    // The part lives in the other document; single-doc resolution must fail,
    // multi-doc resolution must succeed.
    assert!(matches!(
        sub.definition(&design),
        Err(ReferenceError::NotFound(_))
    ));
    let definition = sub.definition(&scope).expect("cross-doc resolves");
    assert_eq!(definition.identity, target.identity);
}

#[test]
fn subcomponent_definition_reports_wrong_type() {
    // Point instanceOf at a Sequence instead of a Component.
    let bad_target = Sequence::new(NS, "wrong_target").unwrap();
    let mut owner = build_component("owner");
    let sub = SubComponent::new(&owner.identity, "sub", bad_target.identity.clone()).unwrap();
    owner.features.push(sub.identity.clone());

    let document = doc(vec![
        SbolObject::Component(owner),
        SbolObject::Sequence(bad_target),
        SbolObject::SubComponent(sub.clone()),
    ]);

    let error = sub.definition(&document).unwrap_err();
    match error {
        ReferenceError::WrongType {
            expected, found, ..
        } => {
            assert_eq!(expected, "Component");
            assert_eq!(found, "Sequence");
        }
        other => panic!("expected WrongType, got {other:?}"),
    }
}

// ---------- ComponentReference::target / trace ----------

#[test]
fn component_reference_target_walks_two_hops() {
    // outer Component has a SubComponent `host` whose instanceOf is inner Component;
    // inner Component has a SubComponent `inner_sub`.
    // outer has a ComponentReference `link` with in_child_of = host, refers_to =
    // a ComponentReference whose refers_to is inner_sub. Two-hop chain to inner_sub.
    let mut inner = build_component("inner");
    let inner_sub = SubComponent::new(
        &inner.identity,
        "inner_sub",
        build_component("inner_target").identity,
    )
    .unwrap();
    inner.features.push(inner_sub.identity.clone());

    let mut outer = build_component("outer");
    let host = SubComponent::new(&outer.identity, "host", inner.identity.clone()).unwrap();
    outer.features.push(host.identity.clone());

    let mid = ComponentReference::new(
        &outer.identity,
        "mid",
        host.identity.clone(),
        inner_sub.identity.clone(),
    )
    .unwrap();
    outer.features.push(mid.identity.clone());

    let link = ComponentReference::new(
        &outer.identity,
        "link",
        host.identity.clone(),
        mid.identity.clone(),
    )
    .unwrap();
    outer.features.push(link.identity.clone());

    let document = doc(vec![
        SbolObject::Component(outer),
        SbolObject::Component(inner),
        SbolObject::SubComponent(host),
        SbolObject::SubComponent(inner_sub.clone()),
        SbolObject::ComponentReference(mid.clone()),
        SbolObject::ComponentReference(link.clone()),
    ]);

    let trace = link.trace(&document).expect("trace resolves");
    assert_eq!(trace.target_iri(), &inner_sub.identity);
    assert_eq!(trace.path().len(), 1, "one intermediate hop expected");
    assert_eq!(&trace.path()[0], &mid.identity);
    match trace.target() {
        FeatureRef::SubComponent(found) => assert_eq!(found.identity, inner_sub.identity),
        other => panic!("expected SubComponent leaf, got {other:?}"),
    }

    // `target()` matches the trace's terminal value.
    match link.target(&document).unwrap() {
        FeatureRef::SubComponent(found) => assert_eq!(found.identity, inner_sub.identity),
        other => panic!("expected SubComponent leaf, got {other:?}"),
    }
}

#[test]
fn component_reference_trace_detects_cycle() {
    let inner = build_component("inner");
    let mut outer = build_component("outer");
    let host = SubComponent::new(&outer.identity, "host", inner.identity.clone()).unwrap();
    outer.features.push(host.identity.clone());

    // a refersTo b; b refersTo a — a cycle.
    let a_iri = Resource::iri(format!("{NS}/outer/a"));
    let b_iri = Resource::iri(format!("{NS}/outer/b"));
    let a = ComponentReference::builder(&outer.identity, "a")
        .unwrap()
        .in_child_of(host.identity.clone())
        .refers_to(b_iri.clone())
        .build()
        .unwrap();
    let b = ComponentReference::builder(&outer.identity, "b")
        .unwrap()
        .in_child_of(host.identity.clone())
        .refers_to(a_iri.clone())
        .build()
        .unwrap();
    outer.features.push(a.identity.clone());
    outer.features.push(b.identity.clone());

    let document = doc(vec![
        SbolObject::Component(outer),
        SbolObject::Component(inner),
        SbolObject::SubComponent(host),
        SbolObject::ComponentReference(a.clone()),
        SbolObject::ComponentReference(b.clone()),
    ]);

    let error = a.trace(&document).unwrap_err();
    assert!(matches!(error, ReferenceError::Cycle(_)));
}

#[test]
fn component_reference_trace_dangling_refers_to() {
    let inner = build_component("inner");
    let mut outer = build_component("outer");
    let host = SubComponent::new(&outer.identity, "host", inner.identity.clone()).unwrap();
    outer.features.push(host.identity.clone());

    let dangling_target = Resource::iri(format!("{NS}/outer/missing"));
    let link = ComponentReference::builder(&outer.identity, "link")
        .unwrap()
        .in_child_of(host.identity.clone())
        .refers_to(dangling_target.clone())
        .build()
        .unwrap();
    outer.features.push(link.identity.clone());

    let document = doc(vec![
        SbolObject::Component(outer),
        SbolObject::Component(inner),
        SbolObject::SubComponent(host),
        SbolObject::ComponentReference(link.clone()),
    ]);

    let error = link.trace(&document).unwrap_err();
    match error {
        ReferenceError::NotFound(iri) => assert_eq!(iri, dangling_target),
        other => panic!("expected NotFound, got {other:?}"),
    }
}

// ---------- CombinatorialDerivation::variants ----------

#[test]
fn variants_expand_explicit_collection_and_derivation_sources() {
    // Explicit variant
    let v_explicit = build_component("v_explicit");

    // Collection variants
    let v_coll_a = build_component("v_coll_a");
    let v_coll_b = build_component("v_coll_b");
    let mut collection = Collection::new(NS, "vc").unwrap();
    collection.members.push(v_coll_a.identity.clone());
    collection.members.push(v_coll_b.identity.clone());

    // Derivation variant (a Component prov:wasDerivedFrom an inner CD)
    let inner_cd_template = build_component("inner_template");
    let inner_cd =
        CombinatorialDerivation::new(NS, "inner_cd", inner_cd_template.identity.clone()).unwrap();
    let mut v_derived = build_component("v_derived");
    v_derived
        .identified
        .derived_from
        .push(inner_cd.identity.clone());

    // Outer CD with one VariableFeature wiring all three sources
    let template = build_component("template");
    let template_feature_iri = Resource::iri(format!("{NS}/template/tf"));
    let mut outer_cd =
        CombinatorialDerivation::new(NS, "outer_cd", template.identity.clone()).unwrap();

    let vf = VariableFeature::builder(&outer_cd.identity, "vf")
        .unwrap()
        .cardinality(CARDINALITY_ONE.clone())
        .variable(template_feature_iri.clone())
        .add_variant(v_explicit.identity.clone())
        .add_variant_collection(collection.identity.clone())
        .add_variant_derivation(inner_cd.identity.clone())
        .build()
        .unwrap();
    outer_cd.variable_features.push(vf.identity.clone());

    let document = doc(vec![
        SbolObject::Component(v_explicit.clone()),
        SbolObject::Component(v_coll_a.clone()),
        SbolObject::Component(v_coll_b.clone()),
        SbolObject::Component(inner_cd_template),
        SbolObject::Component(v_derived.clone()),
        SbolObject::Component(template),
        SbolObject::Collection(collection),
        SbolObject::CombinatorialDerivation(inner_cd.clone()),
        SbolObject::CombinatorialDerivation(outer_cd.clone()),
        SbolObject::VariableFeature(vf),
    ]);

    let variants = outer_cd.variants(&document).expect("variants resolve");
    assert_eq!(variants.len(), 4);
    assert!(!variants.is_empty());

    let explicit_ids: Vec<&Resource> = variants
        .from_variants()
        .iter()
        .map(|c| &c.identity)
        .collect();
    assert_eq!(explicit_ids, vec![&v_explicit.identity]);

    let coll_ids: Vec<&Resource> = variants
        .from_collections()
        .iter()
        .map(|c| &c.identity)
        .collect();
    assert_eq!(coll_ids, vec![&v_coll_a.identity, &v_coll_b.identity]);

    let deriv_ids: Vec<&Resource> = variants
        .from_derivations()
        .iter()
        .map(|c| &c.identity)
        .collect();
    assert_eq!(deriv_ids, vec![&v_derived.identity]);

    let total: Vec<&Resource> = variants.flatten().map(|c| &c.identity).collect();
    assert_eq!(total.len(), 4);
}

// ---------- ObjectGraph abstraction holds over both impls ----------

/// Generic resolver walk parameterized over any `ObjectGraph`. The body
/// stays identical whether the caller passes a single `Document` or a
/// composed `DocumentSet`.
fn collect_sub_definitions<'g, G: ObjectGraph + ?Sized>(
    subs: impl Iterator<Item = &'g SubComponent>,
    graph: &'g G,
) -> Vec<Resource> {
    subs.map(|s| s.definition(graph).unwrap().identity.clone())
        .collect()
}

#[test]
fn object_graph_abstracts_document_and_document_set() {
    let target = build_component("part");
    let mut owner = build_component("owner");
    let sub = SubComponent::new(&owner.identity, "sub", target.identity.clone()).unwrap();
    owner.features.push(sub.identity.clone());

    let document = doc(vec![
        SbolObject::Component(owner),
        SbolObject::Component(target.clone()),
        SbolObject::SubComponent(sub.clone()),
    ]);
    let single = DocumentSet::from_documents([&document]).unwrap();

    let from_doc = collect_sub_definitions(std::iter::once(&sub), &document);
    let from_set = collect_sub_definitions(std::iter::once(&sub), &single);
    assert_eq!(from_doc, from_set);
    assert_eq!(from_doc, vec![target.identity]);
}

// ---------- ObjectGraph::get / iter_typed escape hatches ----------

#[test]
fn object_graph_get_returns_property_bag() {
    let target = build_component("part");
    let document = doc(vec![SbolObject::Component(target.clone())]);
    let object = ObjectGraph::get(&document, &target.identity).expect("present");
    assert_eq!(object.identity(), &target.identity);
}

#[test]
fn object_graph_iter_typed_enumerates_everything() {
    let a = build_component("a");
    let b = build_component("b");
    let document = doc(vec![SbolObject::Component(a), SbolObject::Component(b)]);
    let count = ObjectGraph::iter_typed(&document).count();
    assert_eq!(count, 2);
}
