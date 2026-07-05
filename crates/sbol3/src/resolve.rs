//! Reference resolution across SBOL documents.
//!
//! SBOL objects reference each other by IRI. A [`SubComponent`] points at the
//! [`Component`] it instantiates; a [`ComponentReference`] follows a chain of
//! `refersTo` hops down to a leaf [`FeatureRef`]; a
//! [`CombinatorialDerivation`] expands to a set of `Component` variants
//! sourced from explicit variants, `Collection` members, and child
//! `CombinatorialDerivation`s. This module exposes those traversals as
//! methods on the typed structs themselves so that calling code reads as
//! biology, not as library plumbing.
//!
//! Every typed resolution method takes a [`&impl ObjectGraph`], an
//! abstraction over the in-memory scope in which references should resolve.
//! Both [`Document`] and [`DocumentSet`] implement [`ObjectGraph`], so the
//! same method serves single-document and multi-document workflows:
//!
//! ```no_run
//! use sbol3::prelude::*;
//!
//! # fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let main = Document::read_path("design.ttl")?;
//! let parts = Document::read_path("parts_library.ttl")?;
//! let scope = DocumentSet::from_documents([&main, &parts])?;
//!
//! for sub in main.sub_components() {
//!     let def = sub.definition(&scope)?;
//!     println!("{} is a {}", sub.identity, def.identity);
//! }
//! # Ok(()) }
//! ```
//!
//! Failures during traversal (missing fields, dangling IRIs, cycles, or
//! type mismatches) surface as [`ReferenceError`]. This is distinct from
//! [`crate::validation::ResolutionError`], which signals failure to fetch
//! an *external* resource (HTTP, IO, parse). The two never overlap.

use crate::object::ObjectClasses;
use std::collections::BTreeSet;
use std::fmt;

use crate::client::{
    Collection, CombinatorialDerivation, Component, ComponentReference, FeatureRef, SbolObject,
    SubComponent,
};
use crate::validation::DocumentSet;
use crate::vocab::SBOL_REFERS_TO;
use crate::{Document, Object, Resource, SbolClass};

/// In-memory scope of SBOL objects keyed by identity.
///
/// Implemented for [`Document`] (single document) and [`DocumentSet`]
/// (composition of documents). Reference-resolution methods on typed structs
/// ([`SubComponent::definition`], [`ComponentReference::target`],
/// [`ComponentReference::trace`], [`CombinatorialDerivation::variants`])
/// accept any [`ObjectGraph`], so the same code serves both cases.
///
/// Most callers will not invoke trait methods directly; the typed accessors
/// cover the common paths. The trait methods are escape hatches for
/// annotations, extensions, or unrecognized classes.
pub trait ObjectGraph {
    /// Returns the RDF-backed property bag at `iri`, if present.
    ///
    /// Use this for annotations or extension classes that do not have a
    /// typed [`SbolObject`] representation.
    fn get(&self, iri: &Resource) -> Option<&Object>;

    /// Returns the owned typed [`SbolObject`] at `iri`, if present.
    fn resolve(&self, iri: &Resource) -> Option<&SbolObject>;

    /// Iterates every typed object in the scope.
    ///
    /// Order is implementation-defined. For [`DocumentSet`], objects from
    /// later documents follow earlier ones.
    fn iter_typed(&self) -> Box<dyn Iterator<Item = &SbolObject> + '_>;
}

impl ObjectGraph for Document {
    fn get(&self, iri: &Resource) -> Option<&Object> {
        Document::get(self, iri)
    }

    fn resolve(&self, iri: &Resource) -> Option<&SbolObject> {
        Document::resolve(self, iri)
    }

    fn iter_typed(&self) -> Box<dyn Iterator<Item = &SbolObject> + '_> {
        Box::new(self.typed_objects().iter())
    }
}

impl<'a> ObjectGraph for DocumentSet<'a> {
    fn get(&self, iri: &Resource) -> Option<&Object> {
        DocumentSet::get(self, iri)
    }

    fn resolve(&self, iri: &Resource) -> Option<&SbolObject> {
        self.documents()
            .iter()
            .find_map(|doc| Document::resolve(doc, iri))
    }

    fn iter_typed(&self) -> Box<dyn Iterator<Item = &SbolObject> + '_> {
        Box::new(
            self.documents()
                .iter()
                .flat_map(|doc| doc.typed_objects().iter()),
        )
    }
}

