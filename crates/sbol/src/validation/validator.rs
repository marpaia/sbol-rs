use sbol_ontology::Ontology;

use crate::validation::blocker::Blocker;
use crate::validation::context::{ExternalValidationMode, ValidationContext};
use crate::validation::options::ValidationOptions;
use crate::validation::report::{
    AppliedOptions, CoverageKind, NotApplied, NotAppliedReason, PartialApplication, RuleCoverage,
    ValidationIssue, ValidationReport,
};
use crate::validation::resolver::OwnershipIndex;
use crate::validation::spec::{RuleStatus, ValidationRuleStatus, validation_rule_statuses};
use crate::{Document, Object};

pub(crate) struct Validator<'a> {
    pub(crate) document: &'a Document,
    pub(crate) ownership: OwnershipIndex,
    pub(crate) context: ValidationContext<'a>,
    pub(crate) issues: Vec<ValidationIssue>,
}

impl<'a> Validator<'a> {
    pub(crate) fn new(document: &'a Document, options: ValidationOptions) -> Self {
        Self::new_with_context(document, ValidationContext::with_options(options))
    }

    pub(crate) fn new_with_context(document: &'a Document, context: ValidationContext<'a>) -> Self {
        Self {
            document,
            ownership: OwnershipIndex::new(document),
            context,
            issues: Vec::new(),
        }
    }

    pub(crate) fn ontology(&self) -> &Ontology {
        self.context.ontology()
    }

    pub(crate) fn validate(&mut self) {
        for object in self.document.objects().values() {
            self.validate_sbol_namespace(object);
            self.validate_sbol_types(object);
            self.validate_table_rules(object);
            self.validate_display_id(object);
            self.validate_top_level(object);
            self.validate_sequence(object);
            self.validate_feature_vocabularies(object);
            self.validate_class_specific_rules(object);
            self.validate_workflow_rules(object);
        }
        self.validate_derived_from_cycles();
        self.validate_was_generated_by_cycles();
        self.validate_component_instance_cycles();
        self.validate_variant_derivation_cycles();
        self.validate_top_level_url_prefixes();
        self.validate_child_url_patterns();
        self.validate_location_sequence_membership();
    }

    pub(crate) fn finish(self) -> ValidationReport {
        let coverage = compute_coverage(self.context.external_mode());
        let options = self.context.options();
        let options_summary = AppliedOptions {
            topology_completeness: options.topology_completeness,
            external_mode: self.context.external_mode(),
            document_resolvers: self.context.document_resolvers().len()
                + self.context.documents().len(),
            content_resolvers: self.context.content_resolvers().len(),
            overridden_rules: options.overrides().collect(),
            severity_floor: options.severity_floor(),
            severity_ceiling: options.severity_ceiling(),
        };
        ValidationReport {
            issues: self.issues,
            coverage,
            options_summary,
        }
    }

    pub(crate) fn error(
        &mut self,
        rule: &'static str,
        object: &Object,
        property: Option<&'static str>,
        message: impl Into<String>,
    ) {
        self.emit(rule, object, property, message, crate::Severity::Error);
    }

    pub(crate) fn warning(
        &mut self,
        rule: &'static str,
        object: &Object,
        property: Option<&'static str>,
        message: impl Into<String>,
    ) {
        self.emit(rule, object, property, message, crate::Severity::Warning);
    }

    fn emit(
        &mut self,
        rule: &'static str,
        object: &Object,
        property: Option<&'static str>,
        message: impl Into<String>,
        catalog_default: crate::Severity,
    ) {
        let Some(severity) = self
            .context
            .options()
            .resolved_severity(rule, catalog_default)
        else {
            return;
        };
        let issue = match severity {
            crate::Severity::Error => {
                ValidationIssue::error(rule, object.identity().clone(), property, message)
            }
            crate::Severity::Warning => {
                ValidationIssue::warning(rule, object.identity().clone(), property, message)
            }
        };
        self.issues.push(issue);
    }

    /// Add pre-built issues to the report after applying per-rule overrides.
    /// Used by rule modules (e.g. combinatorial) that compute issues with a
    /// shared immutable borrow of the validator and append them in bulk.
    pub(crate) fn extend_with_overrides(
        &mut self,
        issues: impl IntoIterator<Item = ValidationIssue>,
    ) {
        let options = self.context.options();
        for mut issue in issues {
            let Some(severity) = options.resolved_severity(issue.rule, issue.severity) else {
                continue;
            };
            issue.severity = severity;
            self.issues.push(issue);
        }
    }
}

fn compute_coverage(external_mode: ExternalValidationMode) -> RuleCoverage {
    let mut coverage = RuleCoverage::default();
    for status in validation_rule_statuses() {
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

#[allow(dead_code)] // Reserved for future per-blocker coverage tagging.
fn coverage_kind_for(blocker: Blocker) -> CoverageKind {
    match blocker {
        Blocker::Ontology => CoverageKind::OntologyKnownTermsOnly,
        Blocker::Resolver | Blocker::External => CoverageKind::LocalReferencesOnly,
        Blocker::Policy => CoverageKind::PolicyDefaultUndecided,
        Blocker::StrictDatatype => CoverageKind::LexicalShapeOnly,
    }
}
