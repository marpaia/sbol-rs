use std::collections::{BTreeMap, BTreeSet};

use crate::vocab::*;
use crate::{Document, Iri, Object, Resource, SbolClass, Term};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OwnershipEdge {
    pub parent: Resource,
    pub predicate: &'static str,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct OwnershipIndex {
    parents_by_child: BTreeMap<Resource, Vec<OwnershipEdge>>,
    children_by_parent: BTreeMap<(Resource, &'static str), Vec<Resource>>,
}

impl OwnershipIndex {
    pub(crate) fn new(document: &Document) -> Self {
        let mut index = Self::default();
        for object in document.objects().values() {
            for predicate in COMPOSITE_PREDICATES {
                for child in object.resources(predicate) {
                    index
                        .parents_by_child
                        .entry(child.clone())
                        .or_default()
                        .push(OwnershipEdge {
                            parent: object.identity().clone(),
                            predicate,
                        });
                    index
                        .children_by_parent
                        .entry((object.identity().clone(), predicate))
                        .or_default()
                        .push(child.clone());
                }
            }
        }
        index
    }

    pub(crate) fn parents(&self, child: &Resource, predicate: &str) -> Vec<&Resource> {
        self.parents_by_child
            .get(child)
            .into_iter()
            .flatten()
            .filter(|edge| edge.predicate == predicate)
            .map(|edge| &edge.parent)
            .collect()
    }

    pub(crate) fn single_parent(&self, child: &Resource, predicate: &str) -> Option<&Resource> {
        self.parents(child, predicate).into_iter().next()
    }

    pub(crate) fn children(&self, parent: &Resource, predicate: &str) -> Vec<&Resource> {
        self.children_by_parent
            .get(&(parent.clone(), owned_predicate(predicate)))
            .into_iter()
            .flatten()
            .collect()
    }

    pub(crate) fn contains(&self, parent: &Resource, predicate: &str, child: &Resource) -> bool {
        self.children(parent, predicate)
            .into_iter()
            .any(|candidate| candidate == child)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FeatureResolveError {
    MissingObject(Resource),
    MissingRefersTo(Resource),
    Cycle(Vec<Resource>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FeatureTrace {
    pub requested: Resource,
    pub target: Resource,
    pub path: Vec<Resource>,
}

pub(crate) struct ComponentReferenceResolver<'a> {
    document: &'a Document,
    ownership: &'a OwnershipIndex,
}

impl<'a> ComponentReferenceResolver<'a> {
    pub(crate) fn new(document: &'a Document, ownership: &'a OwnershipIndex) -> Self {
        Self {
            document,
            ownership,
        }
    }

    pub(crate) fn trace_feature(
        &self,
        feature: &Resource,
    ) -> Result<FeatureTrace, FeatureResolveError> {
        let mut current = feature.clone();
        let mut path = Vec::new();
        let mut visited = BTreeSet::new();

        loop {
            if !visited.insert(current.clone()) {
                path.push(current);
                return Err(FeatureResolveError::Cycle(path));
            }

            let Some(object) = self.document.get(&current) else {
                return Err(FeatureResolveError::MissingObject(current));
            };
            if !object.has_class(SbolClass::ComponentReference) {
                return Ok(FeatureTrace {
                    requested: feature.clone(),
                    target: current,
                    path,
                });
            }

            path.push(current.clone());
            let Some(next) = object.first_resource(SBOL_REFERS_TO) else {
                return Err(FeatureResolveError::MissingRefersTo(current));
            };
            current = next.clone();
        }
    }

    pub(crate) fn parent_reference(&self, reference: &Object) -> Option<&'a Object> {
        parent_by_identity_prefix(self.document, reference)
            .filter(|parent| parent.has_class(SbolClass::ComponentReference))
    }

    pub(crate) fn reference_is_child_of(
        &self,
        reference: &Resource,
        parent_reference: &Resource,
    ) -> bool {
        self.document.get(reference).is_some_and(|reference| {
            self.parent_reference(reference)
                .is_some_and(|parent| parent.identity() == parent_reference)
        })
    }

    pub(crate) fn direct_parent_component(&self, feature: &Resource) -> Option<&Resource> {
        self.ownership.single_parent(feature, SBOL_HAS_FEATURE)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DirectionalOrientation {
    Inline,
    ReverseComplement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LinearLocation {
    pub identity: Resource,
    pub sequence: Resource,
    pub start: i64,
    pub end: i64,
    pub orientation: Option<DirectionalOrientation>,
}

pub(crate) struct LocationResolver<'a> {
    document: &'a Document,
    references: ComponentReferenceResolver<'a>,
}

impl<'a> LocationResolver<'a> {
    pub(crate) fn new(document: &'a Document, ownership: &'a OwnershipIndex) -> Self {
        Self {
            document,
            references: ComponentReferenceResolver::new(document, ownership),
        }
    }

    pub(crate) fn locations_for_feature(&self, feature: &Resource) -> Vec<LinearLocation> {
        let target = self
            .references
            .trace_feature(feature)
            .map(|trace| trace.target)
            .unwrap_or_else(|_| feature.clone());
        let Some(feature) = self.document.get(&target) else {
            return Vec::new();
        };

        feature
            .resources(SBOL_HAS_LOCATION)
            .filter_map(|location| self.linear_location(location))
            .collect()
    }

    pub(crate) fn linear_location(&self, identity: &Resource) -> Option<LinearLocation> {
        let location = self.document.get(identity)?;
        let sequence = location.first_resource(SBOL_HAS_SEQUENCE)?.clone();
        let orientation = directional_orientation_value(location.first_iri(SBOL_ORIENTATION));

        if location.has_class(SbolClass::Range) {
            let start = integer_value(location, SBOL_START)?;
            let end = integer_value(location, SBOL_END)?;
            if start <= 0 || end < start {
                return None;
            }
            return Some(LinearLocation {
                identity: identity.clone(),
                sequence,
                start: start - 1,
                end,
                orientation,
            });
        }

        if location.has_class(SbolClass::Cut) {
            let at = integer_value(location, SBOL_AT)?;
            if at < 0 {
                return None;
            }
            return Some(LinearLocation {
                identity: identity.clone(),
                sequence,
                start: at,
                end: at,
                orientation,
            });
        }

        if location.has_class(SbolClass::EntireSequence) {
            let sequence_object = self.document.get(&sequence)?;
            let end = sequence_object.first_literal_value(SBOL_ELEMENTS)?.len() as i64;
            return Some(LinearLocation {
                identity: identity.clone(),
                sequence,
                start: 0,
                end,
                orientation,
            });
        }

        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RelationOutcome {
    Satisfied,
    Contradicted {
        subject_location: Option<Resource>,
        object_location: Option<Resource>,
    },
    Unknown,
    Unsupported,
}

pub(crate) struct ConstraintEngine<'a> {
    document: &'a Document,
    references: ComponentReferenceResolver<'a>,
    locations: LocationResolver<'a>,
}

impl<'a> ConstraintEngine<'a> {
    pub(crate) fn new(document: &'a Document, ownership: &'a OwnershipIndex) -> Self {
        Self {
            document,
            references: ComponentReferenceResolver::new(document, ownership),
            locations: LocationResolver::new(document, ownership),
        }
    }

    pub(crate) fn table8_relation(
        &self,
        restriction: &str,
        subject: &Resource,
        object: &Resource,
    ) -> RelationOutcome {
        match restriction {
            SBOL_VERIFY_IDENTICAL | SBOL_DIFFERENT_FROM => {
                let Some(subject_key) = self.identity_key(subject) else {
                    return RelationOutcome::Unknown;
                };
                let Some(object_key) = self.identity_key(object) else {
                    return RelationOutcome::Unknown;
                };
                let same = subject_key.key == object_key.key;
                match restriction {
                    SBOL_VERIFY_IDENTICAL if same => RelationOutcome::Satisfied,
                    SBOL_VERIFY_IDENTICAL
                        if subject_key.through_reference || object_key.through_reference =>
                    {
                        RelationOutcome::Unknown
                    }
                    SBOL_VERIFY_IDENTICAL => RelationOutcome::Contradicted {
                        subject_location: None,
                        object_location: None,
                    },
                    SBOL_DIFFERENT_FROM if same => RelationOutcome::Contradicted {
                        subject_location: None,
                        object_location: None,
                    },
                    SBOL_DIFFERENT_FROM => RelationOutcome::Satisfied,
                    _ => RelationOutcome::Unsupported,
                }
            }
            SBOL_SAME_ORIENTATION_AS | SBOL_OPPOSITE_ORIENTATION_AS => {
                let Some(subject_orientation) = self.feature_orientation(subject) else {
                    return RelationOutcome::Unknown;
                };
                let Some(object_orientation) = self.feature_orientation(object) else {
                    return RelationOutcome::Unknown;
                };
                let same = subject_orientation == object_orientation;
                match (restriction, same) {
                    (SBOL_SAME_ORIENTATION_AS, true) | (SBOL_OPPOSITE_ORIENTATION_AS, false) => {
                        RelationOutcome::Satisfied
                    }
                    _ => RelationOutcome::Contradicted {
                        subject_location: None,
                        object_location: None,
                    },
                }
            }
            SBOL_REPLACES => RelationOutcome::Unknown,
            _ => RelationOutcome::Unsupported,
        }
    }

    pub(crate) fn table10_relation(
        &self,
        restriction: &str,
        subject: &Resource,
        object: &Resource,
    ) -> RelationOutcome {
        if !SEQUENTIAL_RESTRICTION_IRIS.contains(&restriction) {
            return RelationOutcome::Unsupported;
        }

        let subject_locations = self.locations.locations_for_feature(subject);
        let object_locations = self.locations.locations_for_feature(object);
        let mut comparable_pair = None;

        for subject_location in &subject_locations {
            for object_location in &object_locations {
                if subject_location.sequence != object_location.sequence {
                    continue;
                }
                if sequential_relation_satisfied(restriction, subject_location, object_location) {
                    return RelationOutcome::Satisfied;
                }
                comparable_pair.get_or_insert_with(|| {
                    (
                        subject_location.identity.clone(),
                        object_location.identity.clone(),
                    )
                });
            }
        }

        match comparable_pair {
            Some((subject_location, object_location)) => RelationOutcome::Contradicted {
                subject_location: Some(subject_location),
                object_location: Some(object_location),
            },
            None => RelationOutcome::Unknown,
        }
    }

    fn identity_key(&self, feature: &Resource) -> Option<ResolvedIdentityKey> {
        let trace = self.references.trace_feature(feature).ok()?;
        let object = self.document.get(&trace.target)?;
        let through_reference = !trace.path.is_empty();
        if object.has_class(SbolClass::SubComponent) {
            return object.first_resource(SBOL_INSTANCE_OF).cloned().map(|key| {
                ResolvedIdentityKey {
                    key: IdentityKey::SubComponentInstance(key),
                    through_reference,
                }
            });
        }
        if object.has_class(SbolClass::ExternallyDefined) {
            return object.first_resource(SBOL_DEFINITION).cloned().map(|key| {
                ResolvedIdentityKey {
                    key: IdentityKey::ExternallyDefinedDefinition(key),
                    through_reference,
                }
            });
        }
        None
    }

    fn feature_orientation(&self, feature: &Resource) -> Option<DirectionalOrientation> {
        let trace = self.references.trace_feature(feature).ok()?;
        let object = self.document.get(&trace.target)?;
        directional_orientation_value(object.first_iri(SBOL_ORIENTATION))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum IdentityKey {
    SubComponentInstance(Resource),
    ExternallyDefinedDefinition(Resource),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedIdentityKey {
    key: IdentityKey,
    through_reference: bool,
}

pub(crate) struct TypeReferentResolver<'a> {
    document: &'a Document,
    references: ComponentReferenceResolver<'a>,
}

impl<'a> TypeReferentResolver<'a> {
    pub(crate) fn new(document: &'a Document, ownership: &'a OwnershipIndex) -> Self {
        Self {
            document,
            references: ComponentReferenceResolver::new(document, ownership),
        }
    }

    pub(crate) fn type_properties(&self, feature: &Resource) -> Option<BTreeSet<Iri>> {
        let referent = self.type_referent(feature)?;
        Some(iri_values(self.document.get(&referent)?, SBOL_TYPE))
    }

    fn type_referent(&self, feature: &Resource) -> Option<Resource> {
        let trace = self.references.trace_feature(feature).ok()?;
        let object = self.document.get(&trace.target)?;
        if object.has_class(SbolClass::LocalSubComponent)
            || object.has_class(SbolClass::ExternallyDefined)
        {
            return Some(trace.target);
        }
        if object.has_class(SbolClass::SubComponent) {
            return object.first_resource(SBOL_INSTANCE_OF).cloned();
        }
        None
    }
}

pub(crate) struct DerivationResolver<'a> {
    document: &'a Document,
}

impl<'a> DerivationResolver<'a> {
    pub(crate) fn new(document: &'a Document, _ownership: &'a OwnershipIndex) -> Self {
        Self { document }
    }

    pub(crate) fn context(&self, derivation: &Object) -> Option<DerivationContext> {
        let template = derivation.first_resource(SBOL_TEMPLATE)?.clone();
        let template_object = self.document.get(&template)?;
        let template_features = template_object
            .resources(SBOL_HAS_FEATURE)
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut variables = BTreeMap::new();
        let mut variable_features = Vec::new();

        for variable_feature in derivation.resources(SBOL_HAS_VARIABLE_FEATURE) {
            let Some(variable_feature_object) = self.document.get(variable_feature) else {
                continue;
            };
            let Some(variable) = variable_feature_object.first_resource(SBOL_VARIABLE) else {
                continue;
            };
            variables.insert(variable.clone(), variable_feature.clone());
            variable_features.push(variable_feature.clone());
        }

        let static_features = template_features
            .iter()
            .filter(|feature| !variables.contains_key(*feature))
            .cloned()
            .collect();

        Some(DerivationContext {
            derivation: derivation.identity().clone(),
            template,
            template_features,
            variables,
            variable_features,
            static_features,
        })
    }

    pub(crate) fn derived_components(&self, derivation: &Resource) -> Vec<&'a Object> {
        self.document
            .objects()
            .values()
            .filter(|object| {
                object.has_class(SbolClass::Component)
                    && object
                        .identified()
                        .derived_from
                        .iter()
                        .any(|item| item == derivation)
            })
            .collect()
    }

    pub(crate) fn derived_collections(&self, derivation: &Resource) -> Vec<&'a Object> {
        self.document
            .objects()
            .values()
            .filter(|object| {
                object.has_class(SbolClass::Collection)
                    && object
                        .identified()
                        .derived_from
                        .iter()
                        .any(|item| item == derivation)
            })
            .collect()
    }

    pub(crate) fn features_derived_from(
        &self,
        derived_component: &'a Object,
        template_feature: &Resource,
    ) -> Vec<&'a Resource> {
        derived_component
            .resources(SBOL_HAS_FEATURE)
            .filter(|feature| {
                self.document.get(feature).is_some_and(|object| {
                    object
                        .identified()
                        .derived_from
                        .iter()
                        .any(|candidate| candidate == template_feature)
                })
            })
            .collect()
    }

    pub(crate) fn feature_derives_from_template(
        &self,
        feature: &Resource,
        template_features: &BTreeSet<Resource>,
    ) -> bool {
        self.document.get(feature).is_some_and(|object| {
            object
                .identified()
                .derived_from
                .iter()
                .any(|candidate| template_features.contains(candidate))
        })
    }

    pub(crate) fn static_feature_properties_match(
        &self,
        derived_feature: &Resource,
        template_feature: &Resource,
    ) -> Option<bool> {
        let derived = self.document.get(derived_feature)?;
        let template = self.document.get(template_feature)?;
        Some(
            FEATURE_EQUIVALENCE_PREDICATES.iter().all(|predicate| {
                term_values(derived, predicate) == term_values(template, predicate)
            }),
        )
    }

    pub(crate) fn variable_feature_for(
        &self,
        context: &DerivationContext,
        template_feature: &Resource,
    ) -> Option<&'a Object> {
        let variable_feature = context.variables.get(template_feature)?;
        self.document.get(variable_feature)
    }

    pub(crate) fn cardinality_allows(&self, cardinality: Option<&Iri>, count: usize) -> bool {
        match cardinality.map(Iri::as_str) {
            Some(SBOL_ONE) => count == 1,
            Some(SBOL_ZERO_OR_ONE) => count <= 1,
            Some(SBOL_ONE_OR_MORE) => count >= 1,
            Some(SBOL_ZERO_OR_MORE) => true,
            _ => true,
        }
    }

    pub(crate) fn allowed_variants(&self, variable_feature: &Object) -> BTreeSet<Resource> {
        let mut allowed = variable_feature
            .resources(SBOL_VARIANT)
            .filter(|variant| {
                self.document
                    .get(variant)
                    .is_some_and(|object| object.has_class(SbolClass::Component))
            })
            .cloned()
            .collect::<BTreeSet<_>>();

        for collection in variable_feature.resources(SBOL_VARIANT_COLLECTION) {
            let mut visited = BTreeSet::new();
            self.collect_collection_components(collection, &mut visited, &mut allowed);
        }

        let derivations = variable_feature
            .resources(SBOL_VARIANT_DERIVATION)
            .cloned()
            .collect::<BTreeSet<_>>();
        if !derivations.is_empty() {
            for component in self.document.components() {
                if component
                    .identified
                    .derived_from
                    .iter()
                    .any(|candidate| derivations.contains(candidate))
                {
                    allowed.insert(component.identity.clone());
                }
            }
        }

        allowed
    }

    pub(crate) fn template_constraints(&self, context: &DerivationContext) -> Vec<&'a Object> {
        let Some(template) = self.document.get(&context.template) else {
            return Vec::new();
        };
        template
            .resources(SBOL_HAS_CONSTRAINT)
            .filter_map(|constraint| self.document.get(constraint))
            .collect()
    }

    pub(crate) fn feature_roles(&self, feature: &Resource) -> Option<BTreeSet<Iri>> {
        Some(iri_values(self.document.get(feature)?, SBOL_ROLE))
    }

    pub(crate) fn component_types(&self, component: &Object) -> BTreeSet<Iri> {
        iri_values(component, SBOL_TYPE)
    }

    pub(crate) fn component_roles(&self, component: &Object) -> BTreeSet<Iri> {
        iri_values(component, SBOL_ROLE)
    }

    fn collect_collection_components(
        &self,
        collection: &Resource,
        visited: &mut BTreeSet<Resource>,
        components: &mut BTreeSet<Resource>,
    ) {
        if !visited.insert(collection.clone()) {
            return;
        }
        let Some(collection_object) = self.document.get(collection) else {
            return;
        };
        if !collection_object.has_class(SbolClass::Collection) {
            return;
        }
        for member in collection_object.resources(SBOL_MEMBER) {
            let Some(member_object) = self.document.get(member) else {
                continue;
            };
            if member_object.has_class(SbolClass::Component) {
                components.insert(member.clone());
            } else if member_object.has_class(SbolClass::Collection) {
                self.collect_collection_components(member, visited, components);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DerivationContext {
    pub derivation: Resource,
    pub template: Resource,
    pub template_features: BTreeSet<Resource>,
    pub variables: BTreeMap<Resource, Resource>,
    pub variable_features: Vec<Resource>,
    pub static_features: BTreeSet<Resource>,
}

pub(crate) fn directional_orientation_value(value: Option<&Iri>) -> Option<DirectionalOrientation> {
    match value.map(Iri::as_str)? {
        SBOL_INLINE | SO_INLINE => Some(DirectionalOrientation::Inline),
        SBOL_REVERSE_COMPLEMENT | SO_REVERSE_COMPLEMENT => {
            Some(DirectionalOrientation::ReverseComplement)
        }
        _ => None,
    }
}

pub(crate) fn parent_by_identity_prefix<'a>(
    document: &'a Document,
    object: &Object,
) -> Option<&'a Object> {
    let identity = object.identity().to_string();
    let parent_identity = identity.rsplit_once('/')?.0;
    document.get(&Resource::iri(parent_identity))
}

fn sequential_relation_satisfied(
    restriction: &str,
    subject: &LinearLocation,
    object: &LinearLocation,
) -> bool {
    match restriction {
        SBOL_PRECEDES => subject.start < object.start,
        SBOL_STRICTLY_PRECEDES => subject.end < object.start,
        SBOL_MEETS => subject.end == object.start,
        SBOL_OVERLAPS => {
            subject.start < object.start && subject.end > object.start && subject.end < object.end
        }
        SBOL_CONTAINS => subject.start <= object.start && subject.end >= object.end,
        SBOL_STRICTLY_CONTAINS => subject.start < object.start && subject.end > object.end,
        SBOL_EQUALS => subject.start == object.start && subject.end == object.end,
        SBOL_FINISHES => subject.start > object.start && subject.end == object.end,
        SBOL_STARTS => subject.start == object.start && subject.end < object.end,
        _ => true,
    }
}

fn integer_value(object: &Object, predicate: &str) -> Option<i64> {
    object
        .first_literal_value(predicate)
        .and_then(|value| value.parse::<i64>().ok())
}

fn iri_values(object: &Object, predicate: &str) -> BTreeSet<Iri> {
    object.iris(predicate).cloned().collect()
}

fn term_values(object: &Object, predicate: &str) -> BTreeSet<Term> {
    object.values(predicate).iter().cloned().collect()
}

fn owned_predicate(predicate: &str) -> &'static str {
    COMPOSITE_PREDICATES
        .iter()
        .copied()
        .find(|candidate| *candidate == predicate)
        .unwrap_or(SBOL_HAS_FEATURE)
}

const COMPOSITE_PREDICATES: &[&str] = &[
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

const FEATURE_EQUIVALENCE_PREDICATES: &[&str] = &[
    RDF_TYPE,
    SBOL_ROLE,
    SBOL_ORIENTATION,
    SBOL_TYPE,
    SBOL_INSTANCE_OF,
    SBOL_DEFINITION,
    SBOL_ROLE_INTEGRATION,
];

#[cfg(test)]
mod tests {
    use crate::validation::resolver::*;

    const PREFIXES: &str = r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX sbol: <http://sbols.org/v3#>
"#;

    fn document(body: &str) -> Document {
        Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap()
    }

    #[test]
    fn ownership_index_supports_parent_and_child_lookup() {
        let document = document(
            r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:LocalSubComponent;
    sbol:displayId "feature";
    sbol:type SBO:0000251 .
"#,
        );
        let index = OwnershipIndex::new(&document);
        let component = Resource::iri("https://example.org/component");
        let feature = Resource::iri("https://example.org/component/feature");

        assert_eq!(
            index.single_parent(&feature, SBOL_HAS_FEATURE),
            Some(&component)
        );
        assert!(index.contains(&component, SBOL_HAS_FEATURE, &feature));
    }

    #[test]
    fn ownership_index_reports_multiple_parents() {
        let document = document(
            r#":component_a a sbol:Component;
    sbol:displayId "component_a";
    sbol:hasFeature :shared_feature;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component_b a sbol:Component;
    sbol:displayId "component_b";
    sbol:hasFeature :shared_feature;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:shared_feature a sbol:LocalSubComponent;
    sbol:displayId "shared_feature";
    sbol:type SBO:0000251 .
"#,
        );
        let index = OwnershipIndex::new(&document);
        let feature = Resource::iri("https://example.org/shared_feature");
        let parents = index.parents(&feature, SBOL_HAS_FEATURE);

        assert_eq!(parents.len(), 2);
        assert!(parents.contains(&&Resource::iri("https://example.org/component_a")));
        assert!(parents.contains(&&Resource::iri("https://example.org/component_b")));
    }

    #[test]
    fn component_reference_resolver_walks_nested_references() {
        let document = document(
            r#"<component/reference> a sbol:ComponentReference;
    sbol:displayId "reference";
    sbol:refersTo <definition/reference> .
<definition/reference> a sbol:ComponentReference;
    sbol:displayId "reference";
    sbol:refersTo <leaf_component/feature> .
<leaf_component/feature> a sbol:SequenceFeature;
    sbol:displayId "feature" .
"#,
        );
        let index = OwnershipIndex::new(&document);
        let resolver = ComponentReferenceResolver::new(&document, &index);
        let feature = Resource::iri("https://example.org/component/reference");
        let trace = resolver.trace_feature(&feature).unwrap();

        assert_eq!(
            trace.target,
            Resource::iri("https://example.org/leaf_component/feature")
        );
        assert_eq!(trace.path.len(), 2);
    }

    #[test]
    fn component_reference_resolver_reports_missing_targets() {
        let document = document(
            r#"<component/reference> a sbol:ComponentReference;
    sbol:displayId "reference";
    sbol:refersTo <missing/feature> .
"#,
        );
        let index = OwnershipIndex::new(&document);
        let resolver = ComponentReferenceResolver::new(&document, &index);
        let feature = Resource::iri("https://example.org/component/reference");

        assert!(matches!(
            resolver.trace_feature(&feature),
            Err(FeatureResolveError::MissingObject(resource))
                if resource == Resource::iri("https://example.org/missing/feature")
        ));
    }

    #[test]
    fn component_reference_resolver_reports_cycles() {
        let document = document(
            r#"<component/a> a sbol:ComponentReference;
    sbol:displayId "a";
    sbol:refersTo <component/b> .
<component/b> a sbol:ComponentReference;
    sbol:displayId "b";
    sbol:refersTo <component/a> .
"#,
        );
        let index = OwnershipIndex::new(&document);
        let resolver = ComponentReferenceResolver::new(&document, &index);
        let feature = Resource::iri("https://example.org/component/a");

        assert!(matches!(
            resolver.trace_feature(&feature),
            Err(FeatureResolveError::Cycle(_))
        ));
    }

    #[test]
    fn location_resolver_normalizes_entire_sequence() {
        let document = document(
            r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <component/feature/location> .
<component/feature/location> a sbol:EntireSequence;
    sbol:displayId "location";
    sbol:hasSequence :sequence .
"#,
        );
        let index = OwnershipIndex::new(&document);
        let resolver = LocationResolver::new(&document, &index);
        let feature = Resource::iri("https://example.org/component/feature");
        let locations = resolver.locations_for_feature(&feature);

        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].start, 0);
        assert_eq!(locations[0].end, 4);
    }

    #[test]
    fn location_resolver_follows_component_references_to_target_locations() {
        let document = document(
            r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:hasNamespace <https://example.org> .
<definition/feature> a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <definition/feature/location> .
<definition/feature/location> a sbol:Range;
    sbol:displayId "location";
    sbol:end "3";
    sbol:hasSequence :sequence;
    sbol:start "2" .
<component/reference> a sbol:ComponentReference;
    sbol:displayId "reference";
    sbol:refersTo <definition/feature> .
"#,
        );
        let index = OwnershipIndex::new(&document);
        let resolver = LocationResolver::new(&document, &index);
        let reference = Resource::iri("https://example.org/component/reference");
        let locations = resolver.locations_for_feature(&reference);

        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].start, 1);
        assert_eq!(locations[0].end, 3);
    }

    #[test]
    fn location_resolver_leaves_entire_sequence_without_length_unresolved() {
        let document = document(
            r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:hasNamespace <https://example.org> .
<component/feature> a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <component/feature/location> .
<component/feature/location> a sbol:EntireSequence;
    sbol:displayId "location";
    sbol:hasSequence :sequence .
"#,
        );
        let index = OwnershipIndex::new(&document);
        let resolver = LocationResolver::new(&document, &index);
        let feature = Resource::iri("https://example.org/component/feature");

        assert!(resolver.locations_for_feature(&feature).is_empty());
    }

    #[test]
    fn constraint_engine_detects_direct_identity_contradictions() {
        let document = document(
            r#":definition_a a sbol:Component;
    sbol:displayId "definition_a";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:definition_b a sbol:Component;
    sbol:displayId "definition_b";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/a>, <component/b>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/a> a sbol:SubComponent;
    sbol:displayId "a";
    sbol:instanceOf :definition_a .
<component/b> a sbol:SubComponent;
    sbol:displayId "b";
    sbol:instanceOf :definition_b .
"#,
        );
        let index = OwnershipIndex::new(&document);
        let engine = ConstraintEngine::new(&document, &index);
        let subject = Resource::iri("https://example.org/component/a");
        let object = Resource::iri("https://example.org/component/b");

        assert!(matches!(
            engine.table8_relation(SBOL_VERIFY_IDENTICAL, &subject, &object),
            RelationOutcome::Contradicted { .. }
        ));
    }

    #[test]
    fn constraint_engine_keeps_replaces_and_spatial_table9_relations_undecided() {
        let document = document(
            r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/a>, <component/b>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/a> a sbol:LocalSubComponent;
    sbol:displayId "a";
    sbol:type SBO:0000251 .
<component/b> a sbol:LocalSubComponent;
    sbol:displayId "b";
    sbol:type SBO:0000251 .
"#,
        );
        let index = OwnershipIndex::new(&document);
        let engine = ConstraintEngine::new(&document, &index);
        let subject = Resource::iri("https://example.org/component/a");
        let object = Resource::iri("https://example.org/component/b");

        assert_eq!(
            engine.table8_relation(SBOL_REPLACES, &subject, &object),
            RelationOutcome::Unknown
        );
        assert_eq!(
            engine.table8_relation(SBOL_COVERS, &subject, &object),
            RelationOutcome::Unsupported
        );
    }
}
