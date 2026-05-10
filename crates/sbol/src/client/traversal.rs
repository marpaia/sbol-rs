//! Typed parent → child traversal helpers.
//!
//! Component, Interaction, and SequenceFeature store child identities as
//! `Vec<Resource>`. These helpers resolve those identities against a
//! [`Document`] and return iterators over the concrete typed children,
//! so callers do not need to manually match on `SbolObject`.
//!
//! Unresolved references (a `Resource` pointing outside the document) are
//! silently skipped. To detect broken references, run `Document::validate*`.

use crate::client::{
    Component, Constraint, Interaction, Interface, Participation, Range, SbolObject,
    SequenceFeature, SubComponent,
};
use crate::{Document, Resource};

/// A child feature resolved from a Component's `hasFeature` list.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum FeatureRef<'a> {
    SubComponent(&'a SubComponent),
    SequenceFeature(&'a SequenceFeature),
    LocalSubComponent(&'a crate::LocalSubComponent),
    ComponentReference(&'a crate::ComponentReference),
    ExternallyDefined(&'a crate::ExternallyDefined),
}

impl<'a> FeatureRef<'a> {
    pub(crate) fn from_object(object: &'a SbolObject) -> Option<Self> {
        Some(match object {
            SbolObject::SubComponent(o) => FeatureRef::SubComponent(o),
            SbolObject::SequenceFeature(o) => FeatureRef::SequenceFeature(o),
            SbolObject::LocalSubComponent(o) => FeatureRef::LocalSubComponent(o),
            SbolObject::ComponentReference(o) => FeatureRef::ComponentReference(o),
            SbolObject::ExternallyDefined(o) => FeatureRef::ExternallyDefined(o),
            _ => return None,
        })
    }

    pub fn identity(&self) -> &'a Resource {
        match self {
            FeatureRef::SubComponent(o) => &o.identity,
            FeatureRef::SequenceFeature(o) => &o.identity,
            FeatureRef::LocalSubComponent(o) => &o.identity,
            FeatureRef::ComponentReference(o) => &o.identity,
            FeatureRef::ExternallyDefined(o) => &o.identity,
        }
    }
}

/// A child location resolved from a SequenceFeature's `hasLocation` list.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum LocationRef<'a> {
    Range(&'a Range),
    Cut(&'a crate::Cut),
    EntireSequence(&'a crate::EntireSequence),
}

impl<'a> LocationRef<'a> {
    fn from_object(object: &'a SbolObject) -> Option<Self> {
        Some(match object {
            SbolObject::Range(o) => LocationRef::Range(o),
            SbolObject::Cut(o) => LocationRef::Cut(o),
            SbolObject::EntireSequence(o) => LocationRef::EntireSequence(o),
            _ => return None,
        })
    }
}

impl Component {
    /// Iterate over this Component's child features, resolved against `doc`.
    pub fn features<'doc>(
        &'doc self,
        doc: &'doc Document,
    ) -> impl Iterator<Item = FeatureRef<'doc>> + 'doc {
        self.features
            .iter()
            .filter_map(|identity| doc.resolve(identity))
            .filter_map(FeatureRef::from_object)
    }

    /// Iterate over this Component's child Constraints.
    pub fn constraints<'doc>(
        &'doc self,
        doc: &'doc Document,
    ) -> impl Iterator<Item = &'doc Constraint> + 'doc {
        self.constraints
            .iter()
            .filter_map(|identity| match doc.resolve(identity)? {
                SbolObject::Constraint(c) => Some(c),
                _ => None,
            })
    }

    /// Iterate over this Component's child Interactions.
    pub fn interactions<'doc>(
        &'doc self,
        doc: &'doc Document,
    ) -> impl Iterator<Item = &'doc Interaction> + 'doc {
        self.interactions
            .iter()
            .filter_map(|identity| match doc.resolve(identity)? {
                SbolObject::Interaction(i) => Some(i),
                _ => None,
            })
    }

    /// Iterate over this Component's child Interfaces.
    pub fn interfaces<'doc>(
        &'doc self,
        doc: &'doc Document,
    ) -> impl Iterator<Item = &'doc Interface> + 'doc {
        self.interfaces
            .iter()
            .filter_map(|identity| match doc.resolve(identity)? {
                SbolObject::Interface(i) => Some(i),
                _ => None,
            })
    }
}

impl Interaction {
    /// Iterate over this Interaction's child Participations.
    pub fn participations<'doc>(
        &'doc self,
        doc: &'doc Document,
    ) -> impl Iterator<Item = &'doc Participation> + 'doc {
        self.participations
            .iter()
            .filter_map(|identity| match doc.resolve(identity)? {
                SbolObject::Participation(p) => Some(p),
                _ => None,
            })
    }
}

impl SequenceFeature {
    /// Iterate over this SequenceFeature's child Locations.
    pub fn locations<'doc>(
        &'doc self,
        doc: &'doc Document,
    ) -> impl Iterator<Item = LocationRef<'doc>> + 'doc {
        self.locations
            .iter()
            .filter_map(|identity| doc.resolve(identity))
            .filter_map(LocationRef::from_object)
    }
}

impl SubComponent {
    /// Iterate over this SubComponent's child Locations.
    pub fn locations<'doc>(
        &'doc self,
        doc: &'doc Document,
    ) -> impl Iterator<Item = LocationRef<'doc>> + 'doc {
        self.locations
            .iter()
            .filter_map(|identity| doc.resolve(identity))
            .filter_map(LocationRef::from_object)
    }
}
