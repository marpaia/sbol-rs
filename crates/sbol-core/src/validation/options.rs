//! Version-neutral validation configuration knobs shared by the SBOL
//! validators: severity overrides, policy toggles, topology assumptions, and
//! the external-resolution mode.

use std::fmt;

use crate::validation::report::Severity;

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
