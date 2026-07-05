//! Builders for the SBOL 2 child classes owned by ComponentDefinition and
//! ModuleDefinition: component instances, modules, annotations, locations,
//! interactions, and combinatorial variable components.

use super::{
    child_seed, component_instance_setters, identified_setters, location_setters,
    measured_setters, measured_seed, missing,
};
use crate::client::{
    Component, ComponentInstanceData, Cut, FunctionalComponent, GenericLocation, IdentifiedData,
    Interaction, LocationData, MapsTo, MeasuredData, Module, Participation, Range,
    SequenceAnnotation, SequenceConstraint, VariableComponent,
};
use crate::error::BuildError;
use crate::identity::DisplayId;
use crate::{Iri, Resource, Sbol2Class, Term};
use sbol_core::error::BuildError as LexError;

/// Builder for [`Component`].
#[derive(Clone, Debug)]
pub struct ComponentBuilder {
    identity: Resource,
    identified: IdentifiedData,
    component_instance: ComponentInstanceData,
    roles: Vec<Iri>,
    role_integration: Option<Iri>,
    source_locations: Vec<Resource>,
}

impl ComponentBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            component_instance: ComponentInstanceData::default(),
            roles: Vec::new(),
            role_integration: None,
            source_locations: Vec::new(),
        })
    }

    identified_setters!();
    component_instance_setters!();

    pub fn roles(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.roles = values.into_iter().collect();
        self
    }
    pub fn add_role(mut self, value: Iri) -> Self {
        self.roles.push(value);
        self
    }
    pub fn role_integration(mut self, value: Iri) -> Self {
        self.role_integration = Some(value);
        self
    }
    pub fn source_locations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.source_locations = values.into_iter().collect();
        self
    }
    pub fn add_source_location(mut self, value: Resource) -> Self {
        self.source_locations.push(value);
        self
    }

    pub fn build(self) -> Result<Component, BuildError> {
        if self.component_instance.definition.is_none() {
            return Err(missing(&self.identity, Sbol2Class::Component, "definition"));
        }
        Ok(Component {
            identity: self.identity,
            identified: self.identified,
            component_instance: self.component_instance,
            roles: self.roles,
            role_integration: self.role_integration,
            source_locations: self.source_locations,
        })
    }
}

impl Component {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        definition: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.definition(definition).build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<ComponentBuilder, BuildError> {
        ComponentBuilder::seed(parent, display_id.try_into()?)
    }
}

/// Builder for [`FunctionalComponent`].
#[derive(Clone, Debug)]
pub struct FunctionalComponentBuilder {
    identity: Resource,
    identified: IdentifiedData,
    component_instance: ComponentInstanceData,
    direction: Option<Iri>,
}

impl FunctionalComponentBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            component_instance: ComponentInstanceData::default(),
            direction: None,
        })
    }

    identified_setters!();
    component_instance_setters!();

    pub fn direction(mut self, value: Iri) -> Self {
        self.direction = Some(value);
        self
    }

    pub fn build(self) -> Result<FunctionalComponent, BuildError> {
        if self.component_instance.definition.is_none() {
            return Err(missing(
                &self.identity,
                Sbol2Class::FunctionalComponent,
                "definition",
            ));
        }
        Ok(FunctionalComponent {
            identity: self.identity,
            identified: self.identified,
            component_instance: self.component_instance,
            direction: self.direction,
        })
    }
}

impl FunctionalComponent {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        definition: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.definition(definition).build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<FunctionalComponentBuilder, BuildError> {
        FunctionalComponentBuilder::seed(parent, display_id.try_into()?)
    }
}

/// Builder for [`Module`].
#[derive(Clone, Debug)]
pub struct ModuleBuilder {
    identity: Resource,
    identified: IdentifiedData,
    measured: MeasuredData,
    definition: Option<Resource>,
    maps_tos: Vec<Resource>,
}

impl ModuleBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            measured: measured_seed(),
            definition: None,
            maps_tos: Vec::new(),
        })
    }

    identified_setters!();
    measured_setters!();

    pub fn definition(mut self, value: Resource) -> Self {
        self.definition = Some(value);
        self
    }
    pub fn maps_tos(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.maps_tos = values.into_iter().collect();
        self
    }
    pub fn add_maps_to(mut self, value: Resource) -> Self {
        self.maps_tos.push(value);
        self
    }

    pub fn build(self) -> Result<Module, BuildError> {
        let definition = self
            .definition
            .ok_or_else(|| missing(&self.identity, Sbol2Class::Module, "definition"))?;
        Ok(Module {
            identity: self.identity,
            identified: self.identified,
            measured: self.measured,
            definition: Some(definition),
            maps_tos: self.maps_tos,
        })
    }
}

