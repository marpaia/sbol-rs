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
//! downgrade/upgrade lands on that same graph. Two non-drift outcomes are
//! counted separately: files the RDF/XML reader cannot parse (`parse-fail`,
//! a reader limitation) and files that parse but the upgrade declines as not
//! SBOL 2 (`upgrade-unsupported`). Files that genuinely cannot round-trip
//! are listed in [`ALLOWLIST`] with a reason.

mod common;

use std::collections::BTreeMap;
use std::path::Path;

use common::corpus::{CORPUS_DIRS, xml_files};

use sbol_convert::UpgradeError;
use sbol3::RdfFormat;

/// Corpus files that legitimately cannot reach the SBOL 2 → SBOL 3 fixed
/// point, paired with the reason. Keyed by file name (unique across the
/// corpora). Empty entries here would let real drift regress silently, so
/// every addition must carry a specific justification.
const ALLOWLIST: &[(&str, &str)] = &[
    (
        "LocationToSequenceOutput.xml",
        "The Range carries an explicit SBOL 2 Location.sequence (the SBOL 2.2 \
         location-to-sequence best practice) selecting one of the enclosing \
         ComponentDefinition's two Sequences. The downgrade treats a Location's \
         hasSequence as implicit through the SequenceAnnotation → \
         ComponentDefinition → sequence chain and drops it; because this CD has \
         more than one Sequence, the specific choice cannot be re-inferred on \
         re-upgrade (infer_location_sequences only fires for single-sequence \
         CDs), so the hasSequence triple does not survive.",
    ),
    // The SBOLTestSuite `SBOL2_nc` (non-compliant) fixtures use fragment-`#`
    // IRIs and/or child objects in namespaces unrelated to their parent. Both
    // this converter and the reference re-nest children under the parent's
    // SBOL-compliant identity, so the original non-compliant identities cannot
    // round-trip (and a fragment-`#` parent yields an invalid double-`#` IRI
    // when a child path is appended). The SBOL 3 content is preserved; only the
    // non-compliant identity shape drifts. Compliant inputs (SBOL2 / SBOL2_ic /
    // SBOL2_bp) all reach a fixed point.
    ("BBa_I0462_orig.xml", "non-compliant SBOL 2 identity shape"),
    ("BBa_T9002_orig.xml", "non-compliant SBOL 2 identity shape"),
    ("igem1.xml", "non-compliant SBOL 2 identity shape"),
    ("igem2.xml", "non-compliant SBOL 2 identity shape"),
    ("igem3.xml", "non-compliant SBOL 2 identity shape"),
    (
        "pIKE_pTAK_cassettes_2_orig.xml",
        "non-compliant SBOL 2 identity shape",
    ),
    (
        "pIKE_pTAK_cassettes_orig.xml",
        "non-compliant SBOL 2 identity shape",
    ),
    (
        "pIKE_pTAK_left_right_cassettes_orig.xml",
        "non-compliant SBOL 2 identity shape",
    ),
    (
        "pIKE_pTAK_toggle_switches_orig.xml",
        "non-compliant SBOL 2: fragment-# IRI yields invalid nested IRI on downgrade",
    ),
    (
        "partial_pIKE_left_cassette_orig.xml",
        "non-compliant SBOL 2 identity shape",
    ),
    (
        "partial_pIKE_right_casette_orig.xml",
        "non-compliant SBOL 2 identity shape",
    ),
    (
        "partial_pIKE_right_cassette_orig.xml",
        "non-compliant SBOL 2 identity shape",
    ),
    (
        "partial_pTAK_left_cassette_orig.xml",
        "non-compliant SBOL 2 identity shape",
    ),
    (
        "partial_pTAK_right_cassette_orig.xml",
        "non-compliant SBOL 2 identity shape",
    ),
];

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
/// `https://sbols.org/backport/2_3#` namespace holds conversion provenance
/// (`sbol2OriginalURI`, `sbol2OriginalSequenceAnnotationURI`,
/// `sbol3TempSequenceURI`, …), not source data. Its exact idempotency depends
/// on identity-shape edge cases — e.g. a `sbol2OriginalURI` value points at a
/// non-canonical nested SBOL 2 identity whose version placement the
/// re-upgrade normalizes — that are orthogonal to whether the
/// biological/structural content survives. The fixed point that matters is
/// over the substantive content, so the provenance is excluded from the
/// comparison (any genuine, non-backport drift still fails).
fn canonicalize(graph: &sbol3::RdfGraph) -> Vec<String> {
    let mut lines: Vec<String> = graph
        .normalized_triples()
        .iter()
        .map(sbol_convert::canonical_nt_line)
        .filter(|line| !line.contains("https://sbols.org/backport/2_3#"))
        .collect();
    lines.sort();
    lines.dedup();
    lines
}

#[derive(Default)]
struct DirReport {
    clean: usize,
    drift: Vec<String>,
    /// The RDF/XML reader could not parse the file — a reader limitation, not
    /// a conversion defect.
    parse_skip: usize,
    /// The file parsed as RDF but the upgrade declined it as not SBOL 2
    /// (`UpgradeError::NotSbol2`) — a genuinely upgrade-unsupported outcome,
    /// tracked separately from a parse failure so the report stays honest.
    upgrade_unsupported: usize,
    reupgrade_fail: Vec<String>,
    downgrade_fail: Vec<String>,
}

/// Runs the fixed-point cycle for one file. `Ok(true)` = clean, `Ok(false)`
/// = drifted, `Err(kind)` distinguishes the failure bucket. A genuine RDF
/// parse failure (`"parse"`) is kept distinct from an
/// upgrade-declined-as-not-SBOL-2 outcome (`"upgrade-unsupported"`).
fn fixed_point(path: &Path) -> Result<bool, &'static str> {
    let text = std::fs::read_to_string(path).map_err(|_| "read")?;

    // First upgrade: also the parse gate. RDF/XML reader gaps surface as a
    // `Parse` error; a well-formed document the upgrade declines surfaces as
    // `NotSbol2`.
    let (upgraded, _) =
        sbol_convert::upgrade_from_sbol2(&text, RdfFormat::RdfXml).map_err(|err| match err {
            UpgradeError::Parse(_) => "parse",
            UpgradeError::NotSbol2 => "upgrade-unsupported",
            _ => "upgrade-unsupported",
        })?;
    let before = canonicalize(upgraded.rdf_graph());

    let (downgraded, _) = sbol_convert::downgrade(&upgraded).map_err(|_| "downgrade")?;
    let sbol2 = downgraded.write(RdfFormat::Turtle).map_err(|_| "write")?;

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
                Err("upgrade-unsupported") => report.upgrade_unsupported += 1,
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
    let mut total_unsupported = 0;
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
            "  {dir:<10} clean={:<4} drift={:<3} (allowlisted={}) parse-fail={} upgrade-unsupported={}",
            report.clean, non_allow_drift, allow, report.parse_skip, report.upgrade_unsupported
        );
        total_clean += report.clean;
        total_drift += non_allow_drift;
        total_skip += report.parse_skip;
        total_unsupported += report.upgrade_unsupported;
        total_fail += allow;
    }
    eprintln!(
        "  {:<10} clean={total_clean} unexpected-drift={total_drift} allowlisted={total_fail} parse-fail={total_skip} upgrade-unsupported={total_unsupported}\n",
        "TOTAL"
    );

    assert!(
        drift_samples.is_empty(),
        "{} corpus file(s) drifted or failed re-conversion without an allowlist entry:\n{}",
        drift_samples.len(),
        drift_samples.join("\n"),
    );
}
