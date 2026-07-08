//! Expanding an SBOL 3 `CombinatorialDerivation` into concrete variant
//! `Component`s gathered in a `Collection`.
//!
//! [`expand_derivation`] takes a finished [`Document`] and the IRI of one
//! `CombinatorialDerivation`; [`expand_derivations`] expands every derivation
//! in the document. Both resolve the derivation's template `Component`, gather
//! the candidate variants of each `VariableFeature` (its `variants`, the
//! `Component` members of each `variantCollection`, and the components produced
//! by recursively expanding each `variantDerivation`), take the Cartesian
//! product across the variables, and for every combination clone the template
//! into a new `Component` whose matching `SubComponent`'s `instanceOf` is set to
//! the chosen variant. The derived components are collected as the members of a
//! new `Collection`. This mirrors the Python
//! `sbol_utilities.expand_combinatorial_derivations`.
//!
//! Every variable contributes exactly one variant to each combination. The
//! `strategy` and `cardinality` fields are carried on the model but do not alter
//! the enumeration, matching the Python implementation: `sample` is treated like
//! `enumerate`, and the `zeroOrOne` / `oneOrMore` / `zeroOrMore` cardinalities
//! are enumerated as a single-variant substitution rather than emitting
//! absent-feature or multi-variant combinations.
//!
//! A single `VariableFeature` over a simple template (one feature, no
//! sequences/constraints/interactions/interfaces/models) is treated as a
//! library: the collection lists the variant components themselves instead of
//! minting clones.
//!
//! Derived display IDs are the derivation's `displayId` followed by each chosen
//! variant's `displayId`. Distinct combinations therefore collide when variants
//! of different variables share a `displayId`; such a collision surfaces as a
//! duplicate-identity error while reassembling the document rather than
//! silently overwriting an object.
//!
//! Because the core [`Document`] is immutable, these functions return a new
//! `Document` with the derived objects added rather than mutating in place.

use std::collections::{HashMap, HashSet};

use sbol3::{
    BuildError, Collection, CombinatorialDerivation, Component, Constraint, Document, Resource,
    SbolIdentified, SbolObject, SbolTopLevel, SubComponent, VariableFeature,
};

/// A problem encountered while expanding a combinatorial derivation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ExpandError {
    /// The requested derivation IRI is not in the document.
    #[error("combinatorial derivation `{0}` was not found in the document")]
    DerivationNotFound(String),
    /// The requested IRI resolves to something other than a `CombinatorialDerivation`.
    #[error("`{0}` is not a CombinatorialDerivation")]
    NotACombinatorialDerivation(String),
    /// The `variantDerivation` graph contains a cycle.
    #[error("combinatorial derivation `{0}` participates in a cyclic variantDerivation chain")]
    CyclicDerivation(String),
    /// A derivation has no `template`.
    #[error("combinatorial derivation `{0}` has no template")]
    MissingTemplate(String),
    /// A derivation's `template` does not resolve within the document.
    #[error("template `{0}` was not found in the document")]
    TemplateNotFound(String),
    /// A derivation's `template` resolves to something other than a `Component`.
    #[error("template `{0}` is not a Component")]
    TemplateNotAComponent(String),
    /// A `variableFeature` reference does not resolve to a `VariableFeature`.
    #[error("`{0}` does not resolve to a VariableFeature")]
    VariableFeatureNotFound(String),
    /// A `VariableFeature` does not name the template feature it varies.
    #[error("variable feature `{0}` has no `variable` referencing a template feature")]
    VariableWithoutTarget(String),
    /// A `VariableFeature`'s `variable` is not among the template's features.
    #[error("variable `{variable}` of derivation `{derivation}` is not a feature of the template")]
    VariableNotInTemplate {
        /// The derivation being expanded.
        derivation: String,
        /// The offending `variable` reference.
        variable: String,
    },
    /// A listed variant does not resolve within the document.
    #[error("variant `{0}` was not found in the document")]
    VariantNotFound(String),
    /// A listed variant resolves to something other than a `Component`.
    #[error("variant `{0}` is not a Component")]
    VariantNotAComponent(String),
    /// A `variantCollection` does not resolve within the document.
    #[error("variant collection `{0}` was not found in the document")]
    CollectionNotFound(String),
    /// A `variantCollection` resolves to something other than a `Collection`.
    #[error("`{0}` is not a Collection")]
    NotACollection(String),
    /// The template holds children this expander cannot faithfully re-parent.
    #[error("template `{template}` cannot be expanded: {reason}")]
    UnsupportedTemplate {
        /// The template component.
        template: String,
        /// Why it cannot be expanded.
        reason: String,
    },
    /// A cloned sub-component's `instanceOf` could not be determined.
    #[error("template feature `{0}` has no instanceOf to clone")]
    MissingInstanceOf(String),
    /// A template constraint is missing a subject, object, or restriction.
    #[error("template constraint `{0}` is missing a subject, object, or restriction")]
    MalformedConstraint(String),
    /// An object needed a `displayId` but did not carry one.
    #[error("`{0}` has no displayId; validate the document before expanding")]
    MissingDisplayId(String),
    /// A derivation or template has no namespace.
    #[error("`{0}` has no namespace; validate the document before expanding")]
    MissingNamespace(String),
    /// Building one of the derived objects failed.
    #[error("failed to build a derived object: {0}")]
    Build(#[from] BuildError),
    /// Reassembling the document with the derived objects failed.
    #[error("failed to reassemble the document: {0}")]
    Assembly(BuildError),
}

