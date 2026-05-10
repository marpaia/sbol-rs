use std::collections::{BTreeMap, BTreeSet};

use crate::client::{SbolObject, ToRdf};
use crate::object::collect_objects;
use crate::{BuildError, Document, RdfGraph};

impl Document {
    /// Builds a document from owned typed SBOL objects.
    ///
    /// The supplied typed values are retained as the document's primary
    /// typed surface; the RDF graph and property-bag indexed map are
    /// derived from them so the document is round-trip ready.
    pub fn from_objects(objects: Vec<SbolObject>) -> Result<Self, BuildError> {
        let mut identities = BTreeSet::new();
        let mut triples = Vec::new();

        for object in &objects {
            let identity = object.identity().clone();
            if !identities.insert(identity.clone()) {
                return Err(BuildError::DuplicateIdentity(identity));
            }
            triples.extend(object.to_rdf_triples()?);
        }

        let graph = RdfGraph::new(triples);
        let property_map: BTreeMap<_, _> = collect_objects(&graph);
        Ok(Document::from_parts(graph, property_map, objects))
    }
}
