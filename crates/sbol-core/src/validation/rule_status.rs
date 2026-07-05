//! Per-rule classification metadata shared by the SBOL validators. Each
//! version supplies its own rule catalog built from these types.

use crate::validation::blocker::Blocker;
use crate::validation::report::CoverageKind;

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

/// Which validation family a rule belongs to. libSBOLj dispatches rule
/// families from distinct passes controlled by the four validation-mode
/// flags; the gate names the pass that runs a rule. `Always` rules run in
/// every configuration; the other gates run only when their corresponding
/// `ValidationConfig` flag is set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ValidationGate {
    /// Runs in every validation configuration.
    #[default]
    Always,
    /// Runs only when `ValidationConfig::compliant` is set: the
    /// compliant-URI structural family.
    Compliant,
    /// Runs only when `ValidationConfig::complete` is set: the
    /// document-completeness family (every referenced object present).
    Complete,
    /// Runs only when `ValidationConfig::best_practice` is set: the
    /// SHOULD-level recommendation family.
    BestPractice,
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
    /// rules.
    pub blocker: Option<Blocker>,
    /// Name of the function that enforces this rule. `Some` for
    /// `Implemented*` rules and any `Partial` rule that has a concrete
    /// emission site; `None` for `Deferred` rules and the rare `Partial`
    /// rule whose local check is entirely data-table driven without a
    /// named function.
    pub validator_function: Option<&'static str>,
    /// Per-rule coverage tag for Partial rules. When set, overrides the
    /// default inferred from `blocker`. Used by the coverage signal to
    /// give callers a precise machine-grep-able tag for what the local
    /// subset covered.
    pub coverage_kind: Option<CoverageKind>,
    /// Validation family this rule belongs to. Selects which
    /// `ValidationConfig` flag must be set for the rule to run.
    pub gate: ValidationGate,
}

impl ValidationRuleStatus {
    /// Assemble a rule-status row. Rule catalogs generate one call per
    /// spec rule.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        rule: &'static str,
        status: RuleStatus,
        normative_severity: NormativeSeverity,
        spec_section: &'static str,
        note: &'static str,
        blocker: Option<Blocker>,
        validator_function: Option<&'static str>,
        coverage_kind: Option<CoverageKind>,
        gate: ValidationGate,
    ) -> Self {
        Self {
            rule,
            status,
            normative_severity,
            spec_section,
            note,
            blocker,
            validator_function,
            coverage_kind,
            gate,
        }
    }

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
