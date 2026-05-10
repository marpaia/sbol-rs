use std::env;
use std::fs;
use std::path::PathBuf;

// Extracts the `version` field from `crates/sbol/Cargo.toml` and exposes
// it as the `SBOL_CRATE_VERSION` env var at compile time so the bench
// report can label the native row with the right number. We do a tiny
// manual scan rather than pulling in a TOML parser, since the only
// thing we need is the top-level `version = "..."` line.
fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let sbol_manifest = manifest_dir
        .parent()
        .expect("crates dir")
        .join("sbol/Cargo.toml");
    println!("cargo:rerun-if-changed={}", sbol_manifest.display());

    let contents = fs::read_to_string(&sbol_manifest)
        .unwrap_or_else(|error| panic!("read {}: {error}", sbol_manifest.display()));
    let version = parse_version(&contents).unwrap_or_else(|| {
        panic!(
            "no top-level `version = \"...\"` in {}",
            sbol_manifest.display()
        )
    });
    println!("cargo:rustc-env=SBOL_CRATE_VERSION={version}");
}

fn parse_version(toml: &str) -> Option<String> {
    let mut in_package = false;
    for raw_line in toml.lines() {
        let line = raw_line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let (key, rest) = line.split_once('=')?;
        if key.trim() != "version" {
            continue;
        }
        let value = rest.trim().trim_start_matches('"').trim_end_matches('"');
        if value.is_empty() {
            return None;
        }
        return Some(value.to_owned());
    }
    None
}
