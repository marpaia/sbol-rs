use crate::object::ObjectClasses;
use std::collections::BTreeSet;

use crate::validation::helpers::component_allows_sequence;
use crate::validation::validator::Validator;
use crate::vocab::*;
use crate::{Resource, SbolClass};

impl<'a> Validator<'a> {
    pub(crate) fn validate_derived_from_cycles(&mut self) {
        for object in self.document.objects().values() {
            let mut stack = BTreeSet::new();
            if self.has_cycle_from(object.identity(), PROV_WAS_DERIVED_FROM, &mut stack) {
                self.error(
                    "sbol3-10203",
                    object,
                    Some(PROV_WAS_DERIVED_FROM),
                    "prov:wasDerivedFrom references form a cycle",
                );
            }
        }
    }

    pub(crate) fn validate_was_generated_by_cycles(&mut self) {
        // sbol3-10204 — the chain of `Identified -> prov:wasGeneratedBy ->
        // prov:Activity -> prov:qualifiedUsage -> prov:Usage -> prov:entity
        // -> Identified ...` must be acyclic. The spec's scope is a
        // resolver-aware "provenance history" that crosses documents;
        // here we only enforce the same-document portion. Cross-document
        // detection is tracked separately.
        //
        // A length-1 self-cycle (an Activity's Usage names the same
        // entity that `wasGeneratedBy` it) is tolerated: the canonical
        // SBOLTestSuite `activity` fixture exercises this pattern and
        // libSBOLj3 accepts it. Cycles of length two or more through
        // distinct entities remain rejected.
        for object in self.document.objects().values() {
            let mut stack = BTreeSet::new();
            if self.has_generation_cycle_from(object.identity(), &mut stack) {
                self.error(
                    "sbol3-10204",
                    object,
                    Some(PROV_WAS_GENERATED_BY),
                    "prov:wasGeneratedBy / prov:Usage entity references form a cycle",
                );
            }
        }
    }

    fn has_generation_cycle_from(
        &self,
        identity: &Resource,
        stack: &mut BTreeSet<Resource>,
    ) -> bool {
        if stack.contains(identity) {
            // A revisit through a chain that crossed a distinct entity is
            // a real cycle (length >= 2). A revisit while the only thing
            // on the stack is `identity` itself is the single-activity
            // self-loop tolerated by the policy ADR.
            return stack.len() > 1;
        }
        stack.insert(identity.clone());
        if let Some(object) = self.document.get(identity) {
            for activity_iri in object.resources(PROV_WAS_GENERATED_BY) {
                let Some(activity) = self.document.get(activity_iri) else {
                    continue;
                };
                for usage_iri in activity.resources(PROV_QUALIFIED_USAGE) {
                    let Some(usage) = self.document.get(usage_iri) else {
                        continue;
                    };
                    for entity_iri in usage.resources(PROV_ENTITY) {
                        if self.has_generation_cycle_from(entity_iri, stack) {
                            return true;
                        }
                    }
                }
            }
        }
        stack.remove(identity);
        false
    }

    pub(crate) fn validate_component_instance_cycles(&mut self) {
        for component in self.document.components() {
            let mut stack = BTreeSet::new();
            if self.component_instance_cycle(&component.identity, &mut stack)
                && let Some(object) = self.document.get(&component.identity)
            {
                self.error(
                    "sbol3-10804",
                    object,
                    Some(SBOL_INSTANCE_OF),
                    "SubComponent instanceOf references form a Component cycle",
                );
            }
        }
    }

