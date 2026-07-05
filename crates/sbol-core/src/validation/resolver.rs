//! Version-neutral external-resolution interfaces: the traits a caller
//! implements to dereference document and content references during
//! resolver-aware validation, plus the resolved value and error types they
//! exchange.

use std::fmt;
use std::io;

use sbol_rdf::{Iri, Resource};

use crate::document::RawDocument;

/// Resolved byte content for an Attachment or Model source.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolvedContent {
    pub bytes: Vec<u8>,
    pub media_type: Option<String>,
}

impl ResolvedContent {
    pub fn new(bytes: impl Into<Vec<u8>>, media_type: Option<String>) -> Self {
        Self {
            bytes: bytes.into(),
            media_type,
        }
    }
}

/// Coarse class of a resolution failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolutionErrorKind {
    UnsupportedScheme,
    NotFound,
    InvalidData,
    Io,
    Http,
    Parse,
}

/// A resolver failure with a stable kind and human-readable context.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolutionError {
    kind: ResolutionErrorKind,
    message: String,
}

impl ResolutionError {
    pub fn new(kind: ResolutionErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ResolutionErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ResolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ResolutionError {}

impl From<io::Error> for ResolutionError {
    fn from(error: io::Error) -> Self {
        Self::new(ResolutionErrorKind::Io, error.to_string())
    }
}

/// Resolves an external resource into an SBOL document.
pub trait DocumentResolver {
    fn resolve_document(&self, resource: &Resource) -> Result<RawDocument, ResolutionError>;
}

/// Resolves an Attachment or Model source into bytes.
pub trait ContentResolver {
    fn resolve_content(&self, source: &Iri) -> Result<ResolvedContent, ResolutionError>;
}
