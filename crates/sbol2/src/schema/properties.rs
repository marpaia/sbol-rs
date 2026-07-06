//! Per-class property specifications: the `*_PROPS` field tables consumed by
//! [`super::class_spec`]. Pure declarative data plus the `prop` / `local_ref`
//! / `external_ref` const constructors that build it.

use super::PropertySpec;
use crate::Sbol2Class;
use crate::schema::{Cardinality, ReferenceSpec, TargetClass, ValueKind};
use crate::vocab::*;

/// Sentinel rule identifier for property descriptors that no SBOL 2
/// validation rule governs (the om: Unit/Prefix ontology classes and the
/// shared abstract `measures` descriptor). The structural engine skips
/// descriptors carrying this id because it is absent from the rule catalog.
pub(crate) const NO_RULE: &str = "sbol2-00000";

const fn prop(
    rule: &'static str,
    predicate: &'static str,
    cardinality: Cardinality,
    value_kind: ValueKind,
    reference: Option<ReferenceSpec>,
) -> PropertySpec {
    PropertySpec::new(predicate, rule, cardinality, value_kind, reference)
}

const fn local_ref(target: TargetClass) -> Option<ReferenceSpec> {
    Some(ReferenceSpec::new(target, true))
}

const fn external_ref(target: TargetClass) -> Option<ReferenceSpec> {
    Some(ReferenceSpec::new(target, false))
}

const fn sbol(class: Sbol2Class) -> TargetClass {
    TargetClass::Sbol(class.iri())
}

pub(super) const IDENTIFIED_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-10203",
        SBOL2_PERSISTENT_IDENTITY,
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-10204",
        SBOL2_DISPLAY_ID,
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        "sbol2-10206",
        SBOL2_VERSION,
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        "sbol2-10212",
        DCTERMS_TITLE,
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        "sbol2-10213",
        DCTERMS_DESCRIPTION,
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        "sbol2-10208",
        PROV_WAS_DERIVED_FROM,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-10221",
        PROV_WAS_GENERATED_BY,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(TargetClass::ProvActivity),
    ),
];

pub(super) const TOP_LEVEL_PROPS: &[PropertySpec] = &[prop(
    "sbol2-10306",
    SBOL2_ATTACHMENT,
    Cardinality::ZeroOrMore,
    ValueKind::Uri,
    external_ref(sbol(Sbol2Class::Attachment)),
)];

pub(super) const MEASURED_PROPS: &[PropertySpec] = &[prop(
    NO_RULE,
    SBOL2_MEASURE,
    Cardinality::ZeroOrMore,
    ValueKind::Uri,
    local_ref(TargetClass::OmMeasure),
)];

pub(super) const COMPONENT_INSTANCE_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-10602",
        SBOL2_DEFINITION,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::ComponentDefinition)),
    ),
    prop(
        "sbol2-10607",
        SBOL2_ACCESS,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-10606",
        SBOL2_MAPS_TO,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(sbol(Sbol2Class::MapsTo)),
    ),
];

pub(super) const LOCATION_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-11002",
        SBOL2_ORIENTATION,
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-11003",
        SBOL2_SEQUENCE,
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::Sequence)),
    ),
];

pub(super) const SEQUENCE_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-10402",
        SBOL2_ELEMENTS,
        Cardinality::ExactlyOne,
        ValueKind::String,
        None,
    ),
    prop(
        "sbol2-10403",
        SBOL2_ENCODING,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
];

pub(super) const COMPONENT_DEFINITION_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-10502",
        SBOL2_TYPE,
        Cardinality::OneOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-10507",
        SBOL2_ROLE,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-10512",
        SBOL2_SEQUENCE,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::Sequence)),
    ),
    prop(
        "sbol2-10519",
        SBOL2_COMPONENT,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(sbol(Sbol2Class::Component)),
    ),
    prop(
        "sbol2-10521",
        SBOL2_SEQUENCE_ANNOTATION,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(sbol(Sbol2Class::SequenceAnnotation)),
    ),
    prop(
        "sbol2-10524",
        SBOL2_SEQUENCE_CONSTRAINT,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(sbol(Sbol2Class::SequenceConstraint)),
    ),
];