    pub(crate) fn validate_location_sequence_membership(&mut self) {
        let mut checks = Vec::new();
        for sub_component in self.document.sub_components() {
            if let Some(component) = self
                .ownership
                .single_parent(&sub_component.identity, SBOL_HAS_FEATURE)
                .cloned()
            {
                for location in &sub_component.locations {
                    checks.push((
                        location.clone(),
                        component.clone(),
                        SBOL_HAS_LOCATION,
                        "sbol3-11302",
                    ));
                }
                if let Some(instance_of) = &sub_component.instance_of {
                    for location in &sub_component.source_locations {
                        checks.push((
                            location.clone(),
                            instance_of.clone(),
                            SBOL_SOURCE_LOCATION,
                            "sbol3-11303",
                        ));
                    }
                }
            }
        }
        for sequence_feature in self.document.sequence_features() {
            if let Some(component) = self
                .ownership
                .single_parent(&sequence_feature.identity, SBOL_HAS_FEATURE)
                .cloned()
            {
                for location in &sequence_feature.locations {
                    checks.push((
                        location.clone(),
                        component.clone(),
                        SBOL_HAS_LOCATION,
                        "sbol3-11302",
                    ));
                }
            }
        }
        for local_sub_component in self.document.local_sub_components() {
            if let Some(component) = self
                .ownership
                .single_parent(&local_sub_component.identity, SBOL_HAS_FEATURE)
                .cloned()
            {
                for location in &local_sub_component.locations {
                    checks.push((
                        location.clone(),
                        component.clone(),
                        SBOL_HAS_LOCATION,
                        "sbol3-11302",
                    ));
                }
            }
        }

        for (location, component, predicate, rule) in checks {
            let Some(location_object) = self.document.get(&location) else {
                continue;
            };
            if location_object.has_class(SbolClass::EntireSequence) {
                continue;
            }
            let Some(sequence) = location_object.first_resource(SBOL_HAS_SEQUENCE) else {
                continue;
            };
            if !component_allows_sequence(self.document, &component, sequence) {
                self.error(
                    rule,
                    location_object,
                    Some(predicate),
                    "Location sequence must be declared on the relevant Component or by a sibling EntireSequence",
                );
            }
        }
    }

    pub(crate) fn validate_variant_derivation_cycles(&mut self) {
        for derivation in self.document.combinatorial_derivations() {
            let mut stack = BTreeSet::new();
            if self.variant_derivation_cycle(&derivation.identity, &mut stack)
                && let Some(object) = self.document.get(&derivation.identity)
            {
                self.error(
                    "sbol3-12204",
                    object,
                    Some(SBOL_VARIANT_DERIVATION),
                    "variantDerivation references form a cycle",
                );
            }
        }
    }

    pub(crate) fn variant_derivation_cycle(
        &self,
        derivation: &Resource,
        stack: &mut BTreeSet<Resource>,
    ) -> bool {
        if !stack.insert(derivation.clone()) {
            return true;
        }
        let Some(derivation_object) = self.document.get(derivation) else {
            stack.remove(derivation);
            return false;
        };
        for variable_feature in derivation_object.resources(SBOL_HAS_VARIABLE_FEATURE) {
            let Some(variable_feature_object) = self.document.get(variable_feature) else {
                continue;
            };
            for next_derivation in variable_feature_object.resources(SBOL_VARIANT_DERIVATION) {
                if self.variant_derivation_cycle(next_derivation, stack) {
                    return true;
                }
            }
        }
        stack.remove(derivation);
        false
    }

    pub(crate) fn has_cycle_from(
        &self,
        identity: &Resource,
        predicate: &str,
        stack: &mut BTreeSet<Resource>,
    ) -> bool {
        if !stack.insert(identity.clone()) {
            return true;
        }
        let Some(object) = self.document.get(identity) else {
            stack.remove(identity);
            return false;
        };
        for next in object.resources(predicate) {
            if self.has_cycle_from(next, predicate, stack) {
                return true;
            }
        }
        stack.remove(identity);
        false
    }

    pub(crate) fn component_instance_cycle(
        &self,
        component: &Resource,
        stack: &mut BTreeSet<Resource>,
    ) -> bool {
        if !stack.insert(component.clone()) {
            return true;
        }
        let Some(component_object) = self.document.get(component) else {
            stack.remove(component);
            return false;
        };
        for feature in component_object.resources(SBOL_HAS_FEATURE) {
            let Some(feature_object) = self.document.get(feature) else {
                continue;
            };
            if !feature_object.has_class(SbolClass::SubComponent) {
                continue;
            }
            let Some(instance_of) = feature_object.first_resource(SBOL_INSTANCE_OF) else {
                continue;
            };
            if self.component_instance_cycle(instance_of, stack) {
                return true;
            }
        }
        stack.remove(component);
        false
    }
}
