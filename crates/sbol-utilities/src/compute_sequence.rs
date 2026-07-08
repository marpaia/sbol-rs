//! Computing the sequence of an assembled region from its ordered parts.
//!
//! [`compute_sequence`] takes a finished [`Document`] and a region `Component`
//! whose sub-components are chained head-to-tail with `meets` constraints (as
//! produced by [`engineered_region`](crate::ComponentVerbs::engineered_region)),
//! concatenates the parts' sequences into one `Sequence` for the region, and
//! attaches a 1-based inclusive `Range` location to each sub-component marking
//! where it sits in the composed sequence. It mirrors the Python
//! `sbol_utilities.calculate_sequences.compute_sequence`, which orders parts by
//! `SBOL_MEETS`.
//!
//! Because the core [`Document`] is immutable, these functions return a new
//! `Document` with the computed objects added rather than mutating in place.

use std::collections::{HashMap, HashSet};

use sbol3::constants::{EDAM_IUPAC_DNA, RESTRICTION_MEETS, SO_ENGINEERED_REGION};
use sbol3::{
    BuildError, Component, Document, Iri, Range, Resource, SbolIdentified, SbolObject,
    SbolTopLevel, Sequence, SubComponent,
};

/// A problem encountered while computing an assembled region's sequence.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ComputeSequenceError {
    /// The requested region IRI is not in the document.
    #[error("region `{0}` was not found in the document")]
    RegionNotFound(String),
    /// The requested IRI resolves to something other than a `Component`.
    #[error("`{0}` is not a Component")]
    NotAComponent(String),
    /// The region has no features to compose a sequence from.
    #[error("region `{0}` has no features to compute a sequence from")]
    NoFeatures(String),
    /// A feature is not a `SubComponent`; only sub-components can be ordered.
    #[error("region `{region}` feature `{feature}` is not a SubComponent")]
    UnsupportedFeature {
        /// The region being composed.
        region: String,
        /// The offending feature IRI.
        feature: String,
    },
    /// The sub-components do not form a single unambiguous linear `meets` chain.
    #[error(
        "region `{0}` sub-components cannot be unambiguously ordered by `meets` \
         constraints (a single linear chain is required)"
    )]
    Unorderable(String),
    /// A sub-component does not declare what component it instantiates.
    #[error("sub-component `{0}` has no instanceOf component")]
    MissingInstanceOf(String),
    /// A sub-component's `instanceOf` (or that component's sequence) does not
    /// resolve within the document.
    #[error("`{reference}` referenced from `{owner}` does not resolve to a usable object")]
    Unresolved {
        /// The object holding the dangling reference.
        owner: String,
        /// The reference that failed to resolve.
        reference: String,
    },
    /// A part must have exactly one sequence to be composed.
    #[error("part `{part}` must have exactly one sequence to be composed (found {count})")]
    PartSequenceCount {
        /// The part component.
        part: String,
        /// How many sequences it actually has.
        count: usize,
    },
    /// A part's sequence carries no `elements` string.
    #[error("part `{0}` has a sequence with no elements")]
    PartMissingElements(String),
    /// Parts within a region disagree on their sequence encoding.
    #[error("parts of a region must share one sequence encoding; found more than one")]
    EncodingMismatch,
    /// The region has no namespace (validate the document first).
    #[error("region `{0}` has no namespace; validate the document before computing sequences")]
    MissingNamespace(String),
    /// The region has no `displayId`.
    #[error("region `{0}` has no displayId")]
    MissingDisplayId(String),
    /// Building one of the computed objects failed.
    #[error("failed to build a computed object: {0}")]
    Build(#[from] BuildError),
    /// Reassembling the document with the computed objects failed.
    #[error("failed to reassemble the document: {0}")]
    Assembly(BuildError),
}

/// The computed objects for one region, ready to be merged into a document.
struct RegionPlan {
    region: Component,
    sequence: Sequence,
    ranges: Vec<Range>,
    sub_components: Vec<SubComponent>,
}

