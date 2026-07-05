//! Owned typed structs for the PROV-O classes adopted by SBOL 2 (§13.1).
//!
//! `Activity`, `Agent`, and `Plan` are TopLevel siblings of the SBOL TopLevel
//! hierarchy. `Association` and `Usage` are bare Identified children of an
//! Activity. All five share the SBOL 2 field-metadata catalog, so the
//! descriptor-driven serializer round-trips them through the same `Emitter`.

use crate::client::accessors::{impl_sbol_identified, impl_sbol_top_level};
use crate::client::shared::{iris, resources};
use crate::client::to_rdf::{Emitter, emit_identified, emit_top_level, seed_triples};
use crate::client::{IdentifiedData, ToRdf, TopLevelData, TryFromObject};
use crate::vocab::*;
use crate::{Iri, Object, Resource, Sbol2Class, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Activity {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub types: Vec<Iri>,
    pub started_at_time: Option<String>,
    pub ended_at_time: Option<String>,
    pub qualified_associations: Vec<Resource>,
    pub qualified_usages: Vec<Resource>,
    pub was_informed_by: Vec<Resource>,
}

impl ToRdf for Activity {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::ProvActivity);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::ProvActivity);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.iris(SBOL2_TYPE, &self.types)?;
        e.literal(PROV_STARTED_AT_TIME, self.started_at_time.as_deref())?;
        e.literal(PROV_ENDED_AT_TIME, self.ended_at_time.as_deref())?;
        e.resources(PROV_QUALIFIED_ASSOCIATION, &self.qualified_associations)?;
        e.resources(PROV_QUALIFIED_USAGE, &self.qualified_usages)?;
        e.resources(PROV_WAS_INFORMED_BY, &self.was_informed_by)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Activity {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            types: iris(object, SBOL2_TYPE),
            started_at_time: object
                .first_literal_value(PROV_STARTED_AT_TIME)
                .map(ToOwned::to_owned),
            ended_at_time: object
                .first_literal_value(PROV_ENDED_AT_TIME)
                .map(ToOwned::to_owned),
            qualified_associations: resources(object, PROV_QUALIFIED_ASSOCIATION),
            qualified_usages: resources(object, PROV_QUALIFIED_USAGE),
            was_informed_by: resources(object, PROV_WAS_INFORMED_BY),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Agent {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
}

impl ToRdf for Agent {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::ProvAgent);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::ProvAgent);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Agent {
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
pub struct Plan {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
}

impl ToRdf for Plan {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::ProvPlan);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::ProvPlan);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Plan {
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
pub struct Association {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub agent: Option<Resource>,
    pub had_role: Vec<Iri>,
    pub had_plan: Option<Resource>,
}

impl ToRdf for Association {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::ProvAssociation);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::ProvAssociation);
        emit_identified(&mut e, &self.identified)?;
        e.resource(PROV_AGENT, self.agent.as_ref())?;
        e.iris(PROV_HAD_ROLE, &self.had_role)?;
        e.resource(PROV_HAD_PLAN, self.had_plan.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Association {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            agent: object.first_resource(PROV_AGENT).cloned(),
            had_role: iris(object, PROV_HAD_ROLE),
            had_plan: object.first_resource(PROV_HAD_PLAN).cloned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Usage {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub entity: Option<Resource>,
    pub had_role: Vec<Iri>,
}

impl ToRdf for Usage {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::ProvUsage);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::ProvUsage);
        emit_identified(&mut e, &self.identified)?;
        e.resource(PROV_ENTITY, self.entity.as_ref())?;
        e.iris(PROV_HAD_ROLE, &self.had_role)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Usage {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            entity: object.first_resource(PROV_ENTITY).cloned(),
            had_role: iris(object, PROV_HAD_ROLE),
        })
    }
}

impl_sbol_identified!(Activity, Agent, Plan, Association, Usage);
impl_sbol_top_level!(Activity, Agent, Plan);
