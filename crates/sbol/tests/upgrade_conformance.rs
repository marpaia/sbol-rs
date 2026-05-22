//! Self-snapshot conformance harness for the SBOL 2 → SBOL 3 upgrade.
//!
//! For each fixture in `tests/fixtures/sbol2/real/`, re-runs our upgrade
//! and diffs the sorted N-Triples output against the committed snapshot
//! at `tests/fixtures/sbol2/real/expected/<name>.nt`. Any unintended
//! change in converter output surfaces here.
//!
//! Cross-implementation correctness is covered by:
//!
//! - The round-trip smoke test
//!   ([`crates/sbol/src/bin/generate-round-trip-report.rs`]) and the
//!   round-trip integration tests in
//!   [`crates/sbol/tests/downgrade.rs`], which prove every triple
//!   produced by the upgrade survives a downgrade and re-upgrade
//!   without loss.
//! - The libSBOLj3 and pySBOL3 cross-impl harnesses
//!   ([`crates/sbol/tests/cross_impl.rs`] and
//!   [`crates/sbol/tests/cross_impl_pysbol3.rs`]) for the SBOL 3
//!   surface.
//!
//! See [`docs/sbol2-upgrade-conformance.md`] for the full design.

use std::path::PathBuf;

use sbol::{Document, RdfFormat, Triple};

fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path
}

fn fixtures_dir() -> PathBuf {
    workspace_root().join("tests/fixtures/sbol2/real")
}

fn expected_dir() -> PathBuf {
    workspace_root().join("tests/fixtures/sbol2/real/expected")
}

fn format_triple(triple: &Triple) -> String {
    sbol::upgrade::canonical_nt_line(triple)
}

fn parse_snapshot(text: &str) -> Vec<String> {
    text.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

fn fixture_names() -> Vec<String> {
    let dir = fixtures_dir();
    let expected = expected_dir();
    let mut paths: Vec<PathBuf> = Vec::new();
    walk_fixtures(&dir, &expected, &mut paths).expect("walk fixtures dir");
    let mut names: Vec<String> = paths
        .into_iter()
        .map(|p| {
            let relative = p.strip_prefix(&dir).unwrap_or(&p).with_extension("");
            relative
                .components()
                .filter_map(|c| match c {
                    std::path::Component::Normal(s) => s.to_str(),
                    _ => None,
                })
                .collect::<Vec<&str>>()
                .join("/")
        })
        .collect();
    names.sort();
    names
}

fn walk_fixtures(
    dir: &std::path::Path,
    expected: &std::path::Path,
    out: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path == *expected {
            continue;
        }
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            walk_fixtures(&path, expected, out)?;
        } else if file_type.is_file() && path.extension().and_then(|s| s.to_str()) == Some("xml") {
            out.push(path);
        }
    }
    Ok(())
}

fn input_path_for(name: &str) -> PathBuf {
    let mut path = fixtures_dir();
    for segment in name.split('/') {
        path.push(segment);
    }
    path.set_extension("xml");
    path
}

fn run_upgrade(name: &str) -> Vec<String> {
    let input_path = input_path_for(name);
    let input = std::fs::read_to_string(&input_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", input_path.display()));
    let (document, _report) = Document::upgrade_from_sbol2(&input, RdfFormat::RdfXml)
        .unwrap_or_else(|err| panic!("upgrade {name}: {err}"));
    let mut actual: Vec<String> = document
        .rdf_graph()
        .normalized_triples()
        .iter()
        .map(format_triple)
        .collect();
    actual.sort();
    actual.dedup();
    actual
}

fn snapshot_path_for(base: &std::path::Path, name: &str, extension: &str) -> PathBuf {
    let mut path = base.to_path_buf();
    for segment in name.split('/') {
        path.push(segment);
    }
    path.set_extension(extension);
    path
}

#[test]
fn self_snapshot_diff() {
    let mut drift = Vec::new();
    for name in fixture_names() {
        let snapshot_path = snapshot_path_for(&expected_dir(), &name, "nt");
        let snapshot = std::fs::read_to_string(&snapshot_path).unwrap_or_else(|err| {
            panic!(
                "self-snapshot missing for {name} ({}); run \
                 `cargo run -p sbol --bin regenerate-sbol2-upgrade-snapshots`: {err}",
                snapshot_path.display()
            )
        });
        let actual = run_upgrade(&name);
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
             cargo run -p sbol --bin regenerate-sbol2-upgrade-snapshots\n",
        );
        panic!("self-snapshot drift detected:{report}");
    }
}