/// Failure mode when resolving a typed SBOL reference.
///
/// Distinct from [`crate::validation::ResolutionError`], which represents
/// failure to *fetch* an external resource (HTTP, IO, parse). This error
/// only describes mismatches between a reference and the in-memory
/// [`ObjectGraph`] used to resolve it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReferenceError {
    /// A required reference field was not set on the source object.
    Missing {
        /// Identity of the object whose field is unset.
        on: Resource,
        /// Name of the missing field (`"instanceOf"`, `"refersTo"`, ...).
        field: &'static str,
    },
    /// The referenced IRI has no corresponding object in the graph.
    NotFound(Resource),
    /// The referenced object exists but is the wrong SBOL class.
    WrongType {
        /// IRI of the offending object.
        iri: Resource,
        /// SBOL class name the caller expected.
        expected: &'static str,
        /// SBOL class name actually present.
        found: &'static str,
    },
    /// A cyclic chain of `refersTo` hops was detected.
    ///
    /// The vector lists the chain in the order visited; the final entry is
    /// the one that closed the cycle.
    Cycle(Vec<Resource>),
}

impl fmt::Display for ReferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReferenceError::Missing { on, field } => {
                write!(f, "object {on} is missing required reference `{field}`")
            }
            ReferenceError::NotFound(iri) => {
                write!(f, "referenced object {iri} is not present in the graph")
            }
            ReferenceError::WrongType {
                iri,
                expected,
                found,
            } => write!(
                f,
                "referenced object {iri} has class {found}, expected {expected}"
            ),
            ReferenceError::Cycle(path) => {
                f.write_str("cyclic refersTo chain: ")?;
                for (i, iri) in path.iter().enumerate() {
                    if i > 0 {
                        f.write_str(" -> ")?;
                    }
                    write!(f, "{iri}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ReferenceError {}

/// The result of walking a [`ComponentReference`] chain to its leaf feature.
///
/// `path` lists every `ComponentReference` traversed in order, ending one
/// hop before `target`. For a `ComponentReference` whose `refersTo` already
/// points at a non-reference feature, `path` is empty.
#[derive(Clone, Debug)]
pub struct FeatureTrace<'a> {
    requested: Resource,
    target: FeatureRef<'a>,
    target_iri: Resource,
    path: Vec<Resource>,
}

impl<'a> FeatureTrace<'a> {
    /// IRI the chain was asked to resolve (the first `refersTo` value).
    pub fn requested(&self) -> &Resource {
        &self.requested
    }

    /// The non-`ComponentReference` feature the chain terminates at.
    pub fn target(&self) -> FeatureRef<'a> {
        self.target
    }

    /// Identity of the target feature.
    pub fn target_iri(&self) -> &Resource {
        &self.target_iri
    }

    /// The `ComponentReference` IRIs traversed before reaching the target.
    pub fn path(&self) -> &[Resource] {
        &self.path
    }
}

/// The expanded variant Components for a [`CombinatorialDerivation`].
///
/// Variants are organized by source so callers can distinguish explicitly
/// listed variants from those pulled in through a `Collection` or a child
/// `CombinatorialDerivation`. Use [`VariantSet::flatten`] when source
/// provenance does not matter.
#[derive(Clone, Debug, Default)]
pub struct VariantSet<'a> {
    from_variants: Vec<&'a Component>,
    from_collections: Vec<&'a Component>,
    from_derivations: Vec<&'a Component>,
}

impl<'a> VariantSet<'a> {
    /// Components named explicitly through `VariableFeature.variant`.
    pub fn from_variants(&self) -> &[&'a Component] {
        &self.from_variants
    }

    /// Components reached through `VariableFeature.variantCollection`
    /// members (recursively for nested Collections).
    pub fn from_collections(&self) -> &[&'a Component] {
        &self.from_collections
    }

    /// Components marked `prov:wasDerivedFrom` a CombinatorialDerivation
    /// named in `VariableFeature.variantDerivation`.
    pub fn from_derivations(&self) -> &[&'a Component] {
        &self.from_derivations
    }

    /// All variant Components in source order.
    pub fn flatten(&self) -> impl Iterator<Item = &'a Component> + '_ {
        self.from_variants
            .iter()
            .copied()
            .chain(self.from_collections.iter().copied())
            .chain(self.from_derivations.iter().copied())
    }

    /// `true` when no source produced any variant.
    pub fn is_empty(&self) -> bool {
        self.from_variants.is_empty()
            && self.from_collections.is_empty()
            && self.from_derivations.is_empty()
    }

    /// Total number of variants across every source.
    pub fn len(&self) -> usize {
        self.from_variants.len() + self.from_collections.len() + self.from_derivations.len()
    }
}

