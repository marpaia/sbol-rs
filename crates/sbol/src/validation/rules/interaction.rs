use crate::Object;
use crate::validation::helpers::component_contains;
use crate::validation::tables;
use crate::validation::validator::Validator;
use crate::vocab::*;

impl<'a> Validator<'a> {
    pub(crate) fn validate_participation(&mut self, object: &Object) {
        self.validate_participation_roles(object);
        self.validate_participation_role_conflicts(object);
        self.validate_participation_role_branch_count(object);

        let participant_count = usize::from(object.first_resource(SBOL_PARTICIPANT).is_some());
        let higher_count = usize::from(
            object
                .first_resource(SBOL_HIGHER_ORDER_PARTICIPANT)
                .is_some(),
        );
        if participant_count + higher_count != 1 {
            self.error(
                "sbol3-11901",
                object,
                None,
                "Participation must contain precisely one participant or higherOrderParticipant",
            );
        }
        let Some(interaction) = self
            .ownership
            .single_parent(object.identity(), SBOL_HAS_PARTICIPATION)
        else {
            return;
        };
        let Some(component) = self
            .ownership
            .single_parent(interaction, SBOL_HAS_INTERACTION)
            .cloned()
        else {
            return;
        };
        if let Some(participant) = object.first_resource(SBOL_PARTICIPANT)
            && !component_contains(&component, self.document, SBOL_HAS_FEATURE, participant)
        {
            self.error(
                "sbol3-11902",
                object,
                Some(SBOL_PARTICIPANT),
                "Participation participant must be a Feature of the containing Component",
            );
        }
        if let Some(higher) = object.first_resource(SBOL_HIGHER_ORDER_PARTICIPANT)
            && !component_contains(&component, self.document, SBOL_HAS_INTERACTION, higher)
        {
            self.error(
                "sbol3-11903",
                object,
                Some(SBOL_HIGHER_ORDER_PARTICIPANT),
                "higherOrderParticipant must be an Interaction of the containing Component",
            );
        }
    }

    pub(crate) fn validate_interaction(&mut self, object: &Object) {
        self.validate_interaction_type_terms(object);
        self.validate_interaction_type_conflicts(object);
        self.validate_interaction_type_branch_count(object);
        self.validate_interaction_participation_roles(object);
    }

    pub(crate) fn validate_interaction_type_terms(&mut self, object: &Object) {
        for interaction_type in object.iris(SBOL_TYPE) {
            match tables::is_interaction_type_term(self.ontology(), interaction_type.as_str()) {
                Some(true) => {}
                Some(false) => self.error(
                    "sbol3-11801",
                    object,
                    Some(SBOL_TYPE),
                    format!(
                        "Interaction type `{interaction_type}` is not an ontology Interaction type term"
                    ),
                ),
                None => {}
            }
        }
    }

    pub(crate) fn validate_interaction_type_conflicts(&mut self, object: &Object) {
        let interaction_types = object.iris(SBOL_TYPE).collect::<Vec<_>>();
        for (index, left) in interaction_types.iter().enumerate() {
            for right in interaction_types.iter().skip(index + 1) {
                if !matches!(
                    tables::type_terms_conflict(self.ontology(), left.as_str(), right.as_str()),
                    Some(true)
                ) {
                    continue;
                }
                self.error(
                    "sbol3-11802",
                    object,
                    Some(SBOL_TYPE),
                    format!("Interaction type `{left}` conflicts with type `{right}`"),
                );
            }
        }
    }

    pub(crate) fn validate_interaction_type_branch_count(&mut self, object: &Object) {
        let mut known_interaction_type_count = 0;
        let mut unknown_type_present = false;
        for interaction_type in object.iris(SBOL_TYPE) {
            match tables::is_interaction_type_term(self.ontology(), interaction_type.as_str()) {
                Some(true) => known_interaction_type_count += 1,
                Some(false) => {}
                None => unknown_type_present = true,
            }
        }

        if known_interaction_type_count > 1
            || (known_interaction_type_count == 0 && !unknown_type_present)
        {
            self.warning(
                "sbol3-11803",
                object,
                Some(SBOL_TYPE),
                "Interaction should have exactly one known SBO occurring-entity-relationship type",
            );
        }
    }

