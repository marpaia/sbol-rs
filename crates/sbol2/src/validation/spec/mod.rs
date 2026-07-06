//! The generated SBOL 2 rule catalog and spec metadata. The class/property
//! schema itself lives in [`crate::schema`]; this module only carries the
//! per-rule classification metadata that `sbol-rulegen` emits from
//! `rules.toml`.

pub use sbol_core::validation::{NormativeSeverity, RuleStatus, ValidationRuleStatus};

include!(concat!(env!("OUT_DIR"), "/rule_spec_meta.rs"));

pub fn validation_rule_statuses() -> &'static [ValidationRuleStatus] {
    VALIDATION_RULE_STATUSES
}

/// Whether `rule` is a known SBOL 2 catalog rule id.
pub(crate) fn is_catalog_rule(rule: &str) -> bool {
    VALIDATION_RULE_STATUSES
        .iter()
        .any(|status| status.rule == rule)
}

include!(concat!(env!("OUT_DIR"), "/rule_catalog.rs"));
