use std::collections::BTreeMap;

use crate::Object;
use crate::schema::{ClassDescriptor, FieldDescriptor};
use crate::vocab::*;

mod properties;
use properties::*;

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
