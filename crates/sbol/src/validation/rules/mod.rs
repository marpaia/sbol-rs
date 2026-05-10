//! Per-spec-section validation rule implementations.
//!
//! Each submodule defines an `impl<'a> Validator<'a>` block that owns the
//! rules for one spec section. The top-level orchestration lives in
//! `super::validator`, which calls into these methods.

mod attachment;
mod combinatorial;
mod common;
mod component;
mod constraint;
mod feature;
mod global;
mod interaction;
mod location;
mod om;
mod sequence;
mod sub_component;
mod workflow;
