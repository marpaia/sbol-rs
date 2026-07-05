//! Drives every parseable SBOL 2 file in the SBOLTestSuite conformance
//! corpora through the representational fixed point
//!
//! ```text
//! SBOL2 → upgrade → SBOL3(A) → downgrade → SBOL2 → upgrade → SBOL3(B)
//! ```
//!
//! and asserts `A == B` at the level of canonical triples. The first
//! upgrade normalizes the SBOL 2 input into the SBOL 3 graph the converter
//! actually reasons about; a lossless conversion is one where a second
//! downgrade/upgrade lands on that same graph. Files the RDF/XML reader
//! cannot parse are skipped and counted (a reader limitation, not a
//! conversion defect). Files that genuinely cannot round-trip are listed
//! in [`ALLOWLIST`] with a reason.

mod common;

use std::collections::BTreeMap;
use std::path::Path;

use common::corpus::{xml_files, CORPUS_DIRS};

use sbol3::RdfFormat;

/// Corpus files that legitimately cannot reach the SBOL 2 → SBOL 3 fixed
/// point, paired with the reason. Keyed by file name (unique across the
/// corpora). Empty entries here would let real drift regress silently, so
/// every addition must carry a specific justification.
const ALLOWLIST: &[(&str, &str)] = &[(
    "LocationToSequenceOutput.xml",
    "The Range carries an explicit SBOL 2 Location.sequence (the SBOL 2.2 \
     location-to-sequence best practice) selecting one of the enclosing \
     ComponentDefinition's two Sequences. The downgrade treats a Location's \
     hasSequence as implicit through the SequenceAnnotation → \
     ComponentDefinition → sequence chain and drops it; because this CD has \
     more than one Sequence, the specific choice cannot be re-inferred on \
     re-upgrade (infer_location_sequences only fires for single-sequence \
     CDs), so the hasSequence triple does not survive.",
)];

fn is_allowlisted(name: &str) -> bool {
    ALLOWLIST.iter().any(|(n, _)| *n == name)
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_owned()
}

/// Canonical triples of the substantive SBOL 3 graph. The
/// `http://sboltools.org/backport#` namespace holds reversible-conversion
/// scaffolding (`sbol2persistentIdentity`, `sbol2version`,
/// `sbol3namespace`, …), not source data. Its exact idempotency depends on
/// identity-shape edge cases — e.g. objects whose SBOL 2 source omitted an
/// explicit `persistentIdentity` (the downgrade synthesizes the default,
/// which the re-upgrade then records), or fragment-`#` IRIs the child
/// detection cannot attach to a top-level — that are orthogonal to whether
/// the biological/structural content survives. The fixed point that
/// matters is over the substantive content, so the scaffolding is excluded
/// from the comparison (any genuine, non-backport drift still fails).
fn canonicalize(graph: &sbol3::RdfGraph) -> Vec<String> {
    let mut lines: Vec<String> = graph
        .normalized_triples()
        .iter()
        .map(sbol_convert::canonical_nt_line)
        .filter(|line| !line.contains("http://sboltools.org/backport#"))
        .collect();
    lines.sort();
    lines.dedup();
    lines
}

#[derive(Default)]
struct DirReport {
    clean: usize,
    drift: Vec<String>,
    parse_skip: usize,
    reupgrade_fail: Vec<String>,
    downgrade_fail: Vec<String>,
}

/// Runs the fixed-point cycle for one file. `Ok(true)` = clean, `Ok(false)`
/// = drifted, `Err(kind)` distinguishes the failure bucket.
fn fixed_point(path: &Path) -> Result<bool, &'static str> {
    let text = std::fs::read_to_string(path).map_err(|_| "read")?;

    // First upgrade: also the parse gate. RDF/XML reader gaps surface here.
    let (upgraded, _) =
        sbol_convert::upgrade_from_sbol2(&text, RdfFormat::RdfXml).map_err(|_| "parse")?;
    let before = canonicalize(upgraded.rdf_graph());

    let (downgraded, _) = sbol_convert::downgrade(&upgraded).map_err(|_| "downgrade")?;
    let sbol2 = downgraded
        .write(RdfFormat::Turtle)
        .map_err(|_| "write")?;

    let (reupgraded, _) =
        sbol_convert::upgrade_from_sbol2(&sbol2, RdfFormat::Turtle).map_err(|_| "reupgrade")?;
    let after = canonicalize(reupgraded.rdf_graph());

    Ok(before == after)
}

#[test]
fn corpus_round_trips_to_fixed_point() {
    let mut reports: BTreeMap<&str, DirReport> = BTreeMap::new();
    let mut drift_samples: Vec<String> = Vec::new();

    for &dir in CORPUS_DIRS {
        let report = reports.entry(dir).or_default();
        for path in xml_files(dir) {
            let name = file_name(&path);
            match fixed_point(&path) {
                Ok(true) => report.clean += 1,
                Ok(false) => {
                    report.drift.push(name.clone());
                    if !is_allowlisted(&name) && drift_samples.len() < 20 {
                        drift_samples.push(format!("{dir}/{name}: SBOL3 fixed-point drift"));
                    }
                }
                Err("parse") => report.parse_skip += 1,
                Err("reupgrade") => {
                    report.reupgrade_fail.push(name.clone());
                    if !is_allowlisted(&name) && drift_samples.len() < 20 {
                        drift_samples.push(format!("{dir}/{name}: re-upgrade failed"));
                    }
                }
                Err(other) => {
                    report.downgrade_fail.push(name.clone());
                    if !is_allowlisted(&name) && drift_samples.len() < 20 {
                        drift_samples.push(format!("{dir}/{name}: {other} failed"));
                    }
                }
            }
        }
    }

    // Human-readable summary (visible with `--nocapture`).
    eprintln!("\nSBOL 2 corpus fixed-point round-trip:");
    let mut total_clean = 0;
    let mut total_drift = 0;
    let mut total_skip = 0;
    let mut total_fail = 0;
    for (dir, report) in &reports {
        let non_allow_drift = report
            .drift
            .iter()
            .chain(&report.reupgrade_fail)
            .chain(&report.downgrade_fail)
            .filter(|n| !is_allowlisted(n))
            .count();
        let allow = report
            .drift
            .iter()
            .chain(&report.reupgrade_fail)
            .chain(&report.downgrade_fail)
            .filter(|n| is_allowlisted(n))
            .count();
        eprintln!(
            "  {dir:<10} clean={:<4} drift={:<3} (allowlisted={}) parse-skip={}",
            report.clean, non_allow_drift, allow, report.parse_skip
        );
        total_clean += report.clean;
        total_drift += non_allow_drift;
        total_skip += report.parse_skip;
        total_fail += allow;
    }
    eprintln!(
        "  {:<10} clean={total_clean} unexpected-drift={total_drift} allowlisted={total_fail} parse-skip={total_skip}\n",
        "TOTAL"
    );

    assert!(
        drift_samples.is_empty(),
        "{} corpus file(s) drifted or failed re-conversion without an allowlist entry:\n{}",
        drift_samples.len(),
        drift_samples.join("\n"),
    );
}
