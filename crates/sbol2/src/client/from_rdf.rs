use crate::client::{
    Activity, Agent, Association, Attachment, BinaryPrefix, Collection, CombinatorialDerivation,
    Component, ComponentDefinition, CompoundUnit, Cut, Experiment, ExperimentalData,
    FunctionalComponent, GenericLocation, GenericTopLevel, IdentifiedExtension, Implementation,
    Interaction, MapsTo, Measure, Model, Module, ModuleDefinition, Participation, Plan, Prefix,
    PrefixedUnit, Range, SIPrefix, Sbol2Object, Sequence, SequenceAnnotation, SequenceConstraint,
    SingularUnit, TryFromObject, Unit, UnitDivision, UnitExponentiation, UnitMultiplication, Usage,
    VariableComponent,
};
use crate::object::ObjectClasses;
use crate::{Object, Sbol2Class};

impl TryFromObject for Sbol2Object {
    fn try_from_object(object: &Object) -> Option<Self> {
        // Concrete subclasses dispatch most-specific-first; bare-Identified
        // and bare-TopLevel subjects fall through to `IdentifiedExtension`.
        if object.has_class(Sbol2Class::ComponentDefinition) {
            return Some(Self::ComponentDefinition(
                ComponentDefinition::try_from_object(object)?,
            ));
        }
        if object.has_class(Sbol2Class::ModuleDefinition) {
            return Some(Self::ModuleDefinition(ModuleDefinition::try_from_object(
                object,
            )?));
        }
        if object.has_class(Sbol2Class::Sequence) {
            return Some(Self::Sequence(Sequence::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::Model) {
            return Some(Self::Model(Model::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::Experiment) {
            return Some(Self::Experiment(Experiment::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::Collection) {
            return Some(Self::Collection(Collection::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::CombinatorialDerivation) {
            return Some(Self::CombinatorialDerivation(
                CombinatorialDerivation::try_from_object(object)?,
            ));
        }
        if object.has_class(Sbol2Class::Implementation) {
            return Some(Self::Implementation(Implementation::try_from_object(
                object,
            )?));
        }
        if object.has_class(Sbol2Class::Attachment) {
            return Some(Self::Attachment(Attachment::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::ExperimentalData) {
            return Some(Self::ExperimentalData(ExperimentalData::try_from_object(
                object,
            )?));
        }
        if object.has_class(Sbol2Class::Component) {
            return Some(Self::Component(Component::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::FunctionalComponent) {
            return Some(Self::FunctionalComponent(
                FunctionalComponent::try_from_object(object)?,
            ));
        }
        if object.has_class(Sbol2Class::Module) {
            return Some(Self::Module(Module::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::MapsTo) {
            return Some(Self::MapsTo(MapsTo::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::SequenceAnnotation) {
            return Some(Self::SequenceAnnotation(
                SequenceAnnotation::try_from_object(object)?,
            ));
        }
        if object.has_class(Sbol2Class::SequenceConstraint) {
            return Some(Self::SequenceConstraint(
                SequenceConstraint::try_from_object(object)?,
            ));
        }
        if object.has_class(Sbol2Class::VariableComponent) {
            return Some(Self::VariableComponent(VariableComponent::try_from_object(
                object,
            )?));
        }
        if object.has_class(Sbol2Class::Interaction) {
            return Some(Self::Interaction(Interaction::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::Participation) {
            return Some(Self::Participation(Participation::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::Range) {
            return Some(Self::Range(Range::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::Cut) {
            return Some(Self::Cut(Cut::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::GenericLocation) {
            return Some(Self::GenericLocation(GenericLocation::try_from_object(
                object,
            )?));
        }
        if object.has_class(Sbol2Class::ProvActivity) {
            return Some(Self::Activity(Activity::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::ProvAgent) {
            return Some(Self::Agent(Agent::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::ProvPlan) {
            return Some(Self::Plan(Plan::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::ProvAssociation) {
            return Some(Self::Association(Association::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::ProvUsage) {
            return Some(Self::Usage(Usage::try_from_object(object)?));
        }
        // OM Unit/Prefix subclasses dispatch before their parents so a subject
        // typed as both `om:SingularUnit` and `om:Unit` lands on the more
        // specific variant.
        if object.has_class(Sbol2Class::OmUnitDivision) {
            return Some(Self::UnitDivision(UnitDivision::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::OmUnitExponentiation) {
            return Some(Self::UnitExponentiation(
                UnitExponentiation::try_from_object(object)?,
            ));
        }
        if object.has_class(Sbol2Class::OmUnitMultiplication) {
            return Some(Self::UnitMultiplication(
                UnitMultiplication::try_from_object(object)?,
            ));
        }
        if object.has_class(Sbol2Class::OmSingularUnit) {
            return Some(Self::SingularUnit(SingularUnit::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::OmCompoundUnit) {
            return Some(Self::CompoundUnit(CompoundUnit::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::OmPrefixedUnit) {
            return Some(Self::PrefixedUnit(PrefixedUnit::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::OmUnit) {
            return Some(Self::Unit(Unit::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::OmSiPrefix) {
            return Some(Self::SIPrefix(SIPrefix::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::OmBinaryPrefix) {
            return Some(Self::BinaryPrefix(BinaryPrefix::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::OmPrefix) {
            return Some(Self::Prefix(Prefix::try_from_object(object)?));
        }
        if object.has_class(Sbol2Class::OmMeasure) {
            return Some(Self::Measure(Measure::try_from_object(object)?));
        }
        // A `sbol2:GenericTopLevel` subject with no more specific variant.
        if object.has_class(Sbol2Class::GenericTopLevel) {
            return Some(Self::GenericTopLevel(GenericTopLevel::try_from_object(
                object,
            )?));
        }
        // Catch-all: subjects typed only as `sbol2:Identified` or
        // `sbol2:TopLevel`. Preserves displayId, name, derived_from,
        // extensions, and the original rdf:type set so the round trip is
        // faithful.
        if object
            .classes()
            .iter()
            .any(|class| matches!(class, Sbol2Class::Identified | Sbol2Class::TopLevel))
        {
            return Some(Self::IdentifiedExtension(
                IdentifiedExtension::try_from_object(object)?,
            ));
        }
        None
    }
}
