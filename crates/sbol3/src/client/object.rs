use crate::client::{
    Activity, Agent, Association, Attachment, BinaryPrefix, Collection, CombinatorialDerivation,
    Component, ComponentReference, CompoundUnit, Constraint, Cut, EntireSequence, Experiment,
    ExperimentalData, ExternallyDefined, IdentifiedExtension, Implementation, Interaction,
    Interface, LocalSubComponent, Measure, Model, Participation, Plan, Prefix, PrefixedUnit, Range,
    SIPrefix, Sequence, SequenceFeature, SingularUnit, SubComponent, Unit, UnitDivision,
    UnitExponentiation, UnitMultiplication, Usage, VariableFeature,
};
use crate::{Iri, Resource, SbolClass};

/// Owned SBOL object variants supported by the client API.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SbolObject {
    Attachment(Attachment),
    Collection(Collection),
    CombinatorialDerivation(CombinatorialDerivation),
    Component(Component),
    ComponentReference(ComponentReference),
    Constraint(Constraint),
    Cut(Cut),
    EntireSequence(EntireSequence),
    Experiment(Experiment),
    ExperimentalData(ExperimentalData),
    ExternallyDefined(ExternallyDefined),
    Implementation(Implementation),
    Interaction(Interaction),
    Interface(Interface),
    LocalSubComponent(LocalSubComponent),
    Model(Model),
    Participation(Participation),
    Range(Range),
    Sequence(Sequence),
    SequenceFeature(SequenceFeature),
    SubComponent(SubComponent),
    VariableFeature(VariableFeature),
    Activity(Activity),
    Agent(Agent),
    Association(Association),
    Plan(Plan),
    Usage(Usage),
    Measure(Measure),
    Unit(Unit),
    SingularUnit(SingularUnit),
    CompoundUnit(CompoundUnit),
    UnitDivision(UnitDivision),
    UnitExponentiation(UnitExponentiation),
    UnitMultiplication(UnitMultiplication),
    PrefixedUnit(PrefixedUnit),
    Prefix(Prefix),
    SIPrefix(SIPrefix),
    BinaryPrefix(BinaryPrefix),
    IdentifiedExtension(IdentifiedExtension),
}

impl SbolObject {
    pub fn identity(&self) -> &Resource {
        match self {
            Self::Attachment(object) => &object.identity,
            Self::Collection(object) => &object.identity,
            Self::CombinatorialDerivation(object) => &object.identity,
            Self::Component(object) => &object.identity,
            Self::ComponentReference(object) => &object.identity,
            Self::Constraint(object) => &object.identity,
            Self::Cut(object) => &object.identity,
            Self::EntireSequence(object) => &object.identity,
            Self::Experiment(object) => &object.identity,
            Self::ExperimentalData(object) => &object.identity,
            Self::ExternallyDefined(object) => &object.identity,
            Self::Implementation(object) => &object.identity,
            Self::Interaction(object) => &object.identity,
            Self::Interface(object) => &object.identity,
            Self::LocalSubComponent(object) => &object.identity,
            Self::Model(object) => &object.identity,
            Self::Participation(object) => &object.identity,
            Self::Range(object) => &object.identity,
            Self::Sequence(object) => &object.identity,
            Self::SequenceFeature(object) => &object.identity,
            Self::SubComponent(object) => &object.identity,
            Self::VariableFeature(object) => &object.identity,
            Self::Activity(object) => &object.identity,
            Self::Agent(object) => &object.identity,
            Self::Association(object) => &object.identity,
            Self::Plan(object) => &object.identity,
            Self::Usage(object) => &object.identity,
            Self::Measure(object) => &object.identity,
            Self::Unit(object) => &object.identity,
            Self::SingularUnit(object) => &object.identity,
            Self::CompoundUnit(object) => &object.identity,
            Self::UnitDivision(object) => &object.identity,
            Self::UnitExponentiation(object) => &object.identity,
            Self::UnitMultiplication(object) => &object.identity,
            Self::PrefixedUnit(object) => &object.identity,
            Self::Prefix(object) => &object.identity,
            Self::SIPrefix(object) => &object.identity,
            Self::BinaryPrefix(object) => &object.identity,
            Self::IdentifiedExtension(object) => &object.identity,
        }
    }

