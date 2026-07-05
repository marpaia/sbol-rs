//! Version-neutral SBOL machinery shared by the `sbol2` and `sbol3` crates.
//!
//! `sbol-core` holds the parts of an SBOL implementation that do not depend on
//! a specific data-model version: the field-metadata descriptor types that
//! drive schema-directed serialization and validation, the validation
//! framework (configuration, severity, coverage, and reporting), identity and
//! IRI utilities, and the object and document scaffolding that a versioned
//! model builds on. The `sbol2` and `sbol3` crates supply their own class and
//! rule catalogs on top of these shared primitives.

pub mod error;
pub mod iri;
pub mod object;
pub mod prelude;
pub mod schema;
pub mod syntax;
pub mod validation;
