//! Rust implementation of the SBOL 2.3.0 specification.
//!
//! `sbol2` is the SBOL 2 peer of the [`sbol3`](https://docs.rs/sbol3) crate. It
//! builds on the version-neutral machinery in [`sbol_core`] — the field-metadata
//! descriptors, identity newtypes, RDF-backed document store — and layers on the
//! SBOL 2 typed data model, its RDF serialization, and a shared field-metadata
//! catalog. Most users reach it through the umbrella `sbol` crate as `sbol::v2`.
#![forbid(unsafe_code)]
#![allow(clippy::result_large_err)]

mod client;
pub mod constants;
mod document;
mod error;
pub mod identity;
mod model;
mod object;
pub mod schema;
#[doc(hidden)]
pub mod vocab;

pub use client::{
    Activity, Agent, Association, Attachment, BinaryPrefix, Collection, CombinatorialDerivation,
    Component, ComponentDefinition, ComponentInstanceData, CompoundUnit, Cut, Experiment,
    ExperimentalData, ExtensionTriple, FunctionalComponent, GenericLocation, GenericTopLevel,
    IdentifiedData, IdentifiedExtension, Implementation, Interaction, LocationData, MapsTo,
    Measure, MeasuredData, Model, Module, ModuleDefinition, Participation, Plan, Prefix,
    PrefixData, PrefixedUnit, Range, SIPrefix, Sbol2Object, SbolIdentified, SbolTopLevel, Sequence,
    SequenceAnnotation, SequenceConstraint, SingularUnit, ToRdf, TopLevelData, TryFromObject, Unit,
    UnitData, UnitDivision, UnitExponentiation, UnitMultiplication, Usage, VariableComponent,
};
pub use document::Document;
pub use error::{BuildError, ReadError, WriteError};
pub use identity::{DisplayId, HashAlgorithm, Namespace, SbolIdentity, SequenceElements};
pub use model::{Identified, Sbol2Class, TopLevel};
pub use object::ObjectClasses;
pub use sbol_core::document::{ObjectStore, RawDocument};
pub use sbol_core::object::Object;
pub use sbol_rdf::{Graph as RdfGraph, Iri, Literal, RdfFormat, Resource, Term, Triple};
