//! Identity transforms for the SBOL 2 → SBOL 3 upgrade.
//!
//! SBOL 2 identities embed a version segment as the last path component
//! (compliant pattern: `<namespace>/<displayId>/<version>`). SBOL 3 does not
//! version IRIs, so the upgrade needs to:
//!
//! 1. Compute the canonical SBOL 3 identity (the SBOL 2 `persistentIdentity`
//!    when present, otherwise the original IRI with a trailing
//!    `/digits(.digits)*` segment stripped).
//! 2. Apply that rename to every subject and every reference (object position)
//!    in the graph.
//! 3. For top-level objects, derive `sbol3:hasNamespace` from the
//!    persistentIdentity minus the displayId, falling back to the URL
//!    scheme+host when neither path is available.

use std::collections::{HashMap, HashSet};

use crate::sbol2_vocab as v2;
use sbol_rdf::Graph;
use sbol_rdf::{Iri, Literal, Resource, Term, Triple};
use sbol3::vocab as v3;

/// Result of pre-scanning the SBOL 2 graph for everything needed to compute
/// stable SBOL 3 identities and namespaces.
pub(super) struct IdentityMap {
    /// Map from original SBOL 2 IRI string → SBOL 3 IRI string. Includes
    /// every subject the upgrade may emit; references not in this map remain
    /// unchanged.
    rewrites: HashMap<String, String>,
    /// Map from rewritten (SBOL 3) IRI → derived `hasNamespace` value, for
    /// subjects whose original SBOL 2 form had a `displayId` and either an
    /// explicit `persistentIdentity` or a displayId-shaped identity IRI. Only
    /// populated for top-level objects (caller filters by rdf:type).
    namespaces: HashMap<String, String>,
}

impl IdentityMap {
    /// Builds the identity map from a parsed SBOL 2 graph.
    pub(super) fn build(graph: &Graph) -> Self {
        let mut persistent_identity: HashMap<String, String> = HashMap::new();
        let mut display_id: HashMap<String, String> = HashMap::new();
        let mut rewrite_candidates: HashSet<String> = HashSet::new();
        // Subject IRI → SBOL 3 namespace stashed by an sbol-utilities /
        // sbolgraph downgrade under `backport:sbol3namespace`. When present
        // it overrides the persistentIdentity/displayId derivation below.
        let mut sbol3_namespace_hints: HashMap<String, String> = HashMap::new();

        for triple in graph.triples() {
            let subject = match triple.subject.as_iri() {
                Some(iri) => iri.as_str().to_owned(),
                None => continue,
            };
            if triple.predicate.as_str() == v2::BACKPORT_SBOL3_NAMESPACE {
                if let Some(iri) = triple.object.as_iri() {
                    sbol3_namespace_hints.insert(subject.clone(), iri.as_str().to_owned());
                } else if let Some(value) = literal_value(&triple.object) {
                    sbol3_namespace_hints.insert(subject.clone(), value.to_owned());
                }
            }
            match triple.predicate.as_str() {
                v3::RDF_TYPE => {
                    if triple
                        .object
                        .as_iri()
                        .is_some_and(|iri| iri.as_str().starts_with(v2::SBOL2_NS))
                    {
                        rewrite_candidates.insert(subject);
                    }
                }
                v2::SBOL2_PERSISTENT_IDENTITY => {
                    rewrite_candidates.insert(subject.clone());
                    if let Some(value) = literal_value(&triple.object) {
                        persistent_identity.insert(subject, value.to_owned());
                    } else if let Some(iri) = triple.object.as_iri() {
                        // Some serializations emit persistentIdentity as an IRI
                        // object rather than a string literal.
                        persistent_identity.insert(subject, iri.as_str().to_owned());
                    }
                }
                v2::SBOL2_DISPLAY_ID => {
                    if let Some(value) = literal_value(&triple.object) {
                        display_id.insert(subject, value.to_owned());
                    }
                }
                _ => {}
            }

            if is_sbol2_identity_reference_predicate(triple.predicate.as_str())
                && let Some(iri) = triple.object.as_iri()
            {
                rewrite_candidates.insert(iri.as_str().to_owned());
            }
        }

        let mut rewrites: HashMap<String, String> = HashMap::new();
        let mut namespaces: HashMap<String, String> = HashMap::new();

        for iri in rewrite_candidates {
            let canonical = canonical_identity(&iri, &persistent_identity);
            if canonical != iri {
                rewrites.insert(iri, canonical);
            }
        }

        for (original, did) in display_id.iter() {
            let canonical = rewrites
                .get(original)
                .cloned()
                .unwrap_or_else(|| original.clone());
            let namespace = persistent_identity
                .get(original)
                .and_then(|pid| strip_suffix_segment(pid, did))
                .or_else(|| strip_suffix_segment(&canonical, did))
                .map(str::to_owned);
            if let Some(namespace) = namespace {
                namespaces.insert(canonical, namespace);
            }
        }

        // An explicit `backport:sbol3namespace` annotation wins over the
        // derived namespace: it records exactly what the SBOL 3 source
        // (round-tripped through sbol-rs, sbol-utilities, or sbolgraph)
        // carried, which derivation can only approximate.
        for (subject, namespace) in sbol3_namespace_hints {
            let canonical = rewrites.get(&subject).cloned().unwrap_or(subject);
            namespaces.insert(canonical, namespace);
        }

        Self {
            rewrites,
            namespaces,
        }
    }

