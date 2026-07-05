//! Builders for `Component` children: features (`SubComponent`,
//! `LocalSubComponent`, `SequenceFeature`, `ComponentReference`,
//! `ExternallyDefined`), locations (`Range`, `Cut`, `EntireSequence`),
//! and the constraint/interaction family (`Constraint`, `Interaction`,
//! `Interface`, `Participation`, `VariableFeature`).

use super::{child_seed, feature_setters, identified_setters, location_setters, missing};
use crate::client::{
    ComponentReference, Constraint, Cut, EntireSequence, ExtensionTriple, ExternallyDefined,
    FeatureData, IdentifiedData, Interaction, Interface, LocalSubComponent, LocationData,
    Participation, Range, SequenceFeature, SubComponent, VariableFeature,
};
use crate::error::BuildError;
use crate::identity::DisplayId;
use crate::{Iri, Resource, SbolClass, Term};

/// Builder for [`SubComponent`].
#[derive(Clone, Debug)]
pub struct SubComponentBuilder {
    identity: Resource,
    identified: IdentifiedData,
    feature: FeatureData,
    instance_of: Option<Resource>,
    role_integration: Option<Iri>,
    locations: Vec<Resource>,
    source_locations: Vec<Resource>,
}

impl SubComponentBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            feature: FeatureData::default(),
            instance_of: None,
            role_integration: None,
            locations: Vec::new(),
            source_locations: Vec::new(),
        })
    }

    identified_setters!();
    feature_setters!();

    pub fn instance_of(mut self, value: Resource) -> Self {
        self.instance_of = Some(value);
        self
    }

    pub fn role_integration(mut self, value: Iri) -> Self {
        self.role_integration = Some(value);
        self
    }

    pub fn locations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.locations = values.into_iter().collect();
        self
    }

    pub fn add_location(mut self, value: Resource) -> Self {
        self.locations.push(value);
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

    pub fn build(self) -> Result<SubComponent, BuildError> {
        let instance_of = self
            .instance_of
            .ok_or_else(|| missing(&self.identity, SbolClass::SubComponent, "instanceOf"))?;
        Ok(SubComponent {
            identity: self.identity,
            identified: self.identified,
            feature: self.feature,
            instance_of: Some(instance_of),
            role_integration: self.role_integration,
            locations: self.locations,
            source_locations: self.source_locations,
        })
    }
}

/// Builder for [`LocalSubComponent`].
#[derive(Clone, Debug)]
pub struct LocalSubComponentBuilder {
    identity: Resource,
    identified: IdentifiedData,
    feature: FeatureData,
    types: Vec<Iri>,
    locations: Vec<Resource>,
}

impl LocalSubComponentBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            feature: FeatureData::default(),
            types: Vec::new(),
            locations: Vec::new(),
        })
    }

    identified_setters!();
    feature_setters!();

    pub fn types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.types = values.into_iter().collect();
        self
    }

    pub fn add_type(mut self, value: Iri) -> Self {
        self.types.push(value);
        self
    }

    pub fn locations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.locations = values.into_iter().collect();
        self
    }

    pub fn add_location(mut self, value: Resource) -> Self {
        self.locations.push(value);
        self
    }

    pub fn build(self) -> Result<LocalSubComponent, BuildError> {
        if self.types.is_empty() {
            return Err(missing(
                &self.identity,
                SbolClass::LocalSubComponent,
                "type",
            ));
        }
        Ok(LocalSubComponent {
            identity: self.identity,
            identified: self.identified,
            feature: self.feature,
            types: self.types,
            locations: self.locations,
        })
    }
}

/// Builder for [`SequenceFeature`].
#[derive(Clone, Debug)]
pub struct SequenceFeatureBuilder {
    identity: Resource,
    identified: IdentifiedData,
    feature: FeatureData,
    locations: Vec<Resource>,
}

impl SequenceFeatureBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            feature: FeatureData::default(),
            locations: Vec::new(),
        })
    }

    identified_setters!();
    feature_setters!();

    pub fn locations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.locations = values.into_iter().collect();
        self
    }

    pub fn add_location(mut self, value: Resource) -> Self {
        self.locations.push(value);
        self
    }

    pub fn build(self) -> Result<SequenceFeature, BuildError> {
        if self.locations.is_empty() {
            return Err(missing(
                &self.identity,
                SbolClass::SequenceFeature,
                "hasLocation",
            ));
        }
        Ok(SequenceFeature {
            identity: self.identity,
            identified: self.identified,
            feature: self.feature,
            locations: self.locations,
        })
    }
}

