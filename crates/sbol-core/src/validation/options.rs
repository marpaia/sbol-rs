//! Version-neutral validation configuration knobs shared by the SBOL
//! validators: severity overrides, policy toggles, topology assumptions, and
//! the external-resolution mode.

use std::collections::BTreeMap;
use std::fmt;

use crate::validation::report::Severity;
use crate::validation::rule_status::ValidationRuleStatus;

/// Controls whether validators assume missing nucleic-acid topology is knowable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum TopologyCompleteness {
    /// Missing topology remains undecided, matching the SBOL "if known" wording.
    #[default]
    Conservative,
    /// DNA/RNA objects are expected to state known topology explicitly.
    RequireKnownForNucleicAcids,
}

/// Controls whether validation may inspect resources outside the primary document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExternalValidationMode {
    /// Do not perform external document or content resolution.
    #[default]
    Off,
    /// Resolve only caller-provided documents and explicitly configured providers.
    ProvidedOnly,
    /// Resolve caller-provided data and configured external providers such as HTTP.
    ExternalAllowed,
}

/// Per-rule override applied at issue-emit time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RuleOverride {
    /// Drop diagnostics for this rule. Coverage still records the rule
    /// as applied — the caller asked for the result to be discarded,
    /// not for the check to be skipped.
    Suppress,
    /// Force severity for this rule regardless of the catalog default.
    Severity(Severity),
}

/// Returned when a per-rule override targets a rule that is not in the
/// catalog — almost always a typo at the call site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownRule {
    pub rule: String,
}

impl fmt::Display for UnknownRule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "`{}` is not a recognized SBOL validation rule",
            self.rule
        )
    }
}

impl std::error::Error for UnknownRule {}

/// Version-neutral engine for per-rule severity overrides and global
/// severity floor/ceiling clamping.
///
/// The engine canonicalizes rule ids against a catalog supplied by the
/// caller, so it holds no version-specific rule knowledge itself. A
/// versioned validator embeds this engine and feeds it its own rule
/// catalog.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuleOverrides {
    overrides: BTreeMap<&'static str, RuleOverride>,
    severity_floor: Option<Severity>,
    severity_ceiling: Option<Severity>,
}

impl RuleOverrides {
    /// Record `ovr` for `rule`, canonicalizing `rule` against `catalog`.
    /// Errors if `rule` is not a catalog rule id.
    pub fn set(
        mut self,
        rule: &str,
        ovr: RuleOverride,
        catalog: &[ValidationRuleStatus],
    ) -> Result<Self, UnknownRule> {
        let canonical = canonical_rule_id(rule, catalog)?;
        self.overrides.insert(canonical, ovr);
        Ok(self)
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

    /// Iterate the rule-id → override map. Stable iteration order.
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, RuleOverride)> + '_ {
        self.overrides.iter().map(|(rule, ovr)| (*rule, *ovr))
    }

    pub fn severity_floor(&self) -> Option<Severity> {
        self.severity_floor
    }

    pub fn severity_ceiling(&self) -> Option<Severity> {
        self.severity_ceiling
    }

    /// Resolve the effective severity for a rule emission. Returns
    /// `None` when the rule is suppressed.
    pub fn resolved_severity(
        &self,
        rule: &'static str,
        catalog_default: Severity,
    ) -> Option<Severity> {
        let base = match self.overrides.get(rule).copied() {
            Some(RuleOverride::Suppress) => return None,
            Some(RuleOverride::Severity(severity)) => severity,
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

fn canonical_rule_id(
    rule: &str,
    catalog: &[ValidationRuleStatus],
) -> Result<&'static str, UnknownRule> {
    catalog
        .iter()
        .find(|status| status.rule == rule)
        .map(|status| status.rule)
        .ok_or_else(|| UnknownRule {
            rule: rule.to_owned(),
        })
}

/// Policy decisions that affect rule semantics. Conservative defaults
/// match current behavior bit-for-bit; opting into Strict or Lenient
/// changes the outcome of specific Policy-blocked rules.
///
/// Each knob is wired end-to-end to at least one emit site. Variants
/// that the validator does not yet differentiate on were removed
/// rather than left as speculative API; reintroduce one at a time
/// when a concrete need arises.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct PolicyOptions {
    /// What is accepted as a value for `sbol:hashAlgorithm` (sbol3-12806).
    pub hash_algorithm_registry: HashAlgorithmRegistry,
}

/// How `sbol:hashAlgorithm` values are validated.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum HashAlgorithmRegistry {
    /// Accepts any non-empty string. Matches the spec's open-vocabulary
    /// wording for sbol3-12806.
    #[default]
    Conservative,
    /// Rejects values not in a curated registry of known hash algorithms.
    Strict,
    /// Suppresses the check entirely.
    Lenient,
}