/// Expands a single combinatorial derivation, identified by its IRI, and returns
/// a new [`Document`] with the derived `Component`s and their `Collection` merged
/// in. Any derivations referenced through `variantDerivation` are expanded too.
pub fn expand_derivation(
    document: &Document,
    derivation: &Resource,
) -> Result<Document, ExpandError> {
    match document.resolve(derivation) {
        Some(SbolObject::CombinatorialDerivation(_)) => {}
        Some(_) => {
            return Err(ExpandError::NotACombinatorialDerivation(
                derivation.to_string(),
            ));
        }
        None => return Err(ExpandError::DerivationNotFound(derivation.to_string())),
    }
    let mut expander = Expander::new(document);
    expander.derivation_to_collection(derivation)?;
    expander.finish()
}

/// Expands every combinatorial derivation in the document, returning a new
/// [`Document`] with all derived `Component`s and their `Collection`s merged in.
/// Derivations reached through `variantDerivation` are expanded once and shared.
pub fn expand_derivations(document: &Document) -> Result<Document, ExpandError> {
    let mut expander = Expander::new(document);
    let derivations: Vec<Resource> = document
        .combinatorial_derivations()
        .map(|cd| cd.identity.clone())
        .collect();
    for derivation in &derivations {
        expander.derivation_to_collection(derivation)?;
    }
    expander.finish()
}

/// Accumulates the objects derived from one or more combinatorial derivations,
/// caching each derivation's expansion so shared `variantDerivation`s are
/// expanded once.
struct Expander<'a> {
    doc: &'a Document,
    additions: Vec<SbolObject>,
    /// Derivation IRI to the component members of the collection it produced.
    cache: HashMap<Resource, Vec<Resource>>,
    /// Derivations currently being expanded, to detect cycles.
    visiting: HashSet<Resource>,
}

impl<'a> Expander<'a> {
    fn new(doc: &'a Document) -> Self {
        Self {
            doc,
            additions: Vec::new(),
            cache: HashMap::new(),
            visiting: HashSet::new(),
        }
    }

    /// Merges the accumulated derived objects into a fresh document.
    fn finish(self) -> Result<Document, ExpandError> {
        let mut objects: Vec<SbolObject> = self.doc.typed_objects().to_vec();
        objects.extend(self.additions);
        Document::from_objects(objects).map_err(ExpandError::Assembly)
    }

