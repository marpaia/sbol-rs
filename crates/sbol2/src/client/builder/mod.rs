//! Builder types for owned SBOL 2 classes.
//!
//! Every owned class has a `Class::builder(namespace_or_parent, display_id)`
//! constructor returning a `ClassBuilder` with chainable setters for every
//! property. `build()` returns `Err(BuildError::MissingRequired)` when a
//! cardinality-required field was never set. Setters consume and return `Self`
//! for fluent construction; collection setters come in a `field(values)`
//! (replace) and `add_field(value)` (append) pair.
//!
//! SBOL 2 bakes the version into the compliant identity, so every builder
//! carries a version (default `"1"`); the `version` and `persistent_identity`
//! setters recompute the object identity.

mod feature;
mod om;
mod prov;
mod top_level;

pub use feature::*;
pub use om::*;
pub use prov::*;
pub use top_level::*;

use crate::client::identity::{DEFAULT_VERSION, build_child_identity};
use crate::client::{IdentifiedData, MeasuredData, PrefixData, TopLevelData, UnitData};
use crate::error::BuildError;
use crate::identity::DisplayId;
use crate::{Resource, Sbol2Class};

// ---------------------------------------------------------------------------
// Shared setter macros
// ---------------------------------------------------------------------------

macro_rules! identified_setters {
    () => {
        pub fn name(mut self, value: impl Into<String>) -> Self {
            self.identified.name = Some(value.into());
            self
        }

        pub fn description(mut self, value: impl Into<String>) -> Self {
            self.identified.description = Some(value.into());
            self
        }

        pub fn version(mut self, value: impl Into<String>) -> Self {
            let version = value.into();
            if let Some(persistent) = &self.identified.persistent_identity {
                self.identity = $crate::client::identity::identity_from(persistent, &version);
            }
            self.identified.version = Some(version);
            self
        }

        pub fn persistent_identity(mut self, value: Resource) -> Self {
            let version = self
                .identified
                .version
                .clone()
                .unwrap_or_else(|| $crate::client::identity::DEFAULT_VERSION.to_string());
            self.identity = $crate::client::identity::identity_from(&value, &version);
            self.identified.persistent_identity = Some(value);
            self
        }

        pub fn derived_from(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.identified.derived_from = values.into_iter().collect();
            self
        }

        pub fn add_derived_from(mut self, value: Resource) -> Self {
            self.identified.derived_from.push(value);
            self
        }

        pub fn generated_by(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.identified.generated_by = values.into_iter().collect();
            self
        }

        pub fn add_generated_by(mut self, value: Resource) -> Self {
            self.identified.generated_by.push(value);
            self
        }

        /// Attach a non-SBOL annotation triple. The predicate must be outside
        /// the SBOL 2, PROV, and OM vocabularies and the recognized
        /// `dcterms`/`rdfs` IRIs.
        pub fn extension(mut self, predicate: Iri, value: Term) -> Self {
            self.identified
                .extensions
                .push($crate::client::ExtensionTriple { predicate, object: value });
            self
        }
    };
}

macro_rules! top_level_setters {
    () => {
        pub fn attachments(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.top_level.attachments = values.into_iter().collect();
            self
        }

        pub fn add_attachment(mut self, value: Resource) -> Self {
            self.top_level.attachments.push(value);
            self
        }
    };
}

macro_rules! measured_setters {
    () => {
        pub fn measures(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.measured.measures = values.into_iter().collect();
            self
        }

        pub fn add_measure(mut self, value: Resource) -> Self {
            self.measured.measures.push(value);
            self
        }
    };
}

macro_rules! component_instance_setters {
    () => {
        pub fn definition(mut self, value: Resource) -> Self {
            self.component_instance.definition = Some(value);
            self
        }

        pub fn access(mut self, value: Iri) -> Self {
            self.component_instance.access = Some(value);
            self
        }

        pub fn maps_tos(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.component_instance.maps_tos = values.into_iter().collect();
            self
        }

        pub fn add_maps_to(mut self, value: Resource) -> Self {
            self.component_instance.maps_tos.push(value);
            self
        }

        pub fn measures(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.component_instance.measured.measures = values.into_iter().collect();
            self
        }

        pub fn add_measure(mut self, value: Resource) -> Self {
            self.component_instance.measured.measures.push(value);
            self
        }
    };
}

