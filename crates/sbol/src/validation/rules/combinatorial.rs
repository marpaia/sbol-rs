use std::collections::BTreeSet;

use crate::validation::helpers::{
    collection_contains_only_components_or_collections, component_contains, error_issue,
    warning_issue,
};
use crate::validation::report::ValidationIssue;
use crate::validation::resolver::{
    ComponentReferenceResolver, ConstraintEngine, DerivationResolver, FeatureResolveError,
    RelationOutcome, TypeReferentResolver,
};
use crate::validation::validator::Validator;
use crate::vocab::*;
use crate::{Object, SbolClass};

impl<'a> Validator<'a> {
    pub(crate) fn validate_combinatorial_derivation(&mut self, object: &Object) {
        if let Some(strategy) = object.first_iri(SBOL_STRATEGY)
            && !COMBINATORIAL_DERIVATION_STRATEGY_IRIS.contains(&strategy.as_str())
        {
            self.error(
                "sbol3-12101",
                object,
                Some(SBOL_STRATEGY),
                format!("unsupported CombinatorialDerivation strategy `{strategy}`"),
            );
        }
        if object
            .first_iri(SBOL_STRATEGY)
            .is_some_and(|iri| iri.as_str() == SBOL_ENUMERATE)
        {
            for variable_feature in object.resources(SBOL_HAS_VARIABLE_FEATURE) {
                let Some(variable_feature) = self.document.get(variable_feature) else {
                    continue;
                };
                if variable_feature
                    .first_iri(SBOL_CARDINALITY)
                    .is_some_and(|iri| matches!(iri.as_str(), SBOL_ZERO_OR_MORE | SBOL_ONE_OR_MORE))
                {
                    self.error(
                        "sbol3-12102",
                        variable_feature,
                        Some(SBOL_CARDINALITY),
                        "enumerate strategy cannot use zeroOrMore or oneOrMore cardinality",
                    );
                }
            }
        }
        let mut variables = BTreeSet::new();
        for variable_feature in object.resources(SBOL_HAS_VARIABLE_FEATURE) {
            let Some(variable_feature) = self.document.get(variable_feature) else {
                continue;
            };
            let Some(variable) = variable_feature.first_resource(SBOL_VARIABLE) else {
                continue;
            };
            if !variables.insert(variable.clone()) {
                self.error(
                    "sbol3-12103",
                    object,
                    Some(SBOL_HAS_VARIABLE_FEATURE),
                    format!("duplicate VariableFeature variable `{variable}`"),
                );
            }
        }
        let semantic_issues = self.combinatorial_derivation_semantic_issues(object);
        self.extend_with_overrides(semantic_issues);
    }

