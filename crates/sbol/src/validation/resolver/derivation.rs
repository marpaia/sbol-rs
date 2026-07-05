//! Combinatorial-derivation resolution: template/variable feature context,
//! derived components, and allowed variant expansion.

use crate::object::ObjectClasses;
use std::collections::{BTreeMap, BTreeSet};

use super::{FEATURE_EQUIVALENCE_PREDICATES, OwnershipIndex, iri_values, term_values};
use crate::vocab::*;
use crate::{Document, Iri, Object, Resource, SbolClass};

pub(crate) struct DerivationResolver<'a> {
    document: &'a Document,
}

impl<'a> DerivationResolver<'a> {
    pub(crate) fn new(document: &'a Document, _ownership: &'a OwnershipIndex) -> Self {
        Self { document }
    }

    pub(crate) fn context(&self, derivation: &Object) -> Option<DerivationContext> {
        let template = derivation.first_resource(SBOL_TEMPLATE)?.clone();
        let template_object = self.document.get(&template)?;
        let template_features = template_object
            .resources(SBOL_HAS_FEATURE)
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut variables = BTreeMap::new();
        let mut variable_features = Vec::new();

        for variable_feature in derivation.resources(SBOL_HAS_VARIABLE_FEATURE) {
            let Some(variable_feature_object) = self.document.get(variable_feature) else {
                continue;
            };
            let Some(variable) = variable_feature_object.first_resource(SBOL_VARIABLE) else {
                continue;
            };
            variables.insert(variable.clone(), variable_feature.clone());
            variable_features.push(variable_feature.clone());
        }

        let static_features = template_features
            .iter()
            .filter(|feature| !variables.contains_key(*feature))
            .cloned()
            .collect();

        Some(DerivationContext {
            derivation: derivation.identity().clone(),
            template,
            template_features,
            variables,
            variable_features,
            static_features,
        })
    }

    pub(crate) fn derived_components(&self, derivation: &Resource) -> Vec<&'a Object> {
        self.document
            .objects()
            .values()
            .filter(|object| {
                object.has_class(SbolClass::Component)
                    && object
                        .identified()
                        .derived_from
                        .iter()
                        .any(|item| item == derivation)
            })
            .collect()
    }

    pub(crate) fn derived_collections(&self, derivation: &Resource) -> Vec<&'a Object> {
        self.document
            .objects()
            .values()
            .filter(|object| {
                object.has_class(SbolClass::Collection)
                    && object
                        .identified()
                        .derived_from
                        .iter()
                        .any(|item| item == derivation)
            })
            .collect()
    }

    pub(crate) fn features_derived_from(
        &self,
        derived_component: &'a Object,
        template_feature: &Resource,
    ) -> Vec<&'a Resource> {
        derived_component
            .resources(SBOL_HAS_FEATURE)
            .filter(|feature| {
                self.document.get(feature).is_some_and(|object| {
                    object
                        .identified()
                        .derived_from
                        .iter()
                        .any(|candidate| candidate == template_feature)
                })
            })
            .collect()
    }

    pub(crate) fn feature_derives_from_template(
        &self,
        feature: &Resource,
        template_features: &BTreeSet<Resource>,
    ) -> bool {
        self.document.get(feature).is_some_and(|object| {
            object
                .identified()
                .derived_from
                .iter()
                .any(|candidate| template_features.contains(candidate))
        })
    }

    pub(crate) fn static_feature_properties_match(
        &self,
        derived_feature: &Resource,
        template_feature: &Resource,
    ) -> Option<bool> {
        let derived = self.document.get(derived_feature)?;
        let template = self.document.get(template_feature)?;
        Some(
            FEATURE_EQUIVALENCE_PREDICATES.iter().all(|predicate| {
                term_values(derived, predicate) == term_values(template, predicate)
            }),
        )
    }

    pub(crate) fn variable_feature_for(
        &self,
        context: &DerivationContext,
        template_feature: &Resource,
    ) -> Option<&'a Object> {
        let variable_feature = context.variables.get(template_feature)?;
        self.document.get(variable_feature)
    }

    pub(crate) fn cardinality_allows(&self, cardinality: Option<&Iri>, count: usize) -> bool {
        match cardinality.map(Iri::as_str) {
            Some(SBOL_ONE) => count == 1,
            Some(SBOL_ZERO_OR_ONE) => count <= 1,
            Some(SBOL_ONE_OR_MORE) => count >= 1,
            Some(SBOL_ZERO_OR_MORE) => true,
            _ => true,
        }
    }

    pub(crate) fn allowed_variants(&self, variable_feature: &Object) -> BTreeSet<Resource> {
        let mut allowed = variable_feature
            .resources(SBOL_VARIANT)
            .filter(|variant| {
                self.document
                    .get(variant)
                    .is_some_and(|object| object.has_class(SbolClass::Component))
            })
            .cloned()
            .collect::<BTreeSet<_>>();

        for collection in variable_feature.resources(SBOL_VARIANT_COLLECTION) {
            let mut visited = BTreeSet::new();
            self.collect_collection_components(collection, &mut visited, &mut allowed);
        }

        let derivations = variable_feature
            .resources(SBOL_VARIANT_DERIVATION)
            .cloned()
            .collect::<BTreeSet<_>>();
        if !derivations.is_empty() {
            for component in self.document.components() {
                if component
                    .identified
                    .derived_from
                    .iter()
                    .any(|candidate| derivations.contains(candidate))
                {
                    allowed.insert(component.identity.clone());
                }
            }
        }

        allowed
    }

    pub(crate) fn template_constraints(&self, context: &DerivationContext) -> Vec<&'a Object> {
        let Some(template) = self.document.get(&context.template) else {
            return Vec::new();
        };
        template
            .resources(SBOL_HAS_CONSTRAINT)
            .filter_map(|constraint| self.document.get(constraint))
            .collect()
    }

    pub(crate) fn feature_roles(&self, feature: &Resource) -> Option<BTreeSet<Iri>> {
        Some(iri_values(self.document.get(feature)?, SBOL_ROLE))
    }

    pub(crate) fn component_types(&self, component: &Object) -> BTreeSet<Iri> {
        iri_values(component, SBOL_TYPE)
    }

    pub(crate) fn component_roles(&self, component: &Object) -> BTreeSet<Iri> {
        iri_values(component, SBOL_ROLE)
    }

    fn collect_collection_components(
        &self,
        collection: &Resource,
        visited: &mut BTreeSet<Resource>,
        components: &mut BTreeSet<Resource>,
    ) {
        if !visited.insert(collection.clone()) {
            return;
        }
        let Some(collection_object) = self.document.get(collection) else {
            return;
        };
        if !collection_object.has_class(SbolClass::Collection) {
            return;
        }
        for member in collection_object.resources(SBOL_MEMBER) {
            let Some(member_object) = self.document.get(member) else {
                continue;
            };
            if member_object.has_class(SbolClass::Component) {
                components.insert(member.clone());
            } else if member_object.has_class(SbolClass::Collection) {
                self.collect_collection_components(member, visited, components);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DerivationContext {
    pub derivation: Resource,
    pub template: Resource,
    pub template_features: BTreeSet<Resource>,
    pub variables: BTreeMap<Resource, Resource>,
    pub variable_features: Vec<Resource>,
    pub static_features: BTreeSet<Resource>,
}
