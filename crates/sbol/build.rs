//! Generates the SBOL 3 validation rule catalog from `rules.toml` via the
//! shared `sbol-rulegen` generator.
//!
//! The generated files land in `OUT_DIR`:
//!   - `rule_catalog.rs` — the `VALIDATION_RULE_STATUSES` slice literal.
//!   - `rule_spec_meta.rs` — the `VALIDATION_RULE_SPEC_*` constants.

use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let rules_path = manifest_dir.join("rules.toml");
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR set by cargo"));

    // The policies dir lives in the workspace, not the crate, so it is
    // absent when building from a packaged crates.io tarball. Skip the ADR
    // check in that case; it is a maintainer-side guard against rules.toml
    // drift.
    let policies_dir = manifest_dir.join("../../docs/policies");
    let policies = if policies_dir.is_dir() {
        Some(policies_dir.as_path())
    } else {
        None
    };

    sbol_rulegen::generate(&rules_path, &out_dir, policies);

    println!("cargo:rerun-if-changed=rules.toml");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../docs/policies");
}
