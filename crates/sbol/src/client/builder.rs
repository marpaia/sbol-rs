//! Builder types for owned SBOL classes.
//!
//! Every owned class has a `Class::builder(namespace_or_parent, display_id)`
//! constructor that returns a `ClassBuilder` with chainable setters for every
//! property. Required fields are tracked separately from the underlying `Vec`
//! storage; `build()` returns `Err(BuildError::MissingRequired)` if any
//! cardinality-required field was never set.
//!
//! Setters consume and return `Self` for fluent single-expression construction.
//! Collection setters come in two shapes:
//!
//! - `field(values)` replaces the entire collection.
//! - `add_field(value)` appends a single value.
//!
//! Inherited `Identified` and `TopLevel` properties (`name`, `description`,
//! `derived_from`, `generated_by`, `measures`, `attachments`) are flat methods
//! on each builder — no nested `.identified().name(...)` paths.

use crate::client::identity::{build_child_identity, build_top_level_identity};
use crate::client::{
    Activity, Agent, Association, Attachment, BinaryPrefix, Collection, CombinatorialDerivation,
    Component, ComponentReference, CompoundUnit, Constraint, Cut, EntireSequence, Experiment,
    ExperimentalData, ExtensionTriple, ExternallyDefined, FeatureData, IdentifiedData,
    IdentifiedExtension, Implementation, Interaction, Interface, LocalSubComponent, LocationData,
    Measure, Model, Participation, Plan, Prefix, PrefixData, PrefixedUnit, Range, SIPrefix,
    Sequence, SequenceFeature, SingularUnit, SubComponent, TopLevelData, Unit, UnitData,
    UnitDivision, UnitExponentiation, UnitMultiplication, Usage, VariableFeature,
};
use crate::error::BuildError;
use crate::identity::{DisplayId, HashAlgorithm, Namespace};
use crate::{Iri, Resource, SbolClass, Term};

// ---------------------------------------------------------------------------
// Shared setter macros
// ---------------------------------------------------------------------------

macro_rules! identified_setters {
    () => {
        pub fn name(mut self, value: impl Into<String>) -> Self {
            self.identified.name = Some(value.into());
            self
        }

        pub fn description(mut self, value: impl Into<String>) -> Self {
            self.identified.description = Some(value.into());
            self
        }

        pub fn derived_from(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.identified.derived_from = values.into_iter().collect();
            self
        }

        pub fn add_derived_from(mut self, value: Resource) -> Self {
            self.identified.derived_from.push(value);
            self
        }

        pub fn generated_by(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.identified.generated_by = values.into_iter().collect();
            self
        }

        pub fn add_generated_by(mut self, value: Resource) -> Self {
            self.identified.generated_by.push(value);
            self
        }

        pub fn measures(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.identified.measures = values.into_iter().collect();
            self
        }

        pub fn add_measure(mut self, value: Resource) -> Self {
            self.identified.measures.push(value);
            self
        }

        /// Attach a non-SBOL annotation triple. The predicate must be outside
        /// the SBOL, PROV, and OM vocabularies; predicates inside those
        /// vocabularies belong on dedicated setters and are emitted twice if
        /// pushed here.
        pub fn extension(mut self, predicate: Iri, value: Term) -> Self {
            self.identified.extensions.push(ExtensionTriple {
                predicate,
                object: value,
            });
            self
        }
    };
}

macro_rules! top_level_setters {
    () => {
        pub fn attachments(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.top_level.attachments = values.into_iter().collect();
            self
        }

        pub fn add_attachment(mut self, value: Resource) -> Self {
            self.top_level.attachments.push(value);
            self
        }
    };
}

macro_rules! feature_setters {
    () => {
        pub fn roles(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
            self.feature.roles = values.into_iter().collect();
            self
        }

        pub fn add_role(mut self, value: Iri) -> Self {
            self.feature.roles.push(value);
            self
        }

        pub fn orientation(mut self, value: Iri) -> Self {
            self.feature.orientation = Some(value);
            self
        }
    };
}

macro_rules! location_setters {
    () => {
        pub fn sequence(mut self, value: Resource) -> Self {
            self.location.sequence = Some(value);
            self
        }

        pub fn orientation(mut self, value: Iri) -> Self {
            self.location.orientation = Some(value);
            self
        }

        pub fn order(mut self, value: i64) -> Self {
            self.location.order = Some(value);
            self
        }
    };
}

macro_rules! unit_setters {
    () => {
        pub fn label(mut self, value: impl Into<String>) -> Self {
            self.unit.label = Some(value.into());
            self
        }

        pub fn symbol(mut self, value: impl Into<String>) -> Self {
            self.unit.symbol = Some(value.into());
            self
        }

        pub fn alternative_labels(mut self, values: impl IntoIterator<Item = String>) -> Self {
            self.unit.alternative_labels = values.into_iter().collect();
            self
        }

        pub fn add_alternative_label(mut self, value: impl Into<String>) -> Self {
            self.unit.alternative_labels.push(value.into());
            self
        }

        pub fn alternative_symbols(mut self, values: impl IntoIterator<Item = String>) -> Self {
            self.unit.alternative_symbols = values.into_iter().collect();
            self
        }

        pub fn add_alternative_symbol(mut self, value: impl Into<String>) -> Self {
            self.unit.alternative_symbols.push(value.into());
            self
        }

        pub fn comment(mut self, value: impl Into<String>) -> Self {
            self.unit.comment = Some(value.into());
            self
        }

        pub fn long_comment(mut self, value: impl Into<String>) -> Self {
            self.unit.long_comment = Some(value.into());
            self
        }
    };
}