impl Module {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        definition: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.definition(definition).build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<ModuleBuilder, BuildError> {
        ModuleBuilder::seed(parent, display_id.try_into()?)
    }
}

/// Builder for [`MapsTo`].
#[derive(Clone, Debug)]
pub struct MapsToBuilder {
    identity: Resource,
    identified: IdentifiedData,
    local: Option<Resource>,
    remote: Option<Resource>,
    refinement: Option<Iri>,
}

impl MapsToBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            local: None,
            remote: None,
            refinement: None,
        })
    }

    identified_setters!();

    pub fn local(mut self, value: Resource) -> Self {
        self.local = Some(value);
        self
    }
    pub fn remote(mut self, value: Resource) -> Self {
        self.remote = Some(value);
        self
    }
    pub fn refinement(mut self, value: Iri) -> Self {
        self.refinement = Some(value);
        self
    }

    pub fn build(self) -> Result<MapsTo, BuildError> {
        let local = self
            .local
            .ok_or_else(|| missing(&self.identity, Sbol2Class::MapsTo, "local"))?;
        let remote = self
            .remote
            .ok_or_else(|| missing(&self.identity, Sbol2Class::MapsTo, "remote"))?;
        let refinement = self
            .refinement
            .ok_or_else(|| missing(&self.identity, Sbol2Class::MapsTo, "refinement"))?;
        Ok(MapsTo {
            identity: self.identity,
            identified: self.identified,
            local: Some(local),
            remote: Some(remote),
            refinement: Some(refinement),
        })
    }
}

impl MapsTo {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        local: Resource,
        remote: Resource,
        refinement: Iri,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?
            .local(local)
            .remote(remote)
            .refinement(refinement)
            .build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<MapsToBuilder, BuildError> {
        MapsToBuilder::seed(parent, display_id.try_into()?)
    }
}

/// Builder for [`SequenceAnnotation`].
#[derive(Clone, Debug)]
pub struct SequenceAnnotationBuilder {
    identity: Resource,
    identified: IdentifiedData,
    locations: Vec<Resource>,
    component: Option<Resource>,
    roles: Vec<Iri>,
}

impl SequenceAnnotationBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            locations: Vec::new(),
            component: None,
            roles: Vec::new(),
        })
    }

    identified_setters!();

    pub fn locations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.locations = values.into_iter().collect();
        self
    }
    pub fn add_location(mut self, value: Resource) -> Self {
        self.locations.push(value);
        self
    }
    pub fn component(mut self, value: Resource) -> Self {
        self.component = Some(value);
        self
    }
    pub fn roles(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.roles = values.into_iter().collect();
        self
    }
    pub fn add_role(mut self, value: Iri) -> Self {
        self.roles.push(value);
        self
    }

    pub fn build(self) -> Result<SequenceAnnotation, BuildError> {
        if self.locations.is_empty() {
            return Err(missing(
                &self.identity,
                Sbol2Class::SequenceAnnotation,
                "location",
            ));
        }
        Ok(SequenceAnnotation {
            identity: self.identity,
            identified: self.identified,
            locations: self.locations,
            component: self.component,
            roles: self.roles,
        })
    }
}

impl SequenceAnnotation {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        locations: impl IntoIterator<Item = Resource>,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.locations(locations).build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<SequenceAnnotationBuilder, BuildError> {
        SequenceAnnotationBuilder::seed(parent, display_id.try_into()?)
    }
}

/// Builder for [`SequenceConstraint`].
#[derive(Clone, Debug)]
pub struct SequenceConstraintBuilder {
    identity: Resource,
    identified: IdentifiedData,
    subject: Option<Resource>,
    constrained_object: Option<Resource>,
    restriction: Option<Iri>,
}

