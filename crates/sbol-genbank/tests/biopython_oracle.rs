//! Cross-implementation feature-table oracle: compare `sbol-genbank`'s
//! extracted feature locations against BioPython's `Bio.SeqIO`.
//!
//! BioPython is the reference GenBank parser in the bioinformatics
//! community. For every committed fixture, `tests/oracle/emit_features.py`
//! (run under the pinned Docker image in `tests/oracle/`) records each
//! non-`source` feature's location spans as sorted 1-based-closed
//! `[start, end, strand]` triples. This test reproduces that table from
//! `sbol-genbank`'s output — mapping SBOL inline/reverseComplement
//! orientation back to strand `1`/`-1` — and asserts the two agree.
//!
//! The comparison is over location structure and orientation only; the
//! GenBank-key → Sequence Ontology mapping is grounded separately by the
//! gb2so parity test. Features are matched as an unordered multiset of
//! span-signatures, so neither feature order nor BioPython's part-order
//! reversal for reverse-strand compound locations matters.
//!
//! If the committed oracle JSON is absent (Docker unavailable when it was
//! last regenerated), the test skips cleanly with a count rather than
//! failing. Regenerate with `tests/oracle/regenerate.sh`.

use std::path::PathBuf;

use sbol_genbank::GenbankImporter;
use sbol3::LocationRef;
use serde_json::Value;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    let mut p = manifest_dir();
    p.pop();
    p.pop();
    p
}

/// (oracle-name, fixture path). Real records live at the workspace-root
/// corpus; the clean multi-span fixture is crate-local.
fn fixtures() -> Vec<(&'static str, PathBuf)> {
    let root = workspace_root();
    let crate_dir = manifest_dir();
    vec![
        (
            "BBa_E0040",
            root.join("tests/fixtures/genbank/BBa_E0040.gb"),
        ),
        (
            "BBa_R0010",
            root.join("tests/fixtures/genbank/BBa_R0010.gb"),
        ),
        (
            "BBa_B0034",
            root.join("tests/fixtures/genbank/BBa_B0034.gb"),
        ),
        (
            "BBa_F2620",
            root.join("tests/fixtures/genbank/BBa_F2620.gb"),
        ),
        ("pUC19", root.join("tests/fixtures/genbank/pUC19.gbk")),
        (
            "oracle_join",
            crate_dir.join("tests/fixtures/multispan/oracle_join.gb"),
        ),
    ]
}

type Signature = Vec<(i64, i64, i64)>;

/// A sorted multiset of per-feature span-signatures, order-independent.
fn sorted_signatures(mut sigs: Vec<Signature>) -> Vec<Signature> {
    for s in &mut sigs {
        s.sort();
    }
    sigs.sort();
    sigs
}

fn biopython_signatures(json: &Value) -> Vec<Signature> {
    let features = json["features"].as_array().expect("features array");
    let sigs = features
        .iter()
        .map(|feature| {
            feature["spans"]
                .as_array()
                .expect("spans array")
                .iter()
                .map(|span| {
                    let s = span.as_array().expect("span triple");
                    (
                        s[0].as_i64().expect("start"),
                        s[1].as_i64().expect("end"),
                        s[2].as_i64().expect("strand"),
                    )
                })
                .collect::<Signature>()
        })
        .collect();
    sorted_signatures(sigs)
}

fn importer_signatures(path: &PathBuf) -> Vec<Signature> {
    let (document, _report) = GenbankImporter::new("https://oracle.example.org/lab")
        .expect("namespace")
        .read_path(path)
        .unwrap_or_else(|err| panic!("import {}: {err}", path.display()));

    let sigs = document
        .sequence_features()
        .map(|sf| {
            sf.locations(&document)
                .filter_map(|loc| match loc {
                    LocationRef::Range(r) => {
                        let strand = match r.location.orientation.as_ref().map(|i| i.as_str()) {
                            Some("http://sbols.org/v3#reverseComplement") => -1,
                            _ => 1,
                        };
                        Some((r.start.expect("start"), r.end.expect("end"), strand))
                    }
                    _ => None,
                })
                .collect::<Signature>()
        })
        .collect();
    sorted_signatures(sigs)
}

#[test]
fn feature_tables_agree_with_biopython() {
    let expected_dir = manifest_dir().join("tests/oracle/expected");
    let mut checked = 0usize;
    let mut skipped = Vec::new();

    for (name, path) in fixtures() {
        let json_path = expected_dir.join(format!("{name}.json"));
        let Ok(text) = std::fs::read_to_string(&json_path) else {
            skipped.push(name);
            continue;
        };
        let json: Value =
            serde_json::from_str(&text).unwrap_or_else(|err| panic!("parse oracle {name}: {err}"));

        let expected = biopython_signatures(&json);
        let actual = importer_signatures(&path);

        assert_eq!(
            actual, expected,
            "{name}: sbol-genbank feature locations disagree with the BioPython \
             oracle.\n  sbol-genbank: {actual:?}\n  biopython:    {expected:?}"
        );
        checked += 1;
    }

    if checked == 0 {
        eprintln!(
            "biopython_oracle: skipped {} fixture(s) — no committed oracle JSON \
             (run crates/sbol-genbank/tests/oracle/regenerate.sh)",
            skipped.len()
        );
    } else if !skipped.is_empty() {
        eprintln!(
            "biopython_oracle: checked {checked}, skipped {} (missing JSON: {skipped:?})",
            skipped.len()
        );
    }
}
