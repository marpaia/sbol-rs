//! Schema-conformance regression against the pinned upstream SBOL 3 OWL.
//!
//! The comparison machinery lives in [`sbol3::owl_conformance`]; this file
//! is the offline gate that fails CI on drift. The committed markdown
//! report at `docs/sbol-owl3-conformance.md` is the auditable view; its
//! freshness gate lives in `sbol_owl3_conformance_report.rs`.
//!
//! To refresh the pin run
//! `cargo run -p sbol-ontology --bin update-sbol-owl3-fixture`, then
//! re-triage any new diffs into the allowlists in
//! `crates/sbol/src/owl_conformance.rs`.

use std::path::PathBuf;

use sbol3::owl_conformance::{
    OWL_ONLY_ALLOWLIST, RUST_ONLY_ALLOWLIST, analyze_owl_conformance, extract_owl_identifiers,
};
use sha2::{Digest, Sha256};

#[derive(Debug, serde::Deserialize)]
struct Manifest {
    integrity: Integrity,
    source: Source,
}

#[derive(Debug, serde::Deserialize)]
struct Integrity {
    sha256: String,
}

#[derive(Debug, serde::Deserialize)]
struct Source {
    url: String,
    commit: String,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sbol-owl3")
}

fn load_manifest() -> Manifest {
    let path = fixture_dir().join("manifest.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read manifest {}: {e}", path.display()));
    toml::from_str(&text).unwrap_or_else(|e| panic!("parse manifest: {e}"))
}

fn load_fixture_bytes() -> Vec<u8> {
    let path = fixture_dir().join("sbol3.rdf");
    std::fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

fn format_iri_list(label: &str, items: &std::collections::BTreeSet<String>) -> String {
    if items.is_empty() {
        return String::new();
    }
    let mut out = format!("\n{label} ({} entries):\n", items.len());
    for iri in items {
        out.push_str("  ");
        out.push_str(iri);
        out.push('\n');
    }
    out
}

#[test]
fn pinned_fixture_matches_manifest_sha256() {
    let manifest = load_manifest();
    let bytes = load_fixture_bytes();
    let actual = hex_lower(Sha256::digest(&bytes).as_slice());
    assert_eq!(
        actual, manifest.integrity.sha256,
        "pinned sbol3.rdf sha256 does not match manifest.toml; \
         the fixture was edited without refreshing the manifest \
         (source: {} @ {})",
        manifest.source.url, manifest.source.commit
    );
}

#[test]
fn rust_vocab_covers_every_owl_iri_outside_allowlist() {
    let rdf = String::from_utf8(load_fixture_bytes()).expect("OWL fixture is UTF-8");
    let report = analyze_owl_conformance(&rdf).expect("analyze conformance");
    assert!(
        report.missing_from_vocab.is_empty(),
        "sbol-owl3 declares IRIs that vocab.rs does not surface. Either add \
         constants in vocab.rs, or add an entry to OWL_ONLY_ALLOWLIST with a \
         rationale.{}",
        format_iri_list("Missing from vocab.rs", &report.missing_from_vocab)
    );
    assert!(
        report.stale_owl_only.is_empty(),
        "OWL_ONLY_ALLOWLIST entries are no longer present in the pinned OWL: {:?}",
        report.stale_owl_only
    );
}

#[test]
fn owl_covers_every_rust_iri_outside_allowlist() {
    let rdf = String::from_utf8(load_fixture_bytes()).expect("OWL fixture is UTF-8");
    let report = analyze_owl_conformance(&rdf).expect("analyze conformance");
    assert!(
        report.missing_from_owl.is_empty(),
        "vocab.rs declares SBOL IRIs that the pinned OWL does not. Either \
         the OWL is missing the term (file upstream and add to \
         RUST_ONLY_ALLOWLIST with a spec citation), or the constant is a \
         bug and should be removed (see the #zero / CARDINALITY_ZERO \
         removal in CHANGELOG.md for the reference case).{}",
        format_iri_list("Missing from sbol3.rdf", &report.missing_from_owl)
    );
    assert!(
        report.stale_rust_only.is_empty(),
        "RUST_ONLY_ALLOWLIST entries are now present in the pinned OWL; \
         remove them from the allowlist: {:?}",
        report.stale_rust_only
    );
}

#[test]
fn owl_iri_counts_match_published_shape() {
    let rdf = String::from_utf8(load_fixture_bytes()).expect("OWL fixture is UTF-8");
    let owl = extract_owl_identifiers(&rdf).expect("parse pinned OWL");
    assert!(
        owl.classes.len() >= 30,
        "pinned OWL declares only {} classes; upstream may have been \
         truncated or significantly restructured",
        owl.classes.len()
    );
    assert!(
        owl.object_properties.len() >= 30,
        "pinned OWL declares only {} object properties",
        owl.object_properties.len()
    );
    assert!(
        owl.datatype_properties.len() >= 5,
        "pinned OWL declares only {} datatype properties",
        owl.datatype_properties.len()
    );
}

#[test]
fn allowlists_have_no_duplicates() {
    use std::collections::BTreeSet;
    let owl: BTreeSet<&str> = OWL_ONLY_ALLOWLIST.iter().map(|(iri, _)| *iri).collect();
    assert_eq!(
        owl.len(),
        OWL_ONLY_ALLOWLIST.len(),
        "OWL_ONLY_ALLOWLIST contains duplicate IRIs"
    );
    let rust: BTreeSet<&str> = RUST_ONLY_ALLOWLIST.iter().map(|(iri, _)| *iri).collect();
    assert_eq!(
        rust.len(),
        RUST_ONLY_ALLOWLIST.len(),
        "RUST_ONLY_ALLOWLIST contains duplicate IRIs"
    );
}
