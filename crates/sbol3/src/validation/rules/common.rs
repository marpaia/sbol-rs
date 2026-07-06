use crate::object::ObjectClasses;
use crate::schema::{Cardinality, FieldDescriptor as PropertySpec, TargetClass};
use crate::validation::context::{ExternalValidationMode, ResolutionErrorKind};
use crate::validation::helpers::{
    COMPOSITE_PREDICATES, is_external_top_level_reference, is_url, is_valid_display_id,
    object_matches_target, url_is_child_of, url_matches_namespace_display_id, value_matches_kind,
};
use crate::validation::spec::{is_known_sbol_iri, is_known_sbol_property, property_specs_for};
use crate::validation::validator::Validator;
use crate::vocab::*;
use crate::{Object, Resource, SbolClass};

impl<'a> Validator<'a> {
    pub(crate) fn validate_sbol_namespace(&mut self, object: &Object) {
        for rdf_type in object.rdf_types() {
            if rdf_type.as_str().starts_with(SBOL_NS) && !is_known_sbol_iri(rdf_type.as_str()) {
                self.error(
                    "sbol3-10105",
                    object,
                    Some(RDF_TYPE),
                    format!("unknown SBOL rdf:type `{}`", rdf_type.as_str()),
                );
            }
        }

        let has_sbol_property = object
            .properties()
            .keys()
            .any(|predicate| predicate.as_str().starts_with(SBOL_NS));
        let has_sbol_type = object
            .rdf_types()
            .iter()
            .any(|rdf_type| rdf_type.as_str().starts_with(SBOL_NS));
        if has_sbol_property && !has_sbol_type {
            self.warning(
                "sbol3-10108",
                object,
                None,
                "object has SBOL properties but no SBOL rdf:type",
            );
        }

        let property_specs = property_specs_for(object);
        for predicate in object.properties().keys() {
            if !predicate.as_str().starts_with(SBOL_NS) {
                continue;
            }
            if !is_known_sbol_property(predicate.as_str()) {
                self.error(
                    "sbol3-10105",
                    object,
                    None,
                    format!("unknown SBOL property `{}`", predicate.as_str()),
                );
                continue;
            }
            if !property_specs.contains_key(predicate.as_str()) {
                self.error(
                    "sbol3-10109",
                    object,
                    None,
                    format!(
                        "property `{}` is not allowed for the object's SBOL type",
                        predicate.as_str()
                    ),
                );
            }
        }
    }

    pub(crate) fn validate_sbol_types(&mut self, object: &Object) {
        let sbol_classes = object
            .rdf_types()
            .iter()
            .filter_map(SbolClass::from_iri)
            .collect::<Vec<_>>();

        for (index, left) in sbol_classes.iter().enumerate() {
            for right in sbol_classes.iter().skip(index + 1) {
                if !left.is_a(*right) && !right.is_a(*left) {
                    self.error(
                        "sbol3-10106",
                        object,
                        Some(RDF_TYPE),
                        format!(
                            "SBOL rdf:type values `{}` and `{}` are disjoint",
                            left.local_name(),
                            right.local_name()
                        ),
                    );
                }
            }
        }

        let concrete_count = sbol_classes
            .iter()
            .filter(|class| !matches!(class, SbolClass::Identified | SbolClass::TopLevel))
            .count();
        if concrete_count > 1 {
            self.warning(
                "sbol3-10107",
                object,
                Some(RDF_TYPE),
                "object has more than one concrete SBOL rdf:type",
            );
        }
    }

    pub(crate) fn validate_table_rules(&mut self, object: &Object) {
        let property_specs = property_specs_for(object);
        for property in property_specs.values() {
            self.validate_property_spec(object, *property);
        }
    }

