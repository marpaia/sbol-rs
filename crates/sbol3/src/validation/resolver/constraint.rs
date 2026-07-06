//! Constraint evaluation: Table 8 (identity / orientation) and Table 10
//! (sequential) relation outcomes for `sbol:Constraint`.

use super::{
    ComponentReferenceResolver, DirectionalOrientation, LocationResolver, OwnershipIndex,
    directional_orientation_value, sequential_relation_satisfied,
};
use crate::object::ObjectClasses;
use crate::vocab::*;
use crate::{Document, Resource, SbolClass};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RelationOutcome {
    Satisfied,
    Contradicted {
        subject_location: Option<Resource>,
        object_location: Option<Resource>,
    },
    Unknown,
    Unsupported,
}

pub(crate) struct ConstraintEngine<'a> {
    document: &'a Document,
    references: ComponentReferenceResolver<'a>,
    locations: LocationResolver<'a>,
}

impl<'a> ConstraintEngine<'a> {
    pub(crate) fn new(document: &'a Document, ownership: &'a OwnershipIndex) -> Self {
        Self {
            document,
            references: ComponentReferenceResolver::new(document, ownership),
            locations: LocationResolver::new(document, ownership),
        }
    }

    pub(crate) fn table8_relation(
        &self,
        restriction: &str,
        subject: &Resource,
        object: &Resource,
    ) -> RelationOutcome {
        match restriction {
            SBOL_VERIFY_IDENTICAL | SBOL_DIFFERENT_FROM => {
                let Some(subject_key) = self.identity_key(subject) else {
                    return RelationOutcome::Unknown;
                };
                let Some(object_key) = self.identity_key(object) else {
                    return RelationOutcome::Unknown;
                };
                let same = subject_key.key == object_key.key;
                match restriction {
                    SBOL_VERIFY_IDENTICAL if same => RelationOutcome::Satisfied,
                    SBOL_VERIFY_IDENTICAL
                        if subject_key.through_reference || object_key.through_reference =>
                    {
                        RelationOutcome::Unknown
                    }
                    SBOL_VERIFY_IDENTICAL => RelationOutcome::Contradicted {
                        subject_location: None,
                        object_location: None,
                    },
                    SBOL_DIFFERENT_FROM if same => RelationOutcome::Contradicted {
                        subject_location: None,
                        object_location: None,
                    },
                    SBOL_DIFFERENT_FROM => RelationOutcome::Satisfied,
                    _ => RelationOutcome::Unsupported,
                }
            }
            SBOL_SAME_ORIENTATION_AS | SBOL_OPPOSITE_ORIENTATION_AS => {
                let Some(subject_orientation) = self.feature_orientation(subject) else {
                    return RelationOutcome::Unknown;
                };
                let Some(object_orientation) = self.feature_orientation(object) else {
                    return RelationOutcome::Unknown;
                };
                let same = subject_orientation == object_orientation;
                match (restriction, same) {
                    (SBOL_SAME_ORIENTATION_AS, true) | (SBOL_OPPOSITE_ORIENTATION_AS, false) => {
                        RelationOutcome::Satisfied
                    }
                    _ => RelationOutcome::Contradicted {
                        subject_location: None,
                        object_location: None,
                    },
                }
            }
            SBOL_REPLACES => RelationOutcome::Unknown,
            _ => RelationOutcome::Unsupported,
        }
    }

    pub(crate) fn table10_relation(
        &self,
        restriction: &str,
        subject: &Resource,
        object: &Resource,
    ) -> RelationOutcome {
        if !SEQUENTIAL_RESTRICTION_IRIS.contains(&restriction) {
            return RelationOutcome::Unsupported;
        }

        let subject_locations = self.locations.locations_for_feature(subject);
        let object_locations = self.locations.locations_for_feature(object);
        let mut comparable_pair = None;

        for subject_location in &subject_locations {
            for object_location in &object_locations {
                if subject_location.sequence != object_location.sequence {
                    continue;
                }
                if sequential_relation_satisfied(restriction, subject_location, object_location) {
                    return RelationOutcome::Satisfied;
                }
                comparable_pair.get_or_insert_with(|| {
                    (
                        subject_location.identity.clone(),
                        object_location.identity.clone(),
                    )
                });
            }
        }

        match comparable_pair {
            Some((subject_location, object_location)) => RelationOutcome::Contradicted {
                subject_location: Some(subject_location),
                object_location: Some(object_location),
            },
            None => RelationOutcome::Unknown,
        }
    }

    fn identity_key(&self, feature: &Resource) -> Option<ResolvedIdentityKey> {
        let trace = self.references.trace_feature(feature).ok()?;
        let object = self.document.get(&trace.target)?;
        let through_reference = !trace.path.is_empty();
        if object.has_class(SbolClass::SubComponent) {
            return object.first_resource(SBOL_INSTANCE_OF).cloned().map(|key| {
                ResolvedIdentityKey {
                    key: IdentityKey::SubComponentInstance(key),
                    through_reference,
                }
            });
        }
        if object.has_class(SbolClass::ExternallyDefined) {
            return object.first_resource(SBOL_DEFINITION).cloned().map(|key| {
                ResolvedIdentityKey {
                    key: IdentityKey::ExternallyDefinedDefinition(key),
                    through_reference,
                }
            });
        }
        None
    }

    fn feature_orientation(&self, feature: &Resource) -> Option<DirectionalOrientation> {
        let trace = self.references.trace_feature(feature).ok()?;
        let object = self.document.get(&trace.target)?;
        directional_orientation_value(object.first_iri(SBOL_ORIENTATION))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum IdentityKey {
    SubComponentInstance(Resource),
    ExternallyDefinedDefinition(Resource),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedIdentityKey {
    key: IdentityKey,
    through_reference: bool,
}
