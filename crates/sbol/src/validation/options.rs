use sbol_ontology::Ontology;

use crate::validation::report::Severity;
use crate::validation::spec::validation_rule_statuses;

pub use sbol_core::validation::options::{
    HashAlgorithmRegistry, PolicyOptions, RuleOverride, TopologyCompleteness, UnknownRule,
};
use sbol_core::validation::options::RuleOverrides;

/// Options for local SBOL validation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ValidationOptions {
    pub topology_completeness: TopologyCompleteness,
    pub policy: PolicyOptions,
    pub(crate) rules: RuleOverrides,
    pub(crate) ontology_extensions: Vec<Ontology>,
}

impl ValidationOptions {
    /// Suppress diagnostics for `rule`. Errors if `rule` is not a
    /// catalog rule ID.
    pub fn allow(self, rule: &str) -> Result<Self, UnknownRule> {
        self.override_rule(rule, RuleOverride::Suppress)
    }

    /// Force `rule` to emit at error severity. Errors if `rule` is not
    /// a catalog rule ID.
    pub fn deny(self, rule: &str) -> Result<Self, UnknownRule> {
        self.override_rule(rule, RuleOverride::Severity(Severity::Error))
    }

    /// Force `rule` to emit at warning severity. Errors if `rule` is
    /// not a catalog rule ID.
    pub fn warn(self, rule: &str) -> Result<Self, UnknownRule> {
        self.override_rule(rule, RuleOverride::Severity(Severity::Warning))
    }

    /// Floor on the severity of any emitted issue (warnings can be
    /// promoted to errors but not below).
    pub fn with_severity_floor(mut self, floor: Severity) -> Self {
        self.rules = self.rules.with_severity_floor(floor);
        self
    }

    /// Ceiling on the severity of any emitted issue (errors can be
    /// demoted to warnings but not above).
    pub fn with_severity_ceiling(mut self, ceiling: Severity) -> Self {
        self.rules = self.rules.with_severity_ceiling(ceiling);
        self
    }

    /// Layer an additional [`Ontology`] snapshot on top of the bundled facts
    /// for the duration of this validation run. Use this to recognize terms
    /// from ontologies that are too large to bundle by default — e.g. NCIT
    /// loaded from an [`sbol_ontology::OntologyCache`].
    ///
    /// Bundled terms always win on conflicts; extensions can add new terms
    /// and parent links but cannot rewrite bundled ones.
    pub fn with_ontology_extension(mut self, extension: Ontology) -> Self {
        self.ontology_extensions.push(extension);
        self
    }

    pub(crate) fn take_ontology_extensions(&mut self) -> Vec<Ontology> {
        std::mem::take(&mut self.ontology_extensions)
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

    /// Resolve the effective severity for a rule emission. Returns
    /// `None` when the rule is suppressed.
    pub(crate) fn resolved_severity(
        &self,
        rule: &'static str,
        catalog_default: Severity,
    ) -> Option<Severity> {
        self.rules.resolved_severity(rule, catalog_default)
    }
}
