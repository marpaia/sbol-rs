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

/// How many values a property may carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Cardinality {
    /// Exactly one value is required.
    ExactlyOne,
    /// Zero or one value is allowed.
    ZeroOrOne,
    /// At least one value is required.
    OneOrMore,
    /// Any number of values is allowed.
    ZeroOrMore,
}

/// The lexical kind of a property's value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValueKind {
    /// An IRI of any scheme.
    Uri,
    /// An IRI whose scheme is `http`, `https`, or `urn`.
    Url,
    /// A string literal.
    String,
    /// A signed integer literal.
    Integer,
    /// A signed long literal.
    Long,
    /// A floating-point literal.
    Float,
    /// An xsd:dateTime literal.
    DateTime,
}

/// The kind of object a reference property must point at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TargetClass {
    /// An SBOL class (defined in the SBOL vocabulary).
    Sbol(SbolClass),
    /// `prov:Activity`.
    ProvActivity,
    /// `prov:Agent`.
    ProvAgent,
    /// `prov:Association`.
    ProvAssociation,
    /// `prov:Plan`.
    ProvPlan,
    /// `prov:Usage`.
    ProvUsage,
    /// `om:Measure`.
    OmMeasure,
    /// `om:Unit` (any unit class in the OM hierarchy).
    OmUnit,
    /// `om:Prefix` (any prefix class in the OM hierarchy).
    OmPrefix,
}

/// Describes a reference-typed property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ReferenceSpec {
    /// The kind of object the reference must point at.
    pub target: TargetClass,
    /// `true` if the target must be in the same document (a contained
    /// child); `false` if external references are permitted.
    pub require_local: bool,
}

/// Describes one property on an SBOL class.
///
/// `FieldDescriptor` is the single source of truth shared between the
/// validator and the RDF serialization/deserialization layer. The same
/// descriptor is consulted by table-driven validation rules (10109,
/// 10110, 10111, 10112, 10113) and by the descriptor-driven serializer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FieldDescriptor {
    /// The RDF predicate IRI for this property.
    pub predicate: &'static str,
    /// The SBOL validation rule that polices cardinality/value-kind for
    /// this property (e.g. `"sbol3-10501"`).
    pub rule: &'static str,
    /// How many values are permitted.
    pub cardinality: Cardinality,
    /// The expected lexical kind of values.
    pub value_kind: ValueKind,
    /// If `Some`, this property is a reference whose target must satisfy
    /// the given [`ReferenceSpec`].
    pub reference: Option<ReferenceSpec>,
}

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
