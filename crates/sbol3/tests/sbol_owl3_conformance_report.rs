//! Freshness gate for `docs/sbol-owl3-conformance.md`. Fails CI if the
//! committed report drifts from what
//! `generate-sbol-owl3-conformance-report` would produce against the
//! current pinned OWL fixture and `vocab.rs`. Run
//! `cargo run -p sbol3 --bin generate-sbol-owl3-conformance-report` to
//! refresh.

use std::path::PathBuf;

use sbol3::owl_conformance::{OwlPinInfo, analyze_owl_conformance, render_owl_conformance_report};

#[derive(Debug, serde::Deserialize)]
struct Manifest {
    source: Source,
    integrity: Integrity,
    fetched: Fetched,
}

#[derive(Debug, serde::Deserialize)]
struct Source {
    upstream_repo: String,
    url: String,
    commit: String,
    committer_date: String,
}

#[derive(Debug, serde::Deserialize)]
struct Integrity {
    sha256: String,
}

#[derive(Debug, serde::Deserialize)]
struct Fetched {
    at: String,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sbol-owl3")
}

#[test]
fn sbol_owl3_conformance_report_matches_committed_file() {
    let rdf =
        std::fs::read_to_string(fixture_dir().join("sbol3.rdf")).expect("read pinned OWL fixture");
    let manifest_text = std::fs::read_to_string(fixture_dir().join("manifest.toml"))
        .expect("read pinned OWL manifest");
    let manifest: Manifest = toml::from_str(&manifest_text).expect("parse manifest");

    let report = analyze_owl_conformance(&rdf).expect("analyze conformance");
    let pin = OwlPinInfo {
        upstream_repo: &manifest.source.upstream_repo,
        source_url: &manifest.source.url,
        commit: &manifest.source.commit,
        committer_date: &manifest.source.committer_date,
        sha256: &manifest.integrity.sha256,
        fetched_at: &manifest.fetched.at,
    };
    let rendered = render_owl_conformance_report(&report, &pin);
    let committed = include_str!("../../../docs/sbol-owl3-conformance.md");
    assert_eq!(
        committed, rendered,
        "docs/sbol-owl3-conformance.md is stale. Run \
         `cargo run -p sbol3 --bin generate-sbol-owl3-conformance-report` \
         to refresh."
    );
}