    pub(crate) fn combinatorial_derivation_semantic_issues(
        &self,
        object: &Object,
    ) -> Vec<ValidationIssue> {
        let resolver = DerivationResolver::new(self.document, &self.ownership);
        let type_referents = TypeReferentResolver::new(self.document, &self.ownership);
        let constraint_engine = ConstraintEngine::new(self.document, &self.ownership);
        let Some(context) = resolver.context(object) else {
            return Vec::new();
        };
        let Some(template_object) = self.document.get(&context.template) else {
            return Vec::new();
        };
        let mut issues = Vec::new();

        if context.template_features.is_empty() {
            issues.push(warning_issue(
                "sbol3-12104",
                object.identity(),
                Some(SBOL_TEMPLATE),
                "CombinatorialDerivation template Component should contain at least one Feature",
            ));
        }

        let template_types = resolver.component_types(template_object);
        let template_roles = resolver.component_roles(template_object);

        for collection in resolver.derived_collections(&context.derivation) {
            for member in collection.resources(SBOL_MEMBER) {
                let Some(member_object) = self.document.get(member) else {
                    continue;
                };
                if !member_object
                    .identified()
                    .derived_from
                    .iter()
                    .any(|candidate| candidate == &context.derivation)
                {
                    issues.push(warning_issue(
                        "sbol3-12106",
                        collection.identity(),
                        Some(SBOL_MEMBER),
                        format!(
                            "Collection member `{member}` should derive from CombinatorialDerivation `{}`",
                            context.derivation
                        ),
                    ));
                }
            }
        }

        for derived_component in resolver.derived_components(&context.derivation) {
            let derived_features = derived_component
                .resources(SBOL_HAS_FEATURE)
                .cloned()
                .collect::<Vec<_>>();

            for feature in &derived_features {
                if !resolver.feature_derives_from_template(feature, &context.template_features) {
                    issues.push(warning_issue(
                        "sbol3-12105",
                        feature,
                        Some(PROV_WAS_DERIVED_FROM),
                        format!(
                            "derived Feature should derive from a Feature in template Component `{}`",
                            context.template
                        ),
                    ));
                }
            }

            let derived_types = resolver.component_types(derived_component);
            if !template_types.is_subset(&derived_types) {
                issues.push(warning_issue(
                    "sbol3-12107",
                    derived_component.identity(),
                    Some(SBOL_TYPE),
                    format!(
                        "derived Component should include all type values from template Component `{}`",
                        context.template
                    ),
                ));
            }

            let derived_roles = resolver.component_roles(derived_component);
            if !template_roles.is_subset(&derived_roles) {
                issues.push(warning_issue(
                    "sbol3-12108",
                    derived_component.identity(),
                    Some(SBOL_ROLE),
                    format!(
                        "derived Component should include all role values from template Component `{}`",
                        context.template
                    ),
                ));
            }

            for static_feature in &context.static_features {
                let matching_features =
                    resolver.features_derived_from(derived_component, static_feature);
                if matching_features.len() != 1 {
                    issues.push(warning_issue(
                        "sbol3-12110",
                        derived_component.identity(),
                        Some(SBOL_HAS_FEATURE),
                        format!(
                            "static template Feature `{static_feature}` should have exactly one derived Feature"
                        ),
                    ));
                }

                for derived_feature in matching_features {
                    if resolver
                        .static_feature_properties_match(derived_feature, static_feature)
                        .is_some_and(|matches| !matches)
                    {
                        issues.push(error_issue(
                            "sbol3-12109",
                            derived_feature,
                            Some(PROV_WAS_DERIVED_FROM),
                            format!(
                                "Feature derived from static template Feature `{static_feature}` must preserve its local Feature properties"
                            ),
                        ));
                    }
                }
            }

            for template_feature in context.variables.keys() {
                let matching_features =
                    resolver.features_derived_from(derived_component, template_feature);
                let Some(variable_feature) =
                    resolver.variable_feature_for(&context, template_feature)
                else {
                    continue;
                };

                if !resolver.cardinality_allows(
                    variable_feature.first_iri(SBOL_CARDINALITY),
                    matching_features.len(),
                ) {
                    issues.push(warning_issue(
                        "sbol3-12111",
                        derived_component.identity(),
                        Some(SBOL_HAS_FEATURE),
                        format!(
                            "derived Feature count for variable template Feature `{template_feature}` should satisfy its VariableFeature cardinality"
                        ),
                    ));
                }

                let allowed_variants = resolver.allowed_variants(variable_feature);
                let template_feature_roles =
                    resolver.feature_roles(template_feature).unwrap_or_default();
                let template_referent_types = type_referents.type_properties(template_feature);

                for derived_feature in matching_features {
                    if let Some(derived_feature_object) = self.document.get(derived_feature)
                        && derived_feature_object.has_class(SbolClass::SubComponent)
                        && let Some(instance_of) =
                            derived_feature_object.first_resource(SBOL_INSTANCE_OF)
                        && !allowed_variants.contains(instance_of)
                    {
                        issues.push(error_issue(
                            "sbol3-12112",
                            derived_feature,
                            Some(SBOL_INSTANCE_OF),
                            format!(
                                "derived SubComponent instanceOf `{instance_of}` must be a variant allowed by VariableFeature `{}`",
                                variable_feature.identity()
                            ),
                        ));
                    }

                    let derived_feature_roles =
                        resolver.feature_roles(derived_feature).unwrap_or_default();
                    if !template_feature_roles.is_subset(&derived_feature_roles) {
                        issues.push(warning_issue(
                            "sbol3-12114",
                            derived_feature,
                            Some(SBOL_ROLE),
                            format!(
                                "derived Feature should include all role values from variable template Feature `{template_feature}`"
                            ),
                        ));
                    }

                    if let (Some(template_types), Some(derived_types)) = (
                        template_referent_types.as_ref(),
                        type_referents.type_properties(derived_feature),
                    ) && !template_types.is_subset(&derived_types)
                    {
                        issues.push(warning_issue(
                            "sbol3-12115",
                            derived_feature,
                            Some(SBOL_TYPE),
                            format!(
                                "derived Feature type-determining referent should include all type values from template Feature `{template_feature}`"
                            ),
                        ));
                    }
                }
            }

            for template_constraint in resolver.template_constraints(&context) {
                let Some(restriction) = template_constraint.first_iri(SBOL_RESTRICTION) else {
                    continue;
                };
                let (Some(template_subject), Some(template_object)) = (
                    template_constraint.first_resource(SBOL_SUBJECT),
                    template_constraint.first_resource(SBOL_OBJECT),
                ) else {
                    continue;
                };
                let derived_subjects =
                    resolver.features_derived_from(derived_component, template_subject);
                let derived_objects =
                    resolver.features_derived_from(derived_component, template_object);

                for derived_subject in &derived_subjects {
                    for derived_object in &derived_objects {
                        let table8 = constraint_engine.table8_relation(
                            restriction.as_str(),
                            derived_subject,
                            derived_object,
                        );
                        let table10 = constraint_engine.table10_relation(
                            restriction.as_str(),
                            derived_subject,
                            derived_object,
                        );
                        if matches!(table8, RelationOutcome::Contradicted { .. })
                            || matches!(table10, RelationOutcome::Contradicted { .. })
                        {
                            issues.push(error_issue(
                                "sbol3-12113",
                                derived_component.identity(),
                                Some(SBOL_HAS_CONSTRAINT),
                                format!(
                                    "derived Features `{derived_subject}` and `{derived_object}` must satisfy template Constraint `{}`",
                                    template_constraint.identity()
                                ),
                            ));
                        }
                    }
                }
            }
        }

        issues
    }

