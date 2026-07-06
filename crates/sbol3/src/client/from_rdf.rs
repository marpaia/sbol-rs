use crate::client::{
    Activity, Agent, Association, Attachment, BinaryPrefix, Collection, CombinatorialDerivation,
    Component, ComponentReference, CompoundUnit, Constraint, Cut, EntireSequence, Experiment,
    ExperimentalData, ExternallyDefined, IdentifiedExtension, Implementation, Interaction,
    Interface, LocalSubComponent, Measure, Model, Participation, Plan, Prefix, PrefixedUnit, Range,
    SIPrefix, SbolObject, Sequence, SequenceFeature, SingularUnit, SubComponent, TryFromObject,
    Unit, UnitDivision, UnitExponentiation, UnitMultiplication, Usage, VariableFeature,
};
use crate::object::ObjectClasses;
use crate::{Object, SbolClass};

impl TryFromObject for SbolObject {
    fn try_from_object(object: &Object) -> Option<Self> {
        // Concrete SBOL subclasses dispatch first; bare-Identified
        // subjects fall through to the catch-all `IdentifiedExtension`.
        if object.has_class(SbolClass::Attachment) {
            return Some(Self::Attachment(Attachment::try_from_object(object)?));
        }
        if object.has_class(SbolClass::Experiment) {
            return Some(Self::Experiment(Experiment::try_from_object(object)?));
        }
        if object.has_class(SbolClass::Collection) {
            return Some(Self::Collection(Collection::try_from_object(object)?));
        }
        if object.has_class(SbolClass::CombinatorialDerivation) {
            return Some(Self::CombinatorialDerivation(
                CombinatorialDerivation::try_from_object(object)?,
            ));
        }
        if object.has_class(SbolClass::Component) {
            return Some(Self::Component(Component::try_from_object(object)?));
        }
        if object.has_class(SbolClass::ComponentReference) {
            return Some(Self::ComponentReference(
                ComponentReference::try_from_object(object)?,
            ));
        }
        if object.has_class(SbolClass::Constraint) {
            return Some(Self::Constraint(Constraint::try_from_object(object)?));
        }
        if object.has_class(SbolClass::Cut) {
            return Some(Self::Cut(Cut::try_from_object(object)?));
        }
        if object.has_class(SbolClass::EntireSequence) {
            return Some(Self::EntireSequence(EntireSequence::try_from_object(
                object,
            )?));
        }
        if object.has_class(SbolClass::ExperimentalData) {
            return Some(Self::ExperimentalData(ExperimentalData::try_from_object(
                object,
            )?));
        }
        if object.has_class(SbolClass::ExternallyDefined) {
            return Some(Self::ExternallyDefined(ExternallyDefined::try_from_object(
                object,
            )?));
        }
        if object.has_class(SbolClass::Implementation) {
            return Some(Self::Implementation(Implementation::try_from_object(
                object,
            )?));
        }
        if object.has_class(SbolClass::Interaction) {
            return Some(Self::Interaction(Interaction::try_from_object(object)?));
        }
        if object.has_class(SbolClass::Interface) {
            return Some(Self::Interface(Interface::try_from_object(object)?));
        }
        if object.has_class(SbolClass::LocalSubComponent) {
            return Some(Self::LocalSubComponent(LocalSubComponent::try_from_object(
                object,
            )?));
        }
        if object.has_class(SbolClass::Model) {
            return Some(Self::Model(Model::try_from_object(object)?));
        }
        if object.has_class(SbolClass::Participation) {
            return Some(Self::Participation(Participation::try_from_object(object)?));
        }
        if object.has_class(SbolClass::Range) {
            return Some(Self::Range(Range::try_from_object(object)?));
        }
        if object.has_class(SbolClass::Sequence) {
            return Some(Self::Sequence(Sequence::try_from_object(object)?));
        }
        if object.has_class(SbolClass::SequenceFeature) {
            return Some(Self::SequenceFeature(SequenceFeature::try_from_object(
                object,
            )?));
        }
        if object.has_class(SbolClass::SubComponent) {
            return Some(Self::SubComponent(SubComponent::try_from_object(object)?));
        }
        if object.has_class(SbolClass::VariableFeature) {
            return Some(Self::VariableFeature(VariableFeature::try_from_object(
                object,
            )?));
        }
        if object.has_class(SbolClass::ProvActivity) {
            return Some(Self::Activity(Activity::try_from_object(object)?));
        }
        if object.has_class(SbolClass::ProvAgent) {
            return Some(Self::Agent(Agent::try_from_object(object)?));
        }
        if object.has_class(SbolClass::ProvAssociation) {
            return Some(Self::Association(Association::try_from_object(object)?));
        }
        if object.has_class(SbolClass::ProvPlan) {
            return Some(Self::Plan(Plan::try_from_object(object)?));
        }
        if object.has_class(SbolClass::ProvUsage) {
            return Some(Self::Usage(Usage::try_from_object(object)?));
        }
        // OM Unit/Prefix subclasses dispatch before their parents so a
        // subject typed as both `om:SingularUnit` and `om:Unit` lands on
        // the more specific variant.
        if object.has_class(SbolClass::OmUnitDivision) {
            return Some(Self::UnitDivision(UnitDivision::try_from_object(object)?));
        }
        if object.has_class(SbolClass::OmUnitExponentiation) {
            return Some(Self::UnitExponentiation(
                UnitExponentiation::try_from_object(object)?,
            ));
        }
        if object.has_class(SbolClass::OmUnitMultiplication) {
            return Some(Self::UnitMultiplication(
                UnitMultiplication::try_from_object(object)?,
            ));
        }
        if object.has_class(SbolClass::OmSingularUnit) {
            return Some(Self::SingularUnit(SingularUnit::try_from_object(object)?));
        }
        if object.has_class(SbolClass::OmCompoundUnit) {
            return Some(Self::CompoundUnit(CompoundUnit::try_from_object(object)?));
        }
        if object.has_class(SbolClass::OmPrefixedUnit) {
            return Some(Self::PrefixedUnit(PrefixedUnit::try_from_object(object)?));
        }
        if object.has_class(SbolClass::OmUnit) {
            return Some(Self::Unit(Unit::try_from_object(object)?));
        }
        if object.has_class(SbolClass::OmSiPrefix) {
            return Some(Self::SIPrefix(SIPrefix::try_from_object(object)?));
        }
        if object.has_class(SbolClass::OmBinaryPrefix) {
            return Some(Self::BinaryPrefix(BinaryPrefix::try_from_object(object)?));
        }
        if object.has_class(SbolClass::OmPrefix) {
            return Some(Self::Prefix(Prefix::try_from_object(object)?));
        }
        if object.has_class(SbolClass::OmMeasure) {
            return Some(Self::Measure(Measure::try_from_object(object)?));
        }
        // Catch-all: subjects typed only as `sbol:Identified` (or
        // `sbol:TopLevel` without a concrete subclass). Preserves
        // displayId, name, derived_from, extensions, etc. so the typed
        // round trip is faithful.
        if object
            .classes()
            .iter()
            .any(|class| matches!(class, SbolClass::Identified | SbolClass::TopLevel))
        {
            return Some(Self::IdentifiedExtension(
                IdentifiedExtension::try_from_object(object)?,
            ));
        }
        None
    }
}
