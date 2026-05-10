use std::borrow::Cow;
use std::fmt;

use crate::backends::{Backend, DefaultBackend};
use crate::error::IriError;

pub(crate) const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

/// An RDF IRI.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Iri(Cow<'static, str>);

impl Iri {
    /// Validates and creates an IRI from a dynamic string.
    pub fn new(value: impl Into<String>) -> Result<Self, IriError> {
        let value = value.into();
        DefaultBackend::validate_iri(&value)?;
        Ok(Self(Cow::Owned(value)))
    }

    /// Creates an IRI from a dynamic string without validation.
    ///
    /// Use [`Iri::from_static`] when the value is a `&'static str` to avoid
    /// allocation.
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(Cow::Owned(value.into()))
    }

    /// Creates an IRI from a static string without validation or allocation.
    ///
    /// This is the right constructor for IRI constants (predicate URIs, class
    /// URIs, vocabulary terms). Equivalent to [`Iri::new_unchecked`] but
    /// avoids the heap allocation and documents that the value is known to
    /// be a valid IRI at compile time.
    pub const fn from_static(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0.into_owned()
    }
}

impl fmt::Display for Iri {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// An RDF blank node identifier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlankNode(String);

impl BlankNode {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// RDF subject resource.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Resource {
    Iri(Iri),
    BlankNode(BlankNode),
}

impl Resource {
    pub fn iri(value: impl Into<String>) -> Self {
        Self::Iri(Iri::new_unchecked(value))
    }

    pub fn as_iri(&self) -> Option<&Iri> {
        match self {
            Self::Iri(iri) => Some(iri),
            Self::BlankNode(_) => None,
        }
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Iri(iri) => formatter.write_str(iri.as_str()),
            Self::BlankNode(blank_node) => write!(formatter, "_:{}", blank_node.as_str()),
        }
    }
}

/// RDF literal lexical data.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Literal {
    value: String,
    datatype: Iri,
    language: Option<String>,
}

impl Literal {
    pub fn new(value: impl Into<String>, datatype: Iri, language: Option<String>) -> Self {
        Self {
            value: value.into(),
            datatype,
            language,
        }
    }

    pub fn simple(value: impl Into<String>) -> Self {
        Self::new(value, Iri::from_static(XSD_STRING), None)
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn datatype(&self) -> &Iri {
        &self.datatype
    }

    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }
}

/// RDF object term.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Term {
    Resource(Resource),
    Literal(Literal),
}

impl Term {
    pub fn as_resource(&self) -> Option<&Resource> {
        match self {
            Self::Resource(resource) => Some(resource),
            Self::Literal(_) => None,
        }
    }

    pub fn as_iri(&self) -> Option<&Iri> {
        self.as_resource().and_then(Resource::as_iri)
    }

    pub fn as_literal(&self) -> Option<&Literal> {
        match self {
            Self::Resource(_) => None,
            Self::Literal(literal) => Some(literal),
        }
    }
}

/// An RDF triple in the default graph.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Triple {
    pub subject: Resource,
    pub predicate: Iri,
    pub object: Term,
}
