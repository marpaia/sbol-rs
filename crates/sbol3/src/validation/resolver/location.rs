//! Location normalization: collapsing `Range` / `Cut` / `EntireSequence`
//! into half-open `[start, end)` [`LinearLocation`] spans.

use crate::object::ObjectClasses;
use super::{
    ComponentReferenceResolver, OwnershipIndex, directional_orientation_value, integer_value,
};
use crate::vocab::*;
use crate::{Document, Resource, SbolClass};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DirectionalOrientation {
    Inline,
    ReverseComplement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LinearLocation {
    pub identity: Resource,
    pub sequence: Resource,
    pub start: i64,
    pub end: i64,
    pub orientation: Option<DirectionalOrientation>,
}

pub(crate) struct LocationResolver<'a> {
    document: &'a Document,
    references: ComponentReferenceResolver<'a>,
}

impl<'a> LocationResolver<'a> {
    pub(crate) fn new(document: &'a Document, ownership: &'a OwnershipIndex) -> Self {
        Self {
            document,
            references: ComponentReferenceResolver::new(document, ownership),
        }
    }

    pub(crate) fn locations_for_feature(&self, feature: &Resource) -> Vec<LinearLocation> {
        let target = self
            .references
            .trace_feature(feature)
            .map(|trace| trace.target)
            .unwrap_or_else(|_| feature.clone());
        let Some(feature) = self.document.get(&target) else {
            return Vec::new();
        };

        feature
            .resources(SBOL_HAS_LOCATION)
            .filter_map(|location| self.linear_location(location))
            .collect()
    }

    pub(crate) fn linear_location(&self, identity: &Resource) -> Option<LinearLocation> {
        let location = self.document.get(identity)?;
        let sequence = location.first_resource(SBOL_HAS_SEQUENCE)?.clone();
        let orientation = directional_orientation_value(location.first_iri(SBOL_ORIENTATION));

        if location.has_class(SbolClass::Range) {
            let start = integer_value(location, SBOL_START)?;
            let end = integer_value(location, SBOL_END)?;
            if start <= 0 || end < start {
                return None;
            }
            return Some(LinearLocation {
                identity: identity.clone(),
                sequence,
                start: start - 1,
                end,
                orientation,
            });
        }

        if location.has_class(SbolClass::Cut) {
            let at = integer_value(location, SBOL_AT)?;
            if at < 0 {
                return None;
            }
            return Some(LinearLocation {
                identity: identity.clone(),
                sequence,
                start: at,
                end: at,
                orientation,
            });
        }

        if location.has_class(SbolClass::EntireSequence) {
            let sequence_object = self.document.get(&sequence)?;
            let end = sequence_object.first_literal_value(SBOL_ELEMENTS)?.len() as i64;
            return Some(LinearLocation {
                identity: identity.clone(),
                sequence,
                start: 0,
                end,
                orientation,
            });
        }

        None
    }
}
