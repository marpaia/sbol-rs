//! Rust implementation of the SBOL 2.3.0 specification.
//!
//! `sbol2` is the SBOL 2 peer of the [`sbol3`](https://docs.rs/sbol3) crate. It
//! builds on the version-neutral machinery in [`sbol_core`] — the field-metadata
//! descriptors, identity newtypes, RDF-backed document store, and validation
//! framework — and layers on the SBOL 2 data model, its RDF serialization, and
//! its validation rule catalog. Most users reach it through the umbrella `sbol`
//! crate as `sbol::v2`.
#![forbid(unsafe_code)]
