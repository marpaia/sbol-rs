use crate::vocab::*;
use crate::{Iri, Resource};

/// Shared fields on SBOL Identified objects.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Identified {
    pub display_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub derived_from: Vec<Resource>,
    pub generated_by: Vec<Resource>,
    pub measures: Vec<Resource>,
    pub attachments: Vec<Resource>,
}

/// Shared fields on SBOL TopLevel objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopLevel {
    pub namespace: Option<Iri>,
}

/// SBOL RDF classes handled by this crate.
///
/// In addition to the SBOL 3 vocabulary classes, this enum covers the
/// PROV-O and OM classes the SBOL 3.1.0 spec adopts. Treating them as
/// first-class members of the class hierarchy lets the typed surface,
/// table-driven validator, and `is_top_level` predicate stay in sync
/// with the spec without ad-hoc string comparisons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SbolClass {
    Identified,
    TopLevel,
    Attachment,
    Collection,
    CombinatorialDerivation,
    Component,
    ComponentReference,
    Constraint,
    Cut,
    EntireSequence,
    Experiment,
    ExperimentalData,
    ExternallyDefined,
    Feature,
    Implementation,
    Interaction,
    Interface,
    Location,
    LocalSubComponent,
    Model,
    Participation,
    Range,
    Sequence,
    SequenceFeature,
    SubComponent,
    VariableFeature,
    // PROV-O classes adopted by SBOL 3 (Appendix A.1).
    ProvActivity,
    ProvAgent,
    ProvAssociation,
    ProvPlan,
    ProvUsage,
    // OM classes adopted by SBOL 3 (Appendix A.2).
    OmMeasure,
    OmUnit,
    OmSingularUnit,
    OmCompoundUnit,
    OmUnitDivision,
    OmUnitExponentiation,
    OmUnitMultiplication,
    OmPrefixedUnit,
    OmPrefix,
    OmSiPrefix,
    OmBinaryPrefix,
}

