use std::collections::BTreeMap;

use sbol_ontology::Ontology;

use crate::validation::report::Severity;
use crate::validation::spec::validation_rule_statuses;

pub use sbol_core::validation::options::{
    HashAlgorithmRegistry, PolicyOptions, RuleOverride, TopologyCompleteness, UnknownRule,
};

/// Options for local SBOL validation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ValidationOptions {
    pub topology_completeness: TopologyCompleteness,
    pub policy: PolicyOptions,
    pub(crate) overrides: BTreeMap<&'static str, RuleOverride>,
    pub(crate) severity_floor: Option<Severity>,
    pub(crate) severity_ceiling: Option<Severity>,
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
        self.severity_floor = Some(floor);
        self
    }

    /// Ceiling on the severity of any emitted issue (errors can be
    /// demoted to warnings but not above).
    pub fn with_severity_ceiling(mut self, ceiling: Severity) -> Self {
        self.severity_ceiling = Some(ceiling);
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
        self.overrides.iter().map(|(rule, ovr)| (*rule, *ovr))
    }

    pub fn severity_floor(&self) -> Option<Severity> {
        self.severity_floor
    }

    pub fn severity_ceiling(&self) -> Option<Severity> {
        self.severity_ceiling
    }

    fn override_rule(mut self, rule: &str, ovr: RuleOverride) -> Result<Self, UnknownRule> {
        let canonical = canonical_rule_id(rule)?;
        self.overrides.insert(canonical, ovr);
        Ok(self)
    }

    /// Resolve the effective severity for a rule emission. Returns
    /// `None` when the rule is suppressed.
    pub(crate) fn resolved_severity(
        &self,
        rule: &'static str,
        catalog_default: Severity,
    ) -> Option<Severity> {
        let base = match self.overrides.get(rule).copied() {
            Some(RuleOverride::Suppress) => return None,
            Some(RuleOverride::Severity(severity)) => severity,
            Some(_) => catalog_default,
            None => catalog_default,
        };
        let with_floor = match self.severity_floor {
            Some(floor) if floor > base => floor,
            _ => base,
        };
        let with_ceiling = match self.severity_ceiling {
            Some(ceiling) if ceiling < with_floor => ceiling,
            _ => with_floor,
        };
        Some(with_ceiling)
    }
}

fn canonical_rule_id(rule: &str) -> Result<&'static str, UnknownRule> {
    validation_rule_statuses()
        .iter()
        .find(|status| status.rule == rule)
        .map(|status| status.rule)
        .ok_or_else(|| UnknownRule {
            rule: rule.to_owned(),
        })
}
