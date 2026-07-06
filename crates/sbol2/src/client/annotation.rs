use crate::client::accessors::impl_sbol_identified;
use crate::client::shared::{iris, resources};
use crate::client::to_rdf::{Emitter, emit_identified, seed_triples};
use crate::client::{IdentifiedData, ToRdf, TryFromObject};
use crate::vocab::*;
use crate::{Iri, Object, Resource, Sbol2Class, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SequenceAnnotation {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub locations: Vec<Resource>,
    pub component: Option<Resource>,
    pub roles: Vec<Iri>,
}

impl ToRdf for SequenceAnnotation {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::SequenceAnnotation);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::SequenceAnnotation);
        emit_identified(&mut e, &self.identified)?;
        e.resources(SBOL2_LOCATION, &self.locations)?;
        e.resource(SBOL2_COMPONENT, self.component.as_ref())?;
        e.iris(SBOL2_ROLE, &self.roles)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for SequenceAnnotation {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            locations: resources(object, SBOL2_LOCATION),
            component: object.first_resource(SBOL2_COMPONENT).cloned(),
            roles: iris(object, SBOL2_ROLE),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SequenceConstraint {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub subject: Option<Resource>,
    pub constrained_object: Option<Resource>,
    pub restriction: Option<Iri>,
}

impl ToRdf for SequenceConstraint {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::SequenceConstraint);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::SequenceConstraint);
        emit_identified(&mut e, &self.identified)?;
        e.resource(SBOL2_SUBJECT, self.subject.as_ref())?;
        e.resource(SBOL2_OBJECT, self.constrained_object.as_ref())?;
        e.iri(SBOL2_RESTRICTION, self.restriction.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for SequenceConstraint {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            subject: object.first_resource(SBOL2_SUBJECT).cloned(),
            constrained_object: object.first_resource(SBOL2_OBJECT).cloned(),
            restriction: object.first_iri(SBOL2_RESTRICTION).cloned(),
        })
    }
}

impl_sbol_identified!(SequenceAnnotation, SequenceConstraint);
