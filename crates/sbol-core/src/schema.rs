//! Version-neutral schema descriptors for SBOL classes and properties.
//!
//! These types describe an SBOL data model in terms of fields:
//!
//! - [`FieldDescriptor`] describes one property: its predicate IRI,
//!   cardinality, value kind, optional reference target, and the validation
//!   rule that polices its cardinality and value-kind checks.
//! - [`Cardinality`], [`ValueKind`], [`TargetClass`], and [`ReferenceSpec`]
//!   are supporting metadata.
//!
//! A versioned model supplies its own class hierarchy and groups these field
//! descriptors per class. Validation reads the descriptors to drive
//! table-driven rule checks, and RDF serialization shares the same source of
//! truth so the schema cannot drift between subsystems.

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
    /// A model class, identified by its RDF class IRI.
    Sbol(&'static str),
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

impl ReferenceSpec {
    /// Assemble a reference specification.
    pub const fn new(target: TargetClass, require_local: bool) -> Self {
        Self {
            target,
            require_local,
        }
    }
}

/// Describes one property on an SBOL class.
///
/// `FieldDescriptor` is the single source of truth shared between the
/// validator and the RDF serialization/deserialization layer. The same
/// descriptor is consulted by table-driven validation rules and by the
/// descriptor-driven serializer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FieldDescriptor {
    /// The RDF predicate IRI for this property.
    pub predicate: &'static str,
    /// The validation rule that polices cardinality/value-kind for this
    /// property (e.g. `"sbol3-10501"`).
    pub rule: &'static str,
    /// How many values are permitted.
    pub cardinality: Cardinality,
    /// The expected lexical kind of values.
    pub value_kind: ValueKind,
    /// If `Some`, this property is a reference whose target must satisfy
    /// the given [`ReferenceSpec`].
    pub reference: Option<ReferenceSpec>,
}

impl FieldDescriptor {
    /// Assemble a field descriptor.
    pub const fn new(
        predicate: &'static str,
        rule: &'static str,
        cardinality: Cardinality,
        value_kind: ValueKind,
        reference: Option<ReferenceSpec>,
    ) -> Self {
        Self {
            predicate,
            rule,
            cardinality,
            value_kind,
            reference,
        }
    }
}
