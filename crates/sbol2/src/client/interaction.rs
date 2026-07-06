use crate::client::accessors::impl_sbol_identified;
use crate::client::shared::{iris, resources};
use crate::client::to_rdf::{Emitter, emit_identified, emit_measured, seed_triples};
use crate::client::{IdentifiedData, MeasuredData, ToRdf, TryFromObject};
use crate::vocab::*;
use crate::{Iri, Object, Resource, Sbol2Class, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Interaction {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub measured: MeasuredData,
    pub types: Vec<Iri>,
    pub participations: Vec<Resource>,
}

impl ToRdf for Interaction {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::Interaction);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::Interaction);
        emit_identified(&mut e, &self.identified)?;
        emit_measured(&mut e, &self.measured)?;
        e.iris(SBOL2_TYPE, &self.types)?;
        e.resources(SBOL2_PARTICIPATION, &self.participations)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Interaction {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            measured: MeasuredData::from_object(object),
            types: iris(object, SBOL2_TYPE),
            participations: resources(object, SBOL2_PARTICIPATION),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Participation {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub measured: MeasuredData,
    pub roles: Vec<Iri>,
    pub participant: Option<Resource>,
}

impl ToRdf for Participation {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::Participation);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::Participation);
        emit_identified(&mut e, &self.identified)?;
        emit_measured(&mut e, &self.measured)?;
        e.iris(SBOL2_ROLE, &self.roles)?;
        e.resource(SBOL2_PARTICIPANT, self.participant.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Participation {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            measured: MeasuredData::from_object(object),
            roles: iris(object, SBOL2_ROLE),
            participant: object.first_resource(SBOL2_PARTICIPANT).cloned(),
        })
    }
}

impl_sbol_identified!(Interaction, Participation);
