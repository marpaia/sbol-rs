//! Version-neutral document scaffolding: the RDF-backed object store shared by
//! every SBOL document, independent of the typed surface a versioned model
//! layers on top.

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use sbol_rdf::{Graph, RdfFormat, Resource};

use crate::error::WriteError;
use crate::object::Object;

/// Read access to a document's RDF-backed objects, indexed by identity.
pub trait ObjectStore {
    /// Returns the RDF-backed objects indexed by identity.
    fn objects(&self) -> &BTreeMap<Resource, Object>;

    /// Returns the RDF-backed object at `identity`, if any.
    fn get(&self, identity: &Resource) -> Option<&Object>;
}

/// An RDF graph paired with its parsed, identity-indexed objects.
///
/// `RawDocument` is the version-neutral core of an SBOL document: the RDF
/// graph as the source of truth plus the property-bag [`Object`] values
/// derived from it. A versioned document wraps this with its own typed
/// surface.
#[derive(Clone, Debug)]
pub struct RawDocument {
    graph: Graph,
    objects: BTreeMap<Resource, Object>,
}

impl RawDocument {
    /// Assemble a raw document from an RDF graph and its parsed objects.
    pub fn from_parts(graph: Graph, objects: BTreeMap<Resource, Object>) -> Self {
        Self { graph, objects }
    }

    /// Returns the underlying RDF graph.
    pub fn rdf_graph(&self) -> &Graph {
        &self.graph
    }

    /// Returns RDF-backed objects indexed by identity.
    pub fn objects(&self) -> &BTreeMap<Resource, Object> {
        &self.objects
    }

    /// Returns the RDF-backed object at `identity`, if any.
    pub fn get(&self, identity: &Resource) -> Option<&Object> {
        self.objects.get(identity)
    }

    /// Serializes the document in the given RDF format.
    pub fn write(&self, format: RdfFormat) -> Result<String, WriteError> {
        self.graph.write(format).map_err(WriteError::Rdf)
    }

    /// Writes the document to a file in the given RDF format. The caller
    /// chooses the format explicitly; no inference from the path's
    /// extension is performed.
    pub fn write_path(&self, path: impl AsRef<Path>, format: RdfFormat) -> Result<(), WriteError> {
        let path = path.as_ref();
        let serialized = self.write(format)?;
        std::fs::write(path, serialized).map_err(|source| WriteError::Io {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Serializes the underlying RDF graph as Turtle.
    pub fn write_turtle(&self) -> Result<String, WriteError> {
        self.write(RdfFormat::Turtle)
    }
}

impl ObjectStore for RawDocument {
    fn objects(&self) -> &BTreeMap<Resource, Object> {
        &self.objects
    }

    fn get(&self, identity: &Resource) -> Option<&Object> {
        self.objects.get(identity)
    }
}

/// A set of in-memory documents indexed by object identity.
///
/// Objects across the member documents share one identity space; adding a
/// document whose identities collide with an already-indexed one fails with
/// a [`DocumentSetError`]. Generic over any [`ObjectStore`], so it composes
/// documents from any SBOL version.
#[derive(Debug)]
#[non_exhaustive]
pub struct DocumentSet<'a, S: ObjectStore> {
    documents: Vec<&'a S>,
    objects: BTreeMap<Resource, &'a Object>,
}

impl<'a, S: ObjectStore> Default for DocumentSet<'a, S> {
    fn default() -> Self {
        Self {
            documents: Vec::new(),
            objects: BTreeMap::new(),
        }
    }
}

impl<'a, S: ObjectStore> DocumentSet<'a, S> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_documents(
        documents: impl IntoIterator<Item = &'a S>,
    ) -> Result<Self, DocumentSetError> {
        let mut set = Self::new();
        for document in documents {
            set.add_document(document)?;
        }
        Ok(set)
    }

    pub fn add_document(&mut self, document: &'a S) -> Result<(), DocumentSetError> {
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

    pub fn documents(&self) -> &[&'a S] {
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
