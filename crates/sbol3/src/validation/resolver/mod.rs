use std::collections::{BTreeMap, BTreeSet};

use crate::vocab::*;
use crate::{Document, Iri, Object, Resource, Term};

mod constraint;
mod derivation;
mod feature;
mod location;

pub(crate) use constraint::*;
pub(crate) use derivation::*;
pub(crate) use feature::*;
pub(crate) use location::*;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OwnershipEdge {
    pub parent: Resource,
    pub predicate: &'static str,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct OwnershipIndex {
    parents_by_child: BTreeMap<Resource, Vec<OwnershipEdge>>,
    children_by_parent: BTreeMap<(Resource, &'static str), Vec<Resource>>,
}

impl OwnershipIndex {
    pub(crate) fn new(document: &Document) -> Self {
        let mut index = Self::default();
        for object in document.objects().values() {
            for predicate in COMPOSITE_PREDICATES {
                for child in object.resources(predicate) {
                    index
                        .parents_by_child
                        .entry(child.clone())
                        .or_default()
                        .push(OwnershipEdge {
                            parent: object.identity().clone(),
                            predicate,
                        });
                    index
                        .children_by_parent
                        .entry((object.identity().clone(), predicate))
                        .or_default()
                        .push(child.clone());
                }
            }
        }
        index
    }

    pub(crate) fn parents(&self, child: &Resource, predicate: &str) -> Vec<&Resource> {
        self.parents_by_child
            .get(child)
            .into_iter()
            .flatten()
            .filter(|edge| edge.predicate == predicate)
            .map(|edge| &edge.parent)
            .collect()
    }

    pub(crate) fn single_parent(&self, child: &Resource, predicate: &str) -> Option<&Resource> {
        self.parents(child, predicate).into_iter().next()
    }

    pub(crate) fn children(&self, parent: &Resource, predicate: &str) -> Vec<&Resource> {
        self.children_by_parent
            .get(&(parent.clone(), owned_predicate(predicate)))
            .into_iter()
            .flatten()
            .collect()
    }

    pub(crate) fn contains(&self, parent: &Resource, predicate: &str, child: &Resource) -> bool {
        self.children(parent, predicate)
            .into_iter()
            .any(|candidate| candidate == child)
    }
}

pub(crate) fn directional_orientation_value(value: Option<&Iri>) -> Option<DirectionalOrientation> {
    match value.map(Iri::as_str)? {
        SBOL_INLINE | SO_INLINE => Some(DirectionalOrientation::Inline),
        SBOL_REVERSE_COMPLEMENT | SO_REVERSE_COMPLEMENT => {
            Some(DirectionalOrientation::ReverseComplement)
        }
        _ => None,
    }
}

pub(crate) fn parent_by_identity_prefix<'a>(
    document: &'a Document,
    object: &Object,
) -> Option<&'a Object> {
    let identity = object.identity().to_string();
    let parent_identity = identity.rsplit_once('/')?.0;
    document.get(&Resource::iri(parent_identity))
}

fn sequential_relation_satisfied(
    restriction: &str,
    subject: &LinearLocation,
    object: &LinearLocation,
) -> bool {
    match restriction {
        SBOL_PRECEDES => subject.start < object.start,
        SBOL_STRICTLY_PRECEDES => subject.end < object.start,
        SBOL_MEETS => subject.end == object.start,
        SBOL_OVERLAPS => {
            subject.start < object.start && subject.end > object.start && subject.end < object.end
        }
        SBOL_CONTAINS => subject.start <= object.start && subject.end >= object.end,
        SBOL_STRICTLY_CONTAINS => subject.start < object.start && subject.end > object.end,
        SBOL_EQUALS => subject.start == object.start && subject.end == object.end,
        SBOL_FINISHES => subject.start > object.start && subject.end == object.end,
        SBOL_STARTS => subject.start == object.start && subject.end < object.end,
        _ => true,
    }
}

fn integer_value(object: &Object, predicate: &str) -> Option<i64> {
    object
        .first_literal_value(predicate)
        .and_then(|value| value.parse::<i64>().ok())
}

fn iri_values(object: &Object, predicate: &str) -> BTreeSet<Iri> {
    object.iris(predicate).cloned().collect()
}

fn term_values(object: &Object, predicate: &str) -> BTreeSet<Term> {
    object.values(predicate).iter().cloned().collect()
}

fn owned_predicate(predicate: &str) -> &'static str {
    COMPOSITE_PREDICATES
        .iter()
        .copied()
        .find(|candidate| *candidate == predicate)
        .unwrap_or(SBOL_HAS_FEATURE)
}

const COMPOSITE_PREDICATES: &[&str] = &[
    SBOL_HAS_FEATURE,
    SBOL_HAS_CONSTRAINT,
    SBOL_HAS_INTERACTION,
    SBOL_HAS_INTERFACE,
    SBOL_HAS_LOCATION,
    SBOL_SOURCE_LOCATION,
    SBOL_HAS_PARTICIPATION,
    SBOL_HAS_VARIABLE_FEATURE,
    PROV_QUALIFIED_USAGE,
    PROV_QUALIFIED_ASSOCIATION,
];

const FEATURE_EQUIVALENCE_PREDICATES: &[&str] = &[
    RDF_TYPE,
    SBOL_ROLE,
    SBOL_ORIENTATION,
    SBOL_TYPE,
    SBOL_INSTANCE_OF,
    SBOL_DEFINITION,
    SBOL_ROLE_INTEGRATION,
];
