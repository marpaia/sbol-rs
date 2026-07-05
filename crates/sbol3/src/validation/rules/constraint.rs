use crate::Object;
use crate::validation::helpers::component_contains;
use crate::validation::resolver::{ConstraintEngine, RelationOutcome};
use crate::validation::validator::Validator;
use crate::vocab::*;

impl<'a> Validator<'a> {
    pub(crate) fn validate_constraint(&mut self, object: &Object) {
        let Some(component) = self
            .ownership
            .single_parent(object.identity(), SBOL_HAS_CONSTRAINT)
            .cloned()
        else {
            return;
        };
        if let Some(subject) = object.first_resource(SBOL_SUBJECT)
            && !component_contains(&component, self.document, SBOL_HAS_FEATURE, subject)
        {
            self.error(
                "sbol3-11701",
                object,
                Some(SBOL_SUBJECT),
                "Constraint subject must be a Feature of the containing Component",
            );
        }
        if let Some(constrained_object) = object.first_resource(SBOL_OBJECT)
            && !component_contains(
                &component,
                self.document,
                SBOL_HAS_FEATURE,
                constrained_object,
            )
        {
            self.error(
                "sbol3-11702",
                object,
                Some(SBOL_OBJECT),
                "Constraint object must be a Feature of the containing Component",
            );
        }
        if object.first_resource(SBOL_SUBJECT) == object.first_resource(SBOL_OBJECT) {
            self.error(
                "sbol3-11703",
                object,
                Some(SBOL_OBJECT),
                "Constraint subject and object must not be the same Feature",
            );
        }

        let Some(restriction) = object.first_iri(SBOL_RESTRICTION) else {
            return;
        };
        if !CONSTRAINT_RESTRICTION_IRIS.contains(&restriction.as_str()) {
            self.warning(
                "sbol3-11704",
                object,
                Some(SBOL_RESTRICTION),
                format!(
                    "Constraint restriction `{restriction}` is not a recommended SBOL relation"
                ),
            );
        }

        if let (Some(subject), Some(constrained_object)) = (
            object.first_resource(SBOL_SUBJECT),
            object.first_resource(SBOL_OBJECT),
        ) {
            let (table8, table10) = {
                let engine = ConstraintEngine::new(self.document, &self.ownership);
                (
                    engine.table8_relation(restriction.as_str(), subject, constrained_object),
                    engine.table10_relation(restriction.as_str(), subject, constrained_object),
                )
            };
            if matches!(table8, RelationOutcome::Contradicted { .. }) {
                self.error(
                    "sbol3-11705",
                    object,
                    Some(SBOL_RESTRICTION),
                    format!(
                        "Constraint restriction `{restriction}` is contradicted by resolved Feature semantics"
                    ),
                );
            }
            if let RelationOutcome::Contradicted {
                subject_location: Some(subject_location),
                object_location: Some(object_location),
            } = table10
            {
                self.error(
                    "sbol3-11706",
                    object,
                    Some(SBOL_RESTRICTION),
                    format!(
                        "Constraint restriction `{restriction}` is contradicted by locations `{subject_location}` and `{object_location}` on the same Sequence"
                    ),
                );
            }
        }
    }
}
