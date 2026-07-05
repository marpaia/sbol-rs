//! SBOL 3 validation diagnostics, re-exported from the version-neutral
//! `sbol-core` reporting framework.

pub use sbol_core::validation::report::{
    AppliedOptions, CoverageKind, Hint, NotApplied, NotAppliedReason, PartialApplication,
    RuleCoverage, Severity, ValidationIssue, ValidationReport,
};