pub(super) const MODULE_DEFINITION_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-11602",
        SBOL2_ROLE,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-11604",
        SBOL2_MODULE,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(sbol(Sbol2Class::Module)),
    ),
    prop(
        "sbol2-11606",
        SBOL2_FUNCTIONAL_COMPONENT,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(sbol(Sbol2Class::FunctionalComponent)),
    ),
    prop(
        "sbol2-11605",
        SBOL2_INTERACTION,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(sbol(Sbol2Class::Interaction)),
    ),
    prop(
        "sbol2-11607",
        SBOL2_MODEL,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::Model)),
    ),
];

pub(super) const MODEL_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-11502",
        SBOL2_SOURCE,
        Cardinality::ExactlyOne,
        ValueKind::Url,
        None,
    ),
    prop(
        "sbol2-11504",
        SBOL2_LANGUAGE,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-11508",
        SBOL2_FRAMEWORK,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
];

pub(super) const COLLECTION_PROPS: &[PropertySpec] = &[prop(
    "sbol2-12102",
    SBOL2_MEMBER,
    Cardinality::ZeroOrMore,
    ValueKind::Uri,
    external_ref(sbol(Sbol2Class::TopLevel)),
)];

pub(super) const COMBINATORIAL_DERIVATION_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-12904",
        SBOL2_TEMPLATE,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::ComponentDefinition)),
    ),
    prop(
        "sbol2-12906",
        SBOL2_VARIABLE_COMPONENT,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(sbol(Sbol2Class::VariableComponent)),
    ),
    prop(
        "sbol2-12914",
        SBOL2_STRATEGY,
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        None,
    ),
];

pub(super) const IMPLEMENTATION_PROPS: &[PropertySpec] = &[prop(
    "sbol2-13102",
    SBOL2_BUILT,
    Cardinality::ZeroOrOne,
    ValueKind::Uri,
    None,
)];

pub(super) const ATTACHMENT_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-13202",
        SBOL2_SOURCE,
        Cardinality::ExactlyOne,
        ValueKind::Url,
        None,
    ),
    prop(
        "sbol2-13204",
        SBOL2_FORMAT,
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-13207",
        SBOL2_SIZE,
        Cardinality::ZeroOrOne,
        ValueKind::Long,
        None,
    ),
    prop(
        "sbol2-13208",
        SBOL2_HASH,
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
];

pub(super) const EXPERIMENTAL_DATA_PROPS: &[PropertySpec] = &[];

pub(super) const EXPERIMENT_PROPS: &[PropertySpec] = &[prop(
    "sbol2-13402",
    SBOL2_EXPERIMENTAL_DATA,
    Cardinality::ZeroOrMore,
    ValueKind::Uri,
    external_ref(sbol(Sbol2Class::ExperimentalData)),
)];

pub(super) const GENERIC_TOP_LEVEL_PROPS: &[PropertySpec] = &[prop(
    "sbol2-12302",
    SBOL2_RDF_TYPE,
    Cardinality::ExactlyOne,
    ValueKind::Uri,
    None,
)];

pub(super) const COMPONENT_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-10702",
        SBOL2_ROLE,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-10708",
        SBOL2_ROLE_INTEGRATION,
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-10710",
        SBOL2_SOURCE_LOCATION,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(sbol(Sbol2Class::Location)),
    ),
];

pub(super) const FUNCTIONAL_COMPONENT_PROPS: &[PropertySpec] = &[prop(
    "sbol2-11802",
    SBOL2_DIRECTION,
    Cardinality::ExactlyOne,
    ValueKind::Uri,
    None,
)];

pub(super) const MODULE_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-11702",
        SBOL2_DEFINITION,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::ModuleDefinition)),
    ),
    prop(
        "sbol2-11706",
        SBOL2_MAPS_TO,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(sbol(Sbol2Class::MapsTo)),
    ),
];

pub(super) const MAPS_TO_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-10802",
        SBOL2_LOCAL,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::ComponentInstance)),
    ),
    prop(
        "sbol2-10805",
        SBOL2_REMOTE,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::ComponentInstance)),
    ),
    prop(
        "sbol2-10810",
        SBOL2_REFINEMENT,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
];

