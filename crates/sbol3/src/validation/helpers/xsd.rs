//! XSD datatype and lexical-form validation for Table 23 value-type checks.

use super::*;

/// W3C XML Schema datatype IRIs and the RDF 1.1 language-string IRI
/// that `value_matches_kind` checks against. The set is intentionally
/// small: only datatypes that Table 23 references (or that an SBOL
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
/// rejected. They are well-formed XSD but nonsensical in SBOL, and
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
///      where the value parses correctly; this matches the SBOL
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
        _ => false,
    }
}
