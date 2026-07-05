//! Common re-exports for working with the version-neutral SBOL primitives.
//!
//! `use sbol_core::prelude::*;` brings the shared error, schema-descriptor,
//! object field-struct, and validation-framework types into scope.

pub use crate::error::{ReadError, WriteError};
pub use crate::object::{Identified, TopLevel};
pub use crate::schema::{Cardinality, FieldDescriptor, ReferenceSpec, TargetClass, ValueKind};
pub use crate::validation::{
    AppliedOptions, Blocker, CoverageKind, ExternalValidationMode, HashAlgorithmRegistry, Hint,
    NormativeSeverity, NotApplied, NotAppliedReason, PartialApplication, PolicyOptions,
    RuleCoverage, RuleOverride, RuleStatus, Severity, TopologyCompleteness, UnknownRule,
    VALIDATION_OUTPUT_SCHEMA_VERSION, ValidationIssue, ValidationReport, ValidationRuleStatus,
    compute_coverage, to_json,
};