pub(super) const SEQUENCE_ANNOTATION_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-10902",
        SBOL2_LOCATION,
        Cardinality::OneOrMore,
        ValueKind::Uri,
        local_ref(sbol(Sbol2Class::Location)),
    ),
    prop(
        "sbol2-10904",
        SBOL2_COMPONENT,
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::Component)),
    ),
    prop(
        "sbol2-10906",
        SBOL2_ROLE,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
];

pub(super) const SEQUENCE_CONSTRAINT_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-11402",
        SBOL2_SUBJECT,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::Component)),
    ),
    prop(
        "sbol2-11404",
        SBOL2_OBJECT,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::Component)),
    ),
    prop(
        "sbol2-11407",
        SBOL2_RESTRICTION,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
];

pub(super) const RANGE_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-11102",
        SBOL2_START,
        Cardinality::ExactlyOne,
        ValueKind::Integer,
        None,
    ),
    prop(
        "sbol2-11103",
        SBOL2_END,
        Cardinality::ExactlyOne,
        ValueKind::Integer,
        None,
    ),
];

pub(super) const CUT_PROPS: &[PropertySpec] = &[prop(
    "sbol2-11202",
    SBOL2_AT,
    Cardinality::ExactlyOne,
    ValueKind::Integer,
    None,
)];

pub(super) const GENERIC_LOCATION_PROPS: &[PropertySpec] = &[];

pub(super) const INTERACTION_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-11902",
        SBOL2_TYPE,
        Cardinality::OneOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-11906",
        SBOL2_PARTICIPATION,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(sbol(Sbol2Class::Participation)),
    ),
];

pub(super) const PARTICIPATION_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-12004",
        SBOL2_ROLE,
        Cardinality::OneOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-12002",
        SBOL2_PARTICIPANT,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::FunctionalComponent)),
    ),
];

pub(super) const VARIABLE_COMPONENT_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-13004",
        SBOL2_VARIABLE,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::Component)),
    ),
    prop(
        "sbol2-13007",
        SBOL2_VARIANT,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::ComponentDefinition)),
    ),
    prop(
        "sbol2-13009",
        SBOL2_VARIANT_COLLECTION,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::Collection)),
    ),
    prop(
        "sbol2-13013",
        SBOL2_VARIANT_DERIVATION,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(sbol(Sbol2Class::CombinatorialDerivation)),
    ),
    prop(
        "sbol2-13002",
        SBOL2_OPERATOR,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
];

pub(super) const PROV_ACTIVITY_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-12412",
        SBOL2_TYPE,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-12402",
        PROV_STARTED_AT_TIME,
        Cardinality::ZeroOrOne,
        ValueKind::DateTime,
        None,
    ),
    prop(
        "sbol2-12403",
        PROV_ENDED_AT_TIME,
        Cardinality::ZeroOrOne,
        ValueKind::DateTime,
        None,
    ),
    prop(
        "sbol2-12404",
        PROV_QUALIFIED_ASSOCIATION,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::ProvAssociation),
    ),
    prop(
        "sbol2-12405",
        PROV_QUALIFIED_USAGE,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::ProvUsage),
    ),
    prop(
        "sbol2-12406",
        PROV_WAS_INFORMED_BY,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(TargetClass::ProvActivity),
    ),
];

pub(super) const PROV_ASSOCIATION_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-12605",
        PROV_AGENT,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::ProvAgent),
    ),
    prop(
        "sbol2-12602",
        PROV_HAD_ROLE,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-12603",
        PROV_HAD_PLAN,
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        external_ref(TargetClass::ProvPlan),
    ),
];

pub(super) const PROV_USAGE_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-12502",
        PROV_ENTITY,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        "sbol2-12503",
        PROV_HAD_ROLE,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
];

pub(super) const PROV_AGENT_PROPS: &[PropertySpec] = &[];
pub(super) const PROV_PLAN_PROPS: &[PropertySpec] = &[];