/// Builder for [`ComponentReference`].
#[derive(Clone, Debug)]
pub struct ComponentReferenceBuilder {
    identity: Resource,
    identified: IdentifiedData,
    feature: FeatureData,
    in_child_of: Option<Resource>,
    refers_to: Option<Resource>,
}

impl ComponentReferenceBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            feature: FeatureData::default(),
            in_child_of: None,
            refers_to: None,
        })
    }

    identified_setters!();
    feature_setters!();

    pub fn in_child_of(mut self, value: Resource) -> Self {
        self.in_child_of = Some(value);
        self
    }

    pub fn refers_to(mut self, value: Resource) -> Self {
        self.refers_to = Some(value);
        self
    }

    pub fn build(self) -> Result<ComponentReference, BuildError> {
        let in_child_of = self
            .in_child_of
            .ok_or_else(|| missing(&self.identity, SbolClass::ComponentReference, "inChildOf"))?;
        let refers_to = self
            .refers_to
            .ok_or_else(|| missing(&self.identity, SbolClass::ComponentReference, "refersTo"))?;
        Ok(ComponentReference {
            identity: self.identity,
            identified: self.identified,
            feature: self.feature,
            in_child_of: Some(in_child_of),
            refers_to: Some(refers_to),
        })
    }
}

/// Builder for [`ExternallyDefined`].
#[derive(Clone, Debug)]
pub struct ExternallyDefinedBuilder {
    identity: Resource,
    identified: IdentifiedData,
    feature: FeatureData,
    definition: Option<Resource>,
    types: Vec<Iri>,
}

impl ExternallyDefinedBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            feature: FeatureData::default(),
            definition: None,
            types: Vec::new(),
        })
    }

    identified_setters!();
    feature_setters!();

    pub fn definition(mut self, value: Resource) -> Self {
        self.definition = Some(value);
        self
    }

    pub fn types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.types = values.into_iter().collect();
        self
    }

    pub fn add_type(mut self, value: Iri) -> Self {
        self.types.push(value);
        self
    }

    pub fn build(self) -> Result<ExternallyDefined, BuildError> {
        let definition = self
            .definition
            .ok_or_else(|| missing(&self.identity, SbolClass::ExternallyDefined, "definition"))?;
        if self.types.is_empty() {
            return Err(missing(
                &self.identity,
                SbolClass::ExternallyDefined,
                "type",
            ));
        }
        Ok(ExternallyDefined {
            identity: self.identity,
            identified: self.identified,
            feature: self.feature,
            definition: Some(definition),
            types: self.types,
        })
    }
}

// ---------------------------------------------------------------------------
// Location classes
// ---------------------------------------------------------------------------

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
            .ok_or_else(|| missing(&self.identity, SbolClass::Range, "start"))?;
        let end = self
            .end
            .ok_or_else(|| missing(&self.identity, SbolClass::Range, "end"))?;
        Ok(Range {
            identity: self.identity,
            identified: self.identified,
            location: self.location,
            start: Some(start),
            end: Some(end),
        })
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
            .ok_or_else(|| missing(&self.identity, SbolClass::Cut, "at"))?;
        Ok(Cut {
            identity: self.identity,
            identified: self.identified,
            location: self.location,
            at: Some(at),
        })
    }
}

/// Builder for [`EntireSequence`].
#[derive(Clone, Debug)]
pub struct EntireSequenceBuilder {
    identity: Resource,
    identified: IdentifiedData,
    location: LocationData,
}

impl EntireSequenceBuilder {
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

    pub fn build(self) -> Result<EntireSequence, BuildError> {
        Ok(EntireSequence {
            identity: self.identity,
            identified: self.identified,
            location: self.location,
        })
    }
}

// ---------------------------------------------------------------------------
// Usage classes (child of Component or Interaction)
// ---------------------------------------------------------------------------

/// Builder for [`Constraint`].
#[derive(Clone, Debug)]
pub struct ConstraintBuilder {
    identity: Resource,
    identified: IdentifiedData,
    subject: Option<Resource>,
    constrained_object: Option<Resource>,
    restriction: Option<Iri>,
}

impl ConstraintBuilder {
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

    pub fn build(self) -> Result<Constraint, BuildError> {
        let subject = self
            .subject
            .ok_or_else(|| missing(&self.identity, SbolClass::Constraint, "subject"))?;
        let constrained_object = self
            .constrained_object
            .ok_or_else(|| missing(&self.identity, SbolClass::Constraint, "object"))?;
        let restriction = self
            .restriction
            .ok_or_else(|| missing(&self.identity, SbolClass::Constraint, "restriction"))?;
        Ok(Constraint {
            identity: self.identity,
            identified: self.identified,
            subject: Some(subject),
            constrained_object: Some(constrained_object),
            restriction: Some(restriction),
        })
    }
}

