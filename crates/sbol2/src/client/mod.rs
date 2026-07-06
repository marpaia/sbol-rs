//! Owned typed SBOL 2 client API.
//!
//! This module contains the construction and extraction surface for callers
//! that want Rust structs instead of raw RDF-backed [`crate::Object`] values.
//! `Document::from_objects` serializes owned objects into the RDF graph; the
//! parsed document exposes the typed surface through [`crate::Document`].

mod accessors;
mod annotation;
mod builder;
mod combinatorial;
mod component_instance;
mod document;
mod extension;
mod from_rdf;
mod identity;
mod interaction;
mod location;
mod module;
mod object;
mod om;
mod prov;
mod shared;
mod to_rdf;
mod top_level;
mod traits;

pub use accessors::{SbolIdentified, SbolTopLevel};
pub use annotation::{SequenceAnnotation, SequenceConstraint};
pub use builder::{
    ActivityBuilder, AgentBuilder, AssociationBuilder, AttachmentBuilder, BinaryPrefixBuilder,
    CollectionBuilder, CombinatorialDerivationBuilder, ComponentBuilder,
    ComponentDefinitionBuilder, CompoundUnitBuilder, CutBuilder, ExperimentBuilder,
    ExperimentalDataBuilder, FunctionalComponentBuilder, GenericLocationBuilder,
    GenericTopLevelBuilder, IdentifiedExtensionBuilder, ImplementationBuilder, InteractionBuilder,
    MapsToBuilder, MeasureBuilder, ModelBuilder, ModuleBuilder, ModuleDefinitionBuilder,
    ParticipationBuilder, PlanBuilder, PrefixBuilder, PrefixedUnitBuilder, RangeBuilder,
    SIPrefixBuilder, SequenceAnnotationBuilder, SequenceBuilder, SequenceConstraintBuilder,
    SingularUnitBuilder, UnitBuilder, UnitDivisionBuilder, UnitExponentiationBuilder,
    UnitMultiplicationBuilder, UsageBuilder, VariableComponentBuilder,
};
pub use combinatorial::VariableComponent;
pub use component_instance::{Component, FunctionalComponent};
pub use extension::{GenericTopLevel, IdentifiedExtension};
pub use interaction::{Interaction, Participation};
pub use location::{Cut, GenericLocation, Range};
pub use module::{MapsTo, Module};
pub use object::Sbol2Object;
pub use om::{
    BinaryPrefix, CompoundUnit, Measure, Prefix, PrefixData, PrefixedUnit, SIPrefix, SingularUnit,
    Unit, UnitData, UnitDivision, UnitExponentiation, UnitMultiplication,
};
pub use prov::{Activity, Agent, Association, Plan, Usage};
pub use shared::{
    ComponentInstanceData, ExtensionTriple, IdentifiedData, LocationData, MeasuredData,
    TopLevelData,
};
pub use top_level::{
    Attachment, Collection, CombinatorialDerivation, ComponentDefinition, Experiment,
    ExperimentalData, Implementation, Model, ModuleDefinition, Sequence,
};
pub use traits::{ToRdf, TryFromObject};