macro_rules! prefix_setters {
    () => {
        pub fn label(mut self, value: impl Into<String>) -> Self {
            self.prefix.label = Some(value.into());
            self
        }

        pub fn symbol(mut self, value: impl Into<String>) -> Self {
            self.prefix.symbol = Some(value.into());
            self
        }

        pub fn has_factor(mut self, value: f64) -> Self {
            self.prefix.has_factor = Some(value.to_string());
            self
        }

        pub fn alternative_labels(mut self, values: impl IntoIterator<Item = String>) -> Self {
            self.prefix.alternative_labels = values.into_iter().collect();
            self
        }

        pub fn add_alternative_label(mut self, value: impl Into<String>) -> Self {
            self.prefix.alternative_labels.push(value.into());
            self
        }

        pub fn alternative_symbols(mut self, values: impl IntoIterator<Item = String>) -> Self {
            self.prefix.alternative_symbols = values.into_iter().collect();
            self
        }

        pub fn add_alternative_symbol(mut self, value: impl Into<String>) -> Self {
            self.prefix.alternative_symbols.push(value.into());
            self
        }

        pub fn comment(mut self, value: impl Into<String>) -> Self {
            self.prefix.comment = Some(value.into());
            self
        }

        pub fn long_comment(mut self, value: impl Into<String>) -> Self {
            self.prefix.long_comment = Some(value.into());
            self
        }
    };
}

fn missing(identity: &Resource, class: SbolClass, property: &'static str) -> BuildError {
    BuildError::MissingRequired {
        identity: identity.clone(),
        class,
        property,
    }
}

fn identified_seed(display_id: &DisplayId) -> IdentifiedData {
    IdentifiedData {
        display_id: Some(display_id.as_str().to_string()),
        ..IdentifiedData::default()
    }
}

fn top_level_seed(namespace: &Namespace) -> TopLevelData {
    TopLevelData {
        namespace: Some(namespace.as_iri().clone()),
        ..TopLevelData::default()
    }
}

// ---------------------------------------------------------------------------
// TopLevel classes
// ---------------------------------------------------------------------------

/// Builder for [`Component`].
#[derive(Clone, Debug)]
pub struct ComponentBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    types: Vec<Iri>,
    roles: Vec<Iri>,
    sequences: Vec<Resource>,
    features: Vec<Resource>,
    constraints: Vec<Resource>,
    interactions: Vec<Resource>,
    interfaces: Vec<Resource>,
    models: Vec<Resource>,
}

impl ComponentBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        let identified = identified_seed(&display_id);
        let top_level = top_level_seed(&namespace);
        Self {
            identity,
            identified,
            top_level,
            types: Vec::new(),
            roles: Vec::new(),
            sequences: Vec::new(),
            features: Vec::new(),
            constraints: Vec::new(),
            interactions: Vec::new(),
            interfaces: Vec::new(),
            models: Vec::new(),
        }
    }

    identified_setters!();
    top_level_setters!();

    pub fn types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.types = values.into_iter().collect();
        self
    }

    pub fn add_type(mut self, value: Iri) -> Self {
        self.types.push(value);
        self
    }

    pub fn component_roles(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.roles = values.into_iter().collect();
        self
    }

    pub fn add_component_role(mut self, value: Iri) -> Self {
        self.roles.push(value);
        self
    }

    pub fn sequences(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.sequences = values.into_iter().collect();
        self
    }

    pub fn add_sequence(mut self, value: Resource) -> Self {
        self.sequences.push(value);
        self
    }

    pub fn features(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.features = values.into_iter().collect();
        self
    }

    pub fn add_feature(mut self, value: Resource) -> Self {
        self.features.push(value);
        self
    }

    pub fn constraints(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.constraints = values.into_iter().collect();
        self
    }

    pub fn add_constraint(mut self, value: Resource) -> Self {
        self.constraints.push(value);
        self
    }

    pub fn interactions(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.interactions = values.into_iter().collect();
        self
    }

    pub fn add_interaction(mut self, value: Resource) -> Self {
        self.interactions.push(value);
        self
    }

    pub fn interfaces(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.interfaces = values.into_iter().collect();
        self
    }

    pub fn add_interface(mut self, value: Resource) -> Self {
        self.interfaces.push(value);
        self
    }

    pub fn models(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.models = values.into_iter().collect();
        self
    }

    pub fn add_model(mut self, value: Resource) -> Self {
        self.models.push(value);
        self
    }

    pub fn build(self) -> Result<Component, BuildError> {
        if self.types.is_empty() {
            return Err(missing(&self.identity, SbolClass::Component, "type"));
        }
        Ok(Component {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            types: self.types,
            roles: self.roles,
            sequences: self.sequences,
            features: self.features,
            constraints: self.constraints,
            interactions: self.interactions,
            interfaces: self.interfaces,
            models: self.models,
        })
    }
}

/// Builder for [`Sequence`].
#[derive(Clone, Debug)]
pub struct SequenceBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    elements: Option<String>,
    encoding: Option<Iri>,
}

impl SequenceBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            elements: None,
            encoding: None,
        }
    }

    identified_setters!();
    top_level_setters!();

    pub fn elements(mut self, value: impl Into<String>) -> Self {
        self.elements = Some(value.into());
        self
    }

    pub fn encoding(mut self, value: Iri) -> Self {
        self.encoding = Some(value);
        self
    }

    pub fn build(self) -> Result<Sequence, BuildError> {
        Ok(Sequence {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            elements: self.elements,
            encoding: self.encoding,
        })
    }
}

/// Builder for [`Attachment`].
#[derive(Clone, Debug)]
pub struct AttachmentBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    source: Option<Resource>,
    format: Option<Iri>,
    size: Option<i64>,
    hash: Option<String>,
    hash_algorithm: Option<HashAlgorithm>,
}

impl AttachmentBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            source: None,
            format: None,
            size: None,
            hash: None,
            hash_algorithm: None,
        }
    }

    identified_setters!();
    top_level_setters!();

    pub fn source(mut self, value: Resource) -> Self {
        self.source = Some(value);
        self
    }

    pub fn format(mut self, value: Iri) -> Self {
        self.format = Some(value);
        self
    }

    pub fn size(mut self, value: i64) -> Self {
        self.size = Some(value);
        self
    }

    pub fn hash(mut self, value: impl Into<String>) -> Self {
        self.hash = Some(value.into());
        self
    }

    pub fn hash_algorithm(mut self, value: HashAlgorithm) -> Self {
        self.hash_algorithm = Some(value);
        self
    }

    pub fn build(self) -> Result<Attachment, BuildError> {
        let source = self
            .source
            .ok_or_else(|| missing(&self.identity, SbolClass::Attachment, "source"))?;
        Ok(Attachment {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            source: Some(source),
            format: self.format,
            size: self.size,
            hash: self.hash,
            hash_algorithm: self.hash_algorithm.map(|h| h.as_str().to_string()),
        })
    }
}

