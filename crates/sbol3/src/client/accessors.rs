//! Trait-based ergonomic accessors for the typed SBOL client surface.
//!
//! Typed structs hold shared metadata in nested [`IdentifiedData`] and
//! [`TopLevelData`] fields. These traits expose that metadata directly as
//! method calls so callers can write `component.name()` instead of
//! `component.identified.name.as_deref()`.
//!
//! Bring them into scope through the prelude (`use sbol3::prelude::*;`
//! re-exports both traits), and the accessors light up on every typed
//! SBOL object.
//!
//! ```
//! use sbol3::constants::{SBO_DNA, SO_PROMOTER};
//! use sbol3::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let component = Component::builder("https://example.org/lab", "j23119")?
//!     .types([SBO_DNA])
//!     .add_component_role(SO_PROMOTER)
//!     .name("J23119 promoter")
//!     .description("Strong constitutive promoter")
//!     .build()?;
//!
//! assert_eq!(component.display_id(), Some("j23119"));
//! assert_eq!(component.name(), Some("J23119 promoter"));
//! assert_eq!(component.description(), Some("Strong constitutive promoter"));
//! assert_eq!(
//!     component.namespace().map(|iri| iri.as_str()),
//!     Some("https://example.org/lab"),
//! );
//! # Ok(())
//! # }
//! ```

use crate::client::shared::{ExtensionTriple, IdentifiedData, TopLevelData};
use crate::{Iri, Resource};

/// Read-only access to the shared metadata every SBOL `Identified` object
/// carries: `displayId`, `name`, `description`, the PROV `wasDerivedFrom` /
/// `wasGeneratedBy` resource sets, OM measures, and round-tripped
/// non-SBOL extension triples.
pub trait SbolIdentified {
    /// Returns the underlying [`IdentifiedData`] this accessor reads from.
    /// Implementors only need to forward to their `identified` field.
    fn identified_data(&self) -> &IdentifiedData;

    /// The object's `sbol:displayId`, if present.
    fn display_id(&self) -> Option<&str> {
        self.identified_data().display_id.as_deref()
    }

    /// The object's `sbol:name`, if present.
    fn name(&self) -> Option<&str> {
        self.identified_data().name.as_deref()
    }

    /// The object's `sbol:description`, if present.
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

    /// OM `Measure` resources attached via `sbol:hasMeasure`.
    fn measures(&self) -> &[Resource] {
        &self.identified_data().measures
    }

    /// Non-SBOL annotation triples preserved on this object's identity.
    fn extensions(&self) -> &[ExtensionTriple] {
        &self.identified_data().extensions
    }
}

/// Read-only access to the metadata SBOL `TopLevel` objects carry on top
/// of [`SbolIdentified`]: the document namespace and any attached
/// `sbol:Attachment` resources.
pub trait SbolTopLevel: SbolIdentified {
    /// Returns the underlying [`TopLevelData`] this accessor reads from.
    /// Implementors only need to forward to their `top_level` field.
    fn top_level_data(&self) -> &TopLevelData;

    /// The object's `sbol:hasNamespace`, if present. TopLevel objects in
    /// well-formed SBOL documents always carry a namespace, but the
    /// underlying graph may omit it. Callers that require a namespace
    /// should validate the document first.
    fn namespace(&self) -> Option<&Iri> {
        self.top_level_data().namespace.as_ref()
    }

    /// `sbol:Attachment` resources attached via `sbol:hasAttachment`.
    fn attachments(&self) -> &[Resource] {
        &self.top_level_data().attachments
    }
}

/// Generates the canonical [`SbolIdentified`] impl for a struct with an
/// `identified: IdentifiedData` field.
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

/// Generates the canonical [`SbolTopLevel`] impl for a struct with a
/// `top_level: TopLevelData` field. The struct must also implement
/// [`SbolIdentified`].
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