    /// Returns the TopLevel namespace IRI if this object is a TopLevel.
    /// Returns `None` for child objects (Range, Constraint, ...).
    pub fn top_level_namespace(&self) -> Option<&Iri> {
        match self {
            Self::Attachment(o) => o.top_level.namespace.as_ref(),
            Self::Collection(o) => o.top_level.namespace.as_ref(),
            Self::CombinatorialDerivation(o) => o.top_level.namespace.as_ref(),
            Self::Component(o) => o.top_level.namespace.as_ref(),
            Self::Experiment(o) => o.top_level.namespace.as_ref(),
            Self::ExperimentalData(o) => o.top_level.namespace.as_ref(),
            Self::Implementation(o) => o.top_level.namespace.as_ref(),
            Self::Model(o) => o.top_level.namespace.as_ref(),
            Self::Sequence(o) => o.top_level.namespace.as_ref(),
            Self::Activity(o) => o.top_level.namespace.as_ref(),
            Self::Agent(o) => o.top_level.namespace.as_ref(),
            Self::Plan(o) => o.top_level.namespace.as_ref(),
            Self::Unit(o) => o.top_level.namespace.as_ref(),
            Self::SingularUnit(o) => o.top_level.namespace.as_ref(),
            Self::CompoundUnit(o) => o.top_level.namespace.as_ref(),
            Self::UnitDivision(o) => o.top_level.namespace.as_ref(),
            Self::UnitExponentiation(o) => o.top_level.namespace.as_ref(),
            Self::UnitMultiplication(o) => o.top_level.namespace.as_ref(),
            Self::PrefixedUnit(o) => o.top_level.namespace.as_ref(),
            Self::Prefix(o) => o.top_level.namespace.as_ref(),
            Self::SIPrefix(o) => o.top_level.namespace.as_ref(),
            Self::BinaryPrefix(o) => o.top_level.namespace.as_ref(),
            Self::IdentifiedExtension(o) => {
                o.top_level.as_ref().and_then(|tl| tl.namespace.as_ref())
            }
            _ => None,
        }
    }

    /// Returns the parent identity for child objects.
    ///
    /// For SBOL-compliant URLs of the form `{parent}/{display_id}`, returns
    /// the parent resource. TopLevel objects and objects with a non-IRI
    /// identity (e.g. blank nodes) return `None`. Child objects return
    /// `Some` even when the parent is not in the same document. Callers
    /// pair this with [`Document::resolve`] to look the parent up.
    ///
    /// [`Document::resolve`]: crate::Document::resolve
    pub fn parent_identity(&self) -> Option<Resource> {
        if self.top_level_namespace().is_some() {
            return None;
        }
        let iri = self.identity().as_iri()?;
        let url = iri.as_str();
        let parent = url.rsplit_once('/')?.0;
        if parent.is_empty() {
            return None;
        }
        Some(Resource::iri(parent.to_owned()))
    }

    pub fn class(&self) -> SbolClass {
        match self {
            Self::Attachment(_) => SbolClass::Attachment,
            Self::Collection(_) => SbolClass::Collection,
            Self::CombinatorialDerivation(_) => SbolClass::CombinatorialDerivation,
            Self::Component(_) => SbolClass::Component,
            Self::ComponentReference(_) => SbolClass::ComponentReference,
            Self::Constraint(_) => SbolClass::Constraint,
            Self::Cut(_) => SbolClass::Cut,
            Self::EntireSequence(_) => SbolClass::EntireSequence,
            Self::Experiment(_) => SbolClass::Experiment,
            Self::ExperimentalData(_) => SbolClass::ExperimentalData,
            Self::ExternallyDefined(_) => SbolClass::ExternallyDefined,
            Self::Implementation(_) => SbolClass::Implementation,
            Self::Interaction(_) => SbolClass::Interaction,
            Self::Interface(_) => SbolClass::Interface,
            Self::LocalSubComponent(_) => SbolClass::LocalSubComponent,
            Self::Model(_) => SbolClass::Model,
            Self::Participation(_) => SbolClass::Participation,
            Self::Range(_) => SbolClass::Range,
            Self::Sequence(_) => SbolClass::Sequence,
            Self::SequenceFeature(_) => SbolClass::SequenceFeature,
            Self::SubComponent(_) => SbolClass::SubComponent,
            Self::VariableFeature(_) => SbolClass::VariableFeature,
            Self::Activity(_) => SbolClass::ProvActivity,
            Self::Agent(_) => SbolClass::ProvAgent,
            Self::Association(_) => SbolClass::ProvAssociation,
            Self::Plan(_) => SbolClass::ProvPlan,
            Self::Usage(_) => SbolClass::ProvUsage,
            Self::Measure(_) => SbolClass::OmMeasure,
            Self::Unit(_) => SbolClass::OmUnit,
            Self::SingularUnit(_) => SbolClass::OmSingularUnit,
            Self::CompoundUnit(_) => SbolClass::OmCompoundUnit,
            Self::UnitDivision(_) => SbolClass::OmUnitDivision,
            Self::UnitExponentiation(_) => SbolClass::OmUnitExponentiation,
            Self::UnitMultiplication(_) => SbolClass::OmUnitMultiplication,
            Self::PrefixedUnit(_) => SbolClass::OmPrefixedUnit,
            Self::Prefix(_) => SbolClass::OmPrefix,
            Self::SIPrefix(_) => SbolClass::OmSiPrefix,
            Self::BinaryPrefix(_) => SbolClass::OmBinaryPrefix,
            Self::IdentifiedExtension(_) => SbolClass::Identified,
        }
    }
}
