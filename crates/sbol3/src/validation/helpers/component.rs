//! SBOL component, collection, location, and sequence graph queries.

use super::*;
use crate::object::ObjectClasses;

pub(crate) fn component_sequence_infos(
    ontology: &Ontology,
    document: &Document,
    component: &Object,
) -> Vec<SequenceInfo> {
    component
        .resources(SBOL_HAS_SEQUENCE)
        .filter_map(|identity| {
            let sequence = document.get(identity)?;
            let encoding_iri = sequence.first_iri(SBOL_ENCODING).map(|iri| {
                tables::canonical_known_iri(ontology, iri.as_str())
                    .unwrap_or_else(|| iri.as_str().to_owned())
            });
            let encoding = encoding_iri
                .as_deref()
                .and_then(|iri| tables::sequence_encoding(ontology, iri));
            Some(SequenceInfo {
                identity: identity.clone(),
                encoding,
                encoding_iri,
                elements: sequence
                    .first_literal_value(SBOL_ELEMENTS)
                    .map(ToOwned::to_owned),
            })
        })
        .collect()
}

pub(crate) fn component_contains(
    component: &Resource,
    document: &Document,
    predicate: &str,
    child: &Resource,
) -> bool {
    document
        .get(component)
        .is_some_and(|component| component.resources(predicate).any(|value| value == child))
}

pub(crate) fn component_allows_sequence(
    document: &Document,
    component: &Resource,
    sequence: &Resource,
) -> bool {
    let Some(component) = document.get(component) else {
        return false;
    };
    if component
        .resources(SBOL_HAS_SEQUENCE)
        .any(|component_sequence| component_sequence == sequence)
    {
        return true;
    }
    component.resources(SBOL_HAS_FEATURE).any(|feature| {
        let Some(feature) = document.get(feature) else {
            return false;
        };
        feature.resources(SBOL_HAS_LOCATION).any(|location| {
            let Some(location) = document.get(location) else {
                return false;
            };
            location.has_class(SbolClass::EntireSequence)
                && location
                    .first_resource(SBOL_HAS_SEQUENCE)
                    .is_some_and(|entire_sequence| entire_sequence == sequence)
        })
    })
}

pub(crate) fn collection_contains_only_components_or_collections(
    document: &Document,
    collection: &Resource,
    visited: &mut BTreeSet<Resource>,
) -> bool {
    if !visited.insert(collection.clone()) {
        return true;
    }
    let Some(collection_object) = document.get(collection) else {
        return true;
    };
    collection_object.resources(SBOL_MEMBER).all(|member| {
        let Some(member_object) = document.get(member) else {
            return true;
        };
        if member_object.has_class(SbolClass::Component) {
            return true;
        }
        member_object.has_class(SbolClass::Collection)
            && collection_contains_only_components_or_collections(document, member, visited)
    })
}

pub(crate) fn sum_location_lengths(document: &Document, locations: &[&Resource]) -> Option<usize> {
    let mut total = 0;
    for location in locations {
        total += location_length(document, location)?;
    }
    Some(total)
}

pub(crate) fn location_length(document: &Document, location: &Resource) -> Option<usize> {
    let location = document.get(location)?;
    if location.has_class(SbolClass::Range) {
        let start = integer_value(location, SBOL_START)?;
        let end = integer_value(location, SBOL_END)?;
        if start <= 0 || end < start {
            return None;
        }
        return Some((end - start + 1) as usize);
    }
    if location.has_class(SbolClass::Cut) {
        return Some(0);
    }
    if location.has_class(SbolClass::EntireSequence) {
        return location_sequence_length(document, location);
    }
    None
}

pub(crate) fn first_invalid_sequence_element(
    ontology: &Ontology,
    elements: &str,
    encoding: &str,
) -> Option<char> {
    let canonical_encoding =
        tables::canonical_known_iri(ontology, encoding).unwrap_or_else(|| encoding.to_owned());
    match canonical_encoding.as_str() {
        EDAM_IUPAC_DNA_RNA_ENCODING => elements
            .chars()
            .find(|character| !is_iupac_nucleotide(*character)),
        EDAM_IUPAC_PROTEIN_ENCODING => elements
            .chars()
            .find(|character| !is_iupac_protein(*character)),
        _ => None,
    }
}

pub(crate) fn location_sequence_length(document: &Document, location: &Object) -> Option<usize> {
    let sequence = location.first_resource(SBOL_HAS_SEQUENCE)?;
    let sequence = document.get(sequence)?;
    sequence.first_literal_value(SBOL_ELEMENTS).map(str::len)
}