macro_rules! location_setters {
    () => {
        pub fn orientation(mut self, value: Iri) -> Self {
            self.location.orientation = Some(value);
            self
        }

        pub fn sequence(mut self, value: Resource) -> Self {
            self.location.sequence = Some(value);
            self
        }
    };
}

macro_rules! unit_setters {
    () => {
        pub fn symbol(mut self, value: impl Into<String>) -> Self {
            self.unit.symbol = Some(value.into());
            self
        }

        pub fn label(mut self, value: impl Into<String>) -> Self {
            self.unit.label = Some(value.into());
            self
        }

        pub fn alternative_symbols(mut self, values: impl IntoIterator<Item = String>) -> Self {
            self.unit.alternative_symbols = values.into_iter().collect();
            self
        }

        pub fn alternative_labels(mut self, values: impl IntoIterator<Item = String>) -> Self {
            self.unit.alternative_labels = values.into_iter().collect();
            self
        }

        pub fn comment(mut self, value: impl Into<String>) -> Self {
            self.unit.comment = Some(value.into());
            self
        }

        pub fn long_comment(mut self, value: impl Into<String>) -> Self {
            self.unit.long_comment = Some(value.into());
            self
        }
    };
}

macro_rules! prefix_setters {
    () => {
        pub fn symbol(mut self, value: impl Into<String>) -> Self {
            self.prefix.symbol = Some(value.into());
            self
        }

        pub fn label(mut self, value: impl Into<String>) -> Self {
            self.prefix.label = Some(value.into());
            self
        }

        pub fn alternative_symbols(mut self, values: impl IntoIterator<Item = String>) -> Self {
            self.prefix.alternative_symbols = values.into_iter().collect();
            self
        }

        pub fn alternative_labels(mut self, values: impl IntoIterator<Item = String>) -> Self {
            self.prefix.alternative_labels = values.into_iter().collect();
            self
        }

        pub fn comment(mut self, value: impl Into<String>) -> Self {
            self.prefix.comment = Some(value.into());
            self
        }

        pub fn long_comment(mut self, value: impl Into<String>) -> Self {
            self.prefix.long_comment = Some(value.into());
            self
        }

        pub fn has_factor(mut self, value: f64) -> Self {
            self.prefix.has_factor = Some(value.to_string());
            self
        }
    };
}

pub(crate) use component_instance_setters;
pub(crate) use identified_setters;
pub(crate) use location_setters;
pub(crate) use measured_setters;
pub(crate) use prefix_setters;
pub(crate) use top_level_setters;
pub(crate) use unit_setters;

// ---------------------------------------------------------------------------
// Shared seeds
// ---------------------------------------------------------------------------

fn missing(identity: &Resource, class: Sbol2Class, property: &'static str) -> BuildError {
    BuildError::MissingRequired {
        identity: identity.clone(),
        class,
        property,
    }
}

fn identified_seed(display_id: &DisplayId, persistent: Resource) -> IdentifiedData {
    IdentifiedData {
        persistent_identity: Some(persistent),
        version: Some(DEFAULT_VERSION.to_string()),
        display_id: Some(display_id.as_str().to_string()),
        ..IdentifiedData::default()
    }
}

/// Seeds a child object's `(identity, IdentifiedData)` from the parent's
/// `persistentIdentity`.
fn child_seed(
    parent_persistent: &Resource,
    display_id: DisplayId,
) -> Result<(Resource, IdentifiedData), BuildError> {
    let (identity, persistent) =
        build_child_identity(parent_persistent, &display_id, DEFAULT_VERSION)?;
    Ok((identity, identified_seed(&display_id, persistent)))
}

fn top_level_seed() -> TopLevelData {
    TopLevelData::default()
}

fn measured_seed() -> MeasuredData {
    MeasuredData::default()
}

fn unit_seed() -> UnitData {
    UnitData::default()
}

fn prefix_seed() -> PrefixData {
    PrefixData::default()
}
