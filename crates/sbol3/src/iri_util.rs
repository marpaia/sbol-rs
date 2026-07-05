//! IRI string helpers shared by the upgrade and downgrade engines.
//!
//! The implementation lives in [`sbol_core::iri`]; this module re-exports it so
//! existing `crate::iri_util::…` call sites are unchanged.

pub(crate) use sbol_core::iri::last_iri_segment;
