use crate::Iri;
use crate::vocab::*;

pub use sbol_core::object::{Identified, TopLevel};

/// SBOL RDF classes handled by this crate.
///
/// Alongside the SBOL 2.3.0 vocabulary classes, this enum covers the abstract
/// mixins the specification layers property sets on (`Measured`,
/// `ComponentInstance`, `Location`) and the PROV-O and OM classes the spec
/// adopts. Treating them as first-class members of the class hierarchy lets
/// the typed surface, descriptor catalog, and `is_top_level` predicate stay in
/// sync with the spec without ad-hoc string comparisons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Sbol2Class {
    Identified,
    TopLevel,
    Measured,
    ComponentInstance,
    Location,
    Sequence,
    ComponentDefinition,
    ModuleDefinition,
    Model,
    Collection,
    CombinatorialDerivation,
    Implementation,
    Attachment,
    ExperimentalData,
    Experiment,
    GenericTopLevel,
    Component,
    FunctionalComponent,
    Module,
    MapsTo,
    SequenceAnnotation,
    SequenceConstraint,
    VariableComponent,
    Interaction,
    Participation,
    Range,
    Cut,
    GenericLocation,
    // PROV-O classes adopted by SBOL 2 (§13.1).
    ProvActivity,
    ProvAgent,
    ProvPlan,
    ProvAssociation,
    ProvUsage,
    // OM classes adopted by SBOL 2 (§13.2).
    OmMeasure,
    OmUnit,
    OmSingularUnit,
    OmCompoundUnit,
    OmUnitMultiplication,
    OmUnitDivision,
    OmUnitExponentiation,
    OmPrefixedUnit,
    OmPrefix,
    OmSiPrefix,
    OmBinaryPrefix,
}

impl Sbol2Class {
    pub fn from_iri(iri: &Iri) -> Option<Self> {
        let value = iri.as_str();
        if let Some(local) = value.strip_prefix(SBOL2_NS) {
            return Some(match local {
                "Identified" => Self::Identified,
                "TopLevel" => Self::TopLevel,
                "Measured" => Self::Measured,
                "ComponentInstance" => Self::ComponentInstance,
                "Location" => Self::Location,
                "Sequence" => Self::Sequence,
                "ComponentDefinition" => Self::ComponentDefinition,
                "ModuleDefinition" => Self::ModuleDefinition,
                "Model" => Self::Model,
                "Collection" => Self::Collection,
                "CombinatorialDerivation" => Self::CombinatorialDerivation,
                "Implementation" => Self::Implementation,
                "Attachment" => Self::Attachment,
                "ExperimentalData" => Self::ExperimentalData,
                "Experiment" => Self::Experiment,
                "GenericTopLevel" => Self::GenericTopLevel,
                "Component" => Self::Component,
                "FunctionalComponent" => Self::FunctionalComponent,
                "Module" => Self::Module,
                "MapsTo" => Self::MapsTo,
                "SequenceAnnotation" => Self::SequenceAnnotation,
                "SequenceConstraint" => Self::SequenceConstraint,
                "VariableComponent" => Self::VariableComponent,
                "Interaction" => Self::Interaction,
                "Participation" => Self::Participation,
                "Range" => Self::Range,
                "Cut" => Self::Cut,
                "GenericLocation" => Self::GenericLocation,
                _ => return None,
            });
        }
        if let Some(local) = value.strip_prefix(PROV_NS) {
            return Some(match local {
                "Activity" => Self::ProvActivity,
                "Agent" => Self::ProvAgent,
                "Plan" => Self::ProvPlan,
                "Association" => Self::ProvAssociation,
                "Usage" => Self::ProvUsage,
                _ => return None,
            });
        }
        if let Some(local) = value.strip_prefix(OM_NS) {
            return Some(match local {
                "Measure" => Self::OmMeasure,
                "Unit" => Self::OmUnit,
                "SingularUnit" => Self::OmSingularUnit,
                "CompoundUnit" => Self::OmCompoundUnit,
                "UnitMultiplication" => Self::OmUnitMultiplication,
                "UnitDivision" => Self::OmUnitDivision,
                "UnitExponentiation" => Self::OmUnitExponentiation,
                "PrefixedUnit" => Self::OmPrefixedUnit,
                "Prefix" => Self::OmPrefix,
                "SIPrefix" => Self::OmSiPrefix,
                "BinaryPrefix" => Self::OmBinaryPrefix,
                _ => return None,
            });
        }
        None
    }

