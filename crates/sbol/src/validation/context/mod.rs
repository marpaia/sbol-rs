use std::collections::BTreeMap;
use std::fmt;
use std::io;

use sbol_ontology::{Ontology, OntologyRegistry};

use crate::validation::options::ValidationOptions;
use crate::{Document, Iri, Object, Resource};

mod resolvers;

pub use resolvers::FileResolver;
#[cfg(feature = "http-resolver")]
pub use resolvers::{CachingHttpResolver, HttpResolver};

/// Controls whether validation may inspect resources outside the primary document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExternalValidationMode {
    /// Do not perform external document or content resolution.
    #[default]
    Off,
    /// Resolve only caller-provided documents and explicitly configured providers.
    ProvidedOnly,
    /// Resolve caller-provided data and configured external providers such as HTTP.
    ExternalAllowed,
}

/// Resolver-aware validation inputs.
#[derive(Default)]
#[non_exhaustive]
pub struct ValidationContext<'a> {
    options: ValidationOptions,
    ontology_registry: OntologyRegistry,
    external_mode: ExternalValidationMode,
    documents: Vec<&'a Document>,
    document_resolvers: Vec<&'a dyn DocumentResolver>,
    content_resolvers: Vec<&'a dyn ContentResolver>,
}

impl<'a> ValidationContext<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_options(mut options: ValidationOptions) -> Self {
        let extensions = options.take_ontology_extensions();
        let ontology_registry = if extensions.is_empty() {
            OntologyRegistry::bundled_only()
        } else {
            OntologyRegistry::bundled_with(extensions)
        };
        Self {
            options,
            ontology_registry,
            ..Self::default()
        }
    }

    pub fn options(&self) -> &ValidationOptions {
        &self.options
    }

    /// Returns the ontology view used by validation. Bundled-only by default;
    /// extension snapshots layered in via
    /// [`ValidationOptions::with_ontology_extension`] are merged on top.
    pub fn ontology(&self) -> &Ontology {
        self.ontology_registry.ontology()
    }

    /// Returns the underlying ontology registry.
    pub fn ontology_registry(&self) -> &OntologyRegistry {
        &self.ontology_registry
    }

    pub fn external_mode(&self) -> ExternalValidationMode {
        self.external_mode
    }

    pub fn documents(&self) -> &[&'a Document] {
        &self.documents
    }

    pub fn document_resolvers(&self) -> &[&'a dyn DocumentResolver] {
        &self.document_resolvers
    }

    pub fn content_resolvers(&self) -> &[&'a dyn ContentResolver] {
        &self.content_resolvers
    }

    pub fn with_external_mode(mut self, external_mode: ExternalValidationMode) -> Self {
        self.external_mode = external_mode;
        self
    }

    pub fn with_document(mut self, document: &'a Document) -> Self {
        self.documents.push(document);
        self
    }

    pub fn with_document_resolver(mut self, resolver: &'a dyn DocumentResolver) -> Self {
        self.document_resolvers.push(resolver);
        self
    }

    pub fn with_content_resolver(mut self, resolver: &'a dyn ContentResolver) -> Self {
        self.content_resolvers.push(resolver);
        self
    }
}

/// A set of in-memory SBOL documents indexed by object identity.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct DocumentSet<'a> {
    documents: Vec<&'a Document>,
    objects: BTreeMap<Resource, &'a Object>,
}

impl<'a> DocumentSet<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_documents(
        documents: impl IntoIterator<Item = &'a Document>,
    ) -> Result<Self, DocumentSetError> {
        let mut set = Self::new();
        for document in documents {
            set.add_document(document)?;
        }
        Ok(set)
    }

    pub fn add_document(&mut self, document: &'a Document) -> Result<(), DocumentSetError> {
        for identity in document.objects().keys() {
            if self.objects.contains_key(identity) {
                return Err(DocumentSetError::duplicate(identity.clone()));
            }
        }

        for (identity, object) in document.objects() {
            self.objects.insert(identity.clone(), object);
        }
        self.documents.push(document);
        Ok(())
    }

    pub fn documents(&self) -> &[&'a Document] {
        &self.documents
    }

    pub fn get(&self, identity: &Resource) -> Option<&'a Object> {
        self.objects.get(identity).copied()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DocumentSetError {
    identity: Resource,
}

impl DocumentSetError {
    fn duplicate(identity: Resource) -> Self {
        Self { identity }
    }

    pub fn identity(&self) -> &Resource {
        &self.identity
    }
}

impl fmt::Display for DocumentSetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "duplicate SBOL object identity `{}` in document set",
            self.identity
        )
    }
}

impl std::error::Error for DocumentSetError {}

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
    fn resolve_document(&self, resource: &Resource) -> Result<Document, ResolutionError>;
}

/// Resolves an Attachment or Model source into bytes.
pub trait ContentResolver {
    fn resolve_content(&self, source: &Iri) -> Result<ResolvedContent, ResolutionError>;
}
