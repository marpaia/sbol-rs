//! Builder types for owned SBOL classes.
//!
//! Every owned class has a `Class::builder(namespace_or_parent, display_id)`
//! constructor that returns a `ClassBuilder` with chainable setters for every
//! property. Required fields are tracked separately from the underlying `Vec`
//! storage; `build()` returns `Err(BuildError::MissingRequired)` if any
//! cardinality-required field was never set.
//!
//! Setters consume and return `Self` for fluent single-expression construction.
//! Collection setters come in two shapes:
//!
//! - `field(values)` replaces the entire collection.
//! - `add_field(value)` appends a single value.
//!
//! Inherited `Identified` and `TopLevel` properties (`name`, `description`,
//! `derived_from`, `generated_by`, `measures`, `attachments`) are flat methods
//! on each builder, with no nested `.identified().name(...)` paths.
//!
//! The builders are grouped by domain: [`top_level`] (the TopLevel classes),
//! [`feature`] (Component children: features, locations, and the
//! constraint/interaction family), [`prov`] (PROV-O), and [`om`] (units and
//! measures). The shared setter macros and identity seeds below back every
//! group.

mod feature;
mod om;
mod prov;
mod top_level;

pub use feature::*;
pub use om::*;
pub use prov::*;
pub use top_level::*;

use crate::client::identity::build_child_identity;
use crate::client::{IdentifiedData, PrefixData, TopLevelData, UnitData};
use crate::error::BuildError;
use crate::identity::{DisplayId, Namespace};
use crate::{Resource, SbolClass};

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

        pub fn measures(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.identified.measures = values.into_iter().collect();
            self
        }

        pub fn add_measure(mut self, value: Resource) -> Self {
            self.identified.measures.push(value);
            self
        }

        /// Attach a non-SBOL annotation triple. The predicate must be outside
        /// the SBOL, PROV, and OM vocabularies; predicates inside those
        /// vocabularies belong on dedicated setters and are emitted twice if
        /// pushed here.
        pub fn extension(mut self, predicate: Iri, value: Term) -> Self {
            self.identified.extensions.push(ExtensionTriple {
                predicate,
                object: value,
            });
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

macro_rules! feature_setters {
    () => {
        pub fn roles(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
            self.feature.roles = values.into_iter().collect();
            self
        }

        pub fn add_role(mut self, value: Iri) -> Self {
            self.feature.roles.push(value);
            self
        }

        pub fn orientation(mut self, value: Iri) -> Self {
            self.feature.orientation = Some(value);
            self
        }
    };
}

macro_rules! location_setters {
    () => {
        pub fn sequence(mut self, value: Resource) -> Self {
            self.location.sequence = Some(value);
            self
        }

        pub fn orientation(mut self, value: Iri) -> Self {
            self.location.orientation = Some(value);
            self
        }

        pub fn order(mut self, value: i64) -> Self {
            self.location.order = Some(value);
            self
        }
    };
}

macro_rules! unit_setters {
    () => {
        pub fn label(mut self, value: impl Into<String>) -> Self {
            self.unit.label = Some(value.into());
            self
        }

        pub fn symbol(mut self, value: impl Into<String>) -> Self {
            self.unit.symbol = Some(value.into());
            self
        }

        pub fn alternative_labels(mut self, values: impl IntoIterator<Item = String>) -> Self {
            self.unit.alternative_labels = values.into_iter().collect();
            self
        }

        pub fn add_alternative_label(mut self, value: impl Into<String>) -> Self {
            self.unit.alternative_labels.push(value.into());
            self
        }

        pub fn alternative_symbols(mut self, values: impl IntoIterator<Item = String>) -> Self {
            self.unit.alternative_symbols = values.into_iter().collect();
            self
        }

        pub fn add_alternative_symbol(mut self, value: impl Into<String>) -> Self {
            self.unit.alternative_symbols.push(value.into());
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
        pub fn label(mut self, value: impl Into<String>) -> Self {
            self.prefix.label = Some(value.into());
            self
        }

        pub fn symbol(mut self, value: impl Into<String>) -> Self {
            self.prefix.symbol = Some(value.into());
            self
        }

        pub fn has_factor(mut self, value: f64) -> Self {
            self.prefix.has_factor = Some(value.to_string());
            self
        }

        pub fn alternative_labels(mut self, values: impl IntoIterator<Item = String>) -> Self {
            self.prefix.alternative_labels = values.into_iter().collect();
            self
        }

        pub fn add_alternative_label(mut self, value: impl Into<String>) -> Self {
            self.prefix.alternative_labels.push(value.into());
            self
        }

        pub fn alternative_symbols(mut self, values: impl IntoIterator<Item = String>) -> Self {
            self.prefix.alternative_symbols = values.into_iter().collect();
            self
        }

        pub fn add_alternative_symbol(mut self, value: impl Into<String>) -> Self {
            self.prefix.alternative_symbols.push(value.into());
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
    };
}

pub(crate) use feature_setters;
pub(crate) use identified_setters;
pub(crate) use location_setters;
pub(crate) use prefix_setters;
pub(crate) use top_level_setters;
pub(crate) use unit_setters;

// ---------------------------------------------------------------------------
// Shared identity seeds
// ---------------------------------------------------------------------------

fn missing(identity: &Resource, class: SbolClass, property: &'static str) -> BuildError {
    BuildError::MissingRequired {
        identity: identity.clone(),
        class,
        property,
    }
}

fn identified_seed(display_id: &DisplayId) -> IdentifiedData {
    IdentifiedData {
        display_id: Some(display_id.as_str().to_string()),
        ..IdentifiedData::default()
    }
}

fn top_level_seed(namespace: &Namespace) -> TopLevelData {
    TopLevelData {
        namespace: Some(namespace.as_iri().clone()),
        ..TopLevelData::default()
    }
}

fn child_seed(
    parent: &Resource,
    display_id: DisplayId,
) -> Result<(Resource, IdentifiedData), BuildError> {
    let identity = build_child_identity(parent, &display_id)?;
    Ok((identity, identified_seed(&display_id)))
}

fn unit_seed(_namespace: &Namespace, _display_id: &DisplayId) -> UnitData {
    UnitData::default()
}

fn prefix_seed(_namespace: &Namespace, _display_id: &DisplayId) -> PrefixData {
    PrefixData::default()
}
