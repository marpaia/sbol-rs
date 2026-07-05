//! Version-neutral field-structs shared by the parsed-object representation:
//! the identity-level and top-level metadata every SBOL object carries
//! regardless of model version.

use sbol_rdf::{Iri, Resource};

/// Shared fields on Identified objects.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Identified {
    pub display_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub derived_from: Vec<Resource>,
    pub generated_by: Vec<Resource>,
    pub measures: Vec<Resource>,
    pub attachments: Vec<Resource>,
}

/// Shared fields on TopLevel objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopLevel {
    pub namespace: Option<Iri>,
}