/// Computes the sequence of a single region and returns a new [`Document`] with
/// the region's `Sequence`, its per-sub-component `Range` locations, and the
/// updated `Component`/`SubComponent`s merged in.
///
/// The region's features must be sub-components chained head-to-tail with
/// `meets` constraints and each must instantiate a component carrying exactly
/// one sequence, all sharing an encoding.
pub fn compute_sequence(
    document: &Document,
    region: &Resource,
) -> Result<Document, ComputeSequenceError> {
    let component = match document.resolve(region) {
        Some(SbolObject::Component(component)) => component,
        Some(_) => return Err(ComputeSequenceError::NotAComponent(region.to_string())),
        None => return Err(ComputeSequenceError::RegionNotFound(region.to_string())),
    };
    let plan = plan_region(document, component)?;
    apply(document, vec![plan])
}

/// Computes sequences for every engineered region in the document that has
/// features but no sequence yet, returning a new [`Document`] with all of them
/// merged in. Regions that already have a sequence are left untouched.
pub fn compute_all_sequences(document: &Document) -> Result<Document, ComputeSequenceError> {
    let mut plans = Vec::new();
    for component in document.components() {
        let is_region = component
            .roles
            .iter()
            .any(|role| role == &SO_ENGINEERED_REGION);
        if is_region && component.sequences.is_empty() && !component.features.is_empty() {
            plans.push(plan_region(document, component)?);
        }
    }
    apply(document, plans)
}

fn plan_region(
    document: &Document,
    region: &Component,
) -> Result<RegionPlan, ComputeSequenceError> {
    let region_display = region
        .display_id()
        .ok_or_else(|| ComputeSequenceError::MissingDisplayId(region.identity.to_string()))?;
    let namespace = region
        .namespace()
        .ok_or_else(|| ComputeSequenceError::MissingNamespace(region_display.to_string()))?;

    let mut sub_by_iri: HashMap<&Resource, &SubComponent> = HashMap::new();
    for feature in &region.features {
        match document.resolve(feature) {
            Some(SbolObject::SubComponent(sub)) => {
                sub_by_iri.insert(&sub.identity, sub);
            }
            _ => {
                return Err(ComputeSequenceError::UnsupportedFeature {
                    region: region_display.to_string(),
                    feature: feature.to_string(),
                });
            }
        }
    }
    if sub_by_iri.is_empty() {
        return Err(ComputeSequenceError::NoFeatures(region_display.to_string()));
    }

    let order = order_features(document, region)?;

    // Build the parent sequence first so its canonical identity is available
    // for the per-part range locations; elements/encoding are filled below.
    let seq_display = format!("{region_display}_sequence");
    let mut sequence = Sequence::builder(namespace.as_str(), seq_display.as_str())?.build()?;
    let seq_iri = sequence.identity.clone();

    let mut elements = String::new();
    let mut encoding: Option<Iri> = None;
    let mut ranges = Vec::with_capacity(order.len());
    let mut sub_components = Vec::with_capacity(order.len());

    for sub_iri in &order {
        let sub = sub_by_iri[sub_iri];
        let instance = sub
            .instance_of
            .as_ref()
            .ok_or_else(|| ComputeSequenceError::MissingInstanceOf(sub.identity.to_string()))?;
        let part = match document.resolve(instance) {
            Some(SbolObject::Component(part)) => part,
            _ => {
                return Err(ComputeSequenceError::Unresolved {
                    owner: sub.identity.to_string(),
                    reference: instance.to_string(),
                });
            }
        };
        if part.sequences.len() != 1 {
            return Err(ComputeSequenceError::PartSequenceCount {
                part: label(part),
                count: part.sequences.len(),
            });
        }
        let part_sequence = match document.resolve(&part.sequences[0]) {
            Some(SbolObject::Sequence(sequence)) => sequence,
            _ => {
                return Err(ComputeSequenceError::Unresolved {
                    owner: label(part),
                    reference: part.sequences[0].to_string(),
                });
            }
        };
        let part_elements = part_sequence
            .elements
            .as_deref()
            .ok_or_else(|| ComputeSequenceError::PartMissingElements(label(part)))?;
        match (&encoding, &part_sequence.encoding) {
            (Some(existing), Some(next)) if existing != next => {
                return Err(ComputeSequenceError::EncodingMismatch);
            }
            (None, next) => encoding = next.clone(),
            _ => {}
        }

        // 1-based inclusive coordinates over residue counts.
        let start = elements.chars().count() as i64 + 1;
        let end = start + part_elements.chars().count() as i64 - 1;
        let range = Range::builder(&sub.identity, "range1")?
            .start(start)
            .end(end)
            .sequence(seq_iri.clone())
            .build()?;

        let mut sub = sub.clone();
        sub.locations.push(range.identity.clone());
        sub_components.push(sub);
        ranges.push(range);
        elements.push_str(part_elements);
    }

    sequence.elements = Some(elements);
    sequence.encoding = Some(encoding.unwrap_or(EDAM_IUPAC_DNA));

    let mut region = region.clone();
    region.sequences.push(seq_iri);

    Ok(RegionPlan {
        region,
        sequence,
        ranges,
        sub_components,
    })
}

