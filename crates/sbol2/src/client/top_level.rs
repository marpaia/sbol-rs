use crate::client::accessors::{impl_sbol_identified, impl_sbol_top_level};
use crate::client::shared::{first_i64, iris, resources};
use crate::client::to_rdf::{Emitter, emit_identified, emit_top_level, seed_triples};
use crate::client::{IdentifiedData, ToRdf, TopLevelData, TryFromObject};
use crate::vocab::*;
use crate::{Iri, Object, Resource, Sbol2Class, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Sequence {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub elements: Option<String>,
    pub encoding: Option<Iri>,
}

impl ToRdf for Sequence {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::Sequence);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::Sequence);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.literal(SBOL2_ELEMENTS, self.elements.as_deref())?;
        e.iri(SBOL2_ENCODING, self.encoding.as_ref())?;
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
                .first_literal_value(SBOL2_ELEMENTS)
                .map(ToOwned::to_owned),
            encoding: object.first_iri(SBOL2_ENCODING).cloned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ComponentDefinition {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub types: Vec<Iri>,
    pub roles: Vec<Iri>,
    pub sequences: Vec<Resource>,
    pub components: Vec<Resource>,
    pub sequence_annotations: Vec<Resource>,
    pub sequence_constraints: Vec<Resource>,
}

impl ToRdf for ComponentDefinition {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::ComponentDefinition);
        let mut e = Emitter::new(
            &mut triples,
            &self.identity,
            Sbol2Class::ComponentDefinition,
        );
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.iris(SBOL2_TYPE, &self.types)?;
        e.iris(SBOL2_ROLE, &self.roles)?;
        e.resources(SBOL2_SEQUENCE, &self.sequences)?;
        e.resources(SBOL2_COMPONENT, &self.components)?;
        e.resources(SBOL2_SEQUENCE_ANNOTATION, &self.sequence_annotations)?;
        e.resources(SBOL2_SEQUENCE_CONSTRAINT, &self.sequence_constraints)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for ComponentDefinition {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            types: iris(object, SBOL2_TYPE),
            roles: iris(object, SBOL2_ROLE),
            sequences: resources(object, SBOL2_SEQUENCE),
            components: resources(object, SBOL2_COMPONENT),
            sequence_annotations: resources(object, SBOL2_SEQUENCE_ANNOTATION),
            sequence_constraints: resources(object, SBOL2_SEQUENCE_CONSTRAINT),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ModuleDefinition {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub roles: Vec<Iri>,
    pub modules: Vec<Resource>,
    pub functional_components: Vec<Resource>,
    pub interactions: Vec<Resource>,
    pub models: Vec<Resource>,
}

impl ToRdf for ModuleDefinition {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::ModuleDefinition);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::ModuleDefinition);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.iris(SBOL2_ROLE, &self.roles)?;
        e.resources(SBOL2_MODULE, &self.modules)?;
        e.resources(SBOL2_FUNCTIONAL_COMPONENT, &self.functional_components)?;
        e.resources(SBOL2_INTERACTION, &self.interactions)?;
        e.resources(SBOL2_MODEL, &self.models)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for ModuleDefinition {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            roles: iris(object, SBOL2_ROLE),
            modules: resources(object, SBOL2_MODULE),
            functional_components: resources(object, SBOL2_FUNCTIONAL_COMPONENT),
            interactions: resources(object, SBOL2_INTERACTION),
            models: resources(object, SBOL2_MODEL),
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

impl ToRdf for Model {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::Model);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::Model);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.resource(SBOL2_SOURCE, self.source.as_ref())?;
        e.iri(SBOL2_LANGUAGE, self.language.as_ref())?;
        e.iri(SBOL2_FRAMEWORK, self.framework.as_ref())?;
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
            source: object.first_resource(SBOL2_SOURCE).cloned(),
            language: object.first_iri(SBOL2_LANGUAGE).cloned(),
            framework: object.first_iri(SBOL2_FRAMEWORK).cloned(),
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

impl ToRdf for Collection {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::Collection);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::Collection);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.resources(SBOL2_MEMBER, &self.members)?;
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
            members: resources(object, SBOL2_MEMBER),
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
    pub variable_components: Vec<Resource>,
    pub strategy: Option<Iri>,
}

impl ToRdf for CombinatorialDerivation {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::CombinatorialDerivation);
        let mut e = Emitter::new(
            &mut triples,
            &self.identity,
            Sbol2Class::CombinatorialDerivation,
        );
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.resource(SBOL2_TEMPLATE, self.template.as_ref())?;
        e.resources(SBOL2_VARIABLE_COMPONENT, &self.variable_components)?;
        e.iri(SBOL2_STRATEGY, self.strategy.as_ref())?;
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
            template: object.first_resource(SBOL2_TEMPLATE).cloned(),
            variable_components: resources(object, SBOL2_VARIABLE_COMPONENT),
            strategy: object.first_iri(SBOL2_STRATEGY).cloned(),
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

impl ToRdf for Implementation {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::Implementation);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::Implementation);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.resource(SBOL2_BUILT, self.built.as_ref())?;
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
            built: object.first_resource(SBOL2_BUILT).cloned(),
        })
    }
}

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
}

impl ToRdf for Attachment {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::Attachment);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::Attachment);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.resource(SBOL2_SOURCE, self.source.as_ref())?;
        e.iri(SBOL2_FORMAT, self.format.as_ref())?;
        e.i64(SBOL2_SIZE, self.size)?;
        e.literal(SBOL2_HASH, self.hash.as_deref())?;
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
            source: object.first_resource(SBOL2_SOURCE).cloned(),
            format: object.first_iri(SBOL2_FORMAT).cloned(),
            size: first_i64(object, SBOL2_SIZE),
            hash: object
                .first_literal_value(SBOL2_HASH)
                .map(ToOwned::to_owned),
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

impl ToRdf for ExperimentalData {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::ExperimentalData);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::ExperimentalData);
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
pub struct Experiment {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub experimental_data: Vec<Resource>,
}

impl ToRdf for Experiment {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::Experiment);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::Experiment);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.resources(SBOL2_EXPERIMENTAL_DATA, &self.experimental_data)?;
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
            experimental_data: resources(object, SBOL2_EXPERIMENTAL_DATA),
        })
    }
}

impl_sbol_identified!(
    Sequence,
    ComponentDefinition,
    ModuleDefinition,
    Model,
    Collection,
    CombinatorialDerivation,
    Implementation,
    Attachment,
    ExperimentalData,
    Experiment,
);
impl_sbol_top_level!(
    Sequence,
    ComponentDefinition,
    ModuleDefinition,
    Model,
    Collection,
    CombinatorialDerivation,
    Implementation,
    Attachment,
    ExperimentalData,
    Experiment,
);
