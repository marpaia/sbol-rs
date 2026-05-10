use std::collections::BTreeSet;

use sbol_ontology::Ontology;

use crate::schema::{TargetClass, ValueKind};
use crate::validation::report::ValidationIssue;
use crate::validation::spec::class_spec;
use crate::validation::tables::{self, SequenceEncoding};
use crate::vocab::*;
use crate::{Document, Object, Resource, SbolClass, Term};

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

/// W3C XML Schema datatype IRIs and the RDF 1.1 language-string IRI
/// that `value_matches_kind` checks against. The set is intentionally
/// small — only datatypes that Table 23 references (or that an SBOL
/// document might plausibly use) appear here.
const XSD_NS: &str = "http://www.w3.org/2001/XMLSchema#";
const RDF_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";

fn xsd(name: &str) -> bool {
    name.starts_with(XSD_NS)
}

fn xsd_local(datatype: &str) -> Option<&str> {
    datatype.strip_prefix(XSD_NS)
}

/// Returns `true` when the datatype is `xsd:string`, an untyped string,
/// or an RDF 1.1 language-tagged string. `Literal::datatype()` returns
/// `xsd:string` for plain literals and `rdf:langString` when a language
/// tag is present, so the check covers both common Turtle/JSON-LD
/// serializations.
fn is_string_datatype(datatype: &str) -> bool {
    matches!(xsd_local(datatype), Some("string")) || datatype == RDF_LANG_STRING
}

/// Returns `true` when the datatype is an `xsd:integer` derivative. The
/// XSD integer hierarchy is wider than just `xsd:integer`; we accept
/// any node in that hierarchy because SBOL Table 23 uses `Integer` to
/// mean "any whole number" and libSBOLj3 round-trips fixtures using
/// `xsd:int` for embedded annotations.
fn is_integer_datatype(datatype: &str) -> bool {
    matches!(
        xsd_local(datatype),
        Some(
            "integer"
                | "long"
                | "int"
                | "short"
                | "byte"
                | "nonNegativeInteger"
                | "positiveInteger"
                | "nonPositiveInteger"
                | "negativeInteger"
                | "unsignedLong"
                | "unsignedInt"
                | "unsignedShort"
                | "unsignedByte"
        )
    )
}

/// Returns `true` when the datatype is one of the XSD floating-point
/// types (or `xsd:decimal`, which is a strict numeric type that we
/// accept for `Float` fields since the spec wording is "Float" not
/// "xsd:float").
fn is_float_datatype(datatype: &str) -> bool {
    matches!(xsd_local(datatype), Some("float" | "double" | "decimal"))
}

/// XSD integer lexical form: optional sign, then one-or-more digits.
fn is_xsd_integer_lexical(value: &str) -> bool {
    let bytes = match value.as_bytes() {
        [b'+' | b'-', rest @ ..] if !rest.is_empty() => rest,
        rest if !rest.is_empty() => rest,
        _ => return false,
    };
    bytes.iter().all(u8::is_ascii_digit)
}

/// XSD float lexical form: standard decimal numbers, `INF`, `-INF`,
/// `+INF`, or `NaN`. We let `f64::from_str` handle the numeric forms
/// after checking the special cases, since Rust's parser accepts the
/// same lexical surface (digits, optional decimal point, optional
/// exponent) as XSD float.
fn is_xsd_float_lexical(value: &str) -> bool {
    matches!(value, "INF" | "+INF" | "-INF" | "NaN") || value.parse::<f64>().is_ok()
}