    /// Rewrites a single subject IRI to its SBOL 3 form. Returns the
    /// (possibly unchanged) IRI string.
    pub(super) fn rewrite_iri<'a>(&'a self, iri: &'a str) -> &'a str {
        match self.rewrites.get(iri) {
            Some(new) => new.as_str(),
            None => iri,
        }
    }

    /// Inserts an additional IRI rewrite. Used by the engine after preflight
    /// to record relocations that depend on type-aware analysis (e.g.
    /// migrating a Location IRI from under a collapsed SequenceAnnotation
    /// to its new SubComponent parent).
    pub(super) fn add_rewrite(&mut self, from: String, to: String) {
        self.rewrites.insert(from, to);
    }

    /// Returns the derived `hasNamespace` value for a top-level subject, if
    /// one could be computed from `displayId` plus either
    /// `persistentIdentity` or the subject identity itself.
    pub(super) fn namespace_for(&self, iri: &str) -> Option<&str> {
        self.namespaces.get(iri).map(String::as_str)
    }

    pub(super) fn rewrite_resource(&self, resource: &Resource) -> Resource {
        match resource {
            Resource::Iri(iri) => {
                let new = self.rewrite_iri(iri.as_str());
                if new == iri.as_str() {
                    resource.clone()
                } else {
                    Resource::Iri(Iri::new_unchecked(new))
                }
            }
            _ => resource.clone(),
        }
    }

    pub(super) fn rewrite_term(&self, term: &Term) -> Term {
        match term {
            Term::Resource(resource) => Term::Resource(self.rewrite_resource(resource)),
            _ => term.clone(),
        }
    }

    pub(super) fn rewrite_triple(&self, triple: &Triple) -> Triple {
        Triple {
            subject: self.rewrite_resource(&triple.subject),
            predicate: triple.predicate.clone(),
            object: self.rewrite_term(&triple.object),
        }
    }
}

/// Computes the SBOL 3 identity from an SBOL 2 IRI. Prefers an explicit
/// `persistentIdentity` (canonical case); otherwise falls back to stripping a
/// trailing `/digits(.digits)*` segment.
fn canonical_identity(iri: &str, persistent_identity: &HashMap<String, String>) -> String {
    if let Some(pid) = persistent_identity.get(iri) {
        return pid.clone();
    }
    strip_trailing_version(iri).unwrap_or_else(|| iri.to_owned())
}