impl SubComponent {
    /// Resolves [`SubComponent::instance_of`] to the referenced [`Component`].
    ///
    /// # Errors
    ///
    /// Returns [`ReferenceError::Missing`] when `instance_of` is unset,
    /// [`ReferenceError::NotFound`] when the IRI is not in `graph`, or
    /// [`ReferenceError::WrongType`] when the IRI resolves to a non-Component.
    pub fn definition<'a, G>(&self, graph: &'a G) -> Result<&'a Component, ReferenceError>
    where
        G: ObjectGraph + ?Sized,
    {
        let iri = self
            .instance_of
            .as_ref()
            .ok_or_else(|| ReferenceError::Missing {
                on: self.identity.clone(),
                field: "instanceOf",
            })?;
        let typed = graph
            .resolve(iri)
            .ok_or_else(|| ReferenceError::NotFound(iri.clone()))?;
        match typed {
            SbolObject::Component(component) => Ok(component),
            other => Err(ReferenceError::WrongType {
                iri: iri.clone(),
                expected: "Component",
                found: other.class().local_name(),
            }),
        }
    }
}

impl ComponentReference {
    /// Resolves the leaf [`FeatureRef`] that this reference points at.
    ///
    /// Equivalent to [`ComponentReference::trace`] followed by
    /// [`FeatureTrace::target`]. Use [`trace`](Self::trace) when the
    /// intermediate hops matter.
    pub fn target<'a, G>(&self, graph: &'a G) -> Result<FeatureRef<'a>, ReferenceError>
    where
        G: ObjectGraph + ?Sized,
    {
        Ok(self.trace(graph)?.target())
    }

    /// Walks the `refersTo` chain from this reference to its leaf feature.
    ///
    /// Returns a [`FeatureTrace`] carrying the resolved [`FeatureRef`] plus
    /// the IRIs of every intermediate [`ComponentReference`] crossed.
    ///
    /// # Errors
    ///
    /// - [`ReferenceError::Missing`] when this reference (or a node along
    ///   the chain) has no `refersTo`.
    /// - [`ReferenceError::NotFound`] when an IRI along the chain is absent
    ///   from `graph`.
    /// - [`ReferenceError::WrongType`] when the leaf is not a recognized
    ///   Feature subclass.
    /// - [`ReferenceError::Cycle`] when the chain loops back on itself.
    pub fn trace<'a, G>(&self, graph: &'a G) -> Result<FeatureTrace<'a>, ReferenceError>
    where
        G: ObjectGraph + ?Sized,
    {
        let head = self
            .refers_to
            .as_ref()
            .ok_or_else(|| ReferenceError::Missing {
                on: self.identity.clone(),
                field: "refersTo",
            })?
            .clone();
        let requested = head.clone();
        let mut current = head;
        let mut path = Vec::new();
        let mut visited = BTreeSet::new();

        loop {
            if !visited.insert(current.clone()) {
                path.push(current);
                return Err(ReferenceError::Cycle(path));
            }
            let object = graph
                .get(&current)
                .ok_or_else(|| ReferenceError::NotFound(current.clone()))?;

            if !object.has_class(SbolClass::ComponentReference) {
                let typed = graph
                    .resolve(&current)
                    .ok_or_else(|| ReferenceError::NotFound(current.clone()))?;
                let feature =
                    FeatureRef::from_object(typed).ok_or_else(|| ReferenceError::WrongType {
                        iri: current.clone(),
                        expected: "Feature",
                        found: typed.class().local_name(),
                    })?;
                return Ok(FeatureTrace {
                    requested,
                    target: feature,
                    target_iri: current,
                    path,
                });
            }

            path.push(current.clone());
            let next = object
                .first_resource(SBOL_REFERS_TO)
                .ok_or_else(|| ReferenceError::Missing {
                    on: current.clone(),
                    field: "refersTo",
                })?
                .clone();
            current = next;
        }
    }
}