    pub(crate) fn validate_property_spec(&mut self, object: &Object, property: PropertySpec) {
        let values = object.values(property.predicate);
        match property.cardinality {
            Cardinality::ExactlyOne if values.len() != 1 => self.error(
                property.rule,
                object,
                Some(property.predicate),
                format!(
                    "property `{}` must have exactly one value; found {}",
                    property.predicate,
                    values.len()
                ),
            ),
            Cardinality::ZeroOrOne if values.len() > 1 => self.error(
                property.rule,
                object,
                Some(property.predicate),
                format!(
                    "property `{}` must have zero or one value; found {}",
                    property.predicate,
                    values.len()
                ),
            ),
            Cardinality::OneOrMore if values.is_empty() => self.error(
                property.rule,
                object,
                Some(property.predicate),
                format!(
                    "property `{}` must have one or more values",
                    property.predicate
                ),
            ),
            _ => {}
        }

        for value in values {
            if !value_matches_kind(value, property.value_kind) {
                self.error(
                    "sbol3-10111",
                    object,
                    Some(property.predicate),
                    format!(
                        "property `{}` value does not match expected {:?}",
                        property.predicate, property.value_kind
                    ),
                );
                continue;
            }

            let Some(reference) = property.reference else {
                continue;
            };
            let Some(resource) = value.as_resource() else {
                continue;
            };
            match self.document.get(resource) {
                Some(target) if !object_matches_target(target, reference.target) => self.error(
                    "sbol3-10113",
                    object,
                    Some(property.predicate),
                    format!(
                        "property `{}` refers to `{resource}`, which does not match expected target type",
                        property.predicate
                    ),
                ),
                None if reference.require_local => self.error(
                    "sbol3-10112",
                    object,
                    Some(property.predicate),
                    format!(
                        "property `{}` refers to missing local child `{resource}`",
                        property.predicate
                    ),
                ),
                None if is_external_top_level_reference(reference.target) => {
                    self.validate_external_top_level_reference(
                        object,
                        property.predicate,
                        resource,
                        reference.target,
                    );
                }
                _ => {}
            }
        }
    }

    pub(crate) fn validate_external_top_level_reference(
        &mut self,
        object: &Object,
        predicate: &'static str,
        resource: &Resource,
        expected_target: TargetClass,
    ) {
        let mode = self.context.external_mode();
        if mode == ExternalValidationMode::Off {
            return;
        }

        if let Some(matches_target) = self.context.documents().iter().find_map(|document| {
            document
                .get(resource)
                .map(|target| object_matches_target(target, expected_target))
        }) {
            if !matches_target {
                self.error(
                    "sbol3-10113",
                    object,
                    Some(predicate),
                    format!(
                        "property `{predicate}` resolves `{resource}`, but the target does not match the expected target type"
                    ),
                );
            }
            return;
        }

        if mode == ExternalValidationMode::ExternalAllowed {
            for resolver in self.context.document_resolvers() {
                match resolver.resolve_document(resource) {
                    Ok(document) => {
                        let Some(resolved) = document.get(resource) else {
                            self.warning(
                                "sbol3-10114",
                                object,
                                Some(predicate),
                                format!(
                                    "property `{predicate}` dereferenced `{resource}`, but the resolved document did not contain that object"
                                ),
                            );
                            return;
                        };
                        if !object_matches_target(resolved, expected_target) {
                            self.error(
                                "sbol3-10113",
                                object,
                                Some(predicate),
                                format!(
                                    "property `{predicate}` dereferenced `{resource}`, but the target does not match the expected target type"
                                ),
                            );
                        }
                        return;
                    }
                    Err(error)
                        if matches!(
                            error.kind(),
                            ResolutionErrorKind::UnsupportedScheme | ResolutionErrorKind::NotFound
                        ) => {}
                    Err(error) => {
                        self.warning(
                            "sbol3-10114",
                            object,
                            Some(predicate),
                            format!(
                                "property `{predicate}` could not dereference `{resource}`: {}",
                                error.message()
                            ),
                        );
                        return;
                    }
                }
            }
        }

        self.warning(
            "sbol3-10114",
            object,
            Some(predicate),
            format!("property `{predicate}` refers to unresolved external TopLevel `{resource}`"),
        );
    }

    pub(crate) fn validate_display_id(&mut self, object: &Object) {
        let Some(display_id) = &object.identified().display_id else {
            return;
        };

        if !is_valid_display_id(display_id) {
            self.error(
                "sbol3-10201",
                object,
                Some(SBOL_DISPLAY_ID),
                format!(
                    "displayId `{display_id}` must start with a letter or underscore and contain only letters, digits, or underscores"
                ),
            );
        }
        for derived_from in object.resources(PROV_WAS_DERIVED_FROM) {
            if derived_from == object.identity() {
                self.error(
                    "sbol3-10202",
                    object,
                    Some(PROV_WAS_DERIVED_FROM),
                    "object must not refer to itself via prov:wasDerivedFrom",
                );
            }
        }
    }

