use crate::client::accessors::impl_sbol_identified;
use crate::client::shared::resources;
use crate::client::to_rdf::{Emitter, emit_identified, emit_measured, seed_triples};
use crate::client::{IdentifiedData, MeasuredData, ToRdf, TryFromObject};
use crate::vocab::*;
use crate::{Iri, Object, Resource, Sbol2Class, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Module {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub measured: MeasuredData,
    pub definition: Option<Resource>,
    pub maps_tos: Vec<Resource>,
}

impl ToRdf for Module {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::Module);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::Module);
        emit_identified(&mut e, &self.identified)?;
        emit_measured(&mut e, &self.measured)?;
        e.resource(SBOL2_DEFINITION, self.definition.as_ref())?;
        e.resources(SBOL2_MAPS_TO, &self.maps_tos)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Module {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            measured: MeasuredData::from_object(object),
            definition: object.first_resource(SBOL2_DEFINITION).cloned(),
            maps_tos: resources(object, SBOL2_MAPS_TO),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MapsTo {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub local: Option<Resource>,
    pub remote: Option<Resource>,
    pub refinement: Option<Iri>,
}

impl ToRdf for MapsTo {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::MapsTo);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::MapsTo);
        emit_identified(&mut e, &self.identified)?;
        e.resource(SBOL2_LOCAL, self.local.as_ref())?;
        e.resource(SBOL2_REMOTE, self.remote.as_ref())?;
        e.iri(SBOL2_REFINEMENT, self.refinement.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for MapsTo {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            local: object.first_resource(SBOL2_LOCAL).cloned(),
            remote: object.first_resource(SBOL2_REMOTE).cloned(),
            refinement: object.first_iri(SBOL2_REFINEMENT).cloned(),
        })
    }
}

impl_sbol_identified!(Module, MapsTo);