/// Strips a trailing version segment (one or more dot-separated numeric
/// components) preceded by `/` for URL-form IRIs, or by `:` for URN-form
/// IRIs (`urn:nid:…`). Returns `None` if no such suffix is present.
///
/// URN-form `:` stripping is gated on the `urn:` prefix because URL-form
/// IRIs commonly contain colons in path segments (e.g. ontology terms like
/// `http://identifiers.org/so/SO:0000167`) that must NOT be confused with
/// version suffixes.
pub(super) fn strip_trailing_version(iri: &str) -> Option<String> {
    let is_urn = iri.starts_with("urn:");
    let last_slash = iri.rfind('/');
    let sep_pos = if is_urn {
        let last_colon = iri.rfind(':');
        match (last_slash, last_colon) {
            (Some(s), Some(c)) if s > c => Some(s),
            (Some(s), None) => Some(s),
            (_, Some(c)) => Some(c),
            (None, None) => None,
        }
    } else {
        last_slash
    }?;
    let tail = &iri[sep_pos + 1..];
    if tail.is_empty() {
        return None;
    }
    if !tail
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        return None;
    }
    if !tail.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    Some(iri[..sep_pos].to_owned())
}

/// Removes a trailing `{separator}<segment>` from `value` where `separator`
/// is either `/` (URL form) or `:` (URN form). Returns the namespace prefix
/// without the displayId, or `None` when the value doesn't end with the
/// expected suffix.
fn strip_suffix_segment<'a>(value: &'a str, segment: &str) -> Option<&'a str> {
    for separator in ['/', ':'] {
        let suffix = format!("{separator}{segment}");
        if let Some(stripped) = value.strip_suffix(&suffix) {
            return Some(stripped);
        }
    }
    None
}

fn is_sbol2_identity_reference_predicate(predicate: &str) -> bool {
    matches!(
        predicate,
        v2::SBOL2_BUILT
            | v2::SBOL2_SEQUENCE_PROP
            | v2::SBOL2_SEQUENCE_ANNOTATION_PROP
            | v2::SBOL2_SEQUENCE_CONSTRAINT_PROP
            | v2::SBOL2_COMPONENT_PROP
            | v2::SBOL2_FUNCTIONAL_COMPONENT_PROP
            | v2::SBOL2_MODULE_PROP
            | v2::SBOL2_INTERACTION_PROP
            | v2::SBOL2_PARTICIPATION_PROP
            | v2::SBOL2_LOCATION_PROP
            | v2::SBOL2_DEFINITION
            | v2::SBOL2_VARIABLE_COMPONENT_PROP
            | v2::SBOL2_VARIABLE
            | v2::SBOL2_VARIANT
            | v2::SBOL2_VARIANT_COLLECTION
            | v2::SBOL2_VARIANT_DERIVATION
            | v2::SBOL2_MODEL_PROP
            | v2::SBOL2_ATTACHMENT_PROP
            | v2::SBOL2_SUBJECT
            | v2::SBOL2_OBJECT
            | v2::SBOL2_PARTICIPANT
            | v2::SBOL2_LOCAL
            | v2::SBOL2_REMOTE
            | v2::SBOL2_MAPS_TO_PROP
            | v2::SBOL2_TEMPLATE
            | v2::SBOL2_MEMBER
            | v2::SBOL2_EXPERIMENTAL_DATA_PROP
    )
}

fn literal_value(term: &Term) -> Option<&str> {
    term.as_literal().map(Literal::value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_single_digit_version() {
        assert_eq!(
            strip_trailing_version("https://synbiohub.org/public/igem/BBa_E0040/1").as_deref(),
            Some("https://synbiohub.org/public/igem/BBa_E0040"),
        );
    }

    #[test]
    fn strips_semantic_version() {
        assert_eq!(
            strip_trailing_version("https://example.org/lab/design/1.2.3").as_deref(),
            Some("https://example.org/lab/design"),
        );
    }

    #[test]
    fn preserves_iri_without_version_suffix() {
        assert_eq!(
            strip_trailing_version("https://example.org/lab/design").as_deref(),
            None,
        );
    }

    #[test]
    fn preserves_non_numeric_suffix() {
        assert_eq!(
            strip_trailing_version("https://example.org/lab/promoter_v1").as_deref(),
            None,
        );
    }
}
