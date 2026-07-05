//! Convenient re-exports for typical SBOL construction and inspection workflows.
//!
//! ```
//! use sbol::prelude::*;
//! use sbol::constants::SBO_DNA;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let component = Component::builder("https://example.org/lab", "my_component")?
//!     .types([SBO_DNA])
//!     .name("My component")
//!     .build()?;
//! let document = Document::from_objects(vec![SbolObject::Component(component)])?;
//! assert!(document.validate().is_valid());
//! # Ok(())
//! # }
//! ```
//!
//! The prelude excludes the schema descriptors (in [`crate::schema`]) and the
//! validation rule metadata (in [`crate::validation_rule_statuses`]); import
//! those explicitly when you need them.

pub use crate::client::{
    Activity, ActivityBuilder, Agent, AgentBuilder, Association, AssociationBuilder, Attachment,
    AttachmentBuilder, BinaryPrefix, BinaryPrefixBuilder, Collection, CollectionBuilder,
    CombinatorialDerivation, CombinatorialDerivationBuilder, Component, ComponentBuilder,
    ComponentReference, ComponentReferenceBuilder, CompoundUnit, CompoundUnitBuilder, Constraint,
    ConstraintBuilder, Cut, CutBuilder, EntireSequence, EntireSequenceBuilder, Experiment,
    ExperimentBuilder, ExperimentalData, ExperimentalDataBuilder, ExtensionTriple,
    ExternallyDefined, ExternallyDefinedBuilder, FeatureData, FeatureRef, IdentifiedData,
    IdentifiedExtension, IdentifiedExtensionBuilder, Implementation, ImplementationBuilder,
    Interaction, InteractionBuilder, Interface, InterfaceBuilder, LocalSubComponent,
    LocalSubComponentBuilder, LocationData, LocationRef, Measure, MeasureBuilder, Model,
    ModelBuilder, Participation, ParticipationBuilder, Plan, PlanBuilder, Prefix, PrefixBuilder,
    PrefixData, PrefixedUnit, PrefixedUnitBuilder, Range, RangeBuilder, SIPrefix, SIPrefixBuilder,
    SbolIdentified, SbolObject, SbolTopLevel, Sequence, SequenceBuilder, SequenceFeature,
    SequenceFeatureBuilder, SingularUnit, SingularUnitBuilder, SubComponent, SubComponentBuilder,
    ToRdf, TopLevelData, TryFromObject, Unit, UnitBuilder, UnitData, UnitDivision,
    UnitDivisionBuilder, UnitExponentiation, UnitExponentiationBuilder, UnitMultiplication,
    UnitMultiplicationBuilder, Usage, UsageBuilder, VariableFeature, VariableFeatureBuilder,
};
pub use crate::document::Document;
pub use crate::error::{BuildError, ReadError, WriteError};
pub use crate::identity::{DisplayId, HashAlgorithm, Namespace, SbolIdentity, SequenceElements};
pub use crate::model::{Identified, SbolClass, TopLevel};
pub use crate::object::ObjectClasses;
pub use crate::resolve::{FeatureTrace, ObjectGraph, ReferenceError, VariantSet};
pub use crate::validation::DocumentSet;
pub use crate::{Iri, Literal, Object, RdfFormat, RdfGraph, Resource, Term, Triple};
