use std::fmt;

use crate::Resource;
use crate::validation::blocker::Blocker;
use crate::validation::context::ExternalValidationMode;
use crate::validation::options::TopologyCompleteness;

/// Validation severity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Severity {
    Warning,
    Error,
}

/// A single SBOL validation issue.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ValidationIssue {
    pub severity: Severity,
    pub rule: &'static str,
    pub subject: Resource,
    pub property: Option<&'static str>,
    pub message: String,
    pub hint: Option<Hint>,
}

/// Actionable hint surfaced alongside a diagnostic. Closed enum so renderers
/// can pick a representation rather than scraping prose.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Hint {
    SuggestedTerm {
        table: &'static str,
        iri: &'static str,
        label: &'static str,
    },
    UrlPattern {
        expected: &'static str,
    },
    PreferredAlias {
        canonical: &'static str,
    },
    Note(&'static str),
}

impl ValidationIssue {
    pub(crate) fn error(
        rule: &'static str,
        subject: Resource,
        property: Option<&'static str>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Error,
            rule,
            subject,
            property,
            message: message.into(),
            hint: None,
        }
    }

    pub(crate) fn warning(
        rule: &'static str,
        subject: Resource,
        property: Option<&'static str>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            rule,
            subject,
            property,
            message: message.into(),
            hint: None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn with_hint(mut self, hint: Hint) -> Self {
        self.hint = Some(hint);
        self
    }
}

/// What a Partial rule's local subset actually covered for this validation
/// run. Stable, machine-grep-able tags so downstream tooling can filter
/// rather than parse prose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoverageKind {
    /// Ontology check covered every term in the bundled snapshot; terms
    /// outside the snapshot remain undecided by design.
    OntologyKnownTermsOnly,
    /// Reference integrity check covered document-local targets only;
    /// external references would require a resolver.
    LocalReferencesOnly,
    /// Lexical/datatype shape checked; remote content was not fetched.
    LexicalShapeOnly,
    /// Topology completeness was opt-in; default left the rule undecided
    /// for cases where the local subset has no signal.
    PolicyDefaultUndecided,
}

/// Why a rule did not run for this validation run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NotAppliedReason {
    /// Spec ▲ rule: violations are not to be machine-reported.
    MachineUncheckable,
    /// Catalog status is `Deferred` — no local algorithm exists yet.
    Deferred(Blocker),
}

/// Per-rule partial-application record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PartialApplication {
    pub rule: &'static str,
    pub blocker: Blocker,
    pub coverage_kind: CoverageKind,
}

/// Per-rule not-applied record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NotApplied {
    pub rule: &'static str,
    pub reason: NotAppliedReason,
}

/// Per-rule outcome for a single validation run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct RuleCoverage {
    /// Rules fully evaluated against this document with this configuration.
    pub fully_applied: Vec<&'static str>,
    /// Rules whose local subset ran but whose full coverage is blocked for
    /// this configuration.
    pub partially_applied: Vec<PartialApplication>,
    /// Rules the validator did not evaluate (Deferred, or ▲).
    pub not_applied: Vec<NotApplied>,
}

/// A summary of options that were active for this run. Round-trips through
/// the JSON output schema so downstream readers know what coverage they're
/// looking at.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct AppliedOptions {
    pub topology_completeness: TopologyCompleteness,
    pub external_mode: ExternalValidationMode,
    pub document_resolvers: usize,
    pub content_resolvers: usize,
    pub overridden_rules: Vec<(&'static str, crate::validation::options::RuleOverride)>,
    pub severity_floor: Option<Severity>,
    pub severity_ceiling: Option<Severity>,
}

/// Structured validation result for a document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ValidationReport {
    pub(crate) issues: Vec<ValidationIssue>,
    pub(crate) coverage: RuleCoverage,
    pub(crate) options_summary: AppliedOptions,
}

impl ValidationReport {
    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    pub fn coverage(&self) -> &RuleCoverage {
        &self.coverage
    }

    pub fn options_summary(&self) -> &AppliedOptions {
        &self.options_summary
    }

    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.severity == Severity::Error)
    }

    pub fn is_valid(&self) -> bool {
        !self.has_errors()
    }

    pub fn errors(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Error)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Warning)
    }
}

impl fmt::Display for ValidationReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_count = self.errors().count();
        let warning_count = self.warnings().count();
        write!(
            formatter,
            "SBOL validation failed with {error_count} errors and {warning_count} warnings"
        )?;
        for issue in self.errors().take(5) {
            write!(formatter, "\n{}: {}", issue.rule, issue.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationReport {}
