//! Lexical and RDF-shape helpers shared by the SBOL 2 rule checks: XSD
//! datatype/lexical validation for Table-driven value-kind checks and
//! reference-target classification against the class hierarchy.

use crate::object::ObjectClasses;
use crate::schema::{TargetClass, ValueKind, class_descriptor};
use crate::vocab::*;
use crate::{Iri, Object, Sbol2Class, Term};

const XSD_NS: &str = "http://www.w3.org/2001/XMLSchema#";
const RDF_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";

fn xsd_local(datatype: &str) -> Option<&str> {
    datatype.strip_prefix(XSD_NS)
}

fn is_string_datatype(datatype: &str) -> bool {
    matches!(xsd_local(datatype), Some("string")) || datatype == RDF_LANG_STRING
}

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

fn is_float_datatype(datatype: &str) -> bool {
    matches!(xsd_local(datatype), Some("float" | "double" | "decimal"))
}

fn is_xsd_integer_lexical(value: &str) -> bool {
    let bytes = match value.as_bytes() {
        [b'+' | b'-', rest @ ..] if !rest.is_empty() => rest,
        rest if !rest.is_empty() => rest,
        _ => return false,
    };
    bytes.iter().all(u8::is_ascii_digit)
}

fn is_xsd_float_lexical(value: &str) -> bool {
    matches!(value, "INF" | "+INF" | "-INF" | "NaN") || value.parse::<f64>().is_ok()
}

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
    matches!(tail, [] | [b'Z'])
        || matches!(
            tail,
            [b'+' | b'-', h1, h2, b':', m1, m2]
                if h1.is_ascii_digit()
                    && h2.is_ascii_digit()
                    && m1.is_ascii_digit()
                    && m2.is_ascii_digit()
        )
}

/// Whether a term satisfies the lexical/datatype shape a property's
/// [`ValueKind`] requires. Plain string literals are accepted for numeric
/// kinds when their lexical value parses, matching the SBOL convention of
/// writing numeric bounds as strings in RDF/XML.
pub(crate) fn value_matches_kind(term: &Term, value_kind: ValueKind) -> bool {
    match value_kind {
        // SBOL 2 property rules require a URI; they do not constrain the URI
        // scheme, so `file:` and other schemes are accepted for both kinds.
        ValueKind::Uri | ValueKind::Url => term.as_iri().is_some(),
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
            (matches!(xsd_local(dt), Some("dateTime")) || is_string_datatype(dt))
                && is_xsd_datetime_lexical(literal.value())
        }),
        _ => false,
    }
}

pub(crate) fn object_has_rdf_type(object: &Object, iri: &str) -> bool {
    object
        .rdf_types()
        .iter()
        .any(|rdf_type| rdf_type.as_str() == iri)
}

fn class_inherits(class: &str, target: &str) -> bool {
    if class == target {
        return true;
    }
    let Some(spec) = class_descriptor(class) else {
        return false;
    };
    spec.parents
        .iter()
        .any(|parent| class_inherits(parent, target))
}

pub(crate) fn object_type_is_or_inherits(object: &Object, target: &str) -> bool {
    object
        .rdf_types()
        .iter()
        .any(|rdf_type| class_inherits(rdf_type.as_str(), target))
}

/// Whether an object satisfies the class a reference property targets.
pub(crate) fn object_matches_target(object: &Object, target: TargetClass) -> bool {
    match target {
        TargetClass::Sbol(class_iri) => Sbol2Class::from_iri(&Iri::new_unchecked(class_iri))
            .is_some_and(|class| object.has_class(class)),
        TargetClass::ProvActivity => object_has_rdf_type(object, PROV_ACTIVITY),
        TargetClass::ProvAgent => object_has_rdf_type(object, PROV_AGENT_CLASS),
        TargetClass::ProvAssociation => object_has_rdf_type(object, PROV_ASSOCIATION),
        TargetClass::ProvPlan => object_has_rdf_type(object, PROV_PLAN),
        TargetClass::ProvUsage => object_has_rdf_type(object, PROV_USAGE),
        TargetClass::OmMeasure => object_has_rdf_type(object, OM_MEASURE),
        TargetClass::OmUnit => object_type_is_or_inherits(object, OM_UNIT),
        TargetClass::OmPrefix => object_type_is_or_inherits(object, OM_PREFIX),
        _ => false,
    }
}

/// Whether an object carries at least one RDF type that resolves to a known
/// SBOL 2, PROV, or OM class. Extension/annotation objects (custom RDF types)
/// are not governed by SBOL validation rules and are skipped.
pub(crate) fn is_sbol_object(object: &Object) -> bool {
    object
        .rdf_types()
        .iter()
        .any(|rdf_type| class_descriptor(rdf_type.as_str()).is_some())
}
