//! Public schema descriptors for SBOL classes and properties.
//!
//! These types describe the SBOL data model in terms of fields:
//!
//! - [`FieldDescriptor`] describes one SBOL property: its predicate IRI,
//!   cardinality, value kind, optional reference target, and the SBOL
//!   validation rule that polices its cardinality and value-kind checks.
//! - [`Cardinality`], [`ValueKind`], [`TargetClass`], and [`ReferenceSpec`]
//!   are supporting metadata.
//! - [`ClassDescriptor`] groups parent classes and the field descriptors
//!   that belong to an SBOL class.
//!
//! Validation reads these descriptors to drive table-driven rule checks.
//! RDF serialization, deserialization, and (future) builder APIs share the
//! same source of truth so the schema cannot drift between subsystems.
//!
//! See [`class_descriptor`] for entry-point lookup.

use crate::SbolClass;

pub use sbol_core::schema::{Cardinality, FieldDescriptor, ReferenceSpec, TargetClass, ValueKind};

/// Describes one SBOL class.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct ClassDescriptor {
    /// IRIs of direct parent classes in the SBOL hierarchy.
    pub parents: &'static [&'static str],
    /// Fields declared on this class.
    pub fields: &'static [FieldDescriptor],
}

/// Looks up the [`ClassDescriptor`] for an SBOL class by its RDF type
/// IRI. Returns `None` if the IRI is not a known SBOL, PROV, or OM class.
pub fn class_descriptor(class_iri: &str) -> Option<ClassDescriptor> {
    crate::validation::class_spec(class_iri)
}

/// Returns the field descriptors declared on the given SBOL class.
/// Returns an empty slice for classes that have no fields beyond the
/// inherited ones, or for classes the schema does not model.
pub fn class_fields(class: SbolClass) -> &'static [FieldDescriptor] {
    class_descriptor(class.iri())
        .map(|descriptor| descriptor.fields)
        .unwrap_or(&[])
}
