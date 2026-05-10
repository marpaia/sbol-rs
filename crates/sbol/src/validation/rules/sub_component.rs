use crate::Object;
use crate::validation::helpers::sum_location_lengths;
use crate::validation::validator::Validator;
use crate::vocab::*;

impl<'a> Validator<'a> {
    pub(crate) fn validate_sub_component(&mut self, object: &Object) {
        let Some(parent_component) = self
            .ownership
            .single_parent(object.identity(), SBOL_HAS_FEATURE)
        else {
            return;
        };
        if let Some(instance_of) = object.first_resource(SBOL_INSTANCE_OF)
            && instance_of == parent_component
        {
            self.error(
                "sbol3-10803",
                object,
                Some(SBOL_INSTANCE_OF),
                "SubComponent instanceOf must not refer to its containing Component",
            );
        }
    }

    pub(crate) fn validate_sub_component_location_lengths(&mut self, object: &Object) {
        let locations = object.resources(SBOL_HAS_LOCATION).collect::<Vec<_>>();
        let source_locations = object.resources(SBOL_SOURCE_LOCATION).collect::<Vec<_>>();

        if !locations.is_empty() && !source_locations.is_empty() {
            let Some(location_length) = sum_location_lengths(self.document, &locations) else {
                return;
            };
            let Some(source_location_length) =
                sum_location_lengths(self.document, &source_locations)
            else {
                return;
            };
            if location_length != source_location_length {
                self.error(
                    "sbol3-10806",
                    object,
                    Some(SBOL_HAS_LOCATION),
                    "SubComponent hasLocation and sourceLocation lengths must match",
                );
            }
            return;
        }

        if locations.is_empty() || !source_locations.is_empty() {
            return;
        }
        let Some(instance_of) = object.first_resource(SBOL_INSTANCE_OF) else {
            return;
        };
        let Some(component) = self.document.get(instance_of) else {
            return;
        };
        let sequences = component.resources(SBOL_HAS_SEQUENCE).collect::<Vec<_>>();
        if sequences.len() != 1 {
            return;
        }
        let Some(sequence_length) = self
            .document
            .get(sequences[0])
            .and_then(|sequence| sequence.first_literal_value(SBOL_ELEMENTS))
            .map(str::len)
        else {
            return;
        };
        let Some(location_length) = sum_location_lengths(self.document, &locations) else {
            return;
        };
        if location_length != sequence_length {
            self.error(
                "sbol3-10807",
                object,
                Some(SBOL_HAS_LOCATION),
                "SubComponent location lengths must cover its instanceOf sequence when sourceLocation is absent",
            );
        }
    }
}
