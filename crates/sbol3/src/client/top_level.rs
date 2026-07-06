use crate::client::accessors::{impl_sbol_identified, impl_sbol_top_level};
use crate::client::builder::{
    AttachmentBuilder, CollectionBuilder, CombinatorialDerivationBuilder, ComponentBuilder,
    ExperimentBuilder, ExperimentalDataBuilder, ImplementationBuilder, ModelBuilder,
    SequenceBuilder,
};
use crate::client::shared::{first_i64, iris, resources};
use crate::client::to_rdf::{Emitter, emit_identified, emit_top_level, seed_triples};
use crate::client::{IdentifiedData, ToRdf, TopLevelData, TryFromObject};
use crate::document::Document;
use crate::error::BuildError;
use crate::identity::{DisplayId, Namespace};
use crate::vocab::*;
use crate::{Iri, Object, Resource, SbolClass, Triple};
use sbol_core::error::BuildError as LexError;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Attachment {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub source: Option<Resource>,
    pub format: Option<Iri>,
    pub size: Option<i64>,
    pub hash: Option<String>,
    pub hash_algorithm: Option<String>,
}

impl Attachment {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        source: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.source(source).build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<AttachmentBuilder, BuildError> {
        Ok(AttachmentBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for Attachment {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::Attachment);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::Attachment);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.resource(SBOL_SOURCE, self.source.as_ref())?;
        e.iri(SBOL_FORMAT, self.format.as_ref())?;
        e.i64(SBOL_SIZE, self.size)?;
        e.literal(SBOL_HASH, self.hash.as_deref())?;
        e.literal(SBOL_HASH_ALGORITHM, self.hash_algorithm.as_deref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Attachment {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            source: object.first_resource(SBOL_SOURCE).cloned(),
            format: object.first_iri(SBOL_FORMAT).cloned(),
            size: first_i64(object, SBOL_SIZE),
            hash: object.first_literal_value(SBOL_HASH).map(ToOwned::to_owned),
            hash_algorithm: object
                .first_literal_value(SBOL_HASH_ALGORITHM)
                .map(ToOwned::to_owned),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Collection {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub members: Vec<Resource>,
}

impl Collection {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<CollectionBuilder, BuildError> {
        Ok(CollectionBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for Collection {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::Collection);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::Collection);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.resources(SBOL_MEMBER, &self.members)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Collection {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            members: resources(object, SBOL_MEMBER),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct CombinatorialDerivation {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub template: Option<Resource>,
    pub strategy: Option<Iri>,
    pub variable_features: Vec<Resource>,
}

impl CombinatorialDerivation {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        template: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .template(template)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<CombinatorialDerivationBuilder, BuildError> {
        Ok(CombinatorialDerivationBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for CombinatorialDerivation {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::CombinatorialDerivation);
        let mut e = Emitter::new(
            &mut triples,
            &self.identity,
            SbolClass::CombinatorialDerivation,
        );
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.resource(SBOL_TEMPLATE, self.template.as_ref())?;
        e.iri(SBOL_STRATEGY, self.strategy.as_ref())?;
        e.resources(SBOL_HAS_VARIABLE_FEATURE, &self.variable_features)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for CombinatorialDerivation {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            template: object.first_resource(SBOL_TEMPLATE).cloned(),
            strategy: object.first_iri(SBOL_STRATEGY).cloned(),
            variable_features: resources(object, SBOL_HAS_VARIABLE_FEATURE),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Component {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub types: Vec<Iri>,
    pub roles: Vec<Iri>,
    pub sequences: Vec<Resource>,
    pub features: Vec<Resource>,
    pub constraints: Vec<Resource>,
    pub interactions: Vec<Resource>,
    pub interfaces: Vec<Resource>,
    pub models: Vec<Resource>,
}

impl Component {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        types: impl IntoIterator<Item = Iri>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.types(types).build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<ComponentBuilder, BuildError> {
        Ok(ComponentBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for Component {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::Component);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::Component);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.iris(SBOL_TYPE, &self.types)?;
        e.iris(SBOL_ROLE, &self.roles)?;
        e.resources(SBOL_HAS_SEQUENCE, &self.sequences)?;
        e.resources(SBOL_HAS_FEATURE, &self.features)?;
        e.resources(SBOL_HAS_CONSTRAINT, &self.constraints)?;
        e.resources(SBOL_HAS_INTERACTION, &self.interactions)?;
        e.resources(SBOL_HAS_INTERFACE, &self.interfaces)?;
        e.resources(SBOL_HAS_MODEL, &self.models)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Component {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            types: iris(object, SBOL_TYPE),
            roles: iris(object, SBOL_ROLE),
            sequences: resources(object, SBOL_HAS_SEQUENCE),
            features: resources(object, SBOL_HAS_FEATURE),
            constraints: resources(object, SBOL_HAS_CONSTRAINT),
            interactions: resources(object, SBOL_HAS_INTERACTION),
            interfaces: resources(object, SBOL_HAS_INTERFACE),
            models: resources(object, SBOL_HAS_MODEL),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Experiment {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub members: Vec<Resource>,
}

impl Experiment {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<ExperimentBuilder, BuildError> {
        Ok(ExperimentBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for Experiment {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::Experiment);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::Experiment);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.resources(SBOL_MEMBER, &self.members)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Experiment {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            members: resources(object, SBOL_MEMBER),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExperimentalData {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
}

impl ExperimentalData {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<ExperimentalDataBuilder, BuildError> {
        Ok(ExperimentalDataBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for ExperimentalData {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::ExperimentalData);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::ExperimentalData);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for ExperimentalData {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Implementation {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub built: Option<Resource>,
}

impl Implementation {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<ImplementationBuilder, BuildError> {
        Ok(ImplementationBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for Implementation {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::Implementation);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::Implementation);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.resource(SBOL_BUILT, self.built.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Implementation {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            built: object.first_resource(SBOL_BUILT).cloned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Model {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub source: Option<Resource>,
    pub language: Option<Iri>,
    pub framework: Option<Iri>,
}

impl Model {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        source: Resource,
        language: Iri,
        framework: Iri,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .source(source)
            .language(language)
            .framework(framework)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<ModelBuilder, BuildError> {
        Ok(ModelBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for Model {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::Model);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::Model);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.resource(SBOL_SOURCE, self.source.as_ref())?;
        e.iri(SBOL_LANGUAGE, self.language.as_ref())?;
        e.iri(SBOL_FRAMEWORK, self.framework.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Model {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            source: object.first_resource(SBOL_SOURCE).cloned(),
            language: object.first_iri(SBOL_LANGUAGE).cloned(),
            framework: object.first_iri(SBOL_FRAMEWORK).cloned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Sequence {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub elements: Option<String>,
    pub encoding: Option<Iri>,
}

impl Sequence {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<SequenceBuilder, BuildError> {
        Ok(SequenceBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for Sequence {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::Sequence);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::Sequence);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.literal(SBOL_ELEMENTS, self.elements.as_deref())?;
        e.iri(SBOL_ENCODING, self.encoding.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Sequence {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            elements: object
                .first_literal_value(SBOL_ELEMENTS)
                .map(ToOwned::to_owned),
            encoding: object.first_iri(SBOL_ENCODING).cloned(),
        })
    }
}

impl_sbol_identified!(
    Attachment,
    Collection,
    CombinatorialDerivation,
    Component,
    Experiment,
    ExperimentalData,
    Implementation,
    Model,
    Sequence,
);
impl_sbol_top_level!(
    Attachment,
    Collection,
    CombinatorialDerivation,
    Component,
    Experiment,
    ExperimentalData,
    Implementation,
    Model,
    Sequence,
);

impl Component {
    /// Returns every `Collection` in the document that lists this component
    /// among its `sbol:member` set. A component can belong to many
    /// collections, so the result is a `Vec`.
    ///
    /// Linear in the total number of collection memberships in the
    /// document.
    pub fn parent_collections<'a>(&self, doc: &'a Document) -> Vec<&'a Collection> {
        doc.collections()
            .filter(|collection| collection.members.iter().any(|m| m == &self.identity))
            .collect()
    }
}