    pub(crate) fn validate_top_level(&mut self, object: &Object) {
        if !object.is_top_level() {
            return;
        }

        let Some(identity) = object.identity().as_iri() else {
            self.error(
                "sbol3-10101",
                object,
                None,
                "Identified object identities must be IRIs",
            );
            return;
        };
        let Some(namespace) = object
            .first_resource(SBOL_HAS_NAMESPACE)
            .and_then(Resource::as_iri)
        else {
            return;
        };

        if identity.as_str().starts_with("http://") || identity.as_str().starts_with("https://") {
            let namespace = namespace.as_str().trim_end_matches('/');
            let identity = identity.as_str();
            if identity != namespace && !identity.starts_with(&format!("{namespace}/")) {
                self.error(
                    "sbol3-10301",
                    object,
                    Some(SBOL_HAS_NAMESPACE),
                    format!("TopLevel URL `{identity}` is not prefixed by namespace `{namespace}`"),
                );
            }
        }
    }

    pub(crate) fn validate_top_level_url_prefixes(&mut self) {
        let top_levels = self
            .document
            .objects()
            .values()
            .filter(|object| object.is_top_level())
            .filter_map(|object| {
                let identity = object.identity().as_iri()?;
                is_url(identity.as_str()).then_some((identity.as_str(), object))
            })
            .collect::<Vec<_>>();

        for (index, (left_identity, left_object)) in top_levels.iter().enumerate() {
            self.validate_top_level_url_pattern(left_object);
            for (right_identity, right_object) in top_levels.iter().skip(index + 1) {
                if url_is_child_of(right_identity, left_identity) {
                    self.error(
                        "sbol3-10103",
                        right_object,
                        None,
                        format!(
                            "TopLevel URL `{right_identity}` uses TopLevel URL `{left_identity}` as a prefix"
                        ),
                    );
                }
                if url_is_child_of(left_identity, right_identity) {
                    self.error(
                        "sbol3-10103",
                        left_object,
                        None,
                        format!(
                            "TopLevel URL `{left_identity}` uses TopLevel URL `{right_identity}` as a prefix"
                        ),
                    );
                }
            }
        }
    }

    pub(crate) fn validate_top_level_url_pattern(&mut self, object: &Object) {
        let Some(identity) = object.identity().as_iri() else {
            return;
        };
        if !is_url(identity.as_str()) {
            return;
        }
        let Some(namespace) = object
            .first_resource(SBOL_HAS_NAMESPACE)
            .and_then(Resource::as_iri)
        else {
            return;
        };
        let Some(display_id) = object.identified().display_id.as_deref() else {
            self.error(
                "sbol3-10102",
                object,
                Some(SBOL_DISPLAY_ID),
                "TopLevel URL objects require displayId to satisfy the URL pattern",
            );
            return;
        };

        if !url_matches_namespace_display_id(identity.as_str(), namespace.as_str(), display_id) {
            self.error(
                "sbol3-10102",
                object,
                None,
                format!(
                    "TopLevel URL `{}` does not match [namespace]/[local]/[displayId]",
                    identity.as_str()
                ),
            );
        }
    }

    pub(crate) fn validate_child_url_patterns(&mut self) {
        let mut checks = Vec::new();
        for object in self.document.objects().values() {
            for predicate in COMPOSITE_PREDICATES {
                for child in object.resources(predicate) {
                    checks.push((object.identity().clone(), child.clone()));
                }
            }
        }

        for (parent, child) in checks {
            let Some(parent_iri) = parent.as_iri() else {
                continue;
            };
            let Some(child_object) = self.document.get(&child) else {
                continue;
            };
            let Some(child_iri) = child.as_iri() else {
                continue;
            };
            if !is_url(child_iri.as_str()) {
                continue;
            }
            let Some(display_id) = child_object.identified().display_id.as_deref() else {
                self.error(
                    "sbol3-10104",
                    child_object,
                    Some(SBOL_DISPLAY_ID),
                    "child URL objects require displayId to satisfy the URL pattern",
                );
                continue;
            };
            let expected = format!(
                "{}/{}",
                parent_iri.as_str().trim_end_matches('/'),
                display_id
            );
            if child_iri.as_str() != expected {
                self.error(
                    "sbol3-10104",
                    child_object,
                    None,
                    format!(
                        "child URL `{}` does not match expected URL `{expected}`",
                        child_iri.as_str()
                    ),
                );
            }
        }
    }

