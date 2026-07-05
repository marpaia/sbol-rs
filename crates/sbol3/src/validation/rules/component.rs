use crate::Object;
use crate::validation::helpers::{
    ExternalResource, SequenceInfo, component_sequence_infos, known_external_resource,
};
use crate::validation::options::TopologyCompleteness;
use crate::validation::tables::{self, ComponentType};
use crate::validation::validator::Validator;
use crate::vocab::*;
use sbol_ontology::ComponentTypeFamily;

impl<'a> Validator<'a> {
    pub(crate) fn validate_table_2_type_count(&mut self, object: &Object, rule: &'static str) {
        let count = object
            .iris(SBOL_TYPE)
            .filter(|iri| tables::is_table_2_component_type(self.ontology(), iri.as_str()))
            .count();
        if count > 1 {
            self.error(
                rule,
                object,
                Some(SBOL_TYPE),
                "type property must not contain more than one URI from SBOL Table 2",
            );
        }
    }

    pub(crate) fn validate_component_type_terms(&mut self, object: &Object, rule: &'static str) {
        for component_type in object.iris(SBOL_TYPE) {
            match tables::is_component_type_term(self.ontology(), component_type.as_str()) {
                Some(true) => {}
                Some(false) => self.error(
                    rule,
                    object,
                    Some(SBOL_TYPE),
                    format!("type `{component_type}` is not an ontology Component type term"),
                ),
                None => {}
            }
        }
    }

    pub(crate) fn validate_component_type_physical_entity_branch(&mut self, object: &Object) {
        for component_type in object.iris(SBOL_TYPE) {
            if matches!(
                tables::is_sbo_physical_entity_term(self.ontology(), component_type.as_str()),
                Some(false)
            ) {
                self.warning(
                    "sbol3-10604",
                    object,
                    Some(SBOL_TYPE),
                    format!(
                        "Component type `{component_type}` should refer to a term from the SBO physical entity representation branch (SBO:0000236)"
                    ),
                );
            }
        }
    }

    pub(crate) fn validate_table_2_type_recommendation(
        &mut self,
        object: &Object,
        rule: &'static str,
    ) {
        for component_type in object.iris(SBOL_TYPE) {
            if !matches!(
                tables::is_sbo_physical_entity_term(self.ontology(), component_type.as_str()),
                Some(true)
            ) {
                continue;
            }
            if tables::is_table_2_component_type(self.ontology(), component_type.as_str()) {
                continue;
            }
            self.warning(
                rule,
                object,
                Some(SBOL_TYPE),
                format!("type `{component_type}` should refer to a term from SBOL Table 2"),
            );
        }
    }

    pub(crate) fn validate_component_type_conflicts(
        &mut self,
        object: &Object,
        rule: &'static str,
    ) {
        let component_types = object.iris(SBOL_TYPE).collect::<Vec<_>>();
        for (index, left) in component_types.iter().enumerate() {
            for right in component_types.iter().skip(index + 1) {
                if !matches!(
                    tables::type_terms_conflict(self.ontology(), left.as_str(), right.as_str()),
                    Some(true)
                ) {
                    continue;
                }
                self.error(
                    rule,
                    object,
                    Some(SBOL_TYPE),
                    format!("type `{left}` conflicts with type `{right}`"),
                );
            }
        }
    }

    pub(crate) fn validate_topology_type_count(&mut self, object: &Object, rule: &'static str) {
        if !object
            .iris(SBOL_TYPE)
            .any(|iri| tables::is_nucleic_acid_component_type(self.ontology(), iri.as_str()))
        {
            return;
        }

        let topology_count = object
            .iris(SBOL_TYPE)
            .filter(|iri| {
                tables::is_topology_type_term(self.ontology(), iri.as_str()) == Some(true)
            })
            .count();
        if topology_count > 1 {
            self.warning(
                rule,
                object,
                Some(SBOL_TYPE),
                "DNA/RNA objects should have at most one known SO topology type",
            );
        }
    }

    pub(crate) fn validate_topology_required_when_complete(
        &mut self,
        object: &Object,
        rule: &'static str,
    ) {
        if self.context.options().topology_completeness
            != TopologyCompleteness::RequireKnownForNucleicAcids
        {
            return;
        }
        if !object
            .iris(SBOL_TYPE)
            .any(|iri| tables::is_nucleic_acid_component_type(self.ontology(), iri.as_str()))
        {
            return;
        }
        if object
            .iris(SBOL_TYPE)
            .any(|iri| tables::is_topology_type_term(self.ontology(), iri.as_str()) == Some(true))
        {
            return;
        }

        self.error(
            rule,
            object,
            Some(SBOL_TYPE),
            "DNA/RNA object is missing a known SO topology type under topology-complete validation",
        );
    }