    /// Expands one derivation into a `Collection` and returns the component IRIs
    /// that make up its membership (the derived components, or for a library the
    /// variant components themselves).
    fn derivation_to_collection(
        &mut self,
        cd_iri: &Resource,
    ) -> Result<Vec<Resource>, ExpandError> {
        if let Some(members) = self.cache.get(cd_iri) {
            return Ok(members.clone());
        }
        if !self.visiting.insert(cd_iri.clone()) {
            return Err(ExpandError::CyclicDerivation(cd_iri.to_string()));
        }

        let doc = self.doc;
        let cd = match doc.resolve(cd_iri) {
            Some(SbolObject::CombinatorialDerivation(cd)) => cd.clone(),
            Some(_) => return Err(ExpandError::NotACombinatorialDerivation(cd_iri.to_string())),
            None => return Err(ExpandError::DerivationNotFound(cd_iri.to_string())),
        };

        let template_iri = cd
            .template
            .clone()
            .ok_or_else(|| ExpandError::MissingTemplate(cd_iri.to_string()))?;
        let template = match doc.resolve(&template_iri) {
            Some(SbolObject::Component(component)) => component.clone(),
            Some(_) => return Err(ExpandError::TemplateNotAComponent(template_iri.to_string())),
            None => return Err(ExpandError::TemplateNotFound(template_iri.to_string())),
        };

        // Variables are processed in identity order, which also fixes the shape
        // of the Cartesian product and the derived display IDs.
        let mut vf_iris = cd.variable_features.clone();
        vf_iris.sort();

        let mut vfs: Vec<VariableFeature> = Vec::with_capacity(vf_iris.len());
        let mut values: Vec<Vec<Resource>> = Vec::with_capacity(vf_iris.len());
        for vf_iri in &vf_iris {
            let vf = match doc.resolve(vf_iri) {
                Some(SbolObject::VariableFeature(vf)) => vf.clone(),
                _ => return Err(ExpandError::VariableFeatureNotFound(vf_iri.to_string())),
            };
            values.push(self.cd_variable_values(&vf)?);
            vfs.push(vf);
        }

        let cd_display = cd
            .display_id()
            .ok_or_else(|| ExpandError::MissingDisplayId(cd_iri.to_string()))?
            .to_string();
        let cd_namespace = cd
            .namespace()
            .ok_or_else(|| ExpandError::MissingNamespace(cd_display.clone()))?
            .as_str()
            .to_string();

        let library = is_library(&cd, &template);
        let members: Vec<Resource> = if library {
            values.into_iter().next().unwrap_or_default()
        } else {
            let combos = cartesian_product(&values);
            let mut derived = Vec::with_capacity(combos.len());
            for combo in &combos {
                derived.push(self.build_derived(
                    &template,
                    &cd_display,
                    &cd_namespace,
                    &vfs,
                    combo,
                )?);
            }
            derived
        };

        let coll_display = if library {
            format!("{cd_display}_collection")
        } else {
            format!("{cd_display}_derivatives")
        };
        let mut collection =
            Collection::builder(cd_namespace.as_str(), coll_display.as_str())?.build()?;
        collection.members = members.clone();
        self.additions.push(SbolObject::Collection(collection));

        self.visiting.remove(cd_iri);
        self.cache.insert(cd_iri.clone(), members.clone());
        Ok(members)
    }

    /// Gathers the candidate variant components of one variable feature: its
    /// direct `variants`, the components in its `variantCollection`s, and the
    /// components produced by expanding its `variantDerivation`s.
    fn cd_variable_values(&mut self, vf: &VariableFeature) -> Result<Vec<Resource>, ExpandError> {
        let doc = self.doc;
        let mut out: Vec<Resource> = Vec::new();

        let mut variants = vf.variants.clone();
        variants.sort();
        for variant in &variants {
            match doc.resolve(variant) {
                Some(SbolObject::Component(_)) => out.push(variant.clone()),
                Some(_) => return Err(ExpandError::VariantNotAComponent(variant.to_string())),
                None => return Err(ExpandError::VariantNotFound(variant.to_string())),
            }
        }

        let mut collections = vf.variant_collections.clone();
        collections.sort();
        for collection in &collections {
            out.extend(collection_values(doc, collection, &mut HashSet::new())?);
        }

        let mut derivations = vf.variant_derivations.clone();
        derivations.sort();
        for derivation in &derivations {
            out.extend(self.derivation_to_collection(derivation)?);
        }

        Ok(out)
    }

