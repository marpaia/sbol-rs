//! Owned typed SBOL 2 client API.
//!
//! This module contains the construction and extraction surface for callers
//! that want Rust structs instead of raw RDF-backed [`crate::Object`] values.
//! `Document::from_objects` serializes owned objects into the RDF graph; the
//! parsed document exposes the typed surface through [`crate::Document`].

mod accessors;
mod shared;
mod to_rdf;
mod traits;

pub use accessors::{SbolIdentified, SbolTopLevel};
pub use shared::{
    ComponentInstanceData, ExtensionTriple, IdentifiedData, LocationData, MeasuredData,
    TopLevelData,
};
pub use traits::{ToRdf, TryFromObject};
