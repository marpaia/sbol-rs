//! Validated identity helpers used by the typed builder surface.
//!
//! These newtypes catch invalid SBOL identifiers and namespaces at
//! construction time rather than at document-assembly time, so a
//! `ComponentDefinition::new(...)` call fails fast on inputs the spec rejects.

pub use sbol_core::identity::{
    DisplayId, HashAlgorithm, Namespace, SbolIdentity, SequenceElements,
};
