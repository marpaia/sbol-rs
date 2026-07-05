//! Shared test helpers. The SBOL 2 fixtures are vendored in-repo, so no
//! network download is required.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// Root directory of the vendored SBOL 2 fixture corpus.
pub fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/sbol2")
}

/// Reads a fixture file relative to [`fixture_root`].
pub fn read_fixture(relative: &str) -> String {
    let path = fixture_root().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read fixture {}: {error}", path.display()))
}
