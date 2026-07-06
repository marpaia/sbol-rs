use crate::client::shared::IdentifiedData;
use crate::client::{
    Activity, Agent, Association, Attachment, BinaryPrefix, Collection, CombinatorialDerivation,
    Component, ComponentDefinition, CompoundUnit, Cut, Experiment, ExperimentalData,
    FunctionalComponent, GenericLocation, GenericTopLevel, IdentifiedExtension, Implementation,
    Interaction, MapsTo, Measure, Model, Module, ModuleDefinition, Participation, Plan, Prefix,
    PrefixedUnit, Range, SIPrefix, Sequence, SequenceAnnotation, SequenceConstraint, SingularUnit,
    Unit, UnitDivision, UnitExponentiation, UnitMultiplication, Usage, VariableComponent,
};
use crate::{Iri, Resource, Sbol2Class};

/// Owned SBOL 2 object variants supported by the client API.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Sbol2Object {
    Sequence(Sequence),
    ComponentDefinition(ComponentDefinition),
    ModuleDefinition(ModuleDefinition),
    Model(Model),
    Collection(Collection),
    CombinatorialDerivation(CombinatorialDerivation),
    Implementation(Implementation),
    Attachment(Attachment),
    ExperimentalData(ExperimentalData),
    Experiment(Experiment),
    GenericTopLevel(GenericTopLevel),
    Component(Component),
    FunctionalComponent(FunctionalComponent),
    Module(Module),
    MapsTo(MapsTo),
    SequenceAnnotation(SequenceAnnotation),
    SequenceConstraint(SequenceConstraint),
    VariableComponent(VariableComponent),
    Interaction(Interaction),
    Participation(Participation),
    Range(Range),
    Cut(Cut),
    GenericLocation(GenericLocation),
    Activity(Activity),
    Agent(Agent),
    Plan(Plan),
    Association(Association),
    Usage(Usage),
    Measure(Measure),
    Unit(Unit),
    SingularUnit(SingularUnit),
    CompoundUnit(CompoundUnit),
    UnitMultiplication(UnitMultiplication),
    UnitDivision(UnitDivision),
    UnitExponentiation(UnitExponentiation),
    PrefixedUnit(PrefixedUnit),
    Prefix(Prefix),
    SIPrefix(SIPrefix),
    BinaryPrefix(BinaryPrefix),
    IdentifiedExtension(IdentifiedExtension),
}

macro_rules! for_each_variant {
    ($self:expr, $object:ident => $body:expr) => {
        match $self {
            Self::Sequence($object) => $body,
            Self::ComponentDefinition($object) => $body,
            Self::ModuleDefinition($object) => $body,
            Self::Model($object) => $body,
            Self::Collection($object) => $body,
            Self::CombinatorialDerivation($object) => $body,
            Self::Implementation($object) => $body,
            Self::Attachment($object) => $body,
            Self::ExperimentalData($object) => $body,
            Self::Experiment($object) => $body,
            Self::GenericTopLevel($object) => $body,
            Self::Component($object) => $body,
            Self::FunctionalComponent($object) => $body,
            Self::Module($object) => $body,
            Self::MapsTo($object) => $body,
            Self::SequenceAnnotation($object) => $body,
            Self::SequenceConstraint($object) => $body,
            Self::VariableComponent($object) => $body,
            Self::Interaction($object) => $body,
            Self::Participation($object) => $body,
            Self::Range($object) => $body,
            Self::Cut($object) => $body,
            Self::GenericLocation($object) => $body,
            Self::Activity($object) => $body,
            Self::Agent($object) => $body,
            Self::Plan($object) => $body,
            Self::Association($object) => $body,
            Self::Usage($object) => $body,
            Self::Measure($object) => $body,
            Self::Unit($object) => $body,
            Self::SingularUnit($object) => $body,
            Self::CompoundUnit($object) => $body,
            Self::UnitMultiplication($object) => $body,
            Self::UnitDivision($object) => $body,
            Self::UnitExponentiation($object) => $body,
            Self::PrefixedUnit($object) => $body,
            Self::Prefix($object) => $body,
            Self::SIPrefix($object) => $body,
            Self::BinaryPrefix($object) => $body,
            Self::IdentifiedExtension($object) => $body,
        }
    };
}

pub(crate) use for_each_variant;

impl Sbol2Object {
    pub fn identity(&self) -> &Resource {
        for_each_variant!(self, object => &object.identity)
    }

    pub(crate) fn identified_data(&self) -> &IdentifiedData {
        for_each_variant!(self, object => &object.identified)
    }

