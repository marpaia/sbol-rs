//! Version-neutral field-structs shared by the parsed-object representation:
//! the identity-level and top-level metadata every SBOL object carries
//! regardless of model version, and the [`Object`] value that binds an
//! identity to its RDF types, properties, and parsed metadata.

use std::collections::{BTreeMap, BTreeSet};

use sbol_rdf::{Iri, Literal, Resource, Term};

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

/// An SBOL or SBOL extension object.
///
/// An `Object` binds an identity to its RDF types, its raw property map, and
/// the version-neutral [`Identified`]/[`TopLevel`] metadata parsed from those
/// properties. Extension and annotation triples are retained inside
/// `properties` alongside the recognized SBOL predicates.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Object {
    identity: Resource,
    rdf_types: BTreeSet<Iri>,
    properties: BTreeMap<Iri, Vec<Term>>,
    identified: Identified,
    top_level: Option<TopLevel>,
}

impl Object {
    /// Assemble an `Object` from its parsed parts.
    pub fn from_parts(
        identity: Resource,
        rdf_types: BTreeSet<Iri>,
        properties: BTreeMap<Iri, Vec<Term>>,
        identified: Identified,
        top_level: Option<TopLevel>,
    ) -> Self {
        Self {
            identity,
            rdf_types,
            properties,
            identified,
            top_level,
        }
    }

    pub fn identity(&self) -> &Resource {
        &self.identity
    }

    pub fn rdf_types(&self) -> &BTreeSet<Iri> {
        &self.rdf_types
    }

    pub fn properties(&self) -> &BTreeMap<Iri, Vec<Term>> {
        &self.properties
    }

    pub fn identified(&self) -> &Identified {
        &self.identified
    }

    pub fn top_level(&self) -> Option<&TopLevel> {
        self.top_level.as_ref()
    }

    pub fn is_top_level(&self) -> bool {
        self.top_level.is_some()
    }

    pub fn values(&self, predicate: &str) -> &[Term] {
        terms_for(&self.properties, predicate)
    }

    pub fn resources(&self, predicate: &str) -> impl Iterator<Item = &Resource> {
        self.values(predicate).iter().filter_map(Term::as_resource)
    }

    pub fn iris(&self, predicate: &str) -> impl Iterator<Item = &Iri> {
        self.values(predicate).iter().filter_map(Term::as_iri)
    }

    pub fn literals(&self, predicate: &str) -> impl Iterator<Item = &Literal> {
        self.values(predicate).iter().filter_map(Term::as_literal)
    }

    pub fn first_resource(&self, predicate: &str) -> Option<&Resource> {
        self.resources(predicate).next()
    }

    pub fn first_iri(&self, predicate: &str) -> Option<&Iri> {
        self.iris(predicate).next()
    }

    pub fn first_literal_value(&self, predicate: &str) -> Option<&str> {
        self.literals(predicate).next().map(Literal::value)
    }
}

fn terms_for<'a>(properties: &'a BTreeMap<Iri, Vec<Term>>, predicate: &str) -> &'a [Term] {
    properties
        .get(&Iri::new_unchecked(predicate))
        .map_or(&[], Vec::as_slice)
}