    /// Clones the template into one derived component for a single assignment,
    /// substituting each variable's chosen variant into the matching
    /// sub-component's `instanceOf`. Returns the derived component's IRI.
    fn build_derived(
        &mut self,
        template: &Component,
        cd_display: &str,
        cd_namespace: &str,
        vfs: &[VariableFeature],
        combo: &[Resource],
    ) -> Result<Resource, ExpandError> {
        let doc = self.doc;

        if !template.interactions.is_empty() || !template.interfaces.is_empty() {
            return Err(ExpandError::UnsupportedTemplate {
                template: component_label(template),
                reason: "templates with interactions or interfaces are not supported".to_string(),
            });
        }

        // Which template feature each variable replaces, and with what variant.
        let mut overrides: HashMap<Resource, Resource> = HashMap::new();
        for (vf, chosen) in vfs.iter().zip(combo.iter()) {
            let target = vf
                .variable
                .clone()
                .ok_or_else(|| ExpandError::VariableWithoutTarget(vf.identity.to_string()))?;
            if !template.features.iter().any(|feature| feature == &target) {
                return Err(ExpandError::VariableNotInTemplate {
                    derivation: cd_display.to_string(),
                    variable: target.to_string(),
                });
            }
            overrides.insert(target, chosen.clone());
        }

        // Derived displayId: the derivation's displayId then each chosen variant.
        let mut derived_display = cd_display.to_string();
        for chosen in combo {
            let variant_display = display_id_of(doc, chosen)
                .ok_or_else(|| ExpandError::MissingDisplayId(chosen.to_string()))?;
            derived_display.push('_');
            derived_display.push_str(variant_display);
        }

        // Build the shell first so its canonical identity anchors the children;
        // features and constraints are filled in after they are minted.
        let mut derived = Component::builder(cd_namespace, derived_display.as_str())?
            .types(template.types.iter().cloned())
            .component_roles(template.roles.iter().cloned())
            .sequences(template.sequences.iter().cloned())
            .models(template.models.iter().cloned())
            .build()?;
        let derived_iri = derived.identity.clone();

        // Clone each template feature under the derived component. Variable
        // features become fresh sub-components instancing the chosen variant;
        // every other feature is re-parented as-is (its nested locations are
        // dropped rather than re-parented).
        let mut child_map: HashMap<Resource, Resource> = HashMap::new();
        let mut new_features: Vec<Resource> = Vec::with_capacity(template.features.len());
        for feature in &template.features {
            let resolved = doc.resolve(feature);
            let display = feature_display_id(resolved)
                .ok_or_else(|| ExpandError::MissingDisplayId(feature.to_string()))?
                .to_string();

            let new_sub = if let Some(variant) = overrides.get(feature) {
                SubComponent::builder(&derived_iri, display.as_str())?
                    .instance_of(variant.clone())
                    .build()?
            } else {
                let original = match resolved {
                    Some(SbolObject::SubComponent(sub)) => sub,
                    _ => {
                        return Err(ExpandError::UnsupportedTemplate {
                            template: component_label(template),
                            reason: "non-variable template features must be SubComponents"
                                .to_string(),
                        });
                    }
                };
                let instance = original
                    .instance_of
                    .clone()
                    .ok_or_else(|| ExpandError::MissingInstanceOf(original.identity.to_string()))?;
                let mut sub = SubComponent::builder(&derived_iri, display.as_str())?
                    .instance_of(instance)
                    .build()?;
                sub.feature = original.feature.clone();
                sub.role_integration = original.role_integration.clone();
                sub
            };

            child_map.insert(feature.clone(), new_sub.identity.clone());
            new_features.push(new_sub.identity.clone());
            self.additions.push(SbolObject::SubComponent(new_sub));
        }

        // Clone constraints, rewiring subject/object onto the derived features.
        let mut new_constraints: Vec<Resource> = Vec::with_capacity(template.constraints.len());
        for constraint in &template.constraints {
            let original = match doc.resolve(constraint) {
                Some(SbolObject::Constraint(constraint)) => constraint.clone(),
                _ => {
                    return Err(ExpandError::UnsupportedTemplate {
                        template: component_label(template),
                        reason: "a template constraint did not resolve to a Constraint".to_string(),
                    });
                }
            };
            let display = original
                .display_id()
                .ok_or_else(|| ExpandError::MissingDisplayId(constraint.to_string()))?
                .to_string();
            let subject = remap(&child_map, original.subject.as_ref())
                .ok_or_else(|| ExpandError::MalformedConstraint(constraint.to_string()))?;
            let object = remap(&child_map, original.constrained_object.as_ref())
                .ok_or_else(|| ExpandError::MalformedConstraint(constraint.to_string()))?;
            let restriction = original
                .restriction
                .clone()
                .ok_or_else(|| ExpandError::MalformedConstraint(constraint.to_string()))?;
            let derived_constraint = Constraint::builder(&derived_iri, display.as_str())?
                .subject(subject)
                .constrained_object(object)
                .restriction(restriction)
                .build()?;
            new_constraints.push(derived_constraint.identity.clone());
            self.additions
                .push(SbolObject::Constraint(derived_constraint));
        }

        derived.features = new_features;
        derived.constraints = new_constraints;
        self.additions.push(SbolObject::Component(derived));
        Ok(derived_iri)
    }
}

