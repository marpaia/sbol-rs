//! JSON serialization of SBOL 3 validation reports, delegating to the
//! version-neutral emitter in `sbol-core` and stamping the SBOL 3 rule-spec
//! version.

use crate::validation::report::ValidationReport;
use crate::validation::spec::VALIDATION_RULE_SPEC_VERSION;

pub use sbol_core::validation::output::VALIDATION_OUTPUT_SCHEMA_VERSION;

/// Serialize a [`ValidationReport`] to JSON v1.
pub fn to_json(report: &ValidationReport) -> String {
    sbol_core::validation::to_json(report, VALIDATION_RULE_SPEC_VERSION)
}
