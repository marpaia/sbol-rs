//! Per-spec-area SBOL 2 validation rule regression cases. Each machine-
//! checkable rule in `rules.toml` has a negative fixture here (a minimal
//! document that violates exactly that rule) and a positive fixture (a valid
//! instance that does not). `structural` is schema-driven (per-property
//! cardinality, value-kind, local-reference, and closed-property-set rules);
//! the remaining modules carry the hand-authored semantic rules.

pub mod common;

pub mod combinatorial;
pub mod component;
pub mod identity;
pub mod interaction;
pub mod location;
pub mod mapsto;
pub mod measure;
pub mod module;
pub mod provenance;
pub mod sequence;
pub mod structural;
pub mod toplevel;

pub use common::{
    all_on, all_positive_cases, all_rule_cases, read_case, read_positive_case, reports_any,
    reports_at,
};