/// Builder for [`Collection`].
#[derive(Clone, Debug)]
pub struct CollectionBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    members: Vec<Resource>,
}

impl CollectionBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            members: Vec::new(),
        }
    }

    identified_setters!();
    top_level_setters!();

    pub fn members(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.members = values.into_iter().collect();
        self
    }

    pub fn add_member(mut self, value: Resource) -> Self {
        self.members.push(value);
        self
    }

    pub fn build(self) -> Result<Collection, BuildError> {
        Ok(Collection {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            members: self.members,
        })
    }
}

/// Builder for [`CombinatorialDerivation`].
#[derive(Clone, Debug)]
pub struct CombinatorialDerivationBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    template: Option<Resource>,
    strategy: Option<Iri>,
    variable_features: Vec<Resource>,
}

impl CombinatorialDerivationBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            template: None,
            strategy: None,
            variable_features: Vec::new(),
        }
    }

    identified_setters!();
    top_level_setters!();

    pub fn template(mut self, value: Resource) -> Self {
        self.template = Some(value);
        self
    }

    pub fn strategy(mut self, value: Iri) -> Self {
        self.strategy = Some(value);
        self
    }

    pub fn variable_features(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.variable_features = values.into_iter().collect();
        self
    }

    pub fn add_variable_feature(mut self, value: Resource) -> Self {
        self.variable_features.push(value);
        self
    }

    pub fn build(self) -> Result<CombinatorialDerivation, BuildError> {
        let template = self.template.ok_or_else(|| {
            missing(
                &self.identity,
                SbolClass::CombinatorialDerivation,
                "template",
            )
        })?;
        Ok(CombinatorialDerivation {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            template: Some(template),
            strategy: self.strategy,
            variable_features: self.variable_features,
        })
    }
}

/// Builder for [`Experiment`].
#[derive(Clone, Debug)]
pub struct ExperimentBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    members: Vec<Resource>,
}

impl ExperimentBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            members: Vec::new(),
        }
    }

    identified_setters!();
    top_level_setters!();

    pub fn members(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.members = values.into_iter().collect();
        self
    }

    pub fn add_member(mut self, value: Resource) -> Self {
        self.members.push(value);
        self
    }

    pub fn build(self) -> Result<Experiment, BuildError> {
        Ok(Experiment {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            members: self.members,
        })
    }
}

/// Builder for [`ExperimentalData`].
#[derive(Clone, Debug)]
pub struct ExperimentalDataBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
}

impl ExperimentalDataBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
        }
    }

    identified_setters!();
    top_level_setters!();

    pub fn build(self) -> Result<ExperimentalData, BuildError> {
        Ok(ExperimentalData {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
        })
    }
}

/// Builder for [`Implementation`].
#[derive(Clone, Debug)]
pub struct ImplementationBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    built: Option<Resource>,
}

impl ImplementationBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            built: None,
        }
    }

    identified_setters!();
    top_level_setters!();

    pub fn built(mut self, value: Resource) -> Self {
        self.built = Some(value);
        self
    }

    pub fn build(self) -> Result<Implementation, BuildError> {
        Ok(Implementation {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            built: self.built,
        })
    }
}

/// Builder for [`Model`].
#[derive(Clone, Debug)]
pub struct ModelBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    source: Option<Resource>,
    language: Option<Iri>,
    framework: Option<Iri>,
}

impl ModelBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            source: None,
            language: None,
            framework: None,
        }
    }

    identified_setters!();
    top_level_setters!();

    pub fn source(mut self, value: Resource) -> Self {
        self.source = Some(value);
        self
    }

    pub fn language(mut self, value: Iri) -> Self {
        self.language = Some(value);
        self
    }

    pub fn framework(mut self, value: Iri) -> Self {
        self.framework = Some(value);
        self
    }

    pub fn build(self) -> Result<Model, BuildError> {
        let source = self
            .source
            .ok_or_else(|| missing(&self.identity, SbolClass::Model, "source"))?;
        let language = self
            .language
            .ok_or_else(|| missing(&self.identity, SbolClass::Model, "language"))?;
        let framework = self
            .framework
            .ok_or_else(|| missing(&self.identity, SbolClass::Model, "framework"))?;
        Ok(Model {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            source: Some(source),
            language: Some(language),
            framework: Some(framework),
        })
    }
}

// ---------------------------------------------------------------------------
// Feature classes (child of Component)
// ---------------------------------------------------------------------------

fn child_seed(
    parent: &Resource,
    display_id: DisplayId,
) -> Result<(Resource, IdentifiedData), BuildError> {
    let identity = build_child_identity(parent, &display_id)?;
    Ok((identity, identified_seed(&display_id)))
}

/// Builder for [`SubComponent`].
#[derive(Clone, Debug)]
pub struct SubComponentBuilder {
    identity: Resource,
    identified: IdentifiedData,
    feature: FeatureData,
    instance_of: Option<Resource>,
    role_integration: Option<Iri>,
    locations: Vec<Resource>,
    source_locations: Vec<Resource>,
}

impl SubComponentBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            feature: FeatureData::default(),
            instance_of: None,
            role_integration: None,
            locations: Vec::new(),
            source_locations: Vec::new(),
        })
    }

    identified_setters!();
    feature_setters!();

    pub fn instance_of(mut self, value: Resource) -> Self {
        self.instance_of = Some(value);
        self
    }

    pub fn role_integration(mut self, value: Iri) -> Self {
        self.role_integration = Some(value);
        self
    }

    pub fn locations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.locations = values.into_iter().collect();
        self
    }

    pub fn add_location(mut self, value: Resource) -> Self {
        self.locations.push(value);
        self
    }

    pub fn source_locations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.source_locations = values.into_iter().collect();
        self
    }

    pub fn add_source_location(mut self, value: Resource) -> Self {
        self.source_locations.push(value);
        self
    }

    pub fn build(self) -> Result<SubComponent, BuildError> {
        let instance_of = self
            .instance_of
            .ok_or_else(|| missing(&self.identity, SbolClass::SubComponent, "instanceOf"))?;
        Ok(SubComponent {
            identity: self.identity,
            identified: self.identified,
            feature: self.feature,
            instance_of: Some(instance_of),
            role_integration: self.role_integration,
            locations: self.locations,
            source_locations: self.source_locations,
        })
    }
}

