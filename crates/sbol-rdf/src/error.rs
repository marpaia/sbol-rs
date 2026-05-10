use thiserror::Error;

/// A boxed dynamic error used to carry RDF backend source errors without
/// tightly coupling the public API to a specific backend's error type.
pub type BackendError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Errors produced while validating or parsing an IRI.
#[derive(Debug, Error)]
#[error("invalid IRI `{value}`")]
#[non_exhaustive]
pub struct IriError {
    pub value: String,
    #[source]
    pub source: BackendError,
}

impl IriError {
    pub(crate) fn new(value: impl Into<String>, source: impl Into<BackendError>) -> Self {
        Self {
            value: value.into(),
            source: source.into(),
        }
    }
}

/// Errors produced while parsing an RDF document.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseError {
    #[error("RDF parse failed")]
    Backend(#[source] BackendError),

    #[error("Turtle parser returned a named graph quad")]
    NamedGraphInDefault,

    #[error("RDF-star triple terms are not supported")]
    UnsupportedRdfStar,
}

impl ParseError {
    pub(crate) fn backend(source: impl Into<BackendError>) -> Self {
        Self::Backend(source.into())
    }
}

/// Errors produced while writing an RDF document.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum WriteError {
    #[error("RDF serialization failed")]
    Backend(#[source] BackendError),

    #[error("invalid IRI `{value}` in serialized output")]
    InvalidIri {
        value: String,
        #[source]
        source: BackendError,
    },

    #[error("invalid blank node `{value}` in serialized output")]
    InvalidBlankNode {
        value: String,
        #[source]
        source: BackendError,
    },

    #[error("invalid language tag `{value}` in serialized output")]
    InvalidLanguageTag {
        value: String,
        #[source]
        source: BackendError,
    },

    #[error("serialized output is not valid UTF-8")]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl WriteError {
    pub(crate) fn backend(source: impl Into<BackendError>) -> Self {
        Self::Backend(source.into())
    }

    pub(crate) fn invalid_iri(value: impl Into<String>, source: impl Into<BackendError>) -> Self {
        Self::InvalidIri {
            value: value.into(),
            source: source.into(),
        }
    }

    pub(crate) fn invalid_blank_node(
        value: impl Into<String>,
        source: impl Into<BackendError>,
    ) -> Self {
        Self::InvalidBlankNode {
            value: value.into(),
            source: source.into(),
        }
    }

    pub(crate) fn invalid_language_tag(
        value: impl Into<String>,
        source: impl Into<BackendError>,
    ) -> Self {
        Self::InvalidLanguageTag {
            value: value.into(),
            source: source.into(),
        }
    }
}
