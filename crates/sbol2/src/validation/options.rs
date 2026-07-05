//! Options for SBOL 2 validation: the family-selection [`ValidationConfig`]
//! (which rule gates run) layered with per-rule severity overrides. The
//! defaults match libSBOLj's `SBOLValidate` command line: completeness and
//! compliant-URI checking on, best-practice checking off.

use crate::validation::spec::validation_rule_statuses;

use sbol_core::validation::options::RuleOverrides;
pub use sbol_core::validation::options::{RuleOverride, UnknownRule, ValidationConfig};
use sbol_core::validation::report::Severity;

/// Options for local SBOL 2 validation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ValidationOptions {
    /// Selects which rule families run.
    pub config: ValidationConfig,
    pub(crate) rules: RuleOverrides,
}

impl ValidationOptions {
    /// Suppress diagnostics for `rule`. Errors if `rule` is not a catalog id.
    pub fn allow(self, rule: &str) -> Result<Self, UnknownRule> {
        self.override_rule(rule, RuleOverride::Suppress)
    }

    /// Force `rule` to emit at error severity.
    pub fn deny(self, rule: &str) -> Result<Self, UnknownRule> {
        self.override_rule(rule, RuleOverride::Severity(Severity::Error))
    }

    /// Force `rule` to emit at warning severity.
    pub fn warn(self, rule: &str) -> Result<Self, UnknownRule> {
        self.override_rule(rule, RuleOverride::Severity(Severity::Warning))
    }

    /// Replace the whole validation-family configuration.
    pub fn with_config(mut self, config: ValidationConfig) -> Self {
        self.config = config;
        self
    }

    /// Toggle the completeness family.
    pub fn complete(mut self, on: bool) -> Self {
        self.config.complete = on;
        self
    }

    /// Toggle the compliant-URI family.
    pub fn compliant(mut self, on: bool) -> Self {
        self.config.compliant = on;
        self
    }

    /// Toggle the best-practice family.
    pub fn best_practice(mut self, on: bool) -> Self {
        self.config.best_practice = on;
        self
    }

    /// Iterate the rule-id → override map. Stable iteration order.
    pub fn overrides(&self) -> impl Iterator<Item = (&'static str, RuleOverride)> + '_ {
        self.rules.iter()
    }

    pub fn severity_floor(&self) -> Option<Severity> {
        self.rules.severity_floor()
    }

    pub fn severity_ceiling(&self) -> Option<Severity> {
        self.rules.severity_ceiling()
    }

    fn override_rule(mut self, rule: &str, ovr: RuleOverride) -> Result<Self, UnknownRule> {
        self.rules = self.rules.set(rule, ovr, validation_rule_statuses())?;
        Ok(self)
    }

    /// Resolve the effective severity for a rule emission. Returns `None`
    /// when the rule is suppressed.
    pub(crate) fn resolved_severity(
        &self,
        rule: &'static str,
        catalog_default: Severity,
    ) -> Option<Severity> {
        self.rules.resolved_severity(rule, catalog_default)
    }
}