/// Builder for [`LocalSubComponent`].
#[derive(Clone, Debug)]
pub struct LocalSubComponentBuilder {
    identity: Resource,
    identified: IdentifiedData,
    feature: FeatureData,
    types: Vec<Iri>,
    locations: Vec<Resource>,
}

impl LocalSubComponentBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            feature: FeatureData::default(),
            types: Vec::new(),
            locations: Vec::new(),
        })
    }

    identified_setters!();
    feature_setters!();

    pub fn types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.types = values.into_iter().collect();
        self
    }

    pub fn add_type(mut self, value: Iri) -> Self {
        self.types.push(value);
        self
    }

    pub fn locations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.locations = values.into_iter().collect();
        self
    }

    pub fn add_location(mut self, value: Resource) -> Self {
        self.locations.push(value);
        self
    }

    pub fn build(self) -> Result<LocalSubComponent, BuildError> {
        if self.types.is_empty() {
            return Err(missing(
                &self.identity,
                SbolClass::LocalSubComponent,
                "type",
            ));
        }
        Ok(LocalSubComponent {
            identity: self.identity,
            identified: self.identified,
            feature: self.feature,
            types: self.types,
            locations: self.locations,
        })
    }
}

/// Builder for [`SequenceFeature`].
#[derive(Clone, Debug)]
pub struct SequenceFeatureBuilder {
    identity: Resource,
    identified: IdentifiedData,
    feature: FeatureData,
    locations: Vec<Resource>,
}

impl SequenceFeatureBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            feature: FeatureData::default(),
            locations: Vec::new(),
        })
    }

    identified_setters!();
    feature_setters!();

    pub fn locations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.locations = values.into_iter().collect();
        self
    }

    pub fn add_location(mut self, value: Resource) -> Self {
        self.locations.push(value);
        self
    }

    pub fn build(self) -> Result<SequenceFeature, BuildError> {
        if self.locations.is_empty() {
            return Err(missing(
                &self.identity,
                SbolClass::SequenceFeature,
                "hasLocation",
            ));
        }
        Ok(SequenceFeature {
            identity: self.identity,
            identified: self.identified,
            feature: self.feature,
            locations: self.locations,
        })
    }
}

/// Builder for [`ComponentReference`].
#[derive(Clone, Debug)]
pub struct ComponentReferenceBuilder {
    identity: Resource,
    identified: IdentifiedData,
    feature: FeatureData,
    in_child_of: Option<Resource>,
    refers_to: Option<Resource>,
}

impl ComponentReferenceBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            feature: FeatureData::default(),
            in_child_of: None,
            refers_to: None,
        })
    }

    identified_setters!();
    feature_setters!();

    pub fn in_child_of(mut self, value: Resource) -> Self {
        self.in_child_of = Some(value);
        self
    }

    pub fn refers_to(mut self, value: Resource) -> Self {
        self.refers_to = Some(value);
        self
    }

    pub fn build(self) -> Result<ComponentReference, BuildError> {
        let in_child_of = self
            .in_child_of
            .ok_or_else(|| missing(&self.identity, SbolClass::ComponentReference, "inChildOf"))?;
        let refers_to = self
            .refers_to
            .ok_or_else(|| missing(&self.identity, SbolClass::ComponentReference, "refersTo"))?;
        Ok(ComponentReference {
            identity: self.identity,
            identified: self.identified,
            feature: self.feature,
            in_child_of: Some(in_child_of),
            refers_to: Some(refers_to),
        })
    }
}

/// Builder for [`ExternallyDefined`].
#[derive(Clone, Debug)]
pub struct ExternallyDefinedBuilder {
    identity: Resource,
    identified: IdentifiedData,
    feature: FeatureData,
    definition: Option<Resource>,
    types: Vec<Iri>,
}

impl ExternallyDefinedBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            feature: FeatureData::default(),
            definition: None,
            types: Vec::new(),
        })
    }

    identified_setters!();
    feature_setters!();

    pub fn definition(mut self, value: Resource) -> Self {
        self.definition = Some(value);
        self
    }

    pub fn types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.types = values.into_iter().collect();
        self
    }

    pub fn add_type(mut self, value: Iri) -> Self {
        self.types.push(value);
        self
    }

    pub fn build(self) -> Result<ExternallyDefined, BuildError> {
        let definition = self
            .definition
            .ok_or_else(|| missing(&self.identity, SbolClass::ExternallyDefined, "definition"))?;
        if self.types.is_empty() {
            return Err(missing(
                &self.identity,
                SbolClass::ExternallyDefined,
                "type",
            ));
        }
        Ok(ExternallyDefined {
            identity: self.identity,
            identified: self.identified,
            feature: self.feature,
            definition: Some(definition),
            types: self.types,
        })
    }
}

// ---------------------------------------------------------------------------
// Location classes
// ---------------------------------------------------------------------------

/// Builder for [`Range`].
#[derive(Clone, Debug)]
pub struct RangeBuilder {
    identity: Resource,
    identified: IdentifiedData,
    location: LocationData,
    start: Option<i64>,
    end: Option<i64>,
}

impl RangeBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            location: LocationData::default(),
            start: None,
            end: None,
        })
    }

    identified_setters!();
    location_setters!();

    pub fn start(mut self, value: i64) -> Self {
        self.start = Some(value);
        self
    }

    pub fn end(mut self, value: i64) -> Self {
        self.end = Some(value);
        self
    }

    pub fn build(self) -> Result<Range, BuildError> {
        let start = self
            .start
            .ok_or_else(|| missing(&self.identity, SbolClass::Range, "start"))?;
        let end = self
            .end
            .ok_or_else(|| missing(&self.identity, SbolClass::Range, "end"))?;
        Ok(Range {
            identity: self.identity,
            identified: self.identified,
            location: self.location,
            start: Some(start),
            end: Some(end),
        })
    }
}