/// Builder for [`Interaction`].
#[derive(Clone, Debug)]
pub struct InteractionBuilder {
    identity: Resource,
    identified: IdentifiedData,
    types: Vec<Iri>,
    participations: Vec<Resource>,
}

impl InteractionBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            types: Vec::new(),
            participations: Vec::new(),
        })
    }

    identified_setters!();

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
            return Err(missing(&self.identity, SbolClass::Interaction, "type"));
        }
        Ok(Interaction {
            identity: self.identity,
            identified: self.identified,
            types: self.types,
            participations: self.participations,
        })
    }
}

/// Builder for [`Interface`].
#[derive(Clone, Debug)]
pub struct InterfaceBuilder {
    identity: Resource,
    identified: IdentifiedData,
    inputs: Vec<Resource>,
    outputs: Vec<Resource>,
    nondirectional: Vec<Resource>,
}

impl InterfaceBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            inputs: Vec::new(),
            outputs: Vec::new(),
            nondirectional: Vec::new(),
        })
    }

    identified_setters!();

    pub fn inputs(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.inputs = values.into_iter().collect();
        self
    }

    pub fn add_input(mut self, value: Resource) -> Self {
        self.inputs.push(value);
        self
    }

    pub fn outputs(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.outputs = values.into_iter().collect();
        self
    }

    pub fn add_output(mut self, value: Resource) -> Self {
        self.outputs.push(value);
        self
    }

    pub fn nondirectional(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.nondirectional = values.into_iter().collect();
        self
    }

    pub fn add_nondirectional(mut self, value: Resource) -> Self {
        self.nondirectional.push(value);
        self
    }

    pub fn build(self) -> Result<Interface, BuildError> {
        Ok(Interface {
            identity: self.identity,
            identified: self.identified,
            inputs: self.inputs,
            outputs: self.outputs,
            nondirectional: self.nondirectional,
        })
    }
}

/// Builder for [`Participation`].
#[derive(Clone, Debug)]
pub struct ParticipationBuilder {
    identity: Resource,
    identified: IdentifiedData,
    roles: Vec<Iri>,
    participant: Option<Resource>,
    higher_order_participant: Option<Resource>,
}

impl ParticipationBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            roles: Vec::new(),
            participant: None,
            higher_order_participant: None,
        })
    }

    identified_setters!();

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

    pub fn higher_order_participant(mut self, value: Resource) -> Self {
        self.higher_order_participant = Some(value);
        self
    }

    pub fn build(self) -> Result<Participation, BuildError> {
        if self.roles.is_empty() {
            return Err(missing(&self.identity, SbolClass::Participation, "role"));
        }
        Ok(Participation {
            identity: self.identity,
            identified: self.identified,
            roles: self.roles,
            participant: self.participant,
            higher_order_participant: self.higher_order_participant,
        })
    }
}

/// Builder for [`VariableFeature`].
#[derive(Clone, Debug)]
pub struct VariableFeatureBuilder {
    identity: Resource,
    identified: IdentifiedData,
    cardinality: Option<Iri>,
    variable: Option<Resource>,
    variants: Vec<Resource>,
    variant_collections: Vec<Resource>,
    variant_derivations: Vec<Resource>,
    variant_measures: Vec<Resource>,
}

impl VariableFeatureBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            cardinality: None,
            variable: None,
            variants: Vec::new(),
            variant_collections: Vec::new(),
            variant_derivations: Vec::new(),
            variant_measures: Vec::new(),
        })
    }

    identified_setters!();

    pub fn cardinality(mut self, value: Iri) -> Self {
        self.cardinality = Some(value);
        self
    }

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

    pub fn variant_measures(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
        self.variant_measures = values.into_iter().collect();
        self
    }

    pub fn add_variant_measure(mut self, value: Resource) -> Self {
        self.variant_measures.push(value);
        self
    }

    pub fn build(self) -> Result<VariableFeature, BuildError> {
        let cardinality = self
            .cardinality
            .ok_or_else(|| missing(&self.identity, SbolClass::VariableFeature, "cardinality"))?;
        let variable = self
            .variable
            .ok_or_else(|| missing(&self.identity, SbolClass::VariableFeature, "variable"))?;
        Ok(VariableFeature {
            identity: self.identity,
            identified: self.identified,
            cardinality: Some(cardinality),
            variable: Some(variable),
            variants: self.variants,
            variant_collections: self.variant_collections,
            variant_derivations: self.variant_derivations,
            variant_measures: self.variant_measures,
        })
    }
}