pub(super) const OM_MEASURE_PROPS: &[PropertySpec] = &[
    prop(
        "sbol2-13502",
        OM_HAS_NUMERICAL_VALUE,
        Cardinality::ExactlyOne,
        ValueKind::Float,
        None,
    ),
    prop(
        "sbol2-13503",
        OM_HAS_UNIT,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
    prop(
        "sbol2-13504",
        SBOL2_TYPE,
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
];

pub(super) const OM_UNIT_PROPS: &[PropertySpec] = &[
    prop(
        NO_RULE,
        OM_SYMBOL,
        Cardinality::ExactlyOne,
        ValueKind::String,
        None,
    ),
    prop(
        NO_RULE,
        OM_ALTERNATIVE_SYMBOL,
        Cardinality::ZeroOrMore,
        ValueKind::String,
        None,
    ),
    prop(
        NO_RULE,
        RDFS_LABEL,
        Cardinality::ExactlyOne,
        ValueKind::String,
        None,
    ),
    prop(
        NO_RULE,
        OM_ALTERNATIVE_LABEL,
        Cardinality::ZeroOrMore,
        ValueKind::String,
        None,
    ),
    prop(
        NO_RULE,
        RDFS_COMMENT,
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        NO_RULE,
        OM_LONG_COMMENT,
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
];

pub(super) const OM_SINGULAR_UNIT_PROPS: &[PropertySpec] = &[
    prop(
        NO_RULE,
        OM_HAS_UNIT,
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
    prop(
        NO_RULE,
        OM_HAS_FACTOR,
        Cardinality::ZeroOrOne,
        ValueKind::Float,
        None,
    ),
];

pub(super) const OM_COMPOUND_UNIT_PROPS: &[PropertySpec] = &[];

pub(super) const OM_UNIT_MULTIPLICATION_PROPS: &[PropertySpec] = &[
    prop(
        NO_RULE,
        OM_HAS_TERM1,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
    prop(
        NO_RULE,
        OM_HAS_TERM2,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
];

pub(super) const OM_UNIT_DIVISION_PROPS: &[PropertySpec] = &[
    prop(
        NO_RULE,
        OM_HAS_NUMERATOR,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
    prop(
        NO_RULE,
        OM_HAS_DENOMINATOR,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
];

pub(super) const OM_UNIT_EXPONENTIATION_PROPS: &[PropertySpec] = &[
    prop(
        NO_RULE,
        OM_HAS_BASE,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
    prop(
        NO_RULE,
        OM_HAS_EXPONENT,
        Cardinality::ExactlyOne,
        ValueKind::Integer,
        None,
    ),
];

pub(super) const OM_PREFIXED_UNIT_PROPS: &[PropertySpec] = &[
    prop(
        NO_RULE,
        OM_HAS_UNIT,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
    prop(
        NO_RULE,
        OM_HAS_PREFIX,
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmPrefix),
    ),
];

pub(super) const OM_PREFIX_PROPS: &[PropertySpec] = &[
    prop(
        NO_RULE,
        OM_SYMBOL,
        Cardinality::ExactlyOne,
        ValueKind::String,
        None,
    ),
    prop(
        NO_RULE,
        OM_ALTERNATIVE_SYMBOL,
        Cardinality::ZeroOrMore,
        ValueKind::String,
        None,
    ),
    prop(
        NO_RULE,
        RDFS_LABEL,
        Cardinality::ExactlyOne,
        ValueKind::String,
        None,
    ),
    prop(
        NO_RULE,
        OM_ALTERNATIVE_LABEL,
        Cardinality::ZeroOrMore,
        ValueKind::String,
        None,
    ),
    prop(
        NO_RULE,
        RDFS_COMMENT,
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        NO_RULE,
        OM_LONG_COMMENT,
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        NO_RULE,
        OM_HAS_FACTOR,
        Cardinality::ExactlyOne,
        ValueKind::Float,
        None,
    ),
];

pub(super) const OM_SI_PREFIX_PROPS: &[PropertySpec] = &[];
pub(super) const OM_BINARY_PREFIX_PROPS: &[PropertySpec] = &[];
