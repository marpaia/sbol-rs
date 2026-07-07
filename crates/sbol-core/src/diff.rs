//! Structural comparison of two SBOL documents.
//!
//! A [`Diff`] compares documents by object identity rather than by their
//! serialized text: every SBOL object carries a stable identity IRI, so
//! matching on it ignores serialization order, blank-node labeling, and RDF
//! format. Two serializations of the same document diff clean, and a genuine
//! change surfaces as the object whose properties moved.
//!
//! The comparison is version-neutral: it operates on any [`ObjectStore`], so
//! it applies to SBOL 2 and SBOL 3 documents alike. It is meaningful only
//! *within* a version, because SBOL 2 and SBOL 3 use different identity
//! schemes, RDF type IRIs, and predicate IRIs — a raw comparison across
//! versions reports every object as removed and re-added. To compare a
//! document across versions, upgrade the SBOL 2 document to SBOL 3 first and
//! diff the two SBOL 3 documents.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use sbol_rdf::{Iri, Resource, Term};

use crate::document::ObjectStore;
use crate::object::Object;

/// The identity-keyed difference between two SBOL documents.
///
/// Objects present in the new document but not the old are [`added`], those
/// present in the old but not the new are [`removed`], and those present in
/// both whose types or properties changed are [`changed`]. Objects that are
/// byte-for-byte identical under a given identity appear in none of the three.
///
/// [`added`]: Diff::added
/// [`removed`]: Diff::removed
/// [`changed`]: Diff::changed
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Diff {
    added: Vec<Resource>,
    removed: Vec<Resource>,
    changed: Vec<ObjectDiff>,
}

impl Diff {
    /// Computes the difference between an `old` and a `new` document, matching
    /// objects by identity.
    pub fn compute<O: ObjectStore + ?Sized, N: ObjectStore + ?Sized>(old: &O, new: &N) -> Self {
        let old_objects = old.objects();
        let new_objects = new.objects();

        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();

        for (identity, new_object) in new_objects {
            match old_objects.get(identity) {
                None => added.push(identity.clone()),
                Some(old_object) => {
                    if let Some(object_diff) = ObjectDiff::compute(old_object, new_object) {
                        changed.push(object_diff);
                    }
                }
            }
        }

        for identity in old_objects.keys() {
            if !new_objects.contains_key(identity) {
                removed.push(identity.clone());
            }
        }

        Self {
            added,
            removed,
            changed,
        }
    }

    /// Identities present in the new document but absent from the old.
    pub fn added(&self) -> &[Resource] {
        &self.added
    }

    /// Identities present in the old document but absent from the new.
    pub fn removed(&self) -> &[Resource] {
        &self.removed
    }

    /// Objects present in both documents whose types or properties changed.
    pub fn changed(&self) -> &[ObjectDiff] {
        &self.changed
    }

    /// Returns `true` when the two documents contain the same objects with the
    /// same types and properties.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }
}

impl fmt::Display for Diff {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(formatter, "no differences");
        }

        for identity in &self.added {
            writeln!(formatter, "+ {identity}")?;
        }
        for identity in &self.removed {
            writeln!(formatter, "- {identity}")?;
        }
        for object in &self.changed {
            write!(formatter, "{object}")?;
        }
        Ok(())
    }
}

/// The change to a single object present in both documents.
///
/// RDF types gained appear in [`types_added`] and those lost in
/// [`types_removed`]. Property changes are keyed by predicate; each
/// [`PropertyChange`] records the terms that appeared and disappeared for that
/// predicate.
///
/// [`types_added`]: ObjectDiff::types_added
/// [`types_removed`]: ObjectDiff::types_removed
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ObjectDiff {
    identity: Resource,
    types_added: BTreeSet<Iri>,
    types_removed: BTreeSet<Iri>,
    properties: BTreeMap<Iri, PropertyChange>,
}

