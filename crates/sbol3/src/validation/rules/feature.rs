use crate::object::ObjectClasses;
use crate::validation::tables;
use crate::validation::validator::Validator;
use crate::vocab::*;
use crate::{Object, SbolClass};

impl<'a> Validator<'a> {
    pub(crate) fn validate_feature_vocabularies(&mut self, object: &Object) {
        if object.has_class(SbolClass::Feature) {
            self.validate_feature_roles(object);
            for orientation in object.iris(SBOL_ORIENTATION) {
                if !ORIENTATION_IRIS.contains(&orientation.as_str()) {
                    self.error(
                        "sbol3-10702",
                        object,
                        Some(SBOL_ORIENTATION),
                        format!("unsupported Feature orientation `{orientation}`"),
                    );
                }
            }
        }
        if object.has_class(SbolClass::Location) {
            for orientation in object.iris(SBOL_ORIENTATION) {
                if !ORIENTATION_IRIS.contains(&orientation.as_str()) {
                    self.error(
                        "sbol3-11301",
                        object,
                        Some(SBOL_ORIENTATION),
                        format!("unsupported Location orientation `{orientation}`"),
                    );
                }
            }
        }

        if object.has_class(SbolClass::SubComponent) {
            for role_integration in object.iris(SBOL_ROLE_INTEGRATION) {
                if !ROLE_INTEGRATION_IRIS.contains(&role_integration.as_str()) {
                    self.error(
                        "sbol3-10801",
                        object,
                        Some(SBOL_ROLE_INTEGRATION),
                        format!("unsupported roleIntegration `{role_integration}`"),
                    );
                }
            }
            if !object.values(SBOL_ROLE).is_empty()
                && object.values(SBOL_ROLE_INTEGRATION).is_empty()
            {
                self.error(
                    "sbol3-10802",
                    object,
                    Some(SBOL_ROLE_INTEGRATION),
                    "SubComponent with role values must provide roleIntegration",
                );
            }
        }
    }

    pub(crate) fn validate_feature_roles(&mut self, object: &Object) {
        for role in object.iris(SBOL_ROLE) {
            match tables::is_feature_role_term(self.ontology(), role.as_str()) {
                Some(true) => {}
                Some(false) => self.error(
                    "sbol3-10701",
                    object,
                    Some(SBOL_ROLE),
                    format!("Feature role `{role}` is not an ontology Feature role term"),
                ),
                None => {}
            }
        }
    }
}
