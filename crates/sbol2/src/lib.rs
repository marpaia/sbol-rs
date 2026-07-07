//! Rust implementation of the SBOL 2.3.0 specification.
//!
//! `sbol2` is the SBOL 2 peer of the [`sbol3`](https://docs.rs/sbol3) crate. It
//! builds on the version-neutral machinery in [`sbol_core`] (the field-metadata
//! descriptors, identity newtypes, RDF-backed document store) and layers on the
//! SBOL 2 typed data model, its RDF serialization, and a shared field-metadata
//! catalog. Most users reach it through the umbrella `sbol` crate as `sbol::v2`.
#![forbid(unsafe_code)]
#![allow(clippy::result_large_err)]

mod client;
mod conformance;
pub mod constants;
mod document;
mod error;
pub mod identity;
mod model;
mod object;
pub mod prelude;
pub mod schema;
pub mod validation;
#[doc(hidden)]
pub mod vocab;

pub use client::{
    Activity, ActivityBuilder, Agent, AgentBuilder, Association, AssociationBuilder, Attachment,
    AttachmentBuilder, BinaryPrefix, BinaryPrefixBuilder, Collection, CollectionBuilder,
    CombinatorialDerivation, CombinatorialDerivationBuilder, Component, ComponentBuilder,
    ComponentDefinition, ComponentDefinitionBuilder, ComponentInstanceData, CompoundUnit,
    CompoundUnitBuilder, Cut, CutBuilder, Experiment, ExperimentBuilder, ExperimentalData,
    ExperimentalDataBuilder, ExtensionTriple, FunctionalComponent, FunctionalComponentBuilder,
    GenericLocation, GenericLocationBuilder, GenericTopLevel, GenericTopLevelBuilder,
    IdentifiedData, IdentifiedExtension, IdentifiedExtensionBuilder, Implementation,
    ImplementationBuilder, Interaction, InteractionBuilder, LocationData, MapsTo, MapsToBuilder,
    Measure, MeasureBuilder, MeasuredData, Model, ModelBuilder, Module, ModuleBuilder,
    ModuleDefinition, ModuleDefinitionBuilder, Participation, ParticipationBuilder, Plan,
    PlanBuilder, Prefix, PrefixBuilder, PrefixData, PrefixedUnit, PrefixedUnitBuilder, Range,
    RangeBuilder, SIPrefix, SIPrefixBuilder, Sbol2Object, SbolIdentified, SbolTopLevel, Sequence,
    SequenceAnnotation, SequenceAnnotationBuilder, SequenceBuilder, SequenceConstraint,
    SequenceConstraintBuilder, SingularUnit, SingularUnitBuilder, ToRdf, TopLevelData,
    TryFromObject, Unit, UnitBuilder, UnitData, UnitDivision, UnitDivisionBuilder,
    UnitExponentiation, UnitExponentiationBuilder, UnitMultiplication, UnitMultiplicationBuilder,
    Usage, UsageBuilder, VariableComponent, VariableComponentBuilder,
};
pub use conformance::render_sbol2_conformance_report;
pub use document::Document;
pub use error::{BuildError, ReadError, WriteError};
pub use identity::{DisplayId, HashAlgorithm, Namespace, SbolIdentity, SequenceElements};
pub use model::{Identified, Sbol2Class, TopLevel};
pub use object::ObjectClasses;
pub use sbol_core::diff::{Diff, ObjectDiff, PropertyChange};
pub use sbol_core::document::{ObjectStore, RawDocument};
pub use sbol_core::object::Object;
pub use sbol_rdf::{Graph as RdfGraph, Iri, Literal, RdfFormat, Resource, Term, Triple};
