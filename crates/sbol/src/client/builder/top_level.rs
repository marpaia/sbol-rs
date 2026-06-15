//! Builders for the SBOL 3 TopLevel classes (`Component`, `Sequence`,
//! `Attachment`, `Collection`, `CombinatorialDerivation`, `Experiment`,
//! `ExperimentalData`, `Implementation`, `Model`).

use super::{identified_seed, identified_setters, missing, top_level_seed, top_level_setters};
use crate::client::identity::build_top_level_identity;
use crate::client::{
    Attachment, Collection, CombinatorialDerivation, Component, Experiment, ExperimentalData,
    ExtensionTriple, IdentifiedData, Implementation, Model, Sequence, TopLevelData,
};
use crate::error::BuildError;
use crate::identity::{DisplayId, HashAlgorithm, Namespace};
use crate::{Iri, Resource, SbolClass, Term};

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