    pub(crate) fn validate_interaction_participation_roles(&mut self, object: &Object) {
        let interaction_types = object
            .iris(SBOL_TYPE)
            .filter(|interaction_type| {
                tables::is_interaction_type_term(self.ontology(), interaction_type.as_str())
                    == Some(true)
            })
            .map(|interaction_type| interaction_type.as_str().to_owned())
            .collect::<Vec<_>>();
        if interaction_types.is_empty() {
            return;
        }

        for participation in object.resources(SBOL_HAS_PARTICIPATION) {
            let Some(participation_object) = self.document.get(participation) else {
                continue;
            };
            let mut known_role_count = 0;
            let mut unknown_role_present = false;
            let mut compatible_role_present = false;

            for role in participation_object.iris(SBOL_ROLE) {
                match tables::is_participation_role_term(self.ontology(), role.as_str()) {
                    Some(true) => {
                        known_role_count += 1;
                        compatible_role_present = compatible_role_present
                            || interaction_types.iter().any(|interaction_type| {
                                tables::participation_role_compatible_with_interaction_type(
                                    self.ontology(),
                                    role.as_str(),
                                    interaction_type,
                                ) == Some(true)
                            });
                    }
                    Some(false) => {}
                    None => unknown_role_present = true,
                }
            }

            if !compatible_role_present && (known_role_count == 0 || !unknown_role_present) {
                self.warning(
                    "sbol3-11804",
                    participation_object,
                    Some(SBOL_ROLE),
                    "Participation should have a known role cross-listed with the Interaction type",
                );
            }
        }
    }

    pub(crate) fn validate_participation_roles(&mut self, object: &Object) {
        for role in object.iris(SBOL_ROLE) {
            match tables::is_participation_role_term(self.ontology(), role.as_str()) {
                Some(true) => {}
                Some(false) => self.error(
                    "sbol3-11904",
                    object,
                    Some(SBOL_ROLE),
                    format!(
                        "Participation role `{role}` is not an ontology Participation role term"
                    ),
                ),
                None => {}
            }
        }
    }

    pub(crate) fn validate_participation_role_conflicts(&mut self, object: &Object) {
        let roles = object.iris(SBOL_ROLE).collect::<Vec<_>>();
        for (index, left) in roles.iter().enumerate() {
            for right in roles.iter().skip(index + 1) {
                if !matches!(
                    tables::type_terms_conflict(self.ontology(), left.as_str(), right.as_str()),
                    Some(true)
                ) {
                    continue;
                }
                self.error(
                    "sbol3-11905",
                    object,
                    Some(SBOL_ROLE),
                    format!("Participation role `{left}` conflicts with role `{right}`"),
                );
            }
        }
    }

    pub(crate) fn validate_participation_role_branch_count(&mut self, object: &Object) {
        let mut known_participation_role_count = 0;
        let mut unknown_role_present = false;
        for role in object.iris(SBOL_ROLE) {
            match tables::is_participation_role_term(self.ontology(), role.as_str()) {
                Some(true) => known_participation_role_count += 1,
                Some(false) => {}
                None => unknown_role_present = true,
            }
        }

        if known_participation_role_count > 1
            || (known_participation_role_count == 0 && !unknown_role_present)
        {
            self.warning(
                "sbol3-11906",
                object,
                Some(SBOL_ROLE),
                "Participation should have exactly one known SBO participant role",
            );
        }
    }

    pub(crate) fn validate_interface(&mut self, object: &Object) {
        let Some(component) = self
            .ownership
            .single_parent(object.identity(), SBOL_HAS_INTERFACE)
            .cloned()
        else {
            return;
        };
        for (predicate, rule) in [
            (SBOL_INPUT, "sbol3-12001"),
            (SBOL_OUTPUT, "sbol3-12002"),
            (SBOL_NONDIRECTIONAL, "sbol3-12003"),
        ] {
            for feature in object.resources(predicate) {
                if !component_contains(&component, self.document, SBOL_HAS_FEATURE, feature) {
                    self.error(
                        rule,
                        object,
                        Some(predicate),
                        format!("Interface property `{predicate}` must refer to a Feature of the containing Component"),
                    );
                }
            }
        }
    }
}
