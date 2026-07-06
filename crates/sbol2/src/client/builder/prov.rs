//! Builders for the PROV-O classes adopted by SBOL 2: `Activity`, `Agent`,
//! `Plan`, `Association`, `Usage`.

use super::{
    child_seed, identified_seed, identified_setters, missing, top_level_seed, top_level_setters,
};
use crate::client::identity::{DEFAULT_VERSION, build_top_level_identity};
use crate::client::{Activity, Agent, Association, IdentifiedData, Plan, TopLevelData, Usage};
use crate::error::BuildError;
use crate::identity::{DisplayId, Namespace};
use crate::{Iri, Resource, Sbol2Class, Term};
use sbol_core::error::BuildError as LexError;

fn top_seed(
    namespace: &Namespace,
    display_id: &DisplayId,
) -> (Resource, IdentifiedData, TopLevelData) {
    let (identity, persistent) = build_top_level_identity(namespace, display_id, DEFAULT_VERSION);
    (
        identity,
        identified_seed(display_id, persistent),
        top_level_seed(),
    )
}

/// Builder for [`Activity`].
#[derive(Clone, Debug)]
pub struct ActivityBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    types: Vec<Iri>,
    started_at_time: Option<String>,
    ended_at_time: Option<String>,
    qualified_associations: Vec<Resource>,
    qualified_usages: Vec<Resource>,
    was_informed_by: Vec<Resource>,
}

impl ActivityBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let (identity, identified, top_level) = top_seed(&namespace, &display_id);
        Self {
            identity,
            identified,
            top_level,
            types: Vec::new(),
            started_at_time: None,
            ended_at_time: None,
            qualified_associations: Vec::new(),
            qualified_usages: Vec::new(),
            was_informed_by: Vec::new(),
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
    pub fn qualified_associations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.qualified_associations = values.into_iter().collect();
        self
    }
    pub fn add_qualified_association(mut self, value: Resource) -> Self {
        self.qualified_associations.push(value);
        self
    }
    pub fn qualified_usages(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.qualified_usages = values.into_iter().collect();
        self
    }
    pub fn add_qualified_usage(mut self, value: Resource) -> Self {
        self.qualified_usages.push(value);
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

    pub fn build(self) -> Result<Activity, BuildError> {
        Ok(Activity {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            types: self.types,
            started_at_time: self.started_at_time,
            ended_at_time: self.ended_at_time,
            qualified_associations: self.qualified_associations,
            qualified_usages: self.qualified_usages,
            was_informed_by: self.was_informed_by,
        })
    }
}

impl Activity {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<ActivityBuilder, BuildError> {
        Ok(ActivityBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

macro_rules! bare_top_level {
    ($class:ident, $builder:ident) => {
        /// Builder for the corresponding PROV-O TopLevel class.
        #[derive(Clone, Debug)]
        pub struct $builder {
            identity: Resource,
            identified: IdentifiedData,
            top_level: TopLevelData,
        }

        impl $builder {
            pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
                let (identity, identified, top_level) = top_seed(&namespace, &display_id);
                Self {
                    identity,
                    identified,
                    top_level,
                }
            }

            identified_setters!();
            top_level_setters!();

            pub fn build(self) -> Result<$class, BuildError> {
                Ok($class {
                    identity: self.identity,
                    identified: self.identified,
                    top_level: self.top_level,
                })
            }
        }

        impl $class {
            pub fn new(
                namespace: impl TryInto<Namespace, Error = LexError>,
                display_id: impl TryInto<DisplayId, Error = LexError>,
            ) -> Result<Self, BuildError> {
                Self::builder(namespace, display_id)?.build()
            }

            pub fn builder(
                namespace: impl TryInto<Namespace, Error = LexError>,
                display_id: impl TryInto<DisplayId, Error = LexError>,
            ) -> Result<$builder, BuildError> {
                Ok($builder::seed(
                    namespace.try_into()?,
                    display_id.try_into()?,
                ))
            }
        }
    };
}

bare_top_level!(Agent, AgentBuilder);
bare_top_level!(Plan, PlanBuilder);

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
            .ok_or_else(|| missing(&self.identity, Sbol2Class::ProvAssociation, "agent"))?;
        Ok(Association {
            identity: self.identity,
            identified: self.identified,
            agent: Some(agent),
            had_role: self.had_role,
            had_plan: self.had_plan,
        })
    }
}

impl Association {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        agent: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.agent(agent).build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<AssociationBuilder, BuildError> {
        AssociationBuilder::seed(parent, display_id.try_into()?)
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
            .ok_or_else(|| missing(&self.identity, Sbol2Class::ProvUsage, "entity"))?;
        Ok(Usage {
            identity: self.identity,
            identified: self.identified,
            entity: Some(entity),
            had_role: self.had_role,
        })
    }
}

impl Usage {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        entity: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.entity(entity).build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<UsageBuilder, BuildError> {
        UsageBuilder::seed(parent, display_id.try_into()?)
    }
}