    pub(crate) fn validate_topology_or_strand_requires_nucleic_acid(
        &mut self,
        object: &Object,
        rule: &'static str,
    ) {
        if object
            .iris(SBOL_TYPE)
            .any(|iri| tables::is_nucleic_acid_component_type(self.ontology(), iri.as_str()))
        {
            return;
        }

        for component_type in object.iris(SBOL_TYPE) {
            let is_topology =
                tables::is_topology_type_term(self.ontology(), component_type.as_str())
                    == Some(true);
            let is_strand =
                tables::is_strand_type_term(self.ontology(), component_type.as_str()) == Some(true);
            if !is_topology && !is_strand {
                continue;
            }
            self.warning(
                rule,
                object,
                Some(SBOL_TYPE),
                format!(
                    "type `{component_type}` is an SO topology or strand term but the object is not explicitly DNA or RNA"
                ),
            );
        }
    }

    pub(crate) fn validate_component_role_terms(&mut self, object: &Object, rule: &'static str) {
        for role in object.iris(SBOL_ROLE) {
            match tables::is_component_role_term(self.ontology(), role.as_str()) {
                Some(true) => {}
                Some(false) => self.error(
                    rule,
                    object,
                    Some(SBOL_ROLE),
                    format!("role `{role}` is not an ontology Component role term"),
                ),
                None => {}
            }
        }
    }

    pub(crate) fn validate_component_role_type_compatibility(
        &mut self,
        object: &Object,
        rule: &'static str,
    ) {
        let component_types = object
            .iris(SBOL_TYPE)
            .filter(|iri| tables::is_table_2_component_type(self.ontology(), iri.as_str()))
            .map(|iri| iri.as_str().to_owned())
            .collect::<Vec<_>>();
        if component_types.is_empty() {
            return;
        }

        for role in object.iris(SBOL_ROLE) {
            if tables::is_component_role_term(self.ontology(), role.as_str()) != Some(true) {
                continue;
            }

            let mut decidable = false;
            let mut compatible = false;
            for component_type in &component_types {
                match tables::component_role_compatible_with_component_type(
                    self.ontology(),
                    role.as_str(),
                    component_type,
                ) {
                    Some(true) => {
                        decidable = true;
                        compatible = true;
                    }
                    Some(false) => decidable = true,
                    None => {}
                }
            }

            if decidable && !compatible {
                self.error(
                    rule,
                    object,
                    Some(SBOL_ROLE),
                    format!("role `{role}` is not compatible with the object's known Table 2 type"),
                );
            }
        }
    }

    pub(crate) fn validate_sequence_feature_role_requires_nucleic_acid(
        &mut self,
        object: &Object,
        rule: &'static str,
    ) {
        if object
            .iris(SBOL_TYPE)
            .any(|iri| tables::is_nucleic_acid_component_type(self.ontology(), iri.as_str()))
        {
            return;
        }

        for role in object.iris(SBOL_ROLE) {
            if tables::is_sequence_feature_role_term(self.ontology(), role.as_str()) != Some(true) {
                continue;
            }
            self.warning(
                rule,
                object,
                Some(SBOL_ROLE),
                format!(
                    "role `{role}` is an SO sequence feature term but the object is not explicitly DNA or RNA"
                ),
            );
        }
    }

    pub(crate) fn validate_nucleic_acid_sequence_feature_role_count(
        &mut self,
        object: &Object,
        rule: &'static str,
    ) {
        if !object
            .iris(SBOL_TYPE)
            .any(|iri| tables::is_nucleic_acid_component_type(self.ontology(), iri.as_str()))
        {
            return;
        }

        let mut sequence_feature_role_count = 0;
        let mut unknown_role_present = false;
        for role in object.iris(SBOL_ROLE) {
            match tables::is_sequence_feature_role_term(self.ontology(), role.as_str()) {
                Some(true) => sequence_feature_role_count += 1,
                Some(false) => {}
                None => unknown_role_present = true,
            }
        }

        if sequence_feature_role_count > 1
            || (sequence_feature_role_count == 0 && !unknown_role_present)
        {
            self.warning(
                rule,
                object,
                Some(SBOL_ROLE),
                "DNA/RNA objects should have exactly one known SO sequence feature role",
            );
        }
    }

    pub(crate) fn validate_external_definition_resource(&mut self, object: &Object) {
        let Some(definition) = object.first_iri(SBOL_DEFINITION) else {
            return;
        };
        let Some(resource) = known_external_resource(definition.as_str()) else {
            return;
        };

        for component_type in object.iris(SBOL_TYPE) {
            let Some(family) =
                tables::component_type_family(self.ontology(), component_type.as_str())
            else {
                continue;
            };
            let mismatch = match family {
                ComponentTypeFamily::Protein => resource != ExternalResource::Uniprot,
                ComponentTypeFamily::SimpleChemical => !matches!(
                    resource,
                    ExternalResource::Chebi | ExternalResource::Pubchem
                ),
                _ => false,
            };
            if mismatch {
                self.warning(
                    "sbol3-11109",
                    object,
                    Some(SBOL_DEFINITION),
                    format!(
                        "ExternallyDefined definition `{definition}` uses a resource not recommended for type `{component_type}`"
                    ),
                );
            }
        }
    }