    pub(crate) fn validate_class_specific_rules(&mut self, object: &Object) {
        if object.has_class(SbolClass::Range) {
            self.validate_range(object);
        }
        if object.has_class(SbolClass::Cut) {
            self.validate_cut(object);
        }
        if object.has_class(SbolClass::SubComponent) {
            self.validate_sub_component(object);
            self.validate_sub_component_location_lengths(object);
            self.validate_location_overlap(object, SBOL_HAS_LOCATION, "sbol3-10805");
        }
        if object.has_class(SbolClass::LocalSubComponent) {
            self.validate_location_overlap(object, SBOL_HAS_LOCATION, "sbol3-11013");
        }
        if object.has_class(SbolClass::SequenceFeature) {
            self.validate_location_overlap(object, SBOL_HAS_LOCATION, "sbol3-11201");
        }
        if object.has_class(SbolClass::Constraint) {
            self.validate_constraint(object);
        }
        if super::super::helpers::object_type_is_or_inherits(object, OM_MEASURE) {
            self.validate_om_measure(object);
        }
        if super::super::helpers::object_type_is_or_inherits(object, OM_UNIT) {
            self.validate_om_unit_strings(object);
        }
        if super::super::helpers::object_type_is_or_inherits(object, OM_PREFIX) {
            self.validate_om_prefix_strings(object);
        }
        if object.has_class(SbolClass::Participation) {
            self.validate_participation(object);
        }
        if object.has_class(SbolClass::Interaction) {
            self.validate_interaction(object);
        }
        if object.has_class(SbolClass::Interface) {
            self.validate_interface(object);
        }
        if object.has_class(SbolClass::CombinatorialDerivation) {
            self.validate_combinatorial_derivation(object);
        }
        if object.has_class(SbolClass::ComponentReference) {
            self.validate_component_reference(object);
        }
        if object.has_class(SbolClass::VariableFeature) {
            self.validate_variable_feature(object);
            self.validate_variant_collections(object);
        }
        if object.has_class(SbolClass::Component) {
            self.validate_table_2_type_count(object, "sbol3-10601");
            self.validate_component_type_terms(object, "sbol3-10602");
            self.validate_component_type_physical_entity_branch(object);
            self.validate_component_type_conflicts(object, "sbol3-10605");
            self.validate_topology_required_when_complete(object, "sbol3-10606");
            self.validate_topology_type_count(object, "sbol3-10607");
            self.validate_topology_or_strand_requires_nucleic_acid(object, "sbol3-10608");
            self.validate_component_role_type_compatibility(object, "sbol3-10609");
            self.validate_component_role_terms(object, "sbol3-10610");
            self.validate_sequence_feature_role_requires_nucleic_acid(object, "sbol3-10612");
            self.validate_nucleic_acid_sequence_feature_role_count(object, "sbol3-10613");
            self.validate_component_sequence_rules(object);
        }
        if object.has_class(SbolClass::LocalSubComponent) {
            self.validate_table_2_type_count(object, "sbol3-11001");
            self.validate_component_type_terms(object, "sbol3-11002");
            self.validate_table_2_type_recommendation(object, "sbol3-11004");
            self.validate_component_type_conflicts(object, "sbol3-11005");
            self.validate_topology_required_when_complete(object, "sbol3-11006");
            self.validate_topology_type_count(object, "sbol3-11007");
            self.validate_topology_or_strand_requires_nucleic_acid(object, "sbol3-11008");
            self.validate_component_role_type_compatibility(object, "sbol3-11009");
            self.validate_sequence_feature_role_requires_nucleic_acid(object, "sbol3-11011");
            self.validate_nucleic_acid_sequence_feature_role_count(object, "sbol3-11012");
        }
        if object.has_class(SbolClass::ExternallyDefined) {
            self.validate_table_2_type_count(object, "sbol3-11101");
            self.validate_component_type_terms(object, "sbol3-11102");
            self.validate_table_2_type_recommendation(object, "sbol3-11104");
            self.validate_component_type_conflicts(object, "sbol3-11105");
            self.validate_topology_required_when_complete(object, "sbol3-11106");
            self.validate_topology_type_count(object, "sbol3-11107");
            self.validate_topology_or_strand_requires_nucleic_acid(object, "sbol3-11108");
            self.validate_external_definition_resource(object);
        }
        if object.has_class(SbolClass::Model) {
            self.validate_model(object);
        }
        if object.has_class(SbolClass::Attachment) {
            self.validate_attachment(object);
        }
        if object.has_class(SbolClass::Implementation) {
            self.validate_implementation(object);
        }
    }
}