impl CombinatorialDerivation {
    /// Expands this derivation's [`VariableFeature`](crate::VariableFeature)s
    /// into a [`VariantSet`] of concrete [`Component`]s.
    ///
    /// Variants from every `VariableFeature` are merged into one
    /// [`VariantSet`], grouped by source ([`VariantSet::from_variants`],
    /// [`VariantSet::from_collections`], [`VariantSet::from_derivations`]).
    /// Components are de-duplicated by identity within each source.
    ///
    /// # Errors
    ///
    /// Returns [`ReferenceError::NotFound`] for any IRI on a
    /// `VariableFeature`'s `variant`, `variantCollection`, or
    /// `variantDerivation` that is absent from `graph`, and
    /// [`ReferenceError::WrongType`] when a referenced IRI resolves to
    /// the wrong SBOL class.
    pub fn variants<'a, G>(&self, graph: &'a G) -> Result<VariantSet<'a>, ReferenceError>
    where
        G: ObjectGraph + ?Sized,
    {
        let mut from_variants_iris: BTreeSet<Resource> = BTreeSet::new();
        let mut from_collections_iris: BTreeSet<Resource> = BTreeSet::new();
        let mut from_derivations_iris: BTreeSet<Resource> = BTreeSet::new();
        let mut from_variants = Vec::new();
        let mut from_collections = Vec::new();
        let mut from_derivations = Vec::new();

        for vf_iri in &self.variable_features {
            let vf_typed = graph
                .resolve(vf_iri)
                .ok_or_else(|| ReferenceError::NotFound(vf_iri.clone()))?;
            let variable_feature = match vf_typed {
                SbolObject::VariableFeature(v) => v,
                other => {
                    return Err(ReferenceError::WrongType {
                        iri: vf_iri.clone(),
                        expected: "VariableFeature",
                        found: other.class().local_name(),
                    });
                }
            };

            for variant_iri in &variable_feature.variants {
                if !from_variants_iris.insert(variant_iri.clone()) {
                    continue;
                }
                let typed = graph
                    .resolve(variant_iri)
                    .ok_or_else(|| ReferenceError::NotFound(variant_iri.clone()))?;
                match typed {
                    SbolObject::Component(c) => from_variants.push(c),
                    other => {
                        return Err(ReferenceError::WrongType {
                            iri: variant_iri.clone(),
                            expected: "Component",
                            found: other.class().local_name(),
                        });
                    }
                }
            }

            for collection_iri in &variable_feature.variant_collections {
                let mut visited = BTreeSet::new();
                collect_collection_components(
                    graph,
                    collection_iri,
                    &mut visited,
                    &mut from_collections_iris,
                    &mut from_collections,
                )?;
            }

            for derivation_iri in &variable_feature.variant_derivations {
                let typed = graph
                    .resolve(derivation_iri)
                    .ok_or_else(|| ReferenceError::NotFound(derivation_iri.clone()))?;
                if !matches!(typed, SbolObject::CombinatorialDerivation(_)) {
                    return Err(ReferenceError::WrongType {
                        iri: derivation_iri.clone(),
                        expected: "CombinatorialDerivation",
                        found: typed.class().local_name(),
                    });
                }
                for object in graph.iter_typed() {
                    let SbolObject::Component(component) = object else {
                        continue;
                    };
                    if !component
                        .identified
                        .derived_from
                        .iter()
                        .any(|src| src == derivation_iri)
                    {
                        continue;
                    }
                    if from_derivations_iris.insert(component.identity.clone()) {
                        from_derivations.push(component);
                    }
                }
            }
        }

        Ok(VariantSet {
            from_variants,
            from_collections,
            from_derivations,
        })
    }

    /// Resolves [`CombinatorialDerivation::template`] to the referenced
    /// [`Component`].
    ///
    /// # Errors
    ///
    /// Returns [`ReferenceError::Missing`] when `template` is unset,
    /// [`ReferenceError::NotFound`] when the IRI is not in `graph`, or
    /// [`ReferenceError::WrongType`] when the IRI resolves to a non-Component.
    pub fn template_component<'a, G>(&self, graph: &'a G) -> Result<&'a Component, ReferenceError>
    where
        G: ObjectGraph + ?Sized,
    {
        let iri = self
            .template
            .as_ref()
            .ok_or_else(|| ReferenceError::Missing {
                on: self.identity.clone(),
                field: "template",
            })?;
        let typed = graph
            .resolve(iri)
            .ok_or_else(|| ReferenceError::NotFound(iri.clone()))?;
        match typed {
            SbolObject::Component(c) => Ok(c),
            other => Err(ReferenceError::WrongType {
                iri: iri.clone(),
                expected: "Component",
                found: other.class().local_name(),
            }),
        }
    }
}

fn collect_collection_components<'a, G>(
    graph: &'a G,
    collection_iri: &Resource,
    visited: &mut BTreeSet<Resource>,
    seen: &mut BTreeSet<Resource>,
    out: &mut Vec<&'a Component>,
) -> Result<(), ReferenceError>
where
    G: ObjectGraph + ?Sized,
{
    if !visited.insert(collection_iri.clone()) {
        return Ok(());
    }
    let typed = graph
        .resolve(collection_iri)
        .ok_or_else(|| ReferenceError::NotFound(collection_iri.clone()))?;
    let collection: &Collection = match typed {
        SbolObject::Collection(c) => c,
        other => {
            return Err(ReferenceError::WrongType {
                iri: collection_iri.clone(),
                expected: "Collection",
                found: other.class().local_name(),
            });
        }
    };

    for member_iri in &collection.members {
        let Some(member) = graph.resolve(member_iri) else {
            return Err(ReferenceError::NotFound(member_iri.clone()));
        };
        match member {
            SbolObject::Component(component) => {
                if seen.insert(component.identity.clone()) {
                    out.push(component);
                }
            }
            SbolObject::Collection(_) => {
                collect_collection_components(graph, member_iri, visited, seen, out)?;
            }
            _ => {
                // Members may legitimately be non-Component TopLevels (e.g.
                // Sequences, Models). Variant expansion only takes the
                // Components; other members are silently skipped, matching
                // the validator's behavior.
            }
        }
    }

    Ok(())
}
