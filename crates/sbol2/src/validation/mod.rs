//! SBOL 2 validation: a rule catalog generated from `rules.toml`, the shared
//! reporting and configuration types from `sbol_core`, and the engine that
//! evaluates a document against the catalog.

mod helpers;
mod options;
mod spec;
mod validator;

// Re-exported so the generated rule catalog (included in `spec`) can name
// `super::Blocker`, `super::CoverageKind`, and `super::ValidationGate`.
pub use sbol_core::validation::{
    Blocker, CoverageKind, Hint, NormativeSeverity, NotApplied, NotAppliedReason,
    PartialApplication, RuleCoverage, RuleStatus, Severity, ValidationConfig, ValidationGate,
    ValidationIssue, ValidationReport, ValidationRuleStatus, to_json,
};

pub use options::{RuleOverride, UnknownRule, ValidationOptions};
pub use spec::{
    VALIDATION_RULE_SPEC_CANONICAL_URL, VALIDATION_RULE_SPEC_PATH, VALIDATION_RULE_SPEC_PDF_SHA256,
    VALIDATION_RULE_SPEC_VERSION, validation_rule_statuses,
};

pub(crate) use validator::Validator;