    pub fn local_name(self) -> &'static str {
        match self {
            Self::Identified => "Identified",
            Self::TopLevel => "TopLevel",
            Self::Measured => "Measured",
            Self::ComponentInstance => "ComponentInstance",
            Self::Location => "Location",
            Self::Sequence => "Sequence",
            Self::ComponentDefinition => "ComponentDefinition",
            Self::ModuleDefinition => "ModuleDefinition",
            Self::Model => "Model",
            Self::Collection => "Collection",
            Self::CombinatorialDerivation => "CombinatorialDerivation",
            Self::Implementation => "Implementation",
            Self::Attachment => "Attachment",
            Self::ExperimentalData => "ExperimentalData",
            Self::Experiment => "Experiment",
            Self::GenericTopLevel => "GenericTopLevel",
            Self::Component => "Component",
            Self::FunctionalComponent => "FunctionalComponent",
            Self::Module => "Module",
            Self::MapsTo => "MapsTo",
            Self::SequenceAnnotation => "SequenceAnnotation",
            Self::SequenceConstraint => "SequenceConstraint",
            Self::VariableComponent => "VariableComponent",
            Self::Interaction => "Interaction",
            Self::Participation => "Participation",
            Self::Range => "Range",
            Self::Cut => "Cut",
            Self::GenericLocation => "GenericLocation",
            Self::ProvActivity => "Activity",
            Self::ProvAgent => "Agent",
            Self::ProvPlan => "Plan",
            Self::ProvAssociation => "Association",
            Self::ProvUsage => "Usage",
            Self::OmMeasure => "Measure",
            Self::OmUnit => "Unit",
            Self::OmSingularUnit => "SingularUnit",
            Self::OmCompoundUnit => "CompoundUnit",
            Self::OmUnitMultiplication => "UnitMultiplication",
            Self::OmUnitDivision => "UnitDivision",
            Self::OmUnitExponentiation => "UnitExponentiation",
            Self::OmPrefixedUnit => "PrefixedUnit",
            Self::OmPrefix => "Prefix",
            Self::OmSiPrefix => "SIPrefix",
            Self::OmBinaryPrefix => "BinaryPrefix",
        }
    }

    pub fn is_top_level(self) -> bool {
        matches!(
            self,
            Self::TopLevel
                | Self::Sequence
                | Self::ComponentDefinition
                | Self::ModuleDefinition
                | Self::Model
                | Self::Collection
                | Self::CombinatorialDerivation
                | Self::Implementation
                | Self::Attachment
                | Self::ExperimentalData
                | Self::Experiment
                | Self::GenericTopLevel
                | Self::ProvActivity
                | Self::ProvAgent
                | Self::ProvPlan
                | Self::OmUnit
                | Self::OmSingularUnit
                | Self::OmCompoundUnit
                | Self::OmUnitMultiplication
                | Self::OmUnitDivision
                | Self::OmUnitExponentiation
                | Self::OmPrefixedUnit
                | Self::OmPrefix
                | Self::OmSiPrefix
                | Self::OmBinaryPrefix
        )
    }

    pub const fn iri(self) -> &'static str {
        match self {
            Self::Identified => SBOL2_IDENTIFIED_CLASS,
            Self::TopLevel => SBOL2_TOP_LEVEL_CLASS,
            Self::Measured => SBOL2_MEASURED_CLASS,
            Self::ComponentInstance => SBOL2_COMPONENT_INSTANCE_CLASS,
            Self::Location => SBOL2_LOCATION_CLASS,
            Self::Sequence => SBOL2_SEQUENCE_CLASS,
            Self::ComponentDefinition => SBOL2_COMPONENT_DEFINITION_CLASS,
            Self::ModuleDefinition => SBOL2_MODULE_DEFINITION_CLASS,
            Self::Model => SBOL2_MODEL_CLASS,
            Self::Collection => SBOL2_COLLECTION_CLASS,
            Self::CombinatorialDerivation => SBOL2_COMBINATORIAL_DERIVATION_CLASS,
            Self::Implementation => SBOL2_IMPLEMENTATION_CLASS,
            Self::Attachment => SBOL2_ATTACHMENT_CLASS,
            Self::ExperimentalData => SBOL2_EXPERIMENTAL_DATA_CLASS,
            Self::Experiment => SBOL2_EXPERIMENT_CLASS,
            Self::GenericTopLevel => SBOL2_GENERIC_TOP_LEVEL_CLASS,
            Self::Component => SBOL2_COMPONENT_CLASS,
            Self::FunctionalComponent => SBOL2_FUNCTIONAL_COMPONENT_CLASS,
            Self::Module => SBOL2_MODULE_CLASS,
            Self::MapsTo => SBOL2_MAPS_TO_CLASS,
            Self::SequenceAnnotation => SBOL2_SEQUENCE_ANNOTATION_CLASS,
            Self::SequenceConstraint => SBOL2_SEQUENCE_CONSTRAINT_CLASS,
            Self::VariableComponent => SBOL2_VARIABLE_COMPONENT_CLASS,
            Self::Interaction => SBOL2_INTERACTION_CLASS,
            Self::Participation => SBOL2_PARTICIPATION_CLASS,
            Self::Range => SBOL2_RANGE_CLASS,
            Self::Cut => SBOL2_CUT_CLASS,
            Self::GenericLocation => SBOL2_GENERIC_LOCATION_CLASS,
            Self::ProvActivity => PROV_ACTIVITY,
            Self::ProvAgent => PROV_AGENT_CLASS,
            Self::ProvPlan => PROV_PLAN,
            Self::ProvAssociation => PROV_ASSOCIATION,
            Self::ProvUsage => PROV_USAGE,
            Self::OmMeasure => OM_MEASURE,
            Self::OmUnit => OM_UNIT,
            Self::OmSingularUnit => OM_SINGULAR_UNIT,
            Self::OmCompoundUnit => OM_COMPOUND_UNIT,
            Self::OmUnitMultiplication => OM_UNIT_MULTIPLICATION,
            Self::OmUnitDivision => OM_UNIT_DIVISION,
            Self::OmUnitExponentiation => OM_UNIT_EXPONENTIATION,
            Self::OmPrefixedUnit => OM_PREFIXED_UNIT,
            Self::OmPrefix => OM_PREFIX,
            Self::OmSiPrefix => OM_SI_PREFIX,
            Self::OmBinaryPrefix => OM_BINARY_PREFIX,
        }
    }

    pub fn is_a(self, target: Self) -> bool {
        if self == target {
            return true;
        }
        self.parents().iter().any(|parent| parent.is_a(target))
    }

    fn parents(self) -> &'static [Self] {
        match self {
            Self::Identified => &[],
            Self::TopLevel => &[Self::Identified],
            Self::Measured => &[Self::Identified],
            Self::ComponentInstance => &[Self::Measured],
            Self::Location => &[Self::Identified],
            Self::Sequence
            | Self::ComponentDefinition
            | Self::ModuleDefinition
            | Self::Model
            | Self::Collection
            | Self::CombinatorialDerivation
            | Self::Implementation
            | Self::Attachment
            | Self::ExperimentalData
            | Self::Experiment
            | Self::GenericTopLevel => &[Self::TopLevel],
            Self::Component | Self::FunctionalComponent => &[Self::ComponentInstance],
            Self::Module => &[Self::Measured],
            Self::MapsTo
            | Self::SequenceAnnotation
            | Self::SequenceConstraint
            | Self::VariableComponent => &[Self::Identified],
            Self::Interaction | Self::Participation => &[Self::Measured],
            Self::Range | Self::Cut | Self::GenericLocation => &[Self::Location],
            Self::ProvActivity | Self::ProvAgent | Self::ProvPlan => &[Self::TopLevel],
            Self::ProvAssociation | Self::ProvUsage => &[Self::Identified],
            Self::OmMeasure => &[Self::Identified],
            Self::OmUnit | Self::OmPrefix => &[Self::TopLevel],
            Self::OmSingularUnit | Self::OmCompoundUnit | Self::OmPrefixedUnit => &[Self::OmUnit],
            Self::OmUnitMultiplication | Self::OmUnitDivision | Self::OmUnitExponentiation => {
                &[Self::OmCompoundUnit]
            }
            Self::OmSiPrefix | Self::OmBinaryPrefix => &[Self::OmPrefix],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_inherits_measured_and_identified() {
        assert!(Sbol2Class::Component.is_a(Sbol2Class::ComponentInstance));
        assert!(Sbol2Class::Component.is_a(Sbol2Class::Measured));
        assert!(Sbol2Class::Component.is_a(Sbol2Class::Identified));
        assert!(!Sbol2Class::Component.is_a(Sbol2Class::TopLevel));
    }

    #[test]
    fn top_level_classification_matches_spec() {
        assert!(Sbol2Class::ComponentDefinition.is_top_level());
        assert!(Sbol2Class::Experiment.is_top_level());
        assert!(Sbol2Class::OmSingularUnit.is_top_level());
        assert!(!Sbol2Class::Component.is_top_level());
        assert!(!Sbol2Class::OmMeasure.is_top_level());
        assert!(!Sbol2Class::ProvAssociation.is_top_level());
        assert!(!Sbol2Class::Measured.is_top_level());
    }

    #[test]
    fn from_iri_round_trips_concrete_classes() {
        for class in [
            Sbol2Class::ComponentDefinition,
            Sbol2Class::Range,
            Sbol2Class::ProvActivity,
            Sbol2Class::OmUnitDivision,
        ] {
            let iri = Iri::from_static(class.iri());
            assert_eq!(Sbol2Class::from_iri(&iri), Some(class));
        }
    }

    #[test]
    fn om_subclasses_reach_compound_unit() {
        assert!(Sbol2Class::OmUnitDivision.is_a(Sbol2Class::OmCompoundUnit));
        assert!(Sbol2Class::OmUnitDivision.is_a(Sbol2Class::OmUnit));
        assert!(Sbol2Class::OmUnitDivision.is_a(Sbol2Class::TopLevel));
    }
}
