//! Owned typed SBOL client API.
//!
//! This module contains the construction and extraction surface for callers
//! that want Rust structs instead of raw RDF-backed [`crate::Object`] values.
//! `Document::from_objects` serializes owned objects into the RDF graph; the
//! parsed document exposes the typed surface through [`crate::Document`]
//! directly (`Document::typed_objects`, `Document::resolve`, and the per-class
//! iterators).

mod accessors;
mod builder;
mod document;
mod extension;
mod feature;
mod from_rdf;
mod identity;
mod location;
mod object;
mod om;
mod prov;
mod shared;
mod to_rdf;
mod top_level;
mod traits;
mod traversal;
mod usage;

pub use accessors::{SbolIdentified, SbolTopLevel};
pub use builder::{
    ActivityBuilder, AgentBuilder, AssociationBuilder, AttachmentBuilder, BinaryPrefixBuilder,
    CollectionBuilder, CombinatorialDerivationBuilder, ComponentBuilder, ComponentReferenceBuilder,
    CompoundUnitBuilder, ConstraintBuilder, CutBuilder, EntireSequenceBuilder, ExperimentBuilder,
    ExperimentalDataBuilder, ExternallyDefinedBuilder, IdentifiedExtensionBuilder,
    ImplementationBuilder, InteractionBuilder, InterfaceBuilder, LocalSubComponentBuilder,
    MeasureBuilder, ModelBuilder, ParticipationBuilder, PlanBuilder, PrefixBuilder,
    PrefixedUnitBuilder, RangeBuilder, SIPrefixBuilder, SequenceBuilder, SequenceFeatureBuilder,
    SingularUnitBuilder, SubComponentBuilder, UnitBuilder, UnitDivisionBuilder,
    UnitExponentiationBuilder, UnitMultiplicationBuilder, UsageBuilder, VariableFeatureBuilder,
};
pub use extension::IdentifiedExtension;
pub use feature::{
    ComponentReference, ExternallyDefined, LocalSubComponent, SequenceFeature, SubComponent,
};
pub use location::{Cut, EntireSequence, Range};
pub use object::SbolObject;
pub use om::{
    BinaryPrefix, CompoundUnit, Measure, Prefix, PrefixData, PrefixedUnit, SIPrefix, SingularUnit,
    Unit, UnitData, UnitDivision, UnitExponentiation, UnitMultiplication,
};
pub use prov::{Activity, Agent, Association, Plan, Usage};
pub use shared::{ExtensionTriple, FeatureData, IdentifiedData, LocationData, TopLevelData};
pub use top_level::{
    Attachment, Collection, CombinatorialDerivation, Component, Experiment, ExperimentalData,
    Implementation, Model, Sequence,
};
pub use traits::{ToRdf, TryFromObject};
pub use traversal::{FeatureRef, LocationRef};
pub use usage::{Constraint, Interaction, Interface, Participation, VariableFeature};