/// Builder for [`Cut`].
#[derive(Clone, Debug)]
pub struct CutBuilder {
    identity: Resource,
    identified: IdentifiedData,
    location: LocationData,
    at: Option<i64>,
}

impl CutBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            location: LocationData::default(),
            at: None,
        })
    }

    identified_setters!();
    location_setters!();

    pub fn at(mut self, value: i64) -> Self {
        self.at = Some(value);
        self
    }

    pub fn build(self) -> Result<Cut, BuildError> {
        let at = self
            .at
            .ok_or_else(|| missing(&self.identity, SbolClass::Cut, "at"))?;
        Ok(Cut {
            identity: self.identity,
            identified: self.identified,
            location: self.location,
            at: Some(at),
        })
    }
}

/// Builder for [`EntireSequence`].
#[derive(Clone, Debug)]
pub struct EntireSequenceBuilder {
    identity: Resource,
    identified: IdentifiedData,
    location: LocationData,
}

impl EntireSequenceBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            location: LocationData::default(),
        })
    }

    identified_setters!();
    location_setters!();

    pub fn build(self) -> Result<EntireSequence, BuildError> {
        Ok(EntireSequence {
            identity: self.identity,
            identified: self.identified,
            location: self.location,
        })
    }
}

// ---------------------------------------------------------------------------
// Usage classes (child of Component or Interaction)
// ---------------------------------------------------------------------------

/// Builder for [`Constraint`].
#[derive(Clone, Debug)]
pub struct ConstraintBuilder {
    identity: Resource,
    identified: IdentifiedData,
    subject: Option<Resource>,
    constrained_object: Option<Resource>,
    restriction: Option<Iri>,
}

impl ConstraintBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            subject: None,
            constrained_object: None,
            restriction: None,
        })
    }

    identified_setters!();

    pub fn subject(mut self, value: Resource) -> Self {
        self.subject = Some(value);
        self
    }

    pub fn constrained_object(mut self, value: Resource) -> Self {
        self.constrained_object = Some(value);
        self
    }

    pub fn restriction(mut self, value: Iri) -> Self {
        self.restriction = Some(value);
        self
    }

    pub fn build(self) -> Result<Constraint, BuildError> {
        let subject = self
            .subject
            .ok_or_else(|| missing(&self.identity, SbolClass::Constraint, "subject"))?;
        let constrained_object = self
            .constrained_object
            .ok_or_else(|| missing(&self.identity, SbolClass::Constraint, "object"))?;
        let restriction = self
            .restriction
            .ok_or_else(|| missing(&self.identity, SbolClass::Constraint, "restriction"))?;
        Ok(Constraint {
            identity: self.identity,
            identified: self.identified,
            subject: Some(subject),
            constrained_object: Some(constrained_object),
            restriction: Some(restriction),
        })
    }
}

/// Builder for [`Interaction`].
#[derive(Clone, Debug)]
pub struct InteractionBuilder {
    identity: Resource,
    identified: IdentifiedData,
    types: Vec<Iri>,
    participations: Vec<Resource>,
}

impl InteractionBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            types: Vec::new(),
            participations: Vec::new(),
        })
    }

    identified_setters!();

    pub fn types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.types = values.into_iter().collect();
        self
    }

    pub fn add_type(mut self, value: Iri) -> Self {
        self.types.push(value);
        self
    }

    pub fn participations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.participations = values.into_iter().collect();
        self
    }

    pub fn add_participation(mut self, value: Resource) -> Self {
        self.participations.push(value);
        self
    }

    pub fn build(self) -> Result<Interaction, BuildError> {
        if self.types.is_empty() {
            return Err(missing(&self.identity, SbolClass::Interaction, "type"));
        }
        Ok(Interaction {
            identity: self.identity,
            identified: self.identified,
            types: self.types,
            participations: self.participations,
        })
    }
}

/// Builder for [`Interface`].
#[derive(Clone, Debug)]
pub struct InterfaceBuilder {
    identity: Resource,
    identified: IdentifiedData,
    inputs: Vec<Resource>,
    outputs: Vec<Resource>,
    nondirectional: Vec<Resource>,
}

impl InterfaceBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            inputs: Vec::new(),
            outputs: Vec::new(),
            nondirectional: Vec::new(),
        })
    }

    identified_setters!();

    pub fn inputs(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.inputs = values.into_iter().collect();
        self
    }

    pub fn add_input(mut self, value: Resource) -> Self {
        self.inputs.push(value);
        self
    }

    pub fn outputs(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.outputs = values.into_iter().collect();
        self
    }

    pub fn add_output(mut self, value: Resource) -> Self {
        self.outputs.push(value);
        self
    }

    pub fn nondirectional(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.nondirectional = values.into_iter().collect();
        self
    }

    pub fn add_nondirectional(mut self, value: Resource) -> Self {
        self.nondirectional.push(value);
        self
    }

    pub fn build(self) -> Result<Interface, BuildError> {
        Ok(Interface {
            identity: self.identity,
            identified: self.identified,
            inputs: self.inputs,
            outputs: self.outputs,
            nondirectional: self.nondirectional,
        })
    }
}

/// Builder for [`Participation`].
#[derive(Clone, Debug)]
pub struct ParticipationBuilder {
    identity: Resource,
    identified: IdentifiedData,
    roles: Vec<Iri>,
    participant: Option<Resource>,
    higher_order_participant: Option<Resource>,
}

impl ParticipationBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            roles: Vec::new(),
            participant: None,
            higher_order_participant: None,
        })
    }

    identified_setters!();

    pub fn roles(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.roles = values.into_iter().collect();
        self
    }

    pub fn add_role(mut self, value: Iri) -> Self {
        self.roles.push(value);
        self
    }

    pub fn participant(mut self, value: Resource) -> Self {
        self.participant = Some(value);
        self
    }

    pub fn higher_order_participant(mut self, value: Resource) -> Self {
        self.higher_order_participant = Some(value);
        self
    }

    pub fn build(self) -> Result<Participation, BuildError> {
        if self.roles.is_empty() {
            return Err(missing(&self.identity, SbolClass::Participation, "role"));
        }
        Ok(Participation {
            identity: self.identity,
            identified: self.identified,
            roles: self.roles,
            participant: self.participant,
            higher_order_participant: self.higher_order_participant,
        })
    }
}