impl ObjectDiff {
    /// Computes the change between two objects sharing an identity, returning
    /// `None` when they are identical.
    fn compute(old: &Object, new: &Object) -> Option<Self> {
        let types_added: BTreeSet<Iri> = new
            .rdf_types()
            .difference(old.rdf_types())
            .cloned()
            .collect();
        let types_removed: BTreeSet<Iri> = old
            .rdf_types()
            .difference(new.rdf_types())
            .cloned()
            .collect();

        let mut properties = BTreeMap::new();
        let predicates: BTreeSet<&Iri> = old
            .properties()
            .keys()
            .chain(new.properties().keys())
            .collect();
        for predicate in predicates {
            let old_terms = old
                .properties()
                .get(predicate)
                .map_or(&[][..], Vec::as_slice);
            let new_terms = new
                .properties()
                .get(predicate)
                .map_or(&[][..], Vec::as_slice);
            if let Some(change) = PropertyChange::compute(old_terms, new_terms) {
                properties.insert(predicate.clone(), change);
            }
        }

        if types_added.is_empty() && types_removed.is_empty() && properties.is_empty() {
            return None;
        }

        Some(Self {
            identity: new.identity().clone(),
            types_added,
            types_removed,
            properties,
        })
    }

    /// The identity of the object that changed.
    pub fn identity(&self) -> &Resource {
        &self.identity
    }

    /// RDF types the object gained.
    pub fn types_added(&self) -> &BTreeSet<Iri> {
        &self.types_added
    }

    /// RDF types the object lost.
    pub fn types_removed(&self) -> &BTreeSet<Iri> {
        &self.types_removed
    }

    /// The per-predicate property changes, keyed by predicate IRI.
    pub fn properties(&self) -> &BTreeMap<Iri, PropertyChange> {
        &self.properties
    }
}

impl fmt::Display for ObjectDiff {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "~ {}", self.identity)?;
        for iri in &self.types_added {
            writeln!(formatter, "    + type {iri}")?;
        }
        for iri in &self.types_removed {
            writeln!(formatter, "    - type {iri}")?;
        }
        for (predicate, change) in &self.properties {
            writeln!(formatter, "    {predicate}")?;
            for term in &change.added {
                writeln!(formatter, "        + {}", display_term(term))?;
            }
            for term in &change.removed {
                writeln!(formatter, "        - {}", display_term(term))?;
            }
        }
        Ok(())
    }
}

/// The change to the values of one predicate on a changed object.
///
/// Values are compared as multisets: a predicate carrying the same terms in a
/// different order yields no change. [`added`] holds terms present in the new
/// document, [`removed`] those present in the old.
///
/// [`added`]: PropertyChange::added
/// [`removed`]: PropertyChange::removed
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PropertyChange {
    added: Vec<Term>,
    removed: Vec<Term>,
}

impl PropertyChange {
    /// Computes the multiset difference between the old and new values of a
    /// predicate, returning `None` when they carry the same terms.
    fn compute(old: &[Term], new: &[Term]) -> Option<Self> {
        let mut counts: BTreeMap<&Term, i64> = BTreeMap::new();
        for term in old {
            *counts.entry(term).or_default() -= 1;
        }
        for term in new {
            *counts.entry(term).or_default() += 1;
        }

        let mut added = Vec::new();
        let mut removed = Vec::new();
        for (term, count) in counts {
            if count > 0 {
                added.extend(std::iter::repeat_n(term.clone(), count as usize));
            } else if count < 0 {
                removed.extend(std::iter::repeat_n(term.clone(), (-count) as usize));
            }
        }

        if added.is_empty() && removed.is_empty() {
            return None;
        }
        Some(Self { added, removed })
    }

    /// Terms present in the new document but not the old.
    pub fn added(&self) -> &[Term] {
        &self.added
    }

    /// Terms present in the old document but not the new.
    pub fn removed(&self) -> &[Term] {
        &self.removed
    }
}

