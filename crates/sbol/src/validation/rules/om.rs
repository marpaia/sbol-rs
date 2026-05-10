use crate::Object;
use crate::validation::tables;
use crate::validation::validator::Validator;
use crate::vocab::*;

impl<'a> Validator<'a> {
    pub(crate) fn validate_om_measure(&mut self, object: &Object) {
        for type_iri in object.iris(SBOL_TYPE) {
            if matches!(
                tables::is_sbo_systems_description_parameter_term(
                    self.ontology(),
                    type_iri.as_str()
                ),
                Some(false)
            ) {
                self.warning(
                    "sbol3-13401",
                    object,
                    Some(SBOL_TYPE),
                    format!(
                        "om:Measure type `{type_iri}` should refer to a term from the SBO Systems Description Parameter branch (SBO:0000545)"
                    ),
                );
            }
        }
    }

    pub(crate) fn validate_om_unit_strings(&mut self, object: &Object) {
        self.matching_literals_warning(
            object,
            "sbol3-13501",
            SBOL_NAME,
            OM_LABEL,
            "om:Unit sbol:name and om:label should match",
        );
        self.matching_literals_warning(
            object,
            "sbol3-13502",
            SBOL_DESCRIPTION,
            OM_COMMENT,
            "om:Unit sbol:description and om:comment should match",
        );
    }

    pub(crate) fn validate_om_prefix_strings(&mut self, object: &Object) {
        self.matching_literals_warning(
            object,
            "sbol3-14201",
            SBOL_NAME,
            OM_LABEL,
            "om:Prefix sbol:name and om:label should match",
        );
        self.matching_literals_warning(
            object,
            "sbol3-14202",
            SBOL_DESCRIPTION,
            OM_COMMENT,
            "om:Prefix sbol:description and om:comment should match",
        );
    }

    fn matching_literals_warning(
        &mut self,
        object: &Object,
        rule: &'static str,
        left_predicate: &'static str,
        right_predicate: &'static str,
        message: &'static str,
    ) {
        let Some(left) = object.first_literal_value(left_predicate) else {
            return;
        };
        let Some(right) = object.first_literal_value(right_predicate) else {
            return;
        };
        if left != right {
            self.warning(rule, object, Some(right_predicate), message);
        }
    }
}