impl SequenceConstraintBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            subject: None,
            constrained_object: None,
            restriction: None,
        })
    }

    identified_setters!();

    pub fn subject(mut self, value: Resource) -> Self {
        self.subject = Some(value);
        self
    }
    pub fn constrained_object(mut self, value: Resource) -> Self {
        self.constrained_object = Some(value);
        self
    }
    pub fn restriction(mut self, value: Iri) -> Self {
        self.restriction = Some(value);
        self
    }

    pub fn build(self) -> Result<SequenceConstraint, BuildError> {
        let subject = self
            .subject
            .ok_or_else(|| missing(&self.identity, Sbol2Class::SequenceConstraint, "subject"))?;
        let constrained_object = self
            .constrained_object
            .ok_or_else(|| missing(&self.identity, Sbol2Class::SequenceConstraint, "object"))?;
        let restriction = self.restriction.ok_or_else(|| {
            missing(&self.identity, Sbol2Class::SequenceConstraint, "restriction")
        })?;
        Ok(SequenceConstraint {
            identity: self.identity,
            identified: self.identified,
            subject: Some(subject),
            constrained_object: Some(constrained_object),
            restriction: Some(restriction),
        })
    }
}

impl SequenceConstraint {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        subject: Resource,
        constrained_object: Resource,
        restriction: Iri,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?
            .subject(subject)
            .constrained_object(constrained_object)
            .restriction(restriction)
            .build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<SequenceConstraintBuilder, BuildError> {
        SequenceConstraintBuilder::seed(parent, display_id.try_into()?)
    }
}

/// Builder for [`Range`].
#[derive(Clone, Debug)]
pub struct RangeBuilder {
    identity: Resource,
    identified: IdentifiedData,
    location: LocationData,
    start: Option<i64>,
    end: Option<i64>,
}

impl RangeBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            location: LocationData::default(),
            start: None,
            end: None,
        })
    }

    identified_setters!();
    location_setters!();

    pub fn start(mut self, value: i64) -> Self {
        self.start = Some(value);
        self
    }
    pub fn end(mut self, value: i64) -> Self {
        self.end = Some(value);
        self
    }

    pub fn build(self) -> Result<Range, BuildError> {
        let start = self
            .start
            .ok_or_else(|| missing(&self.identity, Sbol2Class::Range, "start"))?;
        let end = self
            .end
            .ok_or_else(|| missing(&self.identity, Sbol2Class::Range, "end"))?;
        Ok(Range {
            identity: self.identity,
            identified: self.identified,
            location: self.location,
            start: Some(start),
            end: Some(end),
        })
    }
}

impl Range {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        start: i64,
        end: i64,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.start(start).end(end).build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<RangeBuilder, BuildError> {
        RangeBuilder::seed(parent, display_id.try_into()?)
    }
}

/// Builder for [`Cut`].
#[derive(Clone, Debug)]
pub struct CutBuilder {
    identity: Resource,
    identified: IdentifiedData,
    location: LocationData,
    at: Option<i64>,
}

impl CutBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            location: LocationData::default(),
            at: None,
        })
    }

    identified_setters!();
    location_setters!();

    pub fn at(mut self, value: i64) -> Self {
        self.at = Some(value);
        self
    }

    pub fn build(self) -> Result<Cut, BuildError> {
        let at = self
            .at
            .ok_or_else(|| missing(&self.identity, Sbol2Class::Cut, "at"))?;
        Ok(Cut {
            identity: self.identity,
            identified: self.identified,
            location: self.location,
            at: Some(at),
        })
    }
}

impl Cut {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        at: i64,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.at(at).build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<CutBuilder, BuildError> {
        CutBuilder::seed(parent, display_id.try_into()?)
    }
}

/// Builder for [`GenericLocation`].
#[derive(Clone, Debug)]
pub struct GenericLocationBuilder {
    identity: Resource,
    identified: IdentifiedData,
    location: LocationData,
}

impl GenericLocationBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            location: LocationData::default(),
        })
    }

    identified_setters!();
    location_setters!();

    pub fn build(self) -> Result<GenericLocation, BuildError> {
        Ok(GenericLocation {
            identity: self.identity,
            identified: self.identified,
            location: self.location,
        })
    }
}

impl GenericLocation {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<GenericLocationBuilder, BuildError> {
        GenericLocationBuilder::seed(parent, display_id.try_into()?)
    }
}

/// Builder for [`Interaction`].
#[derive(Clone, Debug)]
pub struct InteractionBuilder {
    identity: Resource,
    identified: IdentifiedData,
    measured: MeasuredData,
    types: Vec<Iri>,
    participations: Vec<Resource>,
}