/// Builder for [`VariableFeature`].
#[derive(Clone, Debug)]
pub struct VariableFeatureBuilder {
    identity: Resource,
    identified: IdentifiedData,
    cardinality: Option<Iri>,
    variable: Option<Resource>,
    variants: Vec<Resource>,
    variant_collections: Vec<Resource>,
    variant_derivations: Vec<Resource>,
    variant_measures: Vec<Resource>,
}

impl VariableFeatureBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            cardinality: None,
            variable: None,
            variants: Vec::new(),
            variant_collections: Vec::new(),
            variant_derivations: Vec::new(),
            variant_measures: Vec::new(),
        })
    }

    identified_setters!();

    pub fn cardinality(mut self, value: Iri) -> Self {
        self.cardinality = Some(value);
        self
    }

    pub fn variable(mut self, value: Resource) -> Self {
        self.variable = Some(value);
        self
    }

    pub fn variants(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.variants = values.into_iter().collect();
        self
    }

    pub fn add_variant(mut self, value: Resource) -> Self {
        self.variants.push(value);
        self
    }

    pub fn variant_collections(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.variant_collections = values.into_iter().collect();
        self
    }

    pub fn add_variant_collection(mut self, value: Resource) -> Self {
        self.variant_collections.push(value);
        self
    }

    pub fn variant_derivations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.variant_derivations = values.into_iter().collect();
        self
    }

    pub fn add_variant_derivation(mut self, value: Resource) -> Self {
        self.variant_derivations.push(value);
        self
    }

    pub fn variant_measures(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.variant_measures = values.into_iter().collect();
        self
    }

    pub fn add_variant_measure(mut self, value: Resource) -> Self {
        self.variant_measures.push(value);
        self
    }

    pub fn build(self) -> Result<VariableFeature, BuildError> {
        let cardinality = self
            .cardinality
            .ok_or_else(|| missing(&self.identity, SbolClass::VariableFeature, "cardinality"))?;
        let variable = self
            .variable
            .ok_or_else(|| missing(&self.identity, SbolClass::VariableFeature, "variable"))?;
        Ok(VariableFeature {
            identity: self.identity,
            identified: self.identified,
            cardinality: Some(cardinality),
            variable: Some(variable),
            variants: self.variants,
            variant_collections: self.variant_collections,
            variant_derivations: self.variant_derivations,
            variant_measures: self.variant_measures,
        })
    }
}

// ---------------------------------------------------------------------------
// PROV-O classes (Appendix A.1)
// ---------------------------------------------------------------------------

/// Builder for [`Activity`].
#[derive(Clone, Debug)]
pub struct ActivityBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    types: Vec<Iri>,
    started_at_time: Option<String>,
    ended_at_time: Option<String>,
    was_informed_by: Vec<Resource>,
    qualified_usage: Vec<Resource>,
    qualified_association: Vec<Resource>,
}

impl ActivityBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            types: Vec::new(),
            started_at_time: None,
            ended_at_time: None,
            was_informed_by: Vec::new(),
            qualified_usage: Vec::new(),
            qualified_association: Vec::new(),
        }
    }

    identified_setters!();
    top_level_setters!();

    pub fn types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.types = values.into_iter().collect();
        self
    }

    pub fn add_type(mut self, value: Iri) -> Self {
        self.types.push(value);
        self
    }

    pub fn started_at_time(mut self, value: impl Into<String>) -> Self {
        self.started_at_time = Some(value.into());
        self
    }

    pub fn ended_at_time(mut self, value: impl Into<String>) -> Self {
        self.ended_at_time = Some(value.into());
        self
    }

    pub fn was_informed_by(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.was_informed_by = values.into_iter().collect();
        self
    }

    pub fn add_was_informed_by(mut self, value: Resource) -> Self {
        self.was_informed_by.push(value);
        self
    }

    pub fn qualified_usage(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.qualified_usage = values.into_iter().collect();
        self
    }

    pub fn add_qualified_usage(mut self, value: Resource) -> Self {
        self.qualified_usage.push(value);
        self
    }

    pub fn qualified_association(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.qualified_association = values.into_iter().collect();
        self
    }

    pub fn add_qualified_association(mut self, value: Resource) -> Self {
        self.qualified_association.push(value);
        self
    }

    pub fn build(self) -> Result<Activity, BuildError> {
        Ok(Activity {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            types: self.types,
            started_at_time: self.started_at_time,
            ended_at_time: self.ended_at_time,
            was_informed_by: self.was_informed_by,
            qualified_usage: self.qualified_usage,
            qualified_association: self.qualified_association,
        })
    }
}

/// Builder for [`Agent`].
#[derive(Clone, Debug)]
pub struct AgentBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
}

impl AgentBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
        }
    }

    identified_setters!();
    top_level_setters!();

    pub fn build(self) -> Result<Agent, BuildError> {
        Ok(Agent {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
        })
    }
}

/// Builder for [`Plan`].
#[derive(Clone, Debug)]
pub struct PlanBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
}

impl PlanBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
        }
    }

    identified_setters!();
    top_level_setters!();

    pub fn build(self) -> Result<Plan, BuildError> {
        Ok(Plan {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
        })
    }
}

/// Builder for [`Association`].
#[derive(Clone, Debug)]
pub struct AssociationBuilder {
    identity: Resource,
    identified: IdentifiedData,
    agent: Option<Resource>,
    had_role: Vec<Iri>,
    had_plan: Option<Resource>,
}

impl AssociationBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            agent: None,
            had_role: Vec::new(),
            had_plan: None,
        })
    }

    identified_setters!();

    pub fn agent(mut self, value: Resource) -> Self {
        self.agent = Some(value);
        self
    }

    pub fn had_role(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.had_role = values.into_iter().collect();
        self
    }

    pub fn add_had_role(mut self, value: Iri) -> Self {
        self.had_role.push(value);
        self
    }

    pub fn had_plan(mut self, value: Resource) -> Self {
        self.had_plan = Some(value);
        self
    }

    pub fn build(self) -> Result<Association, BuildError> {
        let agent = self
            .agent
            .ok_or_else(|| missing(&self.identity, SbolClass::ProvAssociation, "agent"))?;
        Ok(Association {
            identity: self.identity,
            identified: self.identified,
            agent: Some(agent),
            had_role: self.had_role,
            had_plan: self.had_plan,
        })
    }
}

