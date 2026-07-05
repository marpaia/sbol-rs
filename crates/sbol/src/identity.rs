//! Validated identity helpers used by the typed builder surface.
//!
//! These newtypes catch invalid SBOL identifiers and namespaces at construction
//! time rather than at `Document::validate*` time, so a `Component::new(...)`
//! call fails fast on inputs the validator would otherwise reject.

pub use sbol_core::identity::{DisplayId, HashAlgorithm, Namespace, SbolIdentity, SequenceElements};
