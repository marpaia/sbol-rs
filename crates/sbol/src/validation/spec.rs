use std::collections::BTreeMap;

use crate::vocab::*;
use crate::{Object, SbolClass};

use crate::schema::{
    Cardinality, ClassDescriptor, FieldDescriptor, ReferenceSpec, TargetClass, ValueKind,
};

pub(crate) type PropertySpec = FieldDescriptor;
pub(crate) type ClassSpec = ClassDescriptor;

/// Per-rule classification. Five categories, each capturing one
/// meaningful axis: algorithm complete with Error severity, algorithm
/// complete with Warning severity, configurable (behavior depends on
/// resolver / ontology snapshot / policy / external context), spec-▲
/// (not to be machine-reported), or unimplemented.
///
/// The `blocker` field carries the secondary axis: for `Configurable`,
/// it names *which* configuration knob (Resolver, Ontology, Policy,
/// External); for `Unimplemented`, it names *what's needed*
/// (Ontology data, Resolver protocol, Policy decision, OWL reasoning).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RuleStatus {
    /// Algorithm complete; MUST violations emit as `Severity::Error`.
    Error,
    /// Algorithm complete; SHOULD violations emit as `Severity::Warning`.
    Warning,
    /// Behavior varies with configuration. The `blocker` field names
    /// the axis: `Resolver` (algorithm needs a resolver for full
    /// scope), `Ontology` (algorithm correct for every snapshot-known
    /// term; out-of-snapshot terms undecided by design), `Policy`
    /// (algorithm runs at the `Conservative` ADR default; `Strict`/
    /// `Lenient` modes change emit behavior), or `External` (local-only
    /// mode is the implementation; the spec scope is structurally
    /// unreachable without external infrastructure like a global
    /// registry).
    Configurable,
    /// Spec ▲: violations are NOT to be machine-reported. The validator
    /// may run a local subset that emits **warnings only** on
    /// positively-decidable cases (e.g. a term that's known to be in
    /// the wrong ontology branch); the broader spec rule is recorded
    /// in coverage as `MachineUncheckable`.
    MachineUncheckable,
    /// No local algorithm. The `blocker` field names what's needed
    /// (Ontology data, Resolver protocol design, OWL axiom reasoning,
    /// Policy decision).
    Unimplemented,
}

/// RFC2119-style normative force of a spec rule. Distinguishes
/// "Partial-but-MUST" rules (gaps that block strict-mode compliance)
/// from "Partial-but-SHOULD" rules (gaps in recommended-but-optional
/// behavior).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NormativeSeverity {
    Must,
    Should,
    May,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ValidationRuleStatus {
    pub rule: &'static str,
    pub status: RuleStatus,
    pub normative_severity: NormativeSeverity,
    pub spec_section: &'static str,
    pub note: &'static str,
    /// Reason this rule is not fully implemented. Always `Some` for
    /// `Partial` / `Deferred` rules, always `None` for `Implemented*`
    /// rules. The invariant is enforced by
    /// `every_partial_or_deferred_rule_has_blocker` in
    /// `crates/sbol/tests/validation_rules.rs`.
    pub blocker: Option<super::Blocker>,
    /// Name of the function in `crates/sbol/src/validation/rules/`
    /// that enforces this rule. `Some` for `Implemented*` rules and
    /// any `Partial` rule that has a concrete emission site; `None`
    /// for `Deferred` rules and the rare `Partial` rule whose local
    /// check is entirely data-table driven without a named function.
    pub validator_function: Option<&'static str>,
    /// Per-rule coverage tag for Partial rules. When set, overrides the
    /// default inferred from `blocker`. Used by the coverage signal to
    /// give callers a precise machine-grep-able tag for what the local
    /// subset covered.
    pub coverage_kind: Option<super::CoverageKind>,
}

