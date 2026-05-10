//! Low-level OBO and RDF/XML parsers used by the bundled snapshot
//! generator and the runtime extension cache.
//!
//! These are deliberately permissive line-based parsers that recover
//! just enough information (term IDs, labels, `is_a` / `subClassOf`
//! parents) to populate the SBOL ontology fact tables. Terms whose
//! prefix is not recognized by [`normalize_term_id`] are dropped.

use std::collections::{BTreeMap, BTreeSet};

use crate::normalize_term_id;

/// Minimum information the parser captures for one ontology term.
#[derive(Clone, Debug, Default)]
pub struct RawTerm {
    pub label: Option<String>,
    pub parents: BTreeSet<String>,
}

/// Parses an OBO Foundry text dump. Mutates `terms` in place.
pub fn parse_obo_terms(text: &str, terms: &mut BTreeMap<String, RawTerm>) {
    let mut current_id = None::<String>;
    let mut current = RawTerm::default();

    for line in text.lines() {
        if line == "[Term]" {
            flush_raw_term(&mut current_id, &mut current, terms);
            continue;
        }
        if let Some(id) = line.strip_prefix("id: ") {
            current_id = normalize_term_id(id.trim());
            continue;
        }
        if let Some(name) = line.strip_prefix("name: ") {
            current.label = Some(name.trim().to_owned());
            continue;
        }
        if let Some(parent) = line.strip_prefix("is_a: ")
            && let Some(parent) = parent.split_whitespace().next().and_then(normalize_term_id)
        {
            current.parents.insert(parent);
        }
    }

    flush_raw_term(&mut current_id, &mut current, terms);
}

/// Parses an RDF/XML OWL dump using a line-oriented scan. Mutates
/// `terms` in place.
pub fn parse_rdfxml_terms(text: &str, terms: &mut BTreeMap<String, RawTerm>) {
    let mut current_id = None::<String>;
    let mut current = RawTerm::default();

    for line in text.lines() {
        if let Some(id) = attribute(line, "rdf:about").and_then(normalize_term_id) {
            flush_raw_term(&mut current_id, &mut current, terms);
            current_id = Some(id);
        }

        if current_id.is_some() {
            if let Some(label) =
                tagged_text(line, "rdfs:label").or_else(|| tagged_text(line, "skos:prefLabel"))
            {
                current.label = Some(label);
            }
            if line.contains("subClassOf")
                && let Some(parent) = attribute(line, "rdf:resource").and_then(normalize_term_id)
            {
                current.parents.insert(parent);
            }
        }

        if line.contains("</owl:Class") || line.contains("</rdf:Description") {
            flush_raw_term(&mut current_id, &mut current, terms);
        }
    }

    flush_raw_term(&mut current_id, &mut current, terms);
}

fn flush_raw_term(
    current_id: &mut Option<String>,
    current: &mut RawTerm,
    terms: &mut BTreeMap<String, RawTerm>,
) {
    if let Some(id) = current_id.take() {
        terms.insert(id, std::mem::take(current));
    }
}

pub(crate) fn attribute<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let start = line.find(&format!("{name}=\""))? + name.len() + 2;
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

pub(crate) fn tagged_text(line: &str, tag: &str) -> Option<String> {
    let start = line.find(&format!("<{tag}"))?;
    let rest = &line[start..];
    let value_start = rest.find('>')? + 1;
    let rest = &rest[value_start..];
    let value_end = rest.find(&format!("</{tag}>"))?;
    Some(unescape_xml(&rest[..value_end]))
}

fn unescape_xml(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_obo_term() {
        let mut terms = BTreeMap::new();
        parse_obo_terms(
            "[Term]\nid: GO:0003700\nname: transcription factor activity\nis_a: GO:0003674 ! molecular_function\n",
            &mut terms,
        );

        let term = terms.get("GO:0003700").unwrap();
        assert_eq!(term.label.as_deref(), Some("transcription factor activity"));
        assert!(term.parents.contains("GO:0003674"));
    }

    #[test]
    fn parses_minimal_rdfxml_term() {
        let mut terms = BTreeMap::new();
        parse_rdfxml_terms(
            r#"<owl:Class rdf:about="http://purl.obolibrary.org/obo/SBO_0000176">
    <rdfs:label>biochemical reaction</rdfs:label>
    <rdfs:subClassOf rdf:resource="http://purl.obolibrary.org/obo/SBO_0000000"/>
</owl:Class>"#,
            &mut terms,
        );

        let term = terms.get("SBO:0000176").unwrap();
        assert_eq!(term.label.as_deref(), Some("biochemical reaction"));
        assert!(term.parents.contains("SBO:0000000"));
    }
}
