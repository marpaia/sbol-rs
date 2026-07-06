use crate::object::is_extension_predicate;
use crate::vocab::*;
use crate::{Iri, Object, Resource, Term};

/// A non-SBOL annotation triple attached to a typed object.
///
/// Extension triples are predicates outside the SBOL, PROV, and OM
/// vocabularies, for example an external lab's `<lab:authoredBy>`
/// predicate. They survive round-tripping through the typed model so callers
/// can read and write extension data without dropping out of the typed API.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExtensionTriple {
    pub predicate: Iri,
    pub object: Term,
}

impl ExtensionTriple {
    pub fn new(predicate: Iri, object: Term) -> Self {
        Self { predicate, object }
    }
}

/// Shared owned fields for SBOL Identified objects.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IdentifiedData {
    pub display_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub derived_from: Vec<Resource>,
    pub generated_by: Vec<Resource>,
    pub measures: Vec<Resource>,
    /// Non-SBOL annotation triples attached to this object's identity.
    pub extensions: Vec<ExtensionTriple>,
}

impl IdentifiedData {
    pub(crate) fn from_object(object: &Object) -> Self {
        let extensions = object
            .properties()
            .iter()
            .filter(|(predicate, _)| is_extension_predicate(predicate.as_str()))
            .flat_map(|(predicate, values)| {
                values.iter().map(move |value| ExtensionTriple {
                    predicate: predicate.clone(),
                    object: value.clone(),
                })
            })
            .collect();

        Self {
            display_id: object.identified().display_id.clone(),
            name: object.identified().name.clone(),
            description: object.identified().description.clone(),
            derived_from: resources(object, PROV_WAS_DERIVED_FROM),
            generated_by: resources(object, PROV_WAS_GENERATED_BY),
            measures: resources(object, SBOL_HAS_MEASURE),
            extensions,
        }
    }
}

/// Shared owned fields for SBOL TopLevel objects.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct TopLevelData {
    pub namespace: Option<Iri>,
    pub attachments: Vec<Resource>,
}

impl TopLevelData {
    pub(crate) fn from_object(object: &Object) -> Self {
        Self {
            namespace: object.first_iri(SBOL_HAS_NAMESPACE).cloned(),
            attachments: resources(object, SBOL_HAS_ATTACHMENT),
        }
    }
}

/// Shared owned fields for SBOL Feature objects.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct FeatureData {
    pub roles: Vec<Iri>,
    pub orientation: Option<Iri>,
}

impl FeatureData {
    pub(crate) fn from_object(object: &Object) -> Self {
        Self {
            roles: iris(object, SBOL_ROLE),
            orientation: object.first_iri(SBOL_ORIENTATION).cloned(),
        }
    }
}

/// Shared owned fields for SBOL Location objects.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct LocationData {
    pub sequence: Option<Resource>,
    pub orientation: Option<Iri>,
    pub order: Option<i64>,
}

impl LocationData {
    pub(crate) fn from_object(object: &Object) -> Self {
        Self {
            sequence: object.first_resource(SBOL_HAS_SEQUENCE).cloned(),
            orientation: object.first_iri(SBOL_ORIENTATION).cloned(),
            order: first_i64(object, SBOL_ORDER),
        }
    }
}

pub(crate) fn resources(object: &Object, predicate: &str) -> Vec<Resource> {
    object.resources(predicate).cloned().collect()
}

pub(crate) fn iris(object: &Object, predicate: &str) -> Vec<Iri> {
    object.iris(predicate).cloned().collect()
}

pub(crate) fn first_i64(object: &Object, predicate: &str) -> Option<i64> {
    object.first_literal_value(predicate)?.parse().ok()
}

pub(crate) fn literals(object: &Object, predicate: &str) -> Vec<String> {
    object
        .literals(predicate)
        .map(|literal| literal.value().to_owned())
        .collect()
}
