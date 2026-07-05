use crate::client::accessors::impl_sbol_identified;
use crate::client::shared::{iris, resources};
use crate::client::to_rdf::{Emitter, emit_component_instance, emit_identified, seed_triples};
use crate::client::{ComponentInstanceData, IdentifiedData, ToRdf, TryFromObject};
use crate::vocab::*;
use crate::{Iri, Object, Resource, Sbol2Class, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Component {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub component_instance: ComponentInstanceData,
    pub roles: Vec<Iri>,
    pub role_integration: Option<Iri>,
    pub source_locations: Vec<Resource>,
}

impl ToRdf for Component {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::Component);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::Component);
        emit_identified(&mut e, &self.identified)?;
        emit_component_instance(&mut e, &self.component_instance)?;
        e.iris(SBOL2_ROLE, &self.roles)?;
        e.iri(SBOL2_ROLE_INTEGRATION, self.role_integration.as_ref())?;
        e.resources(SBOL2_SOURCE_LOCATION, &self.source_locations)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Component {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            component_instance: ComponentInstanceData::from_object(object),
            roles: iris(object, SBOL2_ROLE),
            role_integration: object.first_iri(SBOL2_ROLE_INTEGRATION).cloned(),
            source_locations: resources(object, SBOL2_SOURCE_LOCATION),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FunctionalComponent {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub component_instance: ComponentInstanceData,
    pub direction: Option<Iri>,
}

impl ToRdf for FunctionalComponent {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::FunctionalComponent);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::FunctionalComponent);
        emit_identified(&mut e, &self.identified)?;
        emit_component_instance(&mut e, &self.component_instance)?;
        e.iri(SBOL2_DIRECTION, self.direction.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for FunctionalComponent {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            component_instance: ComponentInstanceData::from_object(object),
            direction: object.first_iri(SBOL2_DIRECTION).cloned(),
        })
    }
}

impl_sbol_identified!(Component, FunctionalComponent);