impl InteractionBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            measured: measured_seed(),
            types: Vec::new(),
            participations: Vec::new(),
        })
    }

    identified_setters!();
    measured_setters!();

    pub fn types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.types = values.into_iter().collect();
        self
    }
    pub fn add_type(mut self, value: Iri) -> Self {
        self.types.push(value);
        self
    }
    pub fn participations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.participations = values.into_iter().collect();
        self
    }
    pub fn add_participation(mut self, value: Resource) -> Self {
        self.participations.push(value);
        self
    }

    pub fn build(self) -> Result<Interaction, BuildError> {
        if self.types.is_empty() {
            return Err(missing(&self.identity, Sbol2Class::Interaction, "type"));
        }
        Ok(Interaction {
            identity: self.identity,
            identified: self.identified,
            measured: self.measured,
            types: self.types,
            participations: self.participations,
        })
    }
}

impl Interaction {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        types: impl IntoIterator<Item = Iri>,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.types(types).build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<InteractionBuilder, BuildError> {
        InteractionBuilder::seed(parent, display_id.try_into()?)
    }
}

/// Builder for [`Participation`].
#[derive(Clone, Debug)]
pub struct ParticipationBuilder {
    identity: Resource,
    identified: IdentifiedData,
    measured: MeasuredData,
    roles: Vec<Iri>,
    participant: Option<Resource>,
}

impl ParticipationBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            measured: measured_seed(),
            roles: Vec::new(),
            participant: None,
        })
    }

    identified_setters!();
    measured_setters!();

    pub fn roles(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.roles = values.into_iter().collect();
        self
    }
    pub fn add_role(mut self, value: Iri) -> Self {
        self.roles.push(value);
        self
    }
    pub fn participant(mut self, value: Resource) -> Self {
        self.participant = Some(value);
        self
    }

    pub fn build(self) -> Result<Participation, BuildError> {
        if self.roles.is_empty() {
            return Err(missing(&self.identity, Sbol2Class::Participation, "role"));
        }
        let participant = self
            .participant
            .ok_or_else(|| missing(&self.identity, Sbol2Class::Participation, "participant"))?;
        Ok(Participation {
            identity: self.identity,
            identified: self.identified,
            measured: self.measured,
            roles: self.roles,
            participant: Some(participant),
        })
    }
}

impl Participation {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        roles: impl IntoIterator<Item = Iri>,
        participant: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?
            .roles(roles)
            .participant(participant)
            .build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<ParticipationBuilder, BuildError> {
        ParticipationBuilder::seed(parent, display_id.try_into()?)
    }
}

/// Builder for [`VariableComponent`].
#[derive(Clone, Debug)]
pub struct VariableComponentBuilder {
    identity: Resource,
    identified: IdentifiedData,
    variable: Option<Resource>,
    variants: Vec<Resource>,
    variant_collections: Vec<Resource>,
    variant_derivations: Vec<Resource>,
    operator: Option<Iri>,
}

impl VariableComponentBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            variable: None,
            variants: Vec::new(),
            variant_collections: Vec::new(),
            variant_derivations: Vec::new(),
            operator: None,
        })
    }

    identified_setters!();

    pub fn variable(mut self, value: Resource) -> Self {
        self.variable = Some(value);
        self
    }
    pub fn variants(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.variants = values.into_iter().collect();
        self
    }
    pub fn add_variant(mut self, value: Resource) -> Self {
        self.variants.push(value);
        self
    }
    pub fn variant_collections(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.variant_collections = values.into_iter().collect();
        self
    }
    pub fn add_variant_collection(mut self, value: Resource) -> Self {
        self.variant_collections.push(value);
        self
    }
    pub fn variant_derivations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.variant_derivations = values.into_iter().collect();
        self
    }
    pub fn add_variant_derivation(mut self, value: Resource) -> Self {
        self.variant_derivations.push(value);
        self
    }
    pub fn operator(mut self, value: Iri) -> Self {
        self.operator = Some(value);
        self
    }

    pub fn build(self) -> Result<VariableComponent, BuildError> {
        let variable = self
            .variable
            .ok_or_else(|| missing(&self.identity, Sbol2Class::VariableComponent, "variable"))?;
        let operator = self
            .operator
            .ok_or_else(|| missing(&self.identity, Sbol2Class::VariableComponent, "operator"))?;
        Ok(VariableComponent {
            identity: self.identity,
            identified: self.identified,
            variable: Some(variable),
            variants: self.variants,
            variant_collections: self.variant_collections,
            variant_derivations: self.variant_derivations,
            operator: Some(operator),
        })
    }
}

impl VariableComponent {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        variable: Resource,
        operator: Iri,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?
            .variable(variable)
            .operator(operator)
            .build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<VariableComponentBuilder, BuildError> {
        VariableComponentBuilder::seed(parent, display_id.try_into()?)
    }
}