impl ValidationRuleStatus {
    /// True iff Appendix B marks this rule with ▲ (status
    /// `MachineUncheckable`).
    pub fn is_machine_uncheckable(&self) -> bool {
        matches!(self.status, RuleStatus::MachineUncheckable)
    }

    /// True iff the algorithm is complete and will fully evaluate the
    /// rule for at least one configuration.
    pub fn is_implemented(&self) -> bool {
        matches!(
            self.status,
            RuleStatus::Error | RuleStatus::Warning | RuleStatus::Configurable
        )
    }

    /// True iff the spec expects tools to machine-check this rule
    /// (Appendix B symbols ☑ ○ ⋆). Excludes ▲ rules.
    pub fn is_machine_checkable(&self) -> bool {
        !matches!(self.status, RuleStatus::MachineUncheckable)
    }
}

include!(concat!(env!("OUT_DIR"), "/rule_spec_meta.rs"));

const fn prop(
    predicate: &'static str,
    rule: &'static str,
    cardinality: Cardinality,
    value_kind: ValueKind,
    reference: Option<ReferenceSpec>,
) -> PropertySpec {
    PropertySpec {
        predicate,
        rule,
        cardinality,
        value_kind,
        reference,
    }
}

const fn local_ref(target: TargetClass) -> Option<ReferenceSpec> {
    Some(ReferenceSpec {
        target,
        require_local: true,
    })
}

const fn external_ref(target: TargetClass) -> Option<ReferenceSpec> {
    Some(ReferenceSpec {
        target,
        require_local: false,
    })
}