/// Renders a term for the human-readable diff: resources as their IRI or blank
/// node, literals as their quoted lexical value.
fn display_term(term: &Term) -> String {
    match term {
        Term::Resource(resource) => resource.to_string(),
        Term::Literal(literal) => {
            let value = format!("{:?}", literal.value());
            match literal.language() {
                Some(language) => format!("{value}@{language}"),
                None => value,
            }
        }
        _ => format!("{term:?}"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use sbol_rdf::{Iri, Literal, Resource, Term};

    use super::*;
    use crate::object::{Identified, Object};

    fn iri(value: &str) -> Iri {
        Iri::new_unchecked(value)
    }

    fn resource(value: &str) -> Resource {
        Resource::Iri(iri(value))
    }

    fn literal(value: &str) -> Term {
        Term::Literal(Literal::simple(value))
    }

    /// A minimal object store backed by an in-memory map.
    struct MapStore(BTreeMap<Resource, Object>);

    impl ObjectStore for MapStore {
        fn objects(&self) -> &BTreeMap<Resource, Object> {
            &self.0
        }

        fn get(&self, identity: &Resource) -> Option<&Object> {
            self.0.get(identity)
        }
    }

    fn object(
        identity: &str,
        types: &[&str],
        properties: &[(&str, &[Term])],
    ) -> (Resource, Object) {
        let id = resource(identity);
        let rdf_types: BTreeSet<Iri> = types.iter().map(|t| iri(t)).collect();
        let props: BTreeMap<Iri, Vec<Term>> = properties
            .iter()
            .map(|(predicate, terms)| (iri(predicate), terms.to_vec()))
            .collect();
        let object = Object::from_parts(id.clone(), rdf_types, props, Identified::default(), None);
        (id, object)
    }

    fn store(objects: impl IntoIterator<Item = (Resource, Object)>) -> MapStore {
        MapStore(objects.into_iter().collect())
    }

    #[test]
    fn identical_documents_have_no_diff() {
        let a = store([object("https://ex.org/a", &["https://ex.org/T"], &[])]);
        let b = store([object("https://ex.org/a", &["https://ex.org/T"], &[])]);
        let diff = Diff::compute(&a, &b);
        assert!(diff.is_empty());
        assert_eq!(diff.to_string(), "no differences");
    }

    #[test]
    fn added_and_removed_objects_are_partitioned() {
        let old = store([object("https://ex.org/a", &[], &[])]);
        let new = store([object("https://ex.org/b", &[], &[])]);
        let diff = Diff::compute(&old, &new);
        assert_eq!(diff.added(), &[resource("https://ex.org/b")]);
        assert_eq!(diff.removed(), &[resource("https://ex.org/a")]);
        assert!(diff.changed().is_empty());
    }

    #[test]
    fn property_value_change_is_reported_per_predicate() {
        let old = store([object(
            "https://ex.org/a",
            &[],
            &[("https://ex.org/name", &[literal("old")])],
        )]);
        let new = store([object(
            "https://ex.org/a",
            &[],
            &[("https://ex.org/name", &[literal("new")])],
        )]);
        let diff = Diff::compute(&old, &new);
        assert!(diff.added().is_empty());
        assert!(diff.removed().is_empty());
        assert_eq!(diff.changed().len(), 1);

        let object = &diff.changed()[0];
        assert_eq!(object.identity(), &resource("https://ex.org/a"));
        let change = &object.properties()[&iri("https://ex.org/name")];
        assert_eq!(change.added(), &[literal("new")]);
        assert_eq!(change.removed(), &[literal("old")]);
    }

    #[test]
    fn type_changes_are_reported() {
        let old = store([object("https://ex.org/a", &["https://ex.org/Old"], &[])]);
        let new = store([object("https://ex.org/a", &["https://ex.org/New"], &[])]);
        let diff = Diff::compute(&old, &new);
        let object = &diff.changed()[0];
        assert_eq!(
            object.types_added(),
            &BTreeSet::from([iri("https://ex.org/New")])
        );
        assert_eq!(
            object.types_removed(),
            &BTreeSet::from([iri("https://ex.org/Old")])
        );
    }

    #[test]
    fn reordered_property_values_are_not_a_change() {
        let old = store([object(
            "https://ex.org/a",
            &[],
            &[("https://ex.org/role", &[literal("x"), literal("y")])],
        )]);
        let new = store([object(
            "https://ex.org/a",
            &[],
            &[("https://ex.org/role", &[literal("y"), literal("x")])],
        )]);
        let diff = Diff::compute(&old, &new);
        assert!(diff.is_empty());
    }

    #[test]
    fn added_value_to_multivalued_predicate_is_reported() {
        let old = store([object(
            "https://ex.org/a",
            &[],
            &[("https://ex.org/role", &[literal("x")])],
        )]);
        let new = store([object(
            "https://ex.org/a",
            &[],
            &[("https://ex.org/role", &[literal("x"), literal("y")])],
        )]);
        let diff = Diff::compute(&old, &new);
        let change = &diff.changed()[0].properties()[&iri("https://ex.org/role")];
        assert_eq!(change.added(), &[literal("y")]);
        assert!(change.removed().is_empty());
    }
}
