//! Per-rule coverage computation: classifies each catalog rule as fully
//! applied, partially applied, or not applied for a given validation run.

use crate::validation::blocker::Blocker;
use crate::validation::options::{ExternalValidationMode, ValidationConfig};
use crate::validation::report::{
    CoverageKind, NotApplied, NotAppliedReason, PartialApplication, RuleCoverage,
};
use crate::validation::rule_status::{RuleStatus, ValidationGate, ValidationRuleStatus};

/// True when `gate`'s family is enabled by `config`.
fn gate_enabled(gate: ValidationGate, config: ValidationConfig) -> bool {
    match gate {
        ValidationGate::Always => true,
        ValidationGate::Compliant => config.compliant,
        ValidationGate::Complete => config.complete,
        ValidationGate::BestPractice => config.best_practice,
    }
}

/// Classify every rule in `catalog` into fully/partially/not applied for a
/// run using `external_mode` and `config`.
pub fn compute_coverage(
    catalog: &[ValidationRuleStatus],
    external_mode: ExternalValidationMode,
    config: ValidationConfig,
) -> RuleCoverage {
    let mut coverage = RuleCoverage::default();
    for status in catalog {
        if !gate_enabled(status.gate, config) {
            coverage.not_applied.push(NotApplied {
                rule: status.rule,
                reason: NotAppliedReason::GatedOff(status.gate),
            });
            continue;
        }
        match coverage_for_rule(status, external_mode) {
            RuleCoverageOutcome::Fully => coverage.fully_applied.push(status.rule),
            RuleCoverageOutcome::Partial(application) => {
                coverage.partially_applied.push(application);
            }
            RuleCoverageOutcome::NotApplied(record) => coverage.not_applied.push(record),
        }
    }
    coverage
}

enum RuleCoverageOutcome {
    Fully,
    Partial(PartialApplication),
    NotApplied(NotApplied),
}

fn coverage_for_rule(
    status: &ValidationRuleStatus,
    external_mode: ExternalValidationMode,
) -> RuleCoverageOutcome {
    let resolver_satisfied = matches!(
        external_mode,
        ExternalValidationMode::ProvidedOnly | ExternalValidationMode::ExternalAllowed
    );
    match status.status {
        // Algorithm complete; unconditionally fires per its severity.
        RuleStatus::Error | RuleStatus::Warning => RuleCoverageOutcome::Fully,
        // Algorithm complete; behavior or scope varies with configuration.
        // The blocker names the axis; per-run coverage records what mode
        // ran.
        RuleStatus::Configurable => {
            let blocker = status
                .blocker
                .expect("Configurable rule must declare a blocker (build invariant)");
            match blocker {
                Blocker::Resolver if resolver_satisfied => RuleCoverageOutcome::Fully,
                Blocker::Resolver => RuleCoverageOutcome::Partial(PartialApplication {
                    rule: status.rule,
                    blocker,
                    coverage_kind: CoverageKind::LocalReferencesOnly,
                }),
                Blocker::External => RuleCoverageOutcome::Partial(PartialApplication {
                    rule: status.rule,
                    blocker,
                    coverage_kind: CoverageKind::LocalReferencesOnly,
                }),
                // Ontology / Policy / StrictDatatype: algorithm runs at
                // the configured mode; coverage is `fully_applied` for
                // that mode (the mode IS the algorithm, not a gap).
                _ => RuleCoverageOutcome::Fully,
            }
        }
        // ▲ rule: spec asks tools not to report violations as the spec
        // rule. The local subset (if any) emits Warnings; coverage
        // signal records the rule as `MachineUncheckable`.
        RuleStatus::MachineUncheckable => RuleCoverageOutcome::NotApplied(NotApplied {
            rule: status.rule,
            reason: NotAppliedReason::MachineUncheckable,
        }),
        // No local algorithm; needs new code, ontology data, resolver
        // protocol design, or a policy decision.
        RuleStatus::Unimplemented => {
            let blocker = status
                .blocker
                .expect("Unimplemented rule must declare a blocker (build invariant)");
            RuleCoverageOutcome::NotApplied(NotApplied {
                rule: status.rule,
                reason: NotAppliedReason::Deferred(blocker),
            })
        }
    }
}

/// Default coverage tag for a blocker axis.
#[allow(dead_code)]
pub fn coverage_kind_for(blocker: Blocker) -> CoverageKind {
    match blocker {
        Blocker::Ontology => CoverageKind::OntologyKnownTermsOnly,
        Blocker::Resolver | Blocker::External => CoverageKind::LocalReferencesOnly,
        Blocker::Policy => CoverageKind::PolicyDefaultUndecided,
        Blocker::StrictDatatype => CoverageKind::LexicalShapeOnly,
    }
}
