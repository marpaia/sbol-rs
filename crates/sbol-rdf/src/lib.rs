//! RDF primitives and Turtle I/O used by the public `sbol` crate.
//!
//! The types in this crate deliberately form a small boundary around the RDF
//! backend. SBOL code should depend on these owned primitives and graph traits
//! instead of on the parser implementation.

#![forbid(unsafe_code)]

mod backends;
mod error;
mod format;
mod graph;
mod terms;
mod traits;

pub use error::{IriError, ParseError, WriteError};
pub use format::RdfFormat;
pub use graph::Graph;
pub use terms::{BlankNode, Iri, Literal, Resource, Term, Triple};
pub use traits::{RdfGraph, RdfIo, TurtleGraph};
