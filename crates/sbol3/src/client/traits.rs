use crate::{BuildError, Object, Triple};

/// Converts an owned typed SBOL value into RDF triples.
pub trait ToRdf {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError>;
}

/// Converts an RDF-collected object into an owned typed SBOL value.
pub trait TryFromObject: Sized {
    fn try_from_object(object: &Object) -> Option<Self>;
}