/// xsd:dateTime lexical form (ISO 8601 subset): `YYYY-MM-DDThh:mm:ss`
/// with an optional fractional-second part and an optional timezone
/// (`Z` or `±hh:mm`). The check rejects partial dates, plain dates,
/// and times without a date. Negative years (`-YYYY-...`) are
/// rejected — they are well-formed XSD but nonsensical in SBOL and
/// pinning to four-digit positive years simplifies parsing.
fn is_xsd_datetime_lexical(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < 19 {
        return false;
    }
    let separators = [
        (0..4, None),
        (4..5, Some(b'-')),
        (5..7, None),
        (7..8, Some(b'-')),
        (8..10, None),
        (10..11, Some(b'T')),
        (11..13, None),
        (13..14, Some(b':')),
        (14..16, None),
        (16..17, Some(b':')),
        (17..19, None),
    ];
    for (range, expected) in separators {
        let segment = &bytes[range];
        match expected {
            Some(sep) => {
                if segment != [sep] {
                    return false;
                }
            }
            None => {
                if !segment.iter().all(u8::is_ascii_digit) {
                    return false;
                }
            }
        }
    }
    let tail = &bytes[19..];
    let tail = if let Some(stripped) = tail.strip_prefix(b".") {
        let split = stripped
            .iter()
            .position(|b| !b.is_ascii_digit())
            .unwrap_or(stripped.len());
        if split == 0 {
            return false;
        }
        &stripped[split..]
    } else {
        tail
    };
    match tail {
        [] | [b'Z'] => true,
        [b'+' | b'-', h1, h2, b':', m1, m2]
            if h1.is_ascii_digit()
                && h2.is_ascii_digit()
                && m1.is_ascii_digit()
                && m2.is_ascii_digit() =>
        {
            true
        }
        _ => false,
    }
}

/// Strict per-value-kind check used by `sbol3-10111` (Table 23 value
/// type compliance). The check requires both:
///
///   1. A compatible RDF datatype (or a permitted plain string literal
///      where the value parses correctly — this matches the SBOL
///      community convention of writing Range bounds as `"123"` in
///      Turtle rather than the typed short-form `123`).
///   2. A well-formed lexical value for the kind.
///
/// Mixing literal kinds (e.g. an `xsd:integer`-typed value on a String
/// property, or a malformed numeric literal on an Integer property)
/// produces a typed `sbol3-10111` error.
pub(crate) fn value_matches_kind(term: &Term, value_kind: ValueKind) -> bool {
    match value_kind {
        ValueKind::Uri => term.as_iri().is_some(),
        ValueKind::Url => term
            .as_iri()
            .is_some_and(|iri| is_namespace_url(iri.as_str())),
        ValueKind::String => term
            .as_literal()
            .is_some_and(|literal| is_string_datatype(literal.datatype().as_str())),
        ValueKind::Integer | ValueKind::Long => term.as_literal().is_some_and(|literal| {
            let dt = literal.datatype().as_str();
            (is_integer_datatype(dt) || is_string_datatype(dt))
                && is_xsd_integer_lexical(literal.value())
                && literal.value().parse::<i64>().is_ok()
        }),
        ValueKind::Float => term.as_literal().is_some_and(|literal| {
            let dt = literal.datatype().as_str();
            (is_float_datatype(dt) || is_string_datatype(dt))
                && is_xsd_float_lexical(literal.value())
        }),
        ValueKind::DateTime => term.as_literal().is_some_and(|literal| {
            let dt = literal.datatype().as_str();
            (xsd(dt) && xsd_local(dt) == Some("dateTime") || is_string_datatype(dt))
                && is_xsd_datetime_lexical(literal.value())
        }),
    }
}

pub(crate) fn known_external_resource(iri: &str) -> Option<ExternalResource> {
    let iri = iri.to_ascii_lowercase();
    if iri.starts_with("https://identifiers.org/uniprot:")
        || iri.starts_with("http://identifiers.org/uniprot:")
        || iri.starts_with("http://purl.uniprot.org/uniprot/")
        || iri.starts_with("https://www.uniprot.org/uniprot/")
        || iri.starts_with("https://www.uniprot.org/uniprotkb/")
    {
        return Some(ExternalResource::Uniprot);
    }
    if iri.starts_with("https://identifiers.org/chebi:")
        || iri.starts_with("http://identifiers.org/chebi:")
        || iri.starts_with("http://purl.obolibrary.org/obo/chebi_")
        || iri.starts_with("https://www.ebi.ac.uk/chebi/")
    {
        return Some(ExternalResource::Chebi);
    }
    if iri.starts_with("https://identifiers.org/pubchem.compound:")
        || iri.starts_with("http://identifiers.org/pubchem.compound:")
        || iri.starts_with("https://pubchem.ncbi.nlm.nih.gov/compound/")
    {
        return Some(ExternalResource::Pubchem);
    }
    None
}

