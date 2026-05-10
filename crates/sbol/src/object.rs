use std::collections::{BTreeMap, BTreeSet};

use crate::model::{Identified, TopLevel};
use crate::vocab::*;
use crate::{Iri, Literal, RdfGraph, Resource, SbolClass, Term};

/// An SBOL or SBOL extension object.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Object {
    identity: Resource,
    rdf_types: BTreeSet<Iri>,
    classes: BTreeSet<SbolClass>,
    properties: BTreeMap<Iri, Vec<Term>>,
    identified: Identified,
    top_level: Option<TopLevel>,
}

impl Object {
    pub fn identity(&self) -> &Resource {
        &self.identity
    }

    pub fn rdf_types(&self) -> &BTreeSet<Iri> {
        &self.rdf_types
    }

    pub fn classes(&self) -> &BTreeSet<SbolClass> {
        &self.classes
    }

    pub fn properties(&self) -> &BTreeMap<Iri, Vec<Term>> {
        &self.properties
    }

    pub fn identified(&self) -> &Identified {
        &self.identified
    }

    pub fn top_level(&self) -> Option<&TopLevel> {
        self.top_level.as_ref()
    }

    pub fn is_top_level(&self) -> bool {
        self.top_level.is_some()
    }

    pub fn has_class(&self, class: SbolClass) -> bool {
        self.classes.iter().any(|candidate| candidate.is_a(class))
    }

    pub fn values(&self, predicate: &str) -> &[Term] {
        terms_for(&self.properties, predicate)
    }

    pub fn resources(&self, predicate: &str) -> impl Iterator<Item = &Resource> {
        self.values(predicate).iter().filter_map(Term::as_resource)
    }

    pub fn iris(&self, predicate: &str) -> impl Iterator<Item = &Iri> {
        self.values(predicate).iter().filter_map(Term::as_iri)
    }

    pub fn literals(&self, predicate: &str) -> impl Iterator<Item = &Literal> {
        self.values(predicate).iter().filter_map(Term::as_literal)
    }

    pub fn first_resource(&self, predicate: &str) -> Option<&Resource> {
        self.resources(predicate).next()
    }

    pub fn first_iri(&self, predicate: &str) -> Option<&Iri> {
        self.iris(predicate).next()
    }

    pub fn first_literal_value(&self, predicate: &str) -> Option<&str> {
        self.literals(predicate).next().map(Literal::value)
    }
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
                .filter_map(SbolClass::from_iri)
                .collect::<BTreeSet<_>>();
            let identified = Identified {
                display_id: first_literal(&properties, SBOL_DISPLAY_ID),
                name: first_literal(&properties, SBOL_NAME),
                description: first_literal(&properties, SBOL_DESCRIPTION),
                derived_from: resources_for(&properties, PROV_WAS_DERIVED_FROM),
                generated_by: resources_for(&properties, PROV_WAS_GENERATED_BY),
                measures: resources_for(&properties, SBOL_HAS_MEASURE),
                attachments: resources_for(&properties, SBOL_HAS_ATTACHMENT),
            };
            let top_level = classes
                .iter()
                .any(|class| class.is_top_level())
                .then(|| TopLevel {
                    namespace: first_iri(&properties, SBOL_HAS_NAMESPACE),
                });

            Some((
                identity.clone(),
                Object {
                    identity,
                    rdf_types,
                    classes,
                    properties,
                    identified,
                    top_level,
                },
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
                .is_some_and(|iri| SbolClass::from_iri(iri).is_some())
        })
}

/// Returns `true` if the given predicate IRI is outside the SBOL, PROV, and
/// OM vocabularies and is not `rdf:type` — i.e. it should be preserved as an
/// extension/annotation triple rather than parsed into a typed field.
pub(crate) fn is_extension_predicate(predicate: &str) -> bool {
    predicate != RDF_TYPE
        && !predicate.starts_with(SBOL_NS)
        && !predicate.starts_with(PROV_NS)
        && !predicate.starts_with(OM_NS)
}

fn first_literal(properties: &BTreeMap<Iri, Vec<Term>>, predicate: &str) -> Option<String> {
    terms_for(properties, predicate)
        .iter()
        .find_map(|term| term.as_literal().map(|literal| literal.value().to_owned()))
}

fn first_iri(properties: &BTreeMap<Iri, Vec<Term>>, predicate: &str) -> Option<Iri> {
    terms_for(properties, predicate)
        .iter()
        .find_map(|term| term.as_iri().cloned())
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