/// Orders a region's sub-components into a single linear chain by walking its
/// `meets` constraints (subject immediately precedes object).
fn order_features(
    document: &Document,
    region: &Component,
) -> Result<Vec<Resource>, ComputeSequenceError> {
    let unorderable = || ComputeSequenceError::Unorderable(label(region));

    let count = region.features.len();
    if count == 1 {
        return Ok(vec![region.features[0].clone()]);
    }

    let feature_set: HashSet<&Resource> = region.features.iter().collect();
    let mut next: HashMap<Resource, Resource> = HashMap::new();
    let mut inbound: HashSet<Resource> = HashSet::new();
    for constraint_iri in &region.constraints {
        let Some(SbolObject::Constraint(constraint)) = document.resolve(constraint_iri) else {
            continue;
        };
        if constraint.restriction.as_ref() != Some(&RESTRICTION_MEETS) {
            continue;
        }
        let (Some(subject), Some(object)) = (&constraint.subject, &constraint.constrained_object)
        else {
            continue;
        };
        if !feature_set.contains(subject) || !feature_set.contains(object) {
            continue;
        }
        // A linear chain has one outgoing and one incoming edge per node at
        // most; a duplicate on either side means the order is ambiguous.
        if next.insert(subject.clone(), object.clone()).is_some() {
            return Err(unorderable());
        }
        if !inbound.insert(object.clone()) {
            return Err(unorderable());
        }
    }
    if next.len() != count - 1 {
        return Err(unorderable());
    }

    let heads: Vec<&Resource> = region
        .features
        .iter()
        .filter(|feature| !inbound.contains(*feature))
        .collect();
    if heads.len() != 1 {
        return Err(unorderable());
    }

    let mut order = Vec::with_capacity(count);
    let mut seen: HashSet<Resource> = HashSet::new();
    let mut current = heads[0].clone();
    loop {
        if !seen.insert(current.clone()) {
            return Err(unorderable());
        }
        order.push(current.clone());
        match next.get(&current) {
            Some(next_iri) => current = next_iri.clone(),
            None => break,
        }
    }
    if order.len() != count {
        return Err(unorderable());
    }
    Ok(order)
}

/// Merges the computed objects for every planned region into a fresh document.
fn apply(document: &Document, plans: Vec<RegionPlan>) -> Result<Document, ComputeSequenceError> {
    let mut regions: HashMap<Resource, Component> = HashMap::new();
    let mut subs: HashMap<Resource, SubComponent> = HashMap::new();
    let mut additions: Vec<SbolObject> = Vec::new();
    for plan in plans {
        regions.insert(plan.region.identity.clone(), plan.region);
        for sub in plan.sub_components {
            subs.insert(sub.identity.clone(), sub);
        }
        additions.push(SbolObject::Sequence(plan.sequence));
        additions.extend(plan.ranges.into_iter().map(SbolObject::Range));
    }

    let mut objects: Vec<SbolObject> = document.typed_objects().to_vec();
    for object in objects.iter_mut() {
        match object {
            SbolObject::Component(component) => {
                if let Some(updated) = regions.remove(&component.identity) {
                    *component = updated;
                }
            }
            SbolObject::SubComponent(sub) => {
                if let Some(updated) = subs.remove(&sub.identity) {
                    *sub = updated;
                }
            }
            _ => {}
        }
    }
    objects.extend(additions);

    Document::from_objects(objects).map_err(ComputeSequenceError::Assembly)
}

/// A human-friendly label for a component: its `displayId`, else its IRI.
fn label(component: &Component) -> String {
    component
        .display_id()
        .map(str::to_string)
        .unwrap_or_else(|| component.identity.to_string())
}

#[cfg(test)]
mod tests;