    pub(crate) fn validate_component_reference(&mut self, object: &Object) {
        let references = ComponentReferenceResolver::new(self.document, &self.ownership);
        let parent_component = references
            .direct_parent_component(object.identity())
            .cloned();
        let parent_reference_in_child_of = references
            .parent_reference(object)
            .and_then(|parent| parent.first_resource(SBOL_IN_CHILD_OF))
            .cloned();

        if let Some(component) = &parent_component
            && let Some(in_child_of) = object.first_resource(SBOL_IN_CHILD_OF)
            && !self
                .ownership
                .contains(component, SBOL_HAS_FEATURE, in_child_of)
        {
            self.error(
                "sbol3-10901",
                object,
                Some(SBOL_IN_CHILD_OF),
                "ComponentReference inChildOf must be a SubComponent of its parent Component",
            );
        }

        if let Some(parent_in_child_of) = parent_reference_in_child_of {
            let Some(parent_subcomponent) = self.document.get(&parent_in_child_of) else {
                return;
            };
            let Some(referenced_component) = parent_subcomponent.first_resource(SBOL_INSTANCE_OF)
            else {
                return;
            };

            if let Some(in_child_of) = object.first_resource(SBOL_IN_CHILD_OF)
                && !component_contains(
                    referenced_component,
                    self.document,
                    SBOL_HAS_FEATURE,
                    in_child_of,
                )
            {
                self.error(
                    "sbol3-10902",
                    object,
                    Some(SBOL_IN_CHILD_OF),
                    "nested ComponentReference inChildOf must be a SubComponent of the referenced Component",
                );
            }
        }

        let Some(in_child_of) = object.first_resource(SBOL_IN_CHILD_OF) else {
            return;
        };
        let Some(subcomponent) = self.document.get(in_child_of) else {
            return;
        };
        let Some(referenced_component) = subcomponent.first_resource(SBOL_INSTANCE_OF) else {
            return;
        };
        let Some(refers_to) = object.first_resource(SBOL_REFERS_TO) else {
            return;
        };
        let Some(refers_to_object) = self.document.get(refers_to) else {
            return;
        };

        let references = ComponentReferenceResolver::new(self.document, &self.ownership);
        if let Err(FeatureResolveError::Cycle(_)) = references.trace_feature(object.identity()) {
            self.error(
                "sbol3-10903",
                object,
                Some(SBOL_REFERS_TO),
                "ComponentReference refersTo chain must not form a cycle",
            );
            return;
        }

        if refers_to_object.has_class(SbolClass::ComponentReference) {
            let child_of_reference = references.reference_is_child_of(refers_to, object.identity());
            let child_of_referenced_component = component_contains(
                referenced_component,
                self.document,
                SBOL_HAS_FEATURE,
                refers_to,
            );
            if !child_of_reference && !child_of_referenced_component {
                self.error(
                    "sbol3-10903",
                    object,
                    Some(SBOL_REFERS_TO),
                    "ComponentReference refersTo target must be a permitted ComponentReference child",
                );
            }
        } else if !component_contains(
            referenced_component,
            self.document,
            SBOL_HAS_FEATURE,
            refers_to,
        ) {
            self.error(
                "sbol3-10904",
                object,
                Some(SBOL_REFERS_TO),
                "ComponentReference refersTo target must be a Feature of the referenced Component",
            );
        }
    }

    pub(crate) fn validate_variable_feature(&mut self, object: &Object) {
        if let Some(cardinality) = object.first_iri(SBOL_CARDINALITY)
            && !VARIABLE_FEATURE_CARDINALITY_IRIS.contains(&cardinality.as_str())
        {
            self.error(
                "sbol3-12201",
                object,
                Some(SBOL_CARDINALITY),
                format!("unsupported VariableFeature cardinality `{cardinality}`"),
            );
        }
        let Some(derivation) = self
            .ownership
            .single_parent(object.identity(), SBOL_HAS_VARIABLE_FEATURE)
        else {
            return;
        };
        let Some(template) = self
            .document
            .get(derivation)
            .and_then(|object| object.first_resource(SBOL_TEMPLATE))
        else {
            return;
        };
        if let Some(variable) = object.first_resource(SBOL_VARIABLE)
            && !component_contains(template, self.document, SBOL_HAS_FEATURE, variable)
        {
            self.error(
                "sbol3-12202",
                object,
                Some(SBOL_VARIABLE),
                "VariableFeature variable must be a Feature of the template Component",
            );
        }
    }

    pub(crate) fn validate_variant_collections(&mut self, object: &Object) {
        for collection in object.resources(SBOL_VARIANT_COLLECTION) {
            let mut visited = BTreeSet::new();
            if !collection_contains_only_components_or_collections(
                self.document,
                collection,
                &mut visited,
            ) {
                self.error(
                    "sbol3-12203",
                    object,
                    Some(SBOL_VARIANT_COLLECTION),
                    format!(
                        "variantCollection `{collection}` contains a member that is not a Component or Collection"
                    ),
                );
            }
        }
    }
}
