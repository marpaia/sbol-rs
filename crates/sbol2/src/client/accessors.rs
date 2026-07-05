//! Trait-based ergonomic accessors for the typed SBOL 2 client surface.
//!
//! Typed structs hold shared metadata in nested [`IdentifiedData`] and
//! [`TopLevelData`] fields. These traits expose that metadata directly as
//! method calls so callers write `cd.name()` instead of
//! `cd.identified.name.as_deref()`. Bring them into scope through the prelude.

use crate::client::shared::{ExtensionTriple, IdentifiedData, TopLevelData};
use crate::Resource;

/// Read-only access to the shared metadata every SBOL 2 `Identified` object
/// carries: `persistentIdentity`, `version`, `displayId`, `name`
/// (`dcterms:title`), `description` (`dcterms:description`), the PROV
/// `wasDerivedFrom` / `wasGeneratedBy` resource sets, and round-tripped
/// non-SBOL extension triples.
pub trait SbolIdentified {
    /// Returns the underlying [`IdentifiedData`] this accessor reads from.
    fn identified_data(&self) -> &IdentifiedData;

    /// The object's `sbol:persistentIdentity`, if present.
    fn persistent_identity(&self) -> Option<&Resource> {
        self.identified_data().persistent_identity.as_ref()
    }

    /// The object's `sbol:version`, if present.
    fn version(&self) -> Option<&str> {
        self.identified_data().version.as_deref()
    }

    /// The object's `sbol:displayId`, if present.
    fn display_id(&self) -> Option<&str> {
        self.identified_data().display_id.as_deref()
    }

    /// The object's `dcterms:title`, if present.
    fn name(&self) -> Option<&str> {
        self.identified_data().name.as_deref()
    }

    /// The object's `dcterms:description`, if present.
    fn description(&self) -> Option<&str> {
        self.identified_data().description.as_deref()
    }

    /// Resources this object lists as `prov:wasDerivedFrom`.
    fn derived_from(&self) -> &[Resource] {
        &self.identified_data().derived_from
    }

    /// Resources this object lists as `prov:wasGeneratedBy`.
    fn generated_by(&self) -> &[Resource] {
        &self.identified_data().generated_by
    }

    /// Non-SBOL annotation triples preserved on this object's identity.
    fn extensions(&self) -> &[ExtensionTriple] {
        &self.identified_data().extensions
    }
}

/// Read-only access to the metadata SBOL 2 `TopLevel` objects carry on top of
/// [`SbolIdentified`]: the attached `sbol:Attachment` resources.
pub trait SbolTopLevel: SbolIdentified {
    /// Returns the underlying [`TopLevelData`] this accessor reads from.
    fn top_level_data(&self) -> &TopLevelData;

    /// `sbol:Attachment` resources attached via `sbol:attachment`.
    fn attachments(&self) -> &[Resource] {
        &self.top_level_data().attachments
    }
}

macro_rules! impl_sbol_identified {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::client::accessors::SbolIdentified for $ty {
                fn identified_data(&self) -> &$crate::client::shared::IdentifiedData {
                    &self.identified
                }
            }
        )+
    };
}

macro_rules! impl_sbol_top_level {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::client::accessors::SbolTopLevel for $ty {
                fn top_level_data(&self) -> &$crate::client::shared::TopLevelData {
                    &self.top_level
                }
            }
        )+
    };
}

pub(crate) use impl_sbol_identified;
pub(crate) use impl_sbol_top_level;
