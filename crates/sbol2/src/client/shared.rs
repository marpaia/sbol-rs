use crate::object::is_extension_predicate;
use crate::vocab::*;
use crate::{Iri, Object, Resource, Term};

/// A non-SBOL annotation triple attached to a typed object.
///
/// Extension triples are predicates outside the SBOL 2, PROV, and OM
/// vocabularies (and outside the four recognized `dcterms`/`rdfs` IRIs) — for
/// example an external lab's `<lab:authoredBy>` predicate. They survive
/// round-tripping through the typed model so callers can read and write
/// extension data without dropping out of the typed API.
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

/// Shared owned fields for SBOL 2 Identified objects.
///
/// SBOL 2 keeps `persistentIdentity` and `version` on every Identified object,
/// and carries the human-readable name and description as `dcterms:title` and
/// `dcterms:description` rather than SBOL-vocabulary predicates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IdentifiedData {
    pub persistent_identity: Option<Resource>,
    pub version: Option<String>,
    pub display_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub derived_from: Vec<Resource>,
    pub generated_by: Vec<Resource>,
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
            persistent_identity: object.first_resource(SBOL2_PERSISTENT_IDENTITY).cloned(),
            version: object
                .first_literal_value(SBOL2_VERSION)
                .map(ToOwned::to_owned),
            display_id: object.identified().display_id.clone(),
            name: object.identified().name.clone(),
            description: object.identified().description.clone(),
            derived_from: resources(object, PROV_WAS_DERIVED_FROM),
            generated_by: resources(object, PROV_WAS_GENERATED_BY),
            extensions,
        }
    }
}

/// Shared owned fields for SBOL 2 TopLevel objects. SBOL 2 has no
/// `hasNamespace`, so only the attachment set lives here.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct TopLevelData {
    pub attachments: Vec<Resource>,
}

impl TopLevelData {
    pub(crate) fn from_object(object: &Object) -> Self {
        Self {
            attachments: resources(object, SBOL2_ATTACHMENT),
        }
    }
}

/// Shared owned fields for the abstract Measured mixin.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct MeasuredData {
    pub measures: Vec<Resource>,
}

impl MeasuredData {
    pub(crate) fn from_object(object: &Object) -> Self {
        Self {
            measures: resources(object, SBOL2_MEASURE),
        }
    }
}

/// Shared owned fields for the abstract ComponentInstance mixin.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ComponentInstanceData {
    pub definition: Option<Resource>,
    pub access: Option<Iri>,
    pub maps_tos: Vec<Resource>,
    pub measured: MeasuredData,
}

impl ComponentInstanceData {
    pub(crate) fn from_object(object: &Object) -> Self {
        Self {
            definition: object.first_resource(SBOL2_DEFINITION).cloned(),
            access: object.first_iri(SBOL2_ACCESS).cloned(),
            maps_tos: resources(object, SBOL2_MAPS_TO),
            measured: MeasuredData::from_object(object),
        }
    }
}

/// Shared owned fields for the abstract Location mixin.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct LocationData {
    pub orientation: Option<Iri>,
    pub sequence: Option<Resource>,
}

impl LocationData {
    pub(crate) fn from_object(object: &Object) -> Self {
        Self {
            orientation: object.first_iri(SBOL2_ORIENTATION).cloned(),
            sequence: object.first_resource(SBOL2_SEQUENCE).cloned(),
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