    pub(crate) fn validate_component_sequence_rules(&mut self, object: &Object) {
        let component_types = object
            .iris(SBOL_TYPE)
            .filter_map(|iri| tables::component_type(self.ontology(), iri.as_str()))
            .collect::<Vec<_>>();
        let sequences = component_sequence_infos(self.ontology(), self.document, object);

        self.validate_component_sequence_type_compatibility(object, &component_types, &sequences);
        self.validate_component_sequence_encoding_conflicts(object, &sequences);
        self.validate_component_required_sequence_encoding(object, &component_types, &sequences);
        self.validate_component_same_encoding_elements(object, &sequences);
    }

    pub(crate) fn validate_component_sequence_type_compatibility(
        &mut self,
        component: &Object,
        component_types: &[ComponentType],
        sequences: &[SequenceInfo],
    ) {
        let cross_listed: Vec<&ComponentType> = component_types
            .iter()
            .filter(|component_type| {
                tables::component_type_has_cross_listed_encoding(self.ontology(), component_type)
            })
            .collect();
        for component_type in cross_listed {
            for sequence in sequences {
                let Some(encoding) = sequence.encoding.as_ref() else {
                    continue;
                };
                if tables::sequence_encoding_compatible_with_component_type(
                    self.ontology(),
                    encoding,
                    component_type,
                ) {
                    continue;
                }
                let component_name = tables::component_type_name(self.ontology(), component_type);
                let encoding_name = tables::sequence_encoding_name(self.ontology(), encoding);
                self.error(
                    "sbol3-10614",
                    component,
                    Some(SBOL_HAS_SEQUENCE),
                    format!(
                        "Component type `{component_name}` is not compatible with Sequence `{}` encoding `{encoding_name}`",
                        sequence.identity,
                    ),
                );
            }
        }
    }

    pub(crate) fn validate_component_sequence_encoding_conflicts(
        &mut self,
        component: &Object,
        sequences: &[SequenceInfo],
    ) {
        for (index, left) in sequences.iter().enumerate() {
            let Some(left_encoding) = left.encoding.as_ref() else {
                continue;
            };
            for right in sequences.iter().skip(index + 1) {
                let Some(right_encoding) = right.encoding.as_ref() else {
                    continue;
                };
                if !tables::sequence_encodings_conflict(
                    self.ontology(),
                    left_encoding,
                    right_encoding,
                ) {
                    continue;
                }
                self.error(
                    "sbol3-10615",
                    component,
                    Some(SBOL_HAS_SEQUENCE),
                    format!(
                        "Sequence `{}` encoding `{}` conflicts with Sequence `{}` encoding `{}`",
                        left.identity,
                        tables::sequence_encoding_name(self.ontology(), left_encoding),
                        right.identity,
                        tables::sequence_encoding_name(self.ontology(), right_encoding)
                    ),
                );
            }
        }
    }

    pub(crate) fn validate_component_required_sequence_encoding(
        &mut self,
        component: &Object,
        component_types: &[ComponentType],
        sequences: &[SequenceInfo],
    ) {
        if sequences.is_empty() || sequences.iter().any(|sequence| sequence.encoding.is_none()) {
            return;
        }

        let cross_listed: Vec<&ComponentType> = component_types
            .iter()
            .filter(|component_type| {
                tables::component_type_has_cross_listed_encoding(self.ontology(), component_type)
            })
            .collect();
        for component_type in cross_listed {
            if sequences.iter().any(|sequence| {
                sequence.encoding.as_ref().is_some_and(|encoding| {
                    tables::sequence_encoding_compatible_with_component_type(
                        self.ontology(),
                        encoding,
                        component_type,
                    )
                })
            }) {
                continue;
            }
            let component_name = tables::component_type_name(self.ontology(), component_type);
            self.error(
                "sbol3-10616",
                component,
                Some(SBOL_HAS_SEQUENCE),
                format!(
                    "Component type `{component_name}` requires at least one compatible Table 1 Sequence encoding",
                ),
            );
        }
    }

    pub(crate) fn validate_component_same_encoding_elements(
        &mut self,
        component: &Object,
        sequences: &[SequenceInfo],
    ) {
        for (index, left) in sequences.iter().enumerate() {
            let (Some(left_encoding_iri), Some(left_elements)) =
                (left.encoding_iri.as_deref(), left.elements.as_deref())
            else {
                continue;
            };
            for right in sequences.iter().skip(index + 1) {
                let (Some(right_encoding_iri), Some(right_elements)) =
                    (right.encoding_iri.as_deref(), right.elements.as_deref())
                else {
                    continue;
                };
                if left_encoding_iri != right_encoding_iri || left_elements == right_elements {
                    continue;
                }
                self.warning(
                    "sbol3-10617",
                    component,
                    Some(SBOL_HAS_SEQUENCE),
                    format!(
                        "Sequences `{}` and `{}` use the same encoding but have different elements",
                        left.identity, right.identity
                    ),
                );
            }
        }
    }
}