/// Flattens a collection into its member components, recursing into nested
/// collections. Members that are neither components nor collections are ignored.
fn collection_values(
    doc: &Document,
    collection: &Resource,
    seen: &mut HashSet<Resource>,
) -> Result<Vec<Resource>, ExpandError> {
    if !seen.insert(collection.clone()) {
        return Ok(Vec::new());
    }
    let members = match doc.resolve(collection) {
        Some(SbolObject::Collection(collection)) => collection.members.clone(),
        Some(_) => return Err(ExpandError::NotACollection(collection.to_string())),
        None => return Err(ExpandError::CollectionNotFound(collection.to_string())),
    };
    let mut sorted = members;
    sorted.sort();
    let mut out: Vec<Resource> = Vec::new();
    for member in &sorted {
        match doc.resolve(member) {
            Some(SbolObject::Component(_)) => out.push(member.clone()),
            Some(SbolObject::Collection(_)) => {
                out.extend(collection_values(doc, member, seen)?);
            }
            _ => {}
        }
    }
    Ok(out)
}

/// A single variable over a simple template is treated as a library: the
/// collection lists the variant components rather than cloning the template.
fn is_library(cd: &CombinatorialDerivation, template: &Component) -> bool {
    let one_variable = cd.variable_features.len() == 1 && template.features.len() == 1;
    let simple = template.sequences.is_empty()
        && template.interactions.is_empty()
        && template.constraints.is_empty()
        && template.interfaces.is_empty()
        && template.models.is_empty();
    one_variable && simple
}

/// The Cartesian product across the per-variable candidate lists, with the last
/// variable varying fastest. An empty candidate list yields no combinations.
fn cartesian_product(values: &[Vec<Resource>]) -> Vec<Vec<Resource>> {
    let mut result: Vec<Vec<Resource>> = vec![Vec::new()];
    for dimension in values {
        let mut next = Vec::with_capacity(result.len() * dimension.len());
        for partial in &result {
            for value in dimension {
                let mut combo = partial.clone();
                combo.push(value.clone());
                next.push(combo);
            }
        }
        result = next;
    }
    result
}

/// Rewrites a reference through the template-to-derived child map, passing
/// through references that name objects outside the template.
fn remap(map: &HashMap<Resource, Resource>, reference: Option<&Resource>) -> Option<Resource> {
    let reference = reference?;
    Some(
        map.get(reference)
            .cloned()
            .unwrap_or_else(|| reference.clone()),
    )
}

/// The `displayId` of a feature object, whatever its concrete type.
fn feature_display_id(object: Option<&SbolObject>) -> Option<&str> {
    match object? {
        SbolObject::SubComponent(feature) => feature.display_id(),
        SbolObject::LocalSubComponent(feature) => feature.display_id(),
        SbolObject::SequenceFeature(feature) => feature.display_id(),
        SbolObject::ComponentReference(feature) => feature.display_id(),
        SbolObject::ExternallyDefined(feature) => feature.display_id(),
        _ => None,
    }
}

/// The `displayId` of a component named by `iri`, if it resolves to one.
fn display_id_of<'a>(doc: &'a Document, iri: &Resource) -> Option<&'a str> {
    match doc.resolve(iri)? {
        SbolObject::Component(component) => component.display_id(),
        _ => None,
    }
}

/// A human-friendly label for a component: its `displayId`, else its IRI.
fn component_label(component: &Component) -> String {
    component
        .display_id()
        .map(str::to_string)
        .unwrap_or_else(|| component.identity.to_string())
}

#[cfg(test)]
mod tests;