pub(crate) fn object_matches_target(object: &Object, target: TargetClass) -> bool {
    match target {
        TargetClass::Sbol(class) => object.has_class(class),
        TargetClass::ProvActivity => object_has_rdf_type(object, PROV_ACTIVITY),
        TargetClass::ProvAgent => object_has_rdf_type(object, PROV_AGENT_CLASS),
        TargetClass::ProvAssociation => object_has_rdf_type(object, PROV_ASSOCIATION),
        TargetClass::ProvPlan => object_has_rdf_type(object, PROV_PLAN),
        TargetClass::ProvUsage => object_has_rdf_type(object, PROV_USAGE),
        TargetClass::OmMeasure => object_has_rdf_type(object, OM_MEASURE),
        TargetClass::OmUnit => object_type_is_or_inherits(object, OM_UNIT),
        TargetClass::OmPrefix => object_type_is_or_inherits(object, OM_PREFIX),
    }
}

pub(crate) fn is_external_top_level_reference(target: TargetClass) -> bool {
    matches!(target, TargetClass::Sbol(class) if class.is_top_level())
}

pub(crate) fn object_has_rdf_type(object: &Object, iri: &str) -> bool {
    object
        .rdf_types()
        .iter()
        .any(|rdf_type| rdf_type.as_str() == iri)
}

pub(crate) fn object_type_is_or_inherits(object: &Object, target: &str) -> bool {
    object
        .rdf_types()
        .iter()
        .any(|rdf_type| class_inherits(rdf_type.as_str(), target))
}

pub(crate) fn class_inherits(class: &str, target: &str) -> bool {
    if class == target {
        return true;
    }
    let Some(spec) = class_spec(class) else {
        return false;
    };
    spec.parents
        .iter()
        .any(|parent| class_inherits(parent, target))
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

pub(crate) fn is_iupac_nucleotide(character: char) -> bool {
    matches!(
        character.to_ascii_uppercase(),
        'A' | 'C'
            | 'G'
            | 'T'
            | 'U'
            | 'R'
            | 'Y'
            | 'S'
            | 'W'
            | 'K'
            | 'M'
            | 'B'
            | 'D'
            | 'H'
            | 'V'
            | 'N'
    )
}

pub(crate) fn is_iupac_protein(character: char) -> bool {
    matches!(
        character.to_ascii_uppercase(),
        'A' | 'C'
            | 'D'
            | 'E'
            | 'F'
            | 'G'
            | 'H'
            | 'I'
            | 'K'
            | 'L'
            | 'M'
            | 'N'
            | 'P'
            | 'Q'
            | 'R'
            | 'S'
            | 'T'
            | 'V'
            | 'W'
            | 'Y'
            | 'B'
            | 'Z'
            | 'J'
            | 'X'
            | 'U'
            | 'O'
            | '*'
    )
}

pub(crate) fn is_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

pub(crate) fn is_namespace_url(value: &str) -> bool {
    is_url(value) || value.starts_with("urn:")
}

pub(crate) fn is_hex_digest(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|character| character.is_ascii_hexdigit())
}

pub(crate) fn is_known_hash_algorithm(value: &str) -> bool {
    matches!(
        value,
        "sha2-256" | "sha3-256" | "blake3" | "sha2-512" | "sha3-512"
    )
}

pub(crate) fn is_hash_algorithm_token(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphanumeric()
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
}

pub(crate) fn hex_digest(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push_str(&format!("{byte:02x}"));
    }
    value
}

pub(crate) fn url_is_child_of(candidate: &str, parent: &str) -> bool {
    candidate
        .strip_prefix(parent.trim_end_matches('/'))
        .is_some_and(|suffix| suffix.starts_with('/'))
}

pub(crate) fn url_matches_namespace_display_id(
    identity: &str,
    namespace: &str,
    display_id: &str,
) -> bool {
    let namespace = namespace.trim_end_matches('/');
    let Some(rest) = identity.strip_prefix(namespace) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix('/') else {
        return false;
    };
    !rest.is_empty()
        && !rest.split('/').any(str::is_empty)
        && rest.rsplit('/').next() == Some(display_id)
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

pub(crate) fn location_sequence_length(document: &Document, location: &Object) -> Option<usize> {
    let sequence = location.first_resource(SBOL_HAS_SEQUENCE)?;
    let sequence = document.get(sequence)?;
    sequence.first_literal_value(SBOL_ELEMENTS).map(str::len)
}

pub(crate) fn is_valid_display_id(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
}
