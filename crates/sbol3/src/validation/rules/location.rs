use crate::Object;
use crate::validation::helpers::{integer_value, location_sequence_length};
use crate::validation::validator::Validator;
use crate::vocab::*;

impl<'a> Validator<'a> {
    pub(crate) fn validate_range(&mut self, object: &Object) {
        let Some(start) = integer_value(object, SBOL_START) else {
            return;
        };
        let Some(end) = integer_value(object, SBOL_END) else {
            return;
        };
        if end < start {
            self.error(
                "sbol3-11403",
                object,
                Some(SBOL_END),
                "Range end must be greater than or equal to start",
            );
        }
        if start <= 0 {
            self.error(
                "sbol3-11401",
                object,
                Some(SBOL_START),
                "Range start must be greater than zero",
            );
        }
        if let Some(length) = location_sequence_length(self.document, object) {
            if start as usize > length {
                self.error(
                    "sbol3-11401",
                    object,
                    Some(SBOL_START),
                    "Range start exceeds referenced Sequence length",
                );
            }
            if end as usize > length {
                self.error(
                    "sbol3-11402",
                    object,
                    Some(SBOL_END),
                    "Range end exceeds referenced Sequence length",
                );
            }
        }
    }

    pub(crate) fn validate_cut(&mut self, object: &Object) {
        let Some(at) = integer_value(object, SBOL_AT) else {
            return;
        };
        if at < 0 {
            self.error(
                "sbol3-11501",
                object,
                Some(SBOL_AT),
                "Cut at must be greater than or equal to zero",
            );
        }
        if let Some(length) = location_sequence_length(self.document, object)
            && at as usize > length
        {
            self.error(
                "sbol3-11501",
                object,
                Some(SBOL_AT),
                "Cut at exceeds referenced Sequence length",
            );
        }
    }

    pub(crate) fn validate_location_overlap(
        &mut self,
        object: &Object,
        predicate: &'static str,
        rule: &'static str,
    ) {
        let mut ranges = Vec::new();
        for location in object.resources(predicate) {
            let Some(location_object) = self.document.get(location) else {
                continue;
            };
            let Some(start) = integer_value(location_object, SBOL_START) else {
                continue;
            };
            let Some(end) = integer_value(location_object, SBOL_END) else {
                continue;
            };
            let sequence = location_object.first_resource(SBOL_HAS_SEQUENCE).cloned();
            ranges.push((location.clone(), sequence, start, end));
        }
        for (index, (left_id, left_sequence, left_start, left_end)) in ranges.iter().enumerate() {
            for (right_id, right_sequence, right_start, right_end) in ranges.iter().skip(index + 1)
            {
                if left_sequence == right_sequence
                    && left_start <= right_end
                    && right_start <= left_end
                {
                    self.error(
                        rule,
                        object,
                        Some(predicate),
                        format!("locations `{left_id}` and `{right_id}` overlap"),
                    );
                }
            }
        }
    }
}
