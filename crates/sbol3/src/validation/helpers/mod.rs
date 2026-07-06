use std::collections::BTreeSet;

use sbol_ontology::Ontology;

use crate::schema::{TargetClass, ValueKind};
use crate::validation::report::ValidationIssue;
use crate::validation::spec::class_spec;
use crate::validation::tables::{self, SequenceEncoding};
use crate::vocab::*;
use crate::{Document, Object, Resource, SbolClass, Term};

mod component;
mod rdf;
mod syntax;
mod xsd;

pub(crate) use component::*;
pub(crate) use rdf::*;
pub(crate) use syntax::*;
pub(crate) use xsd::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExternalResource {
    Chebi,
    Pubchem,
    Uniprot,
}

pub(crate) const COMPOSITE_PREDICATES: &[&str] = &[
    SBOL_HAS_FEATURE,
    SBOL_HAS_CONSTRAINT,
    SBOL_HAS_INTERACTION,
    SBOL_HAS_INTERFACE,
    SBOL_HAS_LOCATION,
    SBOL_SOURCE_LOCATION,
    SBOL_HAS_PARTICIPATION,
    SBOL_HAS_VARIABLE_FEATURE,
    PROV_QUALIFIED_USAGE,
    PROV_QUALIFIED_ASSOCIATION,
];

#[derive(Clone, Debug)]
pub(crate) struct SequenceInfo {
    pub(crate) identity: Resource,
    pub(crate) encoding: Option<SequenceEncoding>,
    pub(crate) encoding_iri: Option<String>,
    pub(crate) elements: Option<String>,
}

pub(crate) fn integer_value(object: &Object, predicate: &str) -> Option<i64> {
    object
        .first_literal_value(predicate)
        .and_then(|value| value.parse::<i64>().ok())
}

pub(crate) fn error_issue(
    rule: &'static str,
    subject: &Resource,
    property: Option<&'static str>,
    message: impl Into<String>,
) -> ValidationIssue {
    ValidationIssue::error(rule, subject.clone(), property, message)
}

pub(crate) fn warning_issue(
    rule: &'static str,
    subject: &Resource,
    property: Option<&'static str>,
    message: impl Into<String>,
) -> ValidationIssue {
    ValidationIssue::warning(rule, subject.clone(), property, message)
}