/// Builder for [`Usage`].
#[derive(Clone, Debug)]
pub struct UsageBuilder {
    identity: Resource,
    identified: IdentifiedData,
    entity: Option<Resource>,
    had_role: Vec<Iri>,
}

impl UsageBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            entity: None,
            had_role: Vec::new(),
        })
    }

    identified_setters!();

    pub fn entity(mut self, value: Resource) -> Self {
        self.entity = Some(value);
        self
    }

    pub fn had_role(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.had_role = values.into_iter().collect();
        self
    }

    pub fn add_had_role(mut self, value: Iri) -> Self {
        self.had_role.push(value);
        self
    }

    pub fn build(self) -> Result<Usage, BuildError> {
        let entity = self
            .entity
            .ok_or_else(|| missing(&self.identity, SbolClass::ProvUsage, "entity"))?;
        Ok(Usage {
            identity: self.identity,
            identified: self.identified,
            entity: Some(entity),
            had_role: self.had_role,
        })
    }
}

// ---------------------------------------------------------------------------
// OM classes (Appendix A.2)
// ---------------------------------------------------------------------------

fn unit_seed(_namespace: &Namespace, _display_id: &DisplayId) -> UnitData {
    UnitData::default()
}

fn prefix_seed(_namespace: &Namespace, _display_id: &DisplayId) -> PrefixData {
    PrefixData::default()
}

/// Builder for [`Measure`].
#[derive(Clone, Debug)]
pub struct MeasureBuilder {
    identity: Resource,
    identified: IdentifiedData,
    types: Vec<Iri>,
    has_unit: Option<Resource>,
    has_numerical_value: Option<String>,
}

impl MeasureBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            types: Vec::new(),
            has_unit: None,
            has_numerical_value: None,
        })
    }

    identified_setters!();

    pub fn types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.types = values.into_iter().collect();
        self
    }

    pub fn add_type(mut self, value: Iri) -> Self {
        self.types.push(value);
        self
    }

    pub fn has_unit(mut self, value: Resource) -> Self {
        self.has_unit = Some(value);
        self
    }

    pub fn has_numerical_value(mut self, value: f64) -> Self {
        self.has_numerical_value = Some(value.to_string());
        self
    }

    pub fn build(self) -> Result<Measure, BuildError> {
        let has_unit = self
            .has_unit
            .ok_or_else(|| missing(&self.identity, SbolClass::OmMeasure, "hasUnit"))?;
        let has_numerical_value = self
            .has_numerical_value
            .ok_or_else(|| missing(&self.identity, SbolClass::OmMeasure, "hasNumericalValue"))?;
        Ok(Measure {
            identity: self.identity,
            identified: self.identified,
            types: self.types,
            has_unit: Some(has_unit),
            has_numerical_value: Some(has_numerical_value),
        })
    }
}

/// Builder for [`Unit`].
#[derive(Clone, Debug)]
pub struct UnitBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
}

impl UnitBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        let unit = unit_seed(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit,
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn build(self) -> Result<Unit, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnit, "label"))?;
        let symbol = self
            .unit
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnit, "symbol"))?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(Unit {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
        })
    }
}

/// Builder for [`SingularUnit`].
#[derive(Clone, Debug)]
pub struct SingularUnitBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
    has_unit: Option<Resource>,
    has_factor: Option<String>,
}

impl SingularUnitBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit: UnitData::default(),
            has_unit: None,
            has_factor: None,
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn has_unit(mut self, value: Resource) -> Self {
        self.has_unit = Some(value);
        self
    }

    pub fn has_factor(mut self, value: f64) -> Self {
        self.has_factor = Some(value.to_string());
        self
    }

    pub fn build(self) -> Result<SingularUnit, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmSingularUnit, "label"))?;
        let symbol = self
            .unit
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmSingularUnit, "symbol"))?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(SingularUnit {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
            has_unit: self.has_unit,
            has_factor: self.has_factor,
        })
    }
}

/// Builder for [`CompoundUnit`].
#[derive(Clone, Debug)]
pub struct CompoundUnitBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
}

impl CompoundUnitBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit: UnitData::default(),
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn build(self) -> Result<CompoundUnit, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmCompoundUnit, "label"))?;
        let symbol = self
            .unit
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmCompoundUnit, "symbol"))?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(CompoundUnit {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
        })
    }
}

/// Builder for [`UnitDivision`].
#[derive(Clone, Debug)]
pub struct UnitDivisionBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
    has_numerator: Option<Resource>,
    has_denominator: Option<Resource>,
}

impl UnitDivisionBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit: UnitData::default(),
            has_numerator: None,
            has_denominator: None,
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn has_numerator(mut self, value: Resource) -> Self {
        self.has_numerator = Some(value);
        self
    }

    pub fn has_denominator(mut self, value: Resource) -> Self {
        self.has_denominator = Some(value);
        self
    }

    pub fn build(self) -> Result<UnitDivision, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitDivision, "label"))?;
        let symbol = self
            .unit
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitDivision, "symbol"))?;
        let has_numerator = self
            .has_numerator
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitDivision, "hasNumerator"))?;
        let has_denominator = self
            .has_denominator
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitDivision, "hasDenominator"))?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(UnitDivision {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
            has_numerator: Some(has_numerator),
            has_denominator: Some(has_denominator),
        })
    }
}

/// Builder for [`UnitExponentiation`].
#[derive(Clone, Debug)]
pub struct UnitExponentiationBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
    has_base: Option<Resource>,
    has_exponent: Option<i64>,
}

