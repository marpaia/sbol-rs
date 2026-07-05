//! The version-neutral validation framework: coverage taxonomy, severity and
//! reporting types, and configuration shared by the SBOL 2 and SBOL 3
//! validators. Each version supplies its own rule catalog and rule
//! implementations on top of these primitives.

mod blocker;
pub mod coverage;
pub mod options;
pub mod output;
pub mod report;
pub mod rule_status;

pub use blocker::Blocker;
pub use coverage::{compute_coverage, coverage_kind_for};
pub use options::{
    ExternalValidationMode, HashAlgorithmRegistry, PolicyOptions, RuleOverride, RuleOverrides,
    TopologyCompleteness, UnknownRule,
};
pub use output::{VALIDATION_OUTPUT_SCHEMA_VERSION, to_json};
pub use report::{
    AppliedOptions, CoverageKind, Hint, NotApplied, NotAppliedReason, PartialApplication,
    RuleCoverage, Severity, ValidationIssue, ValidationReport,
};
pub use rule_status::{NormativeSeverity, RuleStatus, ValidationRuleStatus};
