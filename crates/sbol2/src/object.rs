use std::collections::{BTreeMap, BTreeSet};

use sbol_core::object::Object;

use crate::model::{Identified, TopLevel};
use crate::schema::literal_datatype;
use crate::vocab::*;
use crate::{Iri, Literal, RdfGraph, Resource, Sbol2Class, Term};

/// Classifies an [`Object`] against the SBOL 2 class hierarchy.
///
/// The neutral [`Object`] retains only its raw `rdf_types`; this extension
/// trait interprets those IRIs as SBOL 2 classes, honoring subclass
/// relationships through [`Sbol2Class::is_a`].
pub trait ObjectClasses {
    /// The SBOL 2 classes this object's RDF types resolve to.
    fn classes(&self) -> BTreeSet<Sbol2Class>;

    /// Whether the object is an instance of `class` or any of its subclasses.
    fn has_class(&self, class: Sbol2Class) -> bool;
}

impl ObjectClasses for Object {
    fn classes(&self) -> BTreeSet<Sbol2Class> {
        self.rdf_types()
            .iter()
            .filter_map(Sbol2Class::from_iri)
            .collect()
    }

    fn has_class(&self, class: Sbol2Class) -> bool {
        self.rdf_types()
            .iter()
            .filter_map(Sbol2Class::from_iri)
            .any(|candidate| candidate.is_a(class))
    }
}

/// Rewrites the datatype of every recognized SBOL 2 literal to the datatype
/// the data model assigns it, leaving extension-triple literals untouched.
/// This lets the parsed graph and the graph rebuilt from typed objects agree
/// regardless of how the source serialization typed its literals.
pub(crate) fn canonicalize_literals(graph: &RdfGraph) -> RdfGraph {
    let triples = graph
        .triples()
        .iter()
        .map(|triple| {
            let Term::Literal(literal) = &triple.object else {
                return triple.clone();
            };
            let Some(datatype) = literal_datatype(triple.predicate.as_str()) else {
                return triple.clone();
            };
            if literal.datatype().as_str() == datatype && literal.language().is_none() {
                return triple.clone();
            }
            let mut triple = triple.clone();
            triple.object = Term::Literal(Literal::new(
                literal.value(),
                Iri::from_static(datatype),
                None,
            ));
            triple
        })
        .collect();
    RdfGraph::new(triples)
}

pub(crate) fn collect_objects(graph: &RdfGraph) -> BTreeMap<Resource, Object> {
    let mut properties_by_subject: BTreeMap<Resource, BTreeMap<Iri, Vec<Term>>> = BTreeMap::new();

    for triple in graph.triples() {
        properties_by_subject
            .entry(triple.subject.clone())
            .or_default()
            .entry(triple.predicate.clone())
            .or_default()
            .push(triple.object.clone());
    }

    properties_by_subject
        .into_iter()
        .filter_map(|(identity, properties)| {
            if !is_sbol_relevant(&properties) {
                return None;
            }

            let rdf_types = resources_for(&properties, RDF_TYPE)
                .into_iter()
                .filter_map(|resource| resource.as_iri().cloned())
                .collect::<BTreeSet<_>>();
            let classes = rdf_types
                .iter()
                .filter_map(Sbol2Class::from_iri)
                .collect::<BTreeSet<_>>();
            let identified = Identified {
                display_id: first_literal(&properties, SBOL2_DISPLAY_ID),
                name: first_literal(&properties, DCTERMS_TITLE),
                description: first_literal(&properties, DCTERMS_DESCRIPTION),
                derived_from: resources_for(&properties, PROV_WAS_DERIVED_FROM),
                generated_by: resources_for(&properties, PROV_WAS_GENERATED_BY),
                measures: resources_for(&properties, SBOL2_MEASURE),
                attachments: resources_for(&properties, SBOL2_ATTACHMENT),
            };
            let top_level = classes
                .iter()
                .any(|class| class.is_top_level())
                .then_some(TopLevel { namespace: None });

            Some((
                identity.clone(),
                Object::from_parts(identity, rdf_types, properties, identified, top_level),
            ))
        })
        .collect()
}

fn is_sbol_relevant(properties: &BTreeMap<Iri, Vec<Term>>) -> bool {
    properties
        .keys()
        .any(|predicate| !is_extension_predicate(predicate.as_str()))
        || resources_for(properties, RDF_TYPE).iter().any(|resource| {
            resource
                .as_iri()
                .is_some_and(|iri| Sbol2Class::from_iri(iri).is_some())
        })
}

/// Returns `true` if the given predicate IRI must be preserved as an
/// extension/annotation triple rather than parsed into a typed field.
///
/// The SBOL 2, PROV, and OM vocabularies are recognized wholesale. The four
/// specific Dublin Core Terms and RDF Schema IRIs the model maps to typed
/// fields (`dcterms:title`, `dcterms:description`, `rdfs:label`,
/// `rdfs:comment`) are excluded so they are not re-emitted twice, while any
/// other `dcterms:*` or `rdfs:*` predicate still round-trips as an extension.
pub(crate) fn is_extension_predicate(predicate: &str) -> bool {
    predicate != RDF_TYPE
        && predicate != DCTERMS_TITLE
        && predicate != DCTERMS_DESCRIPTION
        && predicate != RDFS_LABEL
        && predicate != RDFS_COMMENT
        && !predicate.starts_with(SBOL2_NS)
        && !predicate.starts_with(PROV_NS)
        && !predicate.starts_with(OM_NS)
}

fn first_literal(properties: &BTreeMap<Iri, Vec<Term>>, predicate: &str) -> Option<String> {
    terms_for(properties, predicate)
        .iter()
        .find_map(|term| term.as_literal().map(|literal| literal.value().to_owned()))
}

fn resources_for(properties: &BTreeMap<Iri, Vec<Term>>, predicate: &str) -> Vec<Resource> {
    terms_for(properties, predicate)
        .iter()
        .filter_map(|term| term.as_resource().cloned())
        .collect()
}

fn terms_for<'a>(properties: &'a BTreeMap<Iri, Vec<Term>>, predicate: &str) -> &'a [Term] {
    properties
        .get(&Iri::new_unchecked(predicate))
        .map_or(&[], Vec::as_slice)
}