    /// Whether this object is a TopLevel in the SBOL 2 hierarchy.
    pub fn is_top_level_object(&self) -> bool {
        self.class().is_top_level()
    }

    /// The TopLevel namespace IRI for this object, computed from its
    /// `persistentIdentity` (the identity minus its final `displayId`
    /// segment). Returns `None` for child objects or when no
    /// `persistentIdentity` is present.
    pub fn top_level_namespace(&self) -> Option<Iri> {
        if !self.is_top_level_object() {
            return None;
        }
        let persistent = self.identified_data().persistent_identity.as_ref()?;
        let iri = persistent.as_iri()?;
        let namespace = iri.as_str().rsplit_once('/')?.0;
        if namespace.is_empty() {
            return None;
        }
        Some(Iri::new_unchecked(namespace.to_owned()))
    }

    /// The parent object's compliant identity for a child object.
    ///
    /// SBOL 2 compliant child identities have the form
    /// `{parent_identity}/{display_id}/{version}`; this strips the trailing
    /// version and displayId segments. TopLevel objects return `None`.
    pub fn parent_identity(&self) -> Option<Resource> {
        if self.is_top_level_object() {
            return None;
        }
        let iri = self.identity().as_iri()?;
        let without_version = iri.as_str().rsplit_once('/')?.0;
        let parent = without_version.rsplit_once('/')?.0;
        if parent.is_empty() {
            return None;
        }
        Some(Resource::iri(parent.to_owned()))
    }

    pub fn class(&self) -> Sbol2Class {
        match self {
            Self::Sequence(_) => Sbol2Class::Sequence,
            Self::ComponentDefinition(_) => Sbol2Class::ComponentDefinition,
            Self::ModuleDefinition(_) => Sbol2Class::ModuleDefinition,
            Self::Model(_) => Sbol2Class::Model,
            Self::Collection(_) => Sbol2Class::Collection,
            Self::CombinatorialDerivation(_) => Sbol2Class::CombinatorialDerivation,
            Self::Implementation(_) => Sbol2Class::Implementation,
            Self::Attachment(_) => Sbol2Class::Attachment,
            Self::ExperimentalData(_) => Sbol2Class::ExperimentalData,
            Self::Experiment(_) => Sbol2Class::Experiment,
            Self::GenericTopLevel(_) => Sbol2Class::GenericTopLevel,
            Self::Component(_) => Sbol2Class::Component,
            Self::FunctionalComponent(_) => Sbol2Class::FunctionalComponent,
            Self::Module(_) => Sbol2Class::Module,
            Self::MapsTo(_) => Sbol2Class::MapsTo,
            Self::SequenceAnnotation(_) => Sbol2Class::SequenceAnnotation,
            Self::SequenceConstraint(_) => Sbol2Class::SequenceConstraint,
            Self::VariableComponent(_) => Sbol2Class::VariableComponent,
            Self::Interaction(_) => Sbol2Class::Interaction,
            Self::Participation(_) => Sbol2Class::Participation,
            Self::Range(_) => Sbol2Class::Range,
            Self::Cut(_) => Sbol2Class::Cut,
            Self::GenericLocation(_) => Sbol2Class::GenericLocation,
            Self::Activity(_) => Sbol2Class::ProvActivity,
            Self::Agent(_) => Sbol2Class::ProvAgent,
            Self::Plan(_) => Sbol2Class::ProvPlan,
            Self::Association(_) => Sbol2Class::ProvAssociation,
            Self::Usage(_) => Sbol2Class::ProvUsage,
            Self::Measure(_) => Sbol2Class::OmMeasure,
            Self::Unit(_) => Sbol2Class::OmUnit,
            Self::SingularUnit(_) => Sbol2Class::OmSingularUnit,
            Self::CompoundUnit(_) => Sbol2Class::OmCompoundUnit,
            Self::UnitMultiplication(_) => Sbol2Class::OmUnitMultiplication,
            Self::UnitDivision(_) => Sbol2Class::OmUnitDivision,
            Self::UnitExponentiation(_) => Sbol2Class::OmUnitExponentiation,
            Self::PrefixedUnit(_) => Sbol2Class::OmPrefixedUnit,
            Self::Prefix(_) => Sbol2Class::OmPrefix,
            Self::SIPrefix(_) => Sbol2Class::OmSiPrefix,
            Self::BinaryPrefix(_) => Sbol2Class::OmBinaryPrefix,
            Self::IdentifiedExtension(_) => Sbol2Class::Identified,
        }
    }
}
