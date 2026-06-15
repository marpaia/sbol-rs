//! Builders for the PROV-O classes adopted by SBOL 3 (Appendix A.1):
//! `Activity`, `Agent`, `Plan`, `Association`, `Usage`.

use super::{
    child_seed, identified_seed, identified_setters, missing, top_level_seed, top_level_setters,
};
use crate::client::identity::build_top_level_identity;
use crate::client::{
    Activity, Agent, Association, ExtensionTriple, IdentifiedData, Plan, TopLevelData, Usage,
};
use crate::error::BuildError;
use crate::identity::{DisplayId, Namespace};
use crate::{Iri, Resource, SbolClass, Term};

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
