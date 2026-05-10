use crate::validation::helpers::first_invalid_sequence_element;
use crate::validation::tables;
use crate::validation::validator::Validator;
use crate::vocab::*;
use crate::{Object, SbolClass};

impl<'a> Validator<'a> {
    pub(crate) fn validate_sequence(&mut self, object: &Object) {
        if !object.has_class(SbolClass::Sequence) {
            return;
        }

        if object.values(SBOL_ELEMENTS).len() == 1 && object.values(SBOL_ENCODING).is_empty() {
            self.error(
                "sbol3-10501",
                object,
                Some(SBOL_ENCODING),
                "Sequence objects with elements must provide an encoding",
            );
        }
        if let (Some(elements), Some(encoding)) = (
            object.first_literal_value(SBOL_ELEMENTS),
            object.first_iri(SBOL_ENCODING),
        ) && let Some(invalid) =
            first_invalid_sequence_element(self.ontology(), elements, encoding.as_str())
        {
            self.error(
                "sbol3-10503",
                object,
                Some(SBOL_ELEMENTS),
                format!(
                    "Sequence elements contain `{invalid}`, which is inconsistent with encoding `{encoding}`"
                ),
            );
        }
        for encoding in object.iris(SBOL_ENCODING) {
            match tables::is_sequence_encoding_term(self.ontology(), encoding.as_str()) {
                Some(true) => {}
                Some(false) => self.error(
                    "sbol3-10502",
                    object,
                    Some(SBOL_ENCODING),
                    format!(
                        "Sequence encoding `{encoding}` is not an ontology Sequence encoding term"
                    ),
                ),
                None => {}
            }

            if let Some(canonical) =
                tables::canonical_table_1_sequence_encoding_iri(self.ontology(), encoding.as_str())
                && canonical != encoding.as_str()
            {
                self.error(
                    "sbol3-10504",
                    object,
                    Some(SBOL_ENCODING),
                    format!(
                        "Sequence encoding `{encoding}` is equivalent to Table 1 URI `{canonical}`"
                    ),
                );
            }

            if matches!(
                tables::is_edam_textual_format(self.ontology(), encoding.as_str()),
                Some(false)
            ) {
                self.warning(
                    "sbol3-10505",
                    object,
                    Some(SBOL_ENCODING),
                    format!(
                        "Sequence encoding `{encoding}` is not in the EDAM textual format branch"
                    ),
                );
            }
        }
    }
}
