//! Feature-reference resolution: walking `sbol:refersTo` chains
//! (`ComponentReferenceResolver`) and resolving the type-bearing
//! referent of a feature (`TypeReferentResolver`).

use crate::object::ObjectClasses;
use std::collections::BTreeSet;

use super::{OwnershipIndex, iri_values, parent_by_identity_prefix};
use crate::vocab::*;
use crate::{Document, Iri, Object, Resource, SbolClass};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FeatureResolveError {
    MissingObject(Resource),
    MissingRefersTo(Resource),
    Cycle(Vec<Resource>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FeatureTrace {
    pub requested: Resource,
    pub target: Resource,
    pub path: Vec<Resource>,
}

pub(crate) struct ComponentReferenceResolver<'a> {
    document: &'a Document,
    ownership: &'a OwnershipIndex,
}

impl<'a> ComponentReferenceResolver<'a> {
    pub(crate) fn new(document: &'a Document, ownership: &'a OwnershipIndex) -> Self {
        Self {
            document,
            ownership,
        }
    }

    pub(crate) fn trace_feature(
        &self,
        feature: &Resource,
    ) -> Result<FeatureTrace, FeatureResolveError> {
        let mut current = feature.clone();
        let mut path = Vec::new();
        let mut visited = BTreeSet::new();

        loop {
            if !visited.insert(current.clone()) {
                path.push(current);
                return Err(FeatureResolveError::Cycle(path));
            }

            let Some(object) = self.document.get(&current) else {
                return Err(FeatureResolveError::MissingObject(current));
            };
            if !object.has_class(SbolClass::ComponentReference) {
                return Ok(FeatureTrace {
                    requested: feature.clone(),
                    target: current,
                    path,
                });
            }

            path.push(current.clone());
            let Some(next) = object.first_resource(SBOL_REFERS_TO) else {
                return Err(FeatureResolveError::MissingRefersTo(current));
            };
            current = next.clone();
        }
    }

    pub(crate) fn parent_reference(&self, reference: &Object) -> Option<&'a Object> {
        parent_by_identity_prefix(self.document, reference)
            .filter(|parent| parent.has_class(SbolClass::ComponentReference))
    }

    pub(crate) fn reference_is_child_of(
        &self,
        reference: &Resource,
        parent_reference: &Resource,
    ) -> bool {
        self.document.get(reference).is_some_and(|reference| {
            self.parent_reference(reference)
                .is_some_and(|parent| parent.identity() == parent_reference)
        })
    }

    pub(crate) fn direct_parent_component(&self, feature: &Resource) -> Option<&Resource> {
        self.ownership.single_parent(feature, SBOL_HAS_FEATURE)
    }
}

pub(crate) struct TypeReferentResolver<'a> {
    document: &'a Document,
    references: ComponentReferenceResolver<'a>,
}

impl<'a> TypeReferentResolver<'a> {
    pub(crate) fn new(document: &'a Document, ownership: &'a OwnershipIndex) -> Self {
        Self {
            document,
            references: ComponentReferenceResolver::new(document, ownership),
        }
    }

    pub(crate) fn type_properties(&self, feature: &Resource) -> Option<BTreeSet<Iri>> {
        let referent = self.type_referent(feature)?;
        Some(iri_values(self.document.get(&referent)?, SBOL_TYPE))
    }

    fn type_referent(&self, feature: &Resource) -> Option<Resource> {
        let trace = self.references.trace_feature(feature).ok()?;
        let object = self.document.get(&trace.target)?;
        if object.has_class(SbolClass::LocalSubComponent)
            || object.has_class(SbolClass::ExternallyDefined)
        {
            return Some(trace.target);
        }
        if object.has_class(SbolClass::SubComponent) {
            return object.first_resource(SBOL_INSTANCE_OF).cloned();
        }
        None
    }
}
