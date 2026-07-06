//! Self-snapshot conformance gate for `sbol-fasta`.
//!
//! Re-imports every committed `.fasta` / `.fa` / `.fna` / `.faa`
//! file under `tests/fixtures/fasta/` and diffs the deterministically
//! sorted N-Triples output against the committed snapshot at
//! `tests/fixtures/fasta/expected/{name}.nt`. Any drift in the
//! converter's output fails the gate.
//!
//! Pure Rust — no Docker, no Python, no Node. Both sides of the
//! comparison come from `sbol-fasta`, so this gate is about stability
//! and determinism, not cross-implementation agreement.
//!
//! Refresh snapshots after an intentional importer change with:
//!
//! ```sh
//! cargo run -p sbol-fasta --bin regenerate-fasta-import-snapshots
//! ```

use std::path::PathBuf;

use sbol_fasta::FastaImporter;
use sbol3::{Resource, Term, Triple};

fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path
}

fn fixtures_dir() -> PathBuf {
    workspace_root().join("tests/fixtures/fasta")
}

fn expected_dir() -> PathBuf {
    workspace_root().join("tests/fixtures/fasta/expected")
}

fn fixture_entries() -> Vec<(String, PathBuf)> {
    let mut entries: Vec<(String, PathBuf)> = std::fs::read_dir(fixtures_dir())
        .expect("read fixtures dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            matches!(
                p.extension().and_then(|s| s.to_str()),
                Some("fasta") | Some("fa") | Some("fna") | Some("faa")
            )
        })
        .filter_map(|p| {
            p.file_stem()
                .map(|s| (s.to_string_lossy().into_owned(), p.clone()))
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

fn parse_snapshot(text: &str) -> Vec<String> {
    text.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

fn run_import(name: &str, path: &PathBuf) -> Vec<String> {
    let namespace = format!("https://sbol-rs.example.org/fasta/{name}");
    let (document, _report) = FastaImporter::new(&namespace)
        .expect("namespace")
        .read_path(path)
        .unwrap_or_else(|err| panic!("import {name}: {err}"));
    let mut lines: Vec<String> = document
        .rdf_graph()
        .normalized_triples()
        .iter()
        .map(format_triple)
        .collect();
    lines.sort();
    lines.dedup();
    lines
}

fn format_triple(triple: &Triple) -> String {
    const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
    let subject = match &triple.subject {
        Resource::Iri(iri) => format!("<{}>", iri.as_str()),
        Resource::BlankNode(b) => format!("_:{}", b.as_str()),
        _ => format!("{:?}", triple.subject),
    };
    let predicate = format!("<{}>", triple.predicate.as_str());
    let object = match &triple.object {
        Term::Resource(Resource::Iri(iri)) => format!("<{}>", iri.as_str()),
        Term::Resource(Resource::BlankNode(b)) => format!("_:{}", b.as_str()),
        Term::Literal(literal) => {
            let escaped = escape_nt_string(literal.value());
            if let Some(lang) = literal.language() {
                format!("\"{escaped}\"@{lang}")
            } else if literal.datatype().as_str() == XSD_STRING {
                format!("\"{escaped}\"")
            } else {
                format!("\"{escaped}\"^^<{}>", literal.datatype().as_str())
            }
        }
        _ => format!("{:?}", triple.object),
    };
    format!("{subject} {predicate} {object} .")
}

fn escape_nt_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

#[test]
fn self_snapshot_diff() {
    let mut drift = Vec::new();
    for (name, path) in fixture_entries() {
        let snapshot_path = expected_dir().join(format!("{name}.nt"));
        let snapshot = std::fs::read_to_string(&snapshot_path).unwrap_or_else(|err| {
            panic!(
                "snapshot missing for {name} ({}); run \
                 `cargo run -p sbol-fasta --bin regenerate-fasta-import-snapshots`: {err}",
                snapshot_path.display()
            )
        });
        let actual = run_import(&name, &path);
        let expected = parse_snapshot(&snapshot);
        if actual != expected {
            drift.push((name, expected, actual));
        }
    }

    if !drift.is_empty() {
        let mut report = String::new();
        for (name, expected, actual) in &drift {
            report.push_str(&format!("\n=== {name} self-snapshot drift ===\n"));
            let added: Vec<&String> = actual.iter().filter(|t| !expected.contains(t)).collect();
            let removed: Vec<&String> = expected.iter().filter(|t| !actual.contains(t)).collect();
            for t in &added {
                report.push_str(&format!("  +  {t}\n"));
            }
            for t in &removed {
                report.push_str(&format!("  -  {t}\n"));
            }
            if added.is_empty() && removed.is_empty() {
                report.push_str("  (ordering only)\n");
            }
        }
        report.push_str(
            "\nTo accept these changes, run:\n  \
             cargo run -p sbol-fasta --bin regenerate-fasta-import-snapshots\n",
        );
        panic!("fasta import drift detected:{report}");
    }
}