impl SbolClass {
    pub fn from_iri(iri: &Iri) -> Option<Self> {
        let value = iri.as_str();
        if let Some(local) = value.strip_prefix(SBOL_NS) {
            return Some(match local {
                "Identified" => Self::Identified,
                "TopLevel" => Self::TopLevel,
                "Attachment" => Self::Attachment,
                "Collection" => Self::Collection,
                "CombinatorialDerivation" => Self::CombinatorialDerivation,
                "Component" => Self::Component,
                "ComponentReference" => Self::ComponentReference,
                "Constraint" => Self::Constraint,
                "Cut" => Self::Cut,
                "EntireSequence" => Self::EntireSequence,
                "Experiment" => Self::Experiment,
                "ExperimentalData" => Self::ExperimentalData,
                "ExternallyDefined" => Self::ExternallyDefined,
                "Feature" => Self::Feature,
                "Implementation" => Self::Implementation,
                "Interaction" => Self::Interaction,
                "Interface" => Self::Interface,
                "Location" => Self::Location,
                "LocalSubComponent" => Self::LocalSubComponent,
                "Model" => Self::Model,
                "Participation" => Self::Participation,
                "Range" => Self::Range,
                "Sequence" => Self::Sequence,
                "SequenceFeature" => Self::SequenceFeature,
                "SubComponent" => Self::SubComponent,
                "VariableFeature" => Self::VariableFeature,
                _ => return None,
            });
        }
        if let Some(local) = value.strip_prefix(PROV_NS) {
            return Some(match local {
                "Activity" => Self::ProvActivity,
                "Agent" => Self::ProvAgent,
                "Association" => Self::ProvAssociation,
                "Plan" => Self::ProvPlan,
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
                "UnitDivision" => Self::OmUnitDivision,
                "UnitExponentiation" => Self::OmUnitExponentiation,
                "UnitMultiplication" => Self::OmUnitMultiplication,
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
            Self::Attachment => "Attachment",
            Self::Collection => "Collection",
            Self::CombinatorialDerivation => "CombinatorialDerivation",
            Self::Component => "Component",
            Self::ComponentReference => "ComponentReference",
            Self::Constraint => "Constraint",
            Self::Cut => "Cut",
            Self::EntireSequence => "EntireSequence",
            Self::Experiment => "Experiment",
            Self::ExperimentalData => "ExperimentalData",
            Self::ExternallyDefined => "ExternallyDefined",
            Self::Feature => "Feature",
            Self::Implementation => "Implementation",
            Self::Interaction => "Interaction",
            Self::Interface => "Interface",
            Self::Location => "Location",
            Self::LocalSubComponent => "LocalSubComponent",
            Self::Model => "Model",
            Self::Participation => "Participation",
            Self::Range => "Range",
            Self::Sequence => "Sequence",
            Self::SequenceFeature => "SequenceFeature",
            Self::SubComponent => "SubComponent",
            Self::VariableFeature => "VariableFeature",
            Self::ProvActivity => "Activity",
            Self::ProvAgent => "Agent",
            Self::ProvAssociation => "Association",
            Self::ProvPlan => "Plan",
            Self::ProvUsage => "Usage",
            Self::OmMeasure => "Measure",
            Self::OmUnit => "Unit",
            Self::OmSingularUnit => "SingularUnit",
            Self::OmCompoundUnit => "CompoundUnit",
            Self::OmUnitDivision => "UnitDivision",
            Self::OmUnitExponentiation => "UnitExponentiation",
            Self::OmUnitMultiplication => "UnitMultiplication",
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
                | Self::Attachment
                | Self::Collection
                | Self::CombinatorialDerivation
                | Self::Component
                | Self::Experiment
                | Self::ExperimentalData
                | Self::Implementation
                | Self::Model
                | Self::Sequence
                | Self::ProvActivity
                | Self::ProvAgent
                | Self::ProvPlan
                | Self::OmUnit
                | Self::OmSingularUnit
                | Self::OmCompoundUnit
                | Self::OmUnitDivision
                | Self::OmUnitExponentiation
                | Self::OmUnitMultiplication
                | Self::OmPrefixedUnit
                | Self::OmPrefix
                | Self::OmSiPrefix
                | Self::OmBinaryPrefix
        )
    }

    pub const fn iri(self) -> &'static str {
        match self {
            Self::Identified => SBOL_IDENTIFIED_CLASS,
            Self::TopLevel => SBOL_TOP_LEVEL_CLASS,
            Self::Attachment => SBOL_ATTACHMENT_CLASS,
            Self::Collection => SBOL_COLLECTION_CLASS,
            Self::CombinatorialDerivation => SBOL_COMBINATORIAL_DERIVATION_CLASS,
            Self::Component => SBOL_COMPONENT_CLASS,
            Self::ComponentReference => SBOL_COMPONENT_REFERENCE_CLASS,
            Self::Constraint => SBOL_CONSTRAINT_CLASS,
            Self::Cut => SBOL_CUT_CLASS,
            Self::EntireSequence => SBOL_ENTIRE_SEQUENCE_CLASS,
            Self::Experiment => SBOL_EXPERIMENT_CLASS,
            Self::ExperimentalData => SBOL_EXPERIMENTAL_DATA_CLASS,
            Self::ExternallyDefined => SBOL_EXTERNALLY_DEFINED_CLASS,
            Self::Feature => SBOL_FEATURE_CLASS,
            Self::Implementation => SBOL_IMPLEMENTATION_CLASS,
            Self::Interaction => SBOL_INTERACTION_CLASS,
            Self::Interface => SBOL_INTERFACE_CLASS,
            Self::Location => SBOL_LOCATION_CLASS,
            Self::LocalSubComponent => SBOL_LOCAL_SUB_COMPONENT_CLASS,
            Self::Model => SBOL_MODEL_CLASS,
            Self::Participation => SBOL_PARTICIPATION_CLASS,
            Self::Range => SBOL_RANGE_CLASS,
            Self::Sequence => SBOL_SEQUENCE_CLASS,
            Self::SequenceFeature => SBOL_SEQUENCE_FEATURE_CLASS,
            Self::SubComponent => SBOL_SUB_COMPONENT_CLASS,
            Self::VariableFeature => SBOL_VARIABLE_FEATURE_CLASS,
            Self::ProvActivity => PROV_ACTIVITY,
            Self::ProvAgent => PROV_AGENT_CLASS,
            Self::ProvAssociation => PROV_ASSOCIATION,
            Self::ProvPlan => PROV_PLAN,
            Self::ProvUsage => PROV_USAGE,
            Self::OmMeasure => OM_MEASURE,
            Self::OmUnit => OM_UNIT,
            Self::OmSingularUnit => OM_SINGULAR_UNIT,
            Self::OmCompoundUnit => OM_COMPOUND_UNIT,
            Self::OmUnitDivision => OM_UNIT_DIVISION,
            Self::OmUnitExponentiation => OM_UNIT_EXPONENTIATION,
            Self::OmUnitMultiplication => OM_UNIT_MULTIPLICATION,
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
            Self::Attachment
            | Self::Collection
            | Self::CombinatorialDerivation
            | Self::Component
            | Self::ExperimentalData
            | Self::Implementation
            | Self::Model
            | Self::Sequence => &[Self::TopLevel],
            Self::Experiment => &[Self::Collection],
            Self::Feature => &[Self::Identified],
            Self::ComponentReference
            | Self::ExternallyDefined
            | Self::LocalSubComponent
            | Self::SequenceFeature
            | Self::SubComponent => &[Self::Feature],
            Self::Constraint | Self::Interaction | Self::Interface | Self::Participation => {
                &[Self::Identified]
            }
            Self::Location => &[Self::Identified],
            Self::Cut | Self::EntireSequence | Self::Range => &[Self::Location],
            Self::VariableFeature => &[Self::Identified],
            // PROV-O adopted classes mirror the ClassSpec parents declared in
            // `validation/spec.rs`. The SBOL spec adopts these classes as
            // TopLevel siblings (Activity/Agent/Plan) or as bare Identified
            // children (Association/Usage).
            Self::ProvActivity | Self::ProvAgent | Self::ProvPlan => &[Self::TopLevel],
            Self::ProvAssociation | Self::ProvUsage => &[Self::Identified],
            // OM Measure is Identified; Unit/Prefix and their subclasses are
            // TopLevels per the OM ontology's adoption in SBOL Appendix A.2.
            Self::OmMeasure => &[Self::Identified],
            Self::OmUnit | Self::OmPrefix => &[Self::TopLevel],
            Self::OmSingularUnit | Self::OmCompoundUnit | Self::OmPrefixedUnit => &[Self::OmUnit],
            Self::OmUnitDivision | Self::OmUnitExponentiation | Self::OmUnitMultiplication => {
                &[Self::OmCompoundUnit]
            }
            Self::OmSiPrefix | Self::OmBinaryPrefix => &[Self::OmPrefix],
        }
    }
}