impl UnitExponentiationBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit: UnitData::default(),
            has_base: None,
            has_exponent: None,
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn has_base(mut self, value: Resource) -> Self {
        self.has_base = Some(value);
        self
    }

    pub fn has_exponent(mut self, value: i64) -> Self {
        self.has_exponent = Some(value);
        self
    }

    pub fn build(self) -> Result<UnitExponentiation, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitExponentiation, "label"))?;
        let symbol =
            self.unit.symbol.clone().ok_or_else(|| {
                missing(&self.identity, SbolClass::OmUnitExponentiation, "symbol")
            })?;
        let has_base = self
            .has_base
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitExponentiation, "hasBase"))?;
        let has_exponent = self.has_exponent.ok_or_else(|| {
            missing(
                &self.identity,
                SbolClass::OmUnitExponentiation,
                "hasExponent",
            )
        })?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(UnitExponentiation {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
            has_base: Some(has_base),
            has_exponent: Some(has_exponent),
        })
    }
}

/// Builder for [`UnitMultiplication`].
#[derive(Clone, Debug)]
pub struct UnitMultiplicationBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
    has_term1: Option<Resource>,
    has_term2: Option<Resource>,
}

impl UnitMultiplicationBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit: UnitData::default(),
            has_term1: None,
            has_term2: None,
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn has_term1(mut self, value: Resource) -> Self {
        self.has_term1 = Some(value);
        self
    }

    pub fn has_term2(mut self, value: Resource) -> Self {
        self.has_term2 = Some(value);
        self
    }

    pub fn build(self) -> Result<UnitMultiplication, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitMultiplication, "label"))?;
        let symbol =
            self.unit.symbol.clone().ok_or_else(|| {
                missing(&self.identity, SbolClass::OmUnitMultiplication, "symbol")
            })?;
        let has_term1 = self
            .has_term1
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitMultiplication, "hasTerm1"))?;
        let has_term2 = self
            .has_term2
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitMultiplication, "hasTerm2"))?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(UnitMultiplication {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
            has_term1: Some(has_term1),
            has_term2: Some(has_term2),
        })
    }
}

/// Builder for [`PrefixedUnit`].
#[derive(Clone, Debug)]
pub struct PrefixedUnitBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
    has_unit: Option<Resource>,
    has_prefix: Option<Resource>,
}

impl PrefixedUnitBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit: UnitData::default(),
            has_unit: None,
            has_prefix: None,
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn has_unit(mut self, value: Resource) -> Self {
        self.has_unit = Some(value);
        self
    }

    pub fn has_prefix(mut self, value: Resource) -> Self {
        self.has_prefix = Some(value);
        self
    }

    pub fn build(self) -> Result<PrefixedUnit, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefixedUnit, "label"))?;
        let symbol = self
            .unit
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefixedUnit, "symbol"))?;
        let has_unit = self
            .has_unit
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefixedUnit, "hasUnit"))?;
        let has_prefix = self
            .has_prefix
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefixedUnit, "hasPrefix"))?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(PrefixedUnit {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
            has_unit: Some(has_unit),
            has_prefix: Some(has_prefix),
        })
    }
}

/// Builder for [`Prefix`].
#[derive(Clone, Debug)]
pub struct PrefixBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    prefix: PrefixData,
}

impl PrefixBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            prefix: prefix_seed(&namespace, &display_id),
        }
    }

    identified_setters!();
    top_level_setters!();
    prefix_setters!();

    pub fn build(self) -> Result<Prefix, BuildError> {
        let label = self
            .prefix
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefix, "label"))?;
        let symbol = self
            .prefix
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefix, "symbol"))?;
        let has_factor = self
            .prefix
            .has_factor
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefix, "hasFactor"))?;
        let mut prefix = self.prefix;
        prefix.label = Some(label);
        prefix.symbol = Some(symbol);
        prefix.has_factor = Some(has_factor);
        Ok(Prefix {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            prefix,
        })
    }
}

/// Builder for [`SIPrefix`].
#[derive(Clone, Debug)]
pub struct SIPrefixBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    prefix: PrefixData,
}

impl SIPrefixBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            prefix: PrefixData::default(),
        }
    }

    identified_setters!();
    top_level_setters!();
    prefix_setters!();

    pub fn build(self) -> Result<SIPrefix, BuildError> {
        let label = self
            .prefix
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmSiPrefix, "label"))?;
        let symbol = self
            .prefix
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmSiPrefix, "symbol"))?;
        let has_factor = self
            .prefix
            .has_factor
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmSiPrefix, "hasFactor"))?;
        let mut prefix = self.prefix;
        prefix.label = Some(label);
        prefix.symbol = Some(symbol);
        prefix.has_factor = Some(has_factor);
        Ok(SIPrefix {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            prefix,
        })
    }
}

/// Builder for [`BinaryPrefix`].
#[derive(Clone, Debug)]
pub struct BinaryPrefixBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    prefix: PrefixData,
}

impl BinaryPrefixBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            prefix: PrefixData::default(),
        }
    }

    identified_setters!();
    top_level_setters!();
    prefix_setters!();

    pub fn build(self) -> Result<BinaryPrefix, BuildError> {
        let label = self
            .prefix
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmBinaryPrefix, "label"))?;
        let symbol = self
            .prefix
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmBinaryPrefix, "symbol"))?;
        let has_factor = self
            .prefix
            .has_factor
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmBinaryPrefix, "hasFactor"))?;
        let mut prefix = self.prefix;
        prefix.label = Some(label);
        prefix.symbol = Some(symbol);
        prefix.has_factor = Some(has_factor);
        Ok(BinaryPrefix {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            prefix,
        })
    }
}

// ---------------------------------------------------------------------------
// IdentifiedExtension (catch-all for bare sbol:Identified subjects)
// ---------------------------------------------------------------------------

/// Builder for [`IdentifiedExtension`].
#[derive(Clone, Debug)]
pub struct IdentifiedExtensionBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: Option<TopLevelData>,
    rdf_types: Vec<Iri>,
}

impl IdentifiedExtensionBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            top_level: None,
            rdf_types: Vec::new(),
        })
    }

    identified_setters!();

    pub fn top_level(mut self, top_level: TopLevelData) -> Self {
        self.top_level = Some(top_level);
        self
    }

    pub fn rdf_types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.rdf_types = values.into_iter().collect();
        self
    }

    pub fn add_rdf_type(mut self, value: Iri) -> Self {
        self.rdf_types.push(value);
        self
    }

    pub fn build(self) -> Result<IdentifiedExtension, BuildError> {
        Ok(IdentifiedExtension {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            rdf_types: self.rdf_types,
        })
    }
}
