//! Cross-implementation record oracle: compare `sbol-fasta`'s parsed
//! records against BioPython's `Bio.SeqIO`.
//!
//! For every committed FASTA fixture, `tests/oracle/emit_records.py`
//! (run under the pinned Docker image in `tests/oracle/`) records each
//! record's identifier and uppercased sequence. This test reproduces
//! that table from `sbol-fasta`'s output — pairing each Component's name
//! with its Sequence's elements and uppercasing — and asserts they
//! agree, matching records as an unordered multiset.
//!
//! If the committed oracle JSON is absent, the test skips cleanly with a
//! count rather than failing. Regenerate with `tests/oracle/regenerate.sh`.

use std::collections::HashMap;
use std::path::PathBuf;

use sbol3::SbolIdentified;
use sbol_fasta::FastaImporter;
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

fn fixtures() -> Vec<(&'static str, PathBuf)> {
    let root = workspace_root();
    vec![
        ("pUC19", root.join("tests/fixtures/fasta/pUC19.fasta")),
        ("pBR322", root.join("tests/fixtures/fasta/pBR322.fasta")),
        (
            "GFP_protein",
            root.join("tests/fixtures/fasta/GFP_protein.fasta"),
        ),
        (
            "multi_protein",
            root.join("tests/fixtures/fasta/multi_protein.fasta"),
        ),
    ]
}

fn biopython_records(json: &Value) -> Vec<(String, String)> {
    let mut records: Vec<(String, String)> = json["records"]
        .as_array()
        .expect("records array")
        .iter()
        .map(|r| {
            (
                r["id"].as_str().expect("id").to_owned(),
                r["seq"].as_str().expect("seq").to_ascii_uppercase(),
            )
        })
        .collect();
    records.sort();
    records
}

fn importer_records(path: &PathBuf) -> Vec<(String, String)> {
    let (document, _report) = FastaImporter::new("https://oracle.example.org/lab")
        .expect("namespace")
        .read_path(path)
        .unwrap_or_else(|err| panic!("import {}: {err}", path.display()));

    let elements: HashMap<String, String> = document
        .sequences()
        .filter_map(|s| {
            let identity = s.identity.as_iri()?.as_str().to_owned();
            Some((identity, s.elements.clone().unwrap_or_default()))
        })
        .collect();

    let mut records: Vec<(String, String)> = document
        .components()
        .map(|component| {
            let id = component.name().expect("component name").to_owned();
            let sequence_iri = component
                .sequences
                .first()
                .and_then(|r| r.as_iri())
                .map(|i| i.as_str().to_owned())
                .expect("component references a sequence");
            let seq = elements
                .get(&sequence_iri)
                .expect("sequence resolves")
                .to_ascii_uppercase();
            (id, seq)
        })
        .collect();
    records.sort();
    records
}

#[test]
fn records_agree_with_biopython() {
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

        let expected = biopython_records(&json);
        let actual = importer_records(&path);

        assert_eq!(
            actual, expected,
            "{name}: sbol-fasta records disagree with the BioPython oracle"
        );
        checked += 1;
    }

    if checked == 0 {
        eprintln!(
            "biopython_oracle: skipped {} fixture(s) — no committed oracle JSON \
             (run crates/sbol-fasta/tests/oracle/regenerate.sh)",
            skipped.len()
        );
    } else if !skipped.is_empty() {
        eprintln!(
            "biopython_oracle: checked {checked}, skipped {} (missing JSON: {skipped:?})",
            skipped.len()
        );
    }
}