const IDENTIFIED_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_DISPLAY_ID,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        SBOL_NAME,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        SBOL_DESCRIPTION,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        SBOL_HAS_MEASURE,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::OmMeasure),
    ),
    prop(
        PROV_WAS_DERIVED_FROM,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        PROV_WAS_GENERATED_BY,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(TargetClass::ProvActivity),
    ),
];
const TOP_LEVEL_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_HAS_NAMESPACE,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Url,
        None,
    ),
    prop(
        SBOL_HAS_ATTACHMENT,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Attachment)),
    ),
];
const ATTACHMENT_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_SOURCE,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_FORMAT,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_HASH_ALGORITHM,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        SBOL_HASH,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        SBOL_SIZE,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::Long,
        None,
    ),
];
const COLLECTION_PROPS: &[PropertySpec] = &[prop(
    SBOL_MEMBER,
    "sbol3-10110",
    Cardinality::ZeroOrMore,
    ValueKind::Uri,
    external_ref(TargetClass::Sbol(SbolClass::TopLevel)),
)];
const COMBINATORIAL_DERIVATION_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_TEMPLATE,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Component)),
    ),
    prop(
        SBOL_STRATEGY,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_HAS_VARIABLE_FEATURE,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::Sbol(SbolClass::VariableFeature)),
    ),
];
const COMPONENT_REFERENCE_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_REFERS_TO,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Feature)),
    ),
    prop(
        SBOL_IN_CHILD_OF,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::SubComponent)),
    ),
];
const COMPONENT_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_HAS_SEQUENCE,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Sequence)),
    ),
    prop(
        SBOL_ROLE,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_TYPE,
        "sbol3-10110",
        Cardinality::OneOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_HAS_CONSTRAINT,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::Sbol(SbolClass::Constraint)),
    ),
    prop(
        SBOL_HAS_FEATURE,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::Sbol(SbolClass::Feature)),
    ),
    prop(
        SBOL_HAS_INTERACTION,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::Sbol(SbolClass::Interaction)),
    ),
    prop(
        SBOL_HAS_INTERFACE,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::Sbol(SbolClass::Interface)),
    ),
    prop(
        SBOL_HAS_MODEL,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Model)),
    ),
];
const CONSTRAINT_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_OBJECT,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Feature)),
    ),
    prop(
        SBOL_RESTRICTION,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_SUBJECT,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Feature)),
    ),
];
const CUT_PROPS: &[PropertySpec] = &[prop(
    SBOL_AT,
    "sbol3-10110",
    Cardinality::ExactlyOne,
    ValueKind::Integer,
    None,
)];
const EXTERNALLY_DEFINED_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_DEFINITION,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_TYPE,
        "sbol3-10110",
        Cardinality::OneOrMore,
        ValueKind::Uri,
        None,
    ),
];
const FEATURE_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_ROLE,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_ORIENTATION,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        None,
    ),
];
const IMPLEMENTATION_PROPS: &[PropertySpec] = &[prop(
    SBOL_BUILT,
    "sbol3-10110",
    Cardinality::ZeroOrOne,
    ValueKind::Uri,
    external_ref(TargetClass::Sbol(SbolClass::Component)),
)];
const INTERACTION_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_TYPE,
        "sbol3-10110",
        Cardinality::OneOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_HAS_PARTICIPATION,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::Sbol(SbolClass::Participation)),
    ),
];
const INTERFACE_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_INPUT,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Feature)),
    ),
    prop(
        SBOL_OUTPUT,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Feature)),
    ),
    prop(
        SBOL_NONDIRECTIONAL,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Feature)),
    ),
];
const LOCATION_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_ORIENTATION,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_ORDER,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::Integer,
        None,
    ),
    prop(
        SBOL_HAS_SEQUENCE,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Sequence)),
    ),
];
const LOCAL_SUB_COMPONENT_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_HAS_LOCATION,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::Sbol(SbolClass::Location)),
    ),
    prop(
        SBOL_TYPE,
        "sbol3-10110",
        Cardinality::OneOrMore,
        ValueKind::Uri,
        None,
    ),
];
const MODEL_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_SOURCE,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_FRAMEWORK,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_LANGUAGE,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
];
const PARTICIPATION_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_PARTICIPANT,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Feature)),
    ),
    prop(
        SBOL_HIGHER_ORDER_PARTICIPANT,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Interaction)),
    ),
    prop(
        SBOL_ROLE,
        "sbol3-10110",
        Cardinality::OneOrMore,
        ValueKind::Uri,
        None,
    ),
];
const RANGE_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_END,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Integer,
        None,
    ),
    prop(
        SBOL_START,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Integer,
        None,
    ),
];
const SEQUENCE_FEATURE_PROPS: &[PropertySpec] = &[prop(
    SBOL_HAS_LOCATION,
    "sbol3-10110",
    Cardinality::OneOrMore,
    ValueKind::Uri,
    local_ref(TargetClass::Sbol(SbolClass::Location)),
)];
const SEQUENCE_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_ELEMENTS,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        SBOL_ENCODING,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        None,
    ),
];
const SUB_COMPONENT_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_INSTANCE_OF,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Component)),
    ),
    prop(
        SBOL_ROLE_INTEGRATION,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_SOURCE_LOCATION,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::Sbol(SbolClass::Location)),
    ),
    prop(
        SBOL_HAS_LOCATION,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::Sbol(SbolClass::Location)),
    ),
];
const VARIABLE_FEATURE_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_CARDINALITY,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        SBOL_VARIABLE,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Feature)),
    ),
    prop(
        SBOL_VARIANT_COLLECTION,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Collection)),
    ),
    prop(
        SBOL_VARIANT_DERIVATION,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::CombinatorialDerivation)),
    ),
    prop(
        SBOL_VARIANT_MEASURE,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::OmMeasure),
    ),
    prop(
        SBOL_VARIANT,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(TargetClass::Sbol(SbolClass::Component)),
    ),
];
const PROV_ACTIVITY_PROPS: &[PropertySpec] = &[
    prop(
        PROV_ENDED_AT_TIME,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::DateTime,
        None,
    ),
    prop(
        PROV_QUALIFIED_USAGE,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::ProvUsage),
    ),
    prop(
        PROV_STARTED_AT_TIME,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::DateTime,
        None,
    ),
    prop(
        PROV_WAS_INFORMED_BY,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        external_ref(TargetClass::ProvActivity),
    ),
    prop(
        SBOL_TYPE,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        PROV_QUALIFIED_ASSOCIATION,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        local_ref(TargetClass::ProvAssociation),
    ),
];
const PROV_ASSOCIATION_PROPS: &[PropertySpec] = &[
    prop(
        PROV_AGENT,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::ProvAgent),
    ),
    prop(
        PROV_HAD_ROLE,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        PROV_HAD_PLAN,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        external_ref(TargetClass::ProvPlan),
    ),
];
const PROV_USAGE_PROPS: &[PropertySpec] = &[
    prop(
        PROV_ENTITY,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        None,
    ),
    prop(
        PROV_HAD_ROLE,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
];
const OM_MEASURE_PROPS: &[PropertySpec] = &[
    prop(
        SBOL_TYPE,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::Uri,
        None,
    ),
    prop(
        OM_HAS_UNIT,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
    prop(
        OM_HAS_NUMERICAL_VALUE,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Float,
        None,
    ),
];
const OM_UNIT_PROPS: &[PropertySpec] = &[
    prop(
        OM_ALTERNATIVE_LABEL,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::String,
        None,
    ),
    prop(
        OM_LABEL,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::String,
        None,
    ),
    prop(
        OM_LONG_COMMENT,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        OM_SYMBOL,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::String,
        None,
    ),
    prop(
        OM_ALTERNATIVE_SYMBOL,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::String,
        None,
    ),
    prop(
        OM_COMMENT,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
];
const OM_SINGULAR_UNIT_PROPS: &[PropertySpec] = &[
    prop(
        OM_HAS_UNIT,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
    prop(
        OM_HAS_FACTOR,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::Float,
        None,
    ),
];
const OM_PREFIXED_UNIT_PROPS: &[PropertySpec] = &[
    prop(
        OM_HAS_UNIT,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
    prop(
        OM_HAS_PREFIX,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmPrefix),
    ),
];
const OM_PREFIX_PROPS: &[PropertySpec] = &[
    prop(
        OM_ALTERNATIVE_LABEL,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::String,
        None,
    ),
    prop(
        OM_COMMENT,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        OM_HAS_FACTOR,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Float,
        None,
    ),
    prop(
        OM_LABEL,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::String,
        None,
    ),
    prop(
        OM_LONG_COMMENT,
        "sbol3-10110",
        Cardinality::ZeroOrOne,
        ValueKind::String,
        None,
    ),
    prop(
        OM_ALTERNATIVE_SYMBOL,
        "sbol3-10110",
        Cardinality::ZeroOrMore,
        ValueKind::String,
        None,
    ),
    prop(
        OM_SYMBOL,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::String,
        None,
    ),
];
const OM_UNIT_DIVISION_PROPS: &[PropertySpec] = &[
    prop(
        OM_HAS_DENOMINATOR,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
    prop(
        OM_HAS_NUMERATOR,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
];
const OM_UNIT_EXPONENTIATION_PROPS: &[PropertySpec] = &[
    prop(
        OM_HAS_BASE,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
    prop(
        OM_HAS_EXPONENT,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Integer,
        None,
    ),
];
const OM_UNIT_MULTIPLICATION_PROPS: &[PropertySpec] = &[
    prop(
        OM_HAS_TERM1,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
    prop(
        OM_HAS_TERM2,
        "sbol3-10110",
        Cardinality::ExactlyOne,
        ValueKind::Uri,
        external_ref(TargetClass::OmUnit),
    ),
];

pub fn validation_rule_statuses() -> &'static [ValidationRuleStatus] {
    VALIDATION_RULE_STATUSES
}

include!(concat!(env!("OUT_DIR"), "/rule_catalog.rs"));

pub(crate) fn class_spec(iri: &str) -> Option<ClassSpec> {
    Some(match iri {
        SBOL_IDENTIFIED_CLASS => ClassSpec {
            parents: &[],
            fields: IDENTIFIED_PROPS,
        },
        SBOL_TOP_LEVEL_CLASS => ClassSpec {
            parents: &[SBOL_IDENTIFIED_CLASS],
            fields: TOP_LEVEL_PROPS,
        },
        SBOL_ATTACHMENT_CLASS => ClassSpec {
            parents: &[SBOL_TOP_LEVEL_CLASS],
            fields: ATTACHMENT_PROPS,
        },
        SBOL_COLLECTION_CLASS => ClassSpec {
            parents: &[SBOL_TOP_LEVEL_CLASS],
            fields: COLLECTION_PROPS,
        },
        SBOL_COMBINATORIAL_DERIVATION_CLASS => ClassSpec {
            parents: &[SBOL_TOP_LEVEL_CLASS],
            fields: COMBINATORIAL_DERIVATION_PROPS,
        },
        SBOL_COMPONENT_CLASS => ClassSpec {
            parents: &[SBOL_TOP_LEVEL_CLASS],
            fields: COMPONENT_PROPS,
        },
        SBOL_COMPONENT_REFERENCE_CLASS => ClassSpec {
            parents: &[SBOL_FEATURE_CLASS],
            fields: COMPONENT_REFERENCE_PROPS,
        },
        SBOL_CONSTRAINT_CLASS => ClassSpec {
            parents: &[SBOL_IDENTIFIED_CLASS],
            fields: CONSTRAINT_PROPS,
        },
        SBOL_CUT_CLASS => ClassSpec {
            parents: &[SBOL_LOCATION_CLASS],
            fields: CUT_PROPS,
        },
        SBOL_ENTIRE_SEQUENCE_CLASS => ClassSpec {
            parents: &[SBOL_LOCATION_CLASS],
            fields: &[],
        },
        SBOL_EXPERIMENT_CLASS => ClassSpec {
            parents: &[SBOL_COLLECTION_CLASS],
            fields: &[],
        },
        SBOL_EXPERIMENTAL_DATA_CLASS => ClassSpec {
            parents: &[SBOL_TOP_LEVEL_CLASS],
            fields: &[],
        },
        SBOL_EXTERNALLY_DEFINED_CLASS => ClassSpec {
            parents: &[SBOL_FEATURE_CLASS],
            fields: EXTERNALLY_DEFINED_PROPS,
        },
        SBOL_FEATURE_CLASS => ClassSpec {
            parents: &[SBOL_IDENTIFIED_CLASS],
            fields: FEATURE_PROPS,
        },
        SBOL_IMPLEMENTATION_CLASS => ClassSpec {
            parents: &[SBOL_TOP_LEVEL_CLASS],
            fields: IMPLEMENTATION_PROPS,
        },
        SBOL_INTERACTION_CLASS => ClassSpec {
            parents: &[SBOL_IDENTIFIED_CLASS],
            fields: INTERACTION_PROPS,
        },
        SBOL_INTERFACE_CLASS => ClassSpec {
            parents: &[SBOL_IDENTIFIED_CLASS],
            fields: INTERFACE_PROPS,
        },
        SBOL_LOCATION_CLASS => ClassSpec {
            parents: &[SBOL_IDENTIFIED_CLASS],
            fields: LOCATION_PROPS,
        },
        SBOL_LOCAL_SUB_COMPONENT_CLASS => ClassSpec {
            parents: &[SBOL_FEATURE_CLASS],
            fields: LOCAL_SUB_COMPONENT_PROPS,
        },
        SBOL_MODEL_CLASS => ClassSpec {
            parents: &[SBOL_TOP_LEVEL_CLASS],
            fields: MODEL_PROPS,
        },
        SBOL_PARTICIPATION_CLASS => ClassSpec {
            parents: &[SBOL_IDENTIFIED_CLASS],
            fields: PARTICIPATION_PROPS,
        },
        SBOL_RANGE_CLASS => ClassSpec {
            parents: &[SBOL_LOCATION_CLASS],
            fields: RANGE_PROPS,
        },
        SBOL_SEQUENCE_CLASS => ClassSpec {
            parents: &[SBOL_TOP_LEVEL_CLASS],
            fields: SEQUENCE_PROPS,
        },
        SBOL_SEQUENCE_FEATURE_CLASS => ClassSpec {
            parents: &[SBOL_FEATURE_CLASS],
            fields: SEQUENCE_FEATURE_PROPS,
        },
        SBOL_SUB_COMPONENT_CLASS => ClassSpec {
            parents: &[SBOL_FEATURE_CLASS],
            fields: SUB_COMPONENT_PROPS,
        },
        SBOL_VARIABLE_FEATURE_CLASS => ClassSpec {
            parents: &[SBOL_IDENTIFIED_CLASS],
            fields: VARIABLE_FEATURE_PROPS,
        },
        PROV_ACTIVITY => ClassSpec {
            parents: &[SBOL_TOP_LEVEL_CLASS],
            fields: PROV_ACTIVITY_PROPS,
        },
        PROV_AGENT_CLASS => ClassSpec {
            parents: &[SBOL_TOP_LEVEL_CLASS],
            fields: &[],
        },
        PROV_ASSOCIATION => ClassSpec {
            parents: &[SBOL_IDENTIFIED_CLASS],
            fields: PROV_ASSOCIATION_PROPS,
        },
        PROV_PLAN => ClassSpec {
            parents: &[SBOL_TOP_LEVEL_CLASS],
            fields: &[],
        },
        PROV_USAGE => ClassSpec {
            parents: &[SBOL_IDENTIFIED_CLASS],
            fields: PROV_USAGE_PROPS,
        },
        OM_MEASURE => ClassSpec {
            parents: &[SBOL_IDENTIFIED_CLASS],
            fields: OM_MEASURE_PROPS,
        },
        OM_UNIT => ClassSpec {
            parents: &[SBOL_TOP_LEVEL_CLASS],
            fields: OM_UNIT_PROPS,
        },
        OM_SINGULAR_UNIT => ClassSpec {
            parents: &[OM_UNIT],
            fields: OM_SINGULAR_UNIT_PROPS,
        },
        OM_COMPOUND_UNIT => ClassSpec {
            parents: &[OM_UNIT],
            fields: &[],
        },
        OM_UNIT_DIVISION => ClassSpec {
            parents: &[OM_COMPOUND_UNIT],
            fields: OM_UNIT_DIVISION_PROPS,
        },
        OM_UNIT_EXPONENTIATION => ClassSpec {
            parents: &[OM_COMPOUND_UNIT],
            fields: OM_UNIT_EXPONENTIATION_PROPS,
        },
        OM_UNIT_MULTIPLICATION => ClassSpec {
            parents: &[OM_COMPOUND_UNIT],
            fields: OM_UNIT_MULTIPLICATION_PROPS,
        },
        OM_PREFIXED_UNIT => ClassSpec {
            parents: &[OM_UNIT],
            fields: OM_PREFIXED_UNIT_PROPS,
        },
        OM_PREFIX => ClassSpec {
            parents: &[SBOL_TOP_LEVEL_CLASS],
            fields: OM_PREFIX_PROPS,
        },
        OM_SI_PREFIX => ClassSpec {
            parents: &[OM_PREFIX],
            fields: &[],
        },
        OM_BINARY_PREFIX => ClassSpec {
            parents: &[OM_PREFIX],
            fields: &[],
        },
        _ => return None,
    })
}

pub(crate) fn property_specs_for(object: &Object) -> BTreeMap<&'static str, PropertySpec> {
    let mut specs = BTreeMap::new();
    for rdf_type in object.rdf_types() {
        collect_class_properties(rdf_type.as_str(), &mut specs);
    }
    specs
}

fn collect_class_properties(iri: &str, specs: &mut BTreeMap<&'static str, PropertySpec>) {
    let Some(class) = class_spec(iri) else {
        return;
    };
    for parent in class.parents {
        collect_class_properties(parent, specs);
    }
    for property in class.fields {
        specs.insert(property.predicate, *property);
    }
}

pub(crate) fn is_known_sbol_iri(iri: &str) -> bool {
    if class_spec(iri).is_some() || is_known_sbol_property(iri) {
        return true;
    }

    known_value_iris().contains(iri)
}

pub(crate) fn is_known_sbol_property(iri: &str) -> bool {
    known_property_predicates().contains(iri)
}

/// Set of every property predicate appearing in any class's `PropertySpec`
/// row. Built once and cached for the lifetime of the process so the
/// validator hot path does not allocate or scan on every lookup.
fn known_property_predicates() -> &'static std::collections::BTreeSet<&'static str> {
    static CACHE: std::sync::OnceLock<std::collections::BTreeSet<&'static str>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| {
        ALL_CLASS_IRIS
            .iter()
            .filter_map(|class_iri| class_spec(class_iri))
            .flat_map(|class| class.fields.iter().map(|property| property.predicate))
            .collect()
    })
}

/// Set view of `KNOWN_SBOL_VALUE_IRIS` for O(log n) lookup in the validator
/// hot path.
fn known_value_iris() -> &'static std::collections::BTreeSet<&'static str> {
    static CACHE: std::sync::OnceLock<std::collections::BTreeSet<&'static str>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| KNOWN_SBOL_VALUE_IRIS.iter().copied().collect())
}

const ALL_CLASS_IRIS: &[&str] = &[
    SBOL_IDENTIFIED_CLASS,
    SBOL_TOP_LEVEL_CLASS,
    SBOL_ATTACHMENT_CLASS,
    SBOL_COLLECTION_CLASS,
    SBOL_COMBINATORIAL_DERIVATION_CLASS,
    SBOL_COMPONENT_CLASS,
    SBOL_COMPONENT_REFERENCE_CLASS,
    SBOL_CONSTRAINT_CLASS,
    SBOL_CUT_CLASS,
    SBOL_ENTIRE_SEQUENCE_CLASS,
    SBOL_EXPERIMENT_CLASS,
    SBOL_EXPERIMENTAL_DATA_CLASS,
    SBOL_EXTERNALLY_DEFINED_CLASS,
    SBOL_FEATURE_CLASS,
    SBOL_IMPLEMENTATION_CLASS,
    SBOL_INTERACTION_CLASS,
    SBOL_INTERFACE_CLASS,
    SBOL_LOCATION_CLASS,
    SBOL_LOCAL_SUB_COMPONENT_CLASS,
    SBOL_MODEL_CLASS,
    SBOL_PARTICIPATION_CLASS,
    SBOL_RANGE_CLASS,
    SBOL_SEQUENCE_CLASS,
    SBOL_SEQUENCE_FEATURE_CLASS,
    SBOL_SUB_COMPONENT_CLASS,
    SBOL_VARIABLE_FEATURE_CLASS,
    PROV_ACTIVITY,
    PROV_AGENT_CLASS,
    PROV_ASSOCIATION,
    PROV_PLAN,
    PROV_USAGE,
    OM_MEASURE,
    OM_UNIT,
    OM_SINGULAR_UNIT,
    OM_COMPOUND_UNIT,
    OM_UNIT_DIVISION,
    OM_UNIT_EXPONENTIATION,
    OM_UNIT_MULTIPLICATION,
    OM_PREFIXED_UNIT,
    OM_PREFIX,
    OM_SI_PREFIX,
    OM_BINARY_PREFIX,
];
