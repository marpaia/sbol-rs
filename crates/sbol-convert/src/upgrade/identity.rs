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
use crate::uri;
use sbol_rdf::Graph;
use sbol_rdf::{Iri, Resource, Term, Triple};
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
    ///
    /// A top-level SBOL 2 object maps to `<namespace>/<version>/<displayId>`
    /// ([`uri::create_sbol3_uri`]), and its `hasNamespace` is its SBOL 2 prefix
    /// ([`uri::uri_prefix_sbol2`]). A child nests directly under its owning
    /// top-level's SBOL 3 identity as `<topLevelSbol3>/<relativePath>` — the
    /// version appears once, on the top-level, exactly as the reference
    /// converter's object model produces it. References to objects outside the
    /// document fall back to structural decomposition.
    pub(super) fn build(graph: &Graph) -> Self {
        let mut identified: HashSet<String> = HashSet::new();
        let mut rewrite_candidates: HashSet<String> = HashSet::new();
        // child IRI → owning parent IRI, via SBOL 2 containment predicates.
        let mut owned_by: HashMap<String, String> = HashMap::new();
        // subject IRI → its `persistentIdentity` and `version`, to resolve an
        // unversioned reference to the highest-version object carrying it.
        let mut persistent_identity_of: HashMap<String, String> = HashMap::new();
        let mut version_of: HashMap<String, String> = HashMap::new();

        for triple in graph.triples() {
            let subject = match triple.subject.as_iri() {
                Some(iri) => iri.as_str().to_owned(),
                None => continue,
            };
            match triple.predicate.as_str() {
                v3::RDF_TYPE => {
                    if triple
                        .object
                        .as_iri()
                        .is_some_and(|iri| iri.as_str().starts_with(v2::SBOL2_NS))
                    {
                        rewrite_candidates.insert(subject.clone());
                    }
                }
                v2::SBOL2_PERSISTENT_IDENTITY => {
                    rewrite_candidates.insert(subject.clone());
                    if let Some(iri) = triple.object.as_iri() {
                        persistent_identity_of.insert(subject.clone(), iri.as_str().to_owned());
                    }
                }
                v2::SBOL2_VERSION => {
                    if let Some(lit) = triple.object.as_literal() {
                        version_of.insert(subject.clone(), lit.value().to_owned());
                    }
                }
                v2::SBOL2_DISPLAY_ID => {
                    rewrite_candidates.insert(subject.clone());
                    identified.insert(subject.clone());
                }
                _ => {}
            }

            if is_sbol2_containment_predicate(triple.predicate.as_str())
                && let Some(child) = triple.object.as_iri()
            {
                owned_by.insert(child.as_str().to_owned(), subject.clone());
            }

            if is_sbol2_identity_reference_predicate(triple.predicate.as_str())
                && let Some(iri) = triple.object.as_iri()
            {
                rewrite_candidates.insert(iri.as_str().to_owned());
            }
        }

        // For each `persistentIdentity`, the highest-version subject that
        // carries it. A reference to a bare persistentIdentity resolves to
        // this latest object (the reference converter's `getLatestUri`).
        let mut latest: HashMap<String, String> = HashMap::new();
        for (subject, pid) in &persistent_identity_of {
            let entry = latest.entry(pid.clone()).or_insert_with(|| subject.clone());
            let current = version_of.get(subject).map(String::as_str).unwrap_or("");
            let best = version_of.get(entry).map(String::as_str).unwrap_or("");
            if compare_versions(current, best).is_gt() {
                *entry = subject.clone();
            }
        }

        // A top-level is an identified subject that nothing owns.
        let mut top_info: HashMap<String, TopLevelInfo> = HashMap::new();
        for subject in &identified {
            if owned_by.contains_key(subject) {
                continue;
            }
            let Some(namespace) = uri::uri_prefix_sbol2(subject) else {
                continue;
            };
            let display_id = uri::display_id_sbol2(subject).unwrap_or("");
            top_info.insert(
                subject.clone(),
                TopLevelInfo {
                    namespace: namespace.to_owned(),
                    persistent_identity: format!("{namespace}/{display_id}"),
                    sbol3: uri::create_sbol3_uri(subject),
                },
            );
        }

        let mut rewrites: HashMap<String, String> = HashMap::new();
        let mut namespaces: HashMap<String, String> = HashMap::new();

        for iri in &rewrite_candidates {
            // Resolve a bare persistentIdentity reference to the latest object
            // that carries it before computing the SBOL 3 identity.
            let resolved: &str = latest.get(iri).map(String::as_str).unwrap_or(iri);
            let sbol3 = match owning_top_level(resolved, &owned_by, &top_info) {
                Some(top) => {
                    let info = &top_info[&top];
                    if resolved == top {
                        info.sbol3.clone()
                    } else if let Some(relative) =
                        child_relative_path(resolved, &info.persistent_identity)
                    {
                        format!("{}/{}", info.sbol3, relative)
                    } else {
                        uri::create_sbol3_uri(resolved)
                    }
                }
                None => uri::create_sbol3_uri(resolved),
            };
            if &sbol3 != iri {
                rewrites.insert(iri.clone(), sbol3);
            }
        }

        // `hasNamespace`, keyed by SBOL 3 identity; the caller emits it only
        // for top-levels.
        for (top, info) in &top_info {
            let sbol3 = rewrites.get(top).cloned().unwrap_or_else(|| top.clone());
            namespaces.insert(sbol3, info.namespace.clone());
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

/// Compares dotted version strings (`"1"`, `"1.0"`, `"2.10.3"`) segment by
/// segment: numerically when both segments parse as integers, otherwise
/// lexically. Missing trailing segments count as `0` so `"1"` == `"1.0"`.
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let a_parts: Vec<&str> = a.trim().split('.').collect();
    let b_parts: Vec<&str> = b.trim().split('.').collect();
    for i in 0..a_parts.len().max(b_parts.len()) {
        let sa = a_parts.get(i).copied().unwrap_or("0");
        let sb = b_parts.get(i).copied().unwrap_or("0");
        let ord = match (sa.parse::<i64>(), sb.parse::<i64>()) {
            (Ok(ia), Ok(ib)) => ia.cmp(&ib),
            _ => sa.cmp(sb),
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    Ordering::Equal
}

/// Namespace, persistent identity, and SBOL 3 identity of a top-level object,
/// used to nest its children.
struct TopLevelInfo {
    namespace: String,
    persistent_identity: String,
    sbol3: String,
}

/// SBOL 2 predicates by which a parent owns a child object.
fn is_sbol2_containment_predicate(predicate: &str) -> bool {
    matches!(
        predicate,
        v2::SBOL2_SEQUENCE_ANNOTATION_PROP
            | v2::SBOL2_SEQUENCE_CONSTRAINT_PROP
            | v2::SBOL2_LOCATION_PROP
            | v2::SBOL2_COMPONENT_PROP
            | v2::SBOL2_FUNCTIONAL_COMPONENT_PROP
            | v2::SBOL2_MODULE_PROP
            | v2::SBOL2_INTERACTION_PROP
            | v2::SBOL2_PARTICIPATION_PROP
            | v2::SBOL2_MAPS_TO_PROP
            | v2::SBOL2_VARIABLE_COMPONENT_PROP
    )
}

/// Walks the ownership chain from `iri` to the top-level that transitively
/// owns it. Returns the top-level IRI when it is a known top-level.
fn owning_top_level(
    iri: &str,
    owned_by: &HashMap<String, String>,
    top_info: &HashMap<String, TopLevelInfo>,
) -> Option<String> {
    if top_info.contains_key(iri) {
        return Some(iri.to_owned());
    }
    let mut current = iri.to_owned();
    for _ in 0..64 {
        let parent = owned_by.get(&current)?;
        if top_info.contains_key(parent) {
            return Some(parent.clone());
        }
        current = parent.clone();
    }
    None
}

/// Strips `<persistent_identity>/` and any trailing version segment from a
/// child IRI, yielding its path relative to the owning top-level. `None` when
/// the child does not sit under that identity.
fn child_relative_path(iri: &str, persistent_identity: &str) -> Option<String> {
    let rest = iri.strip_prefix(&format!("{persistent_identity}/"))?;
    match uri::version_sbol2(iri) {
        Some(v) if !v.is_empty() => {
            Some(rest.strip_suffix(&format!("/{v}")).unwrap_or(rest).to_owned())
        }
        _ => Some(rest.to_owned()),
    }
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

