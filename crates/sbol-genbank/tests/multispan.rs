//! Hand-verified coverage of GenBank multi-span locations
//! (`join` / `order` / `complement` and their nestings) and the lossy
//! location shapes (`gap` / external reference / between-sites).
//!
//! Expected coordinates and orientations are hand-derived from the
//! INSDC Feature Table spec and the importer's documented coordinate
//! conversion (gb-io's 0-based half-open `[start, end)` becomes SBOL
//! 1-based closed: `sbol_start = gb_start + 1`, `sbol_end = gb_end`),
//! not regenerated from the importer. A GenBank span `a..b` therefore
//! yields an SBOL Range with `start = a`, `end = b`.

use std::path::PathBuf;

use sbol3::{LocationRef, RdfFormat, Severity};
use sbol_genbank::{GenbankImporter, ImportWarning};

const INLINE: &str = "http://sbols.org/v3#inline";
const REVERSE_COMPLEMENT: &str = "http://sbols.org/v3#reverseComplement";

fn fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/multispan");
    p.push(name);
    p
}

fn import(file: &str) -> (sbol3::Document, sbol_genbank::ImportReport) {
    GenbankImporter::new("https://example.org/lab")
        .expect("namespace")
        .read_path(fixture(file))
        .unwrap_or_else(|err| panic!("import {file}: {err}"))
}

fn assert_no_validation_errors(document: &sbol3::Document, name: &str) {
    let errors: Vec<_> = document
        .validate()
        .issues()
        .iter()
        .filter(|i| matches!(i.severity, Severity::Error))
        .cloned()
        .collect();
    assert!(errors.is_empty(), "{name}: validation errors: {errors:?}");
}

/// Returns a feature's Ranges as ordered `(start, end, orientation)`
/// tuples, preserving the order the importer emitted them (which mirrors
/// the textual span order in the GenBank location).
fn ranges_of(document: &sbol3::Document, feature_display_id: &str) -> Vec<(i64, i64, String)> {
    let identity = format!("https://example.org/lab/MULTISPAN/{feature_display_id}");
    let sf = document
        .sequence_features()
        .find(|sf| sf.identity.as_iri().map(|i| i.as_str()) == Some(identity.as_str()))
        .unwrap_or_else(|| panic!("feature {feature_display_id} missing"));
    sf.locations(document)
        .filter_map(|loc| match loc {
            LocationRef::Range(r) => Some((
                r.start.expect("range start"),
                r.end.expect("range end"),
                r.location
                    .orientation
                    .as_ref()
                    .expect("orientation")
                    .as_str()
                    .to_owned(),
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn multi_exon_join_yields_ordered_inline_ranges() {
    let (document, _report) = import("multispan.gb");
    assert_no_validation_errors(&document, "multispan");
    // join(1..6,16..24): two inline spans, first span first.
    assert_eq!(
        ranges_of(&document, "exon_join"),
        vec![(1, 6, INLINE.to_owned()), (16, 24, INLINE.to_owned())]
    );
}

#[test]
fn complement_flips_orientation_to_reverse_complement() {
    let (document, _report) = import("multispan.gb");
    // complement(1..6): a single reverse-complement span, coords unchanged.
    assert_eq!(
        ranges_of(&document, "rev_gene"),
        vec![(1, 6, REVERSE_COMPLEMENT.to_owned())]
    );
}

#[test]
fn complement_of_join_marks_every_span_reverse_complement() {
    let (document, _report) = import("multispan.gb");
    // complement(join(1..6,16..24)): both spans reverse-complement,
    // textual order preserved.
    assert_eq!(
        ranges_of(&document, "rev_join"),
        vec![
            (1, 6, REVERSE_COMPLEMENT.to_owned()),
            (16, 24, REVERSE_COMPLEMENT.to_owned())
        ]
    );
}

#[test]
fn join_of_complement_mixes_orientations_per_span() {
    let (document, _report) = import("multispan.gb");
    // join(complement(1..6),16..24): first span reverse-complement,
    // second inline.
    assert_eq!(
        ranges_of(&document, "mixed_join"),
        vec![
            (1, 6, REVERSE_COMPLEMENT.to_owned()),
            (16, 24, INLINE.to_owned())
        ]
    );
}

#[test]
fn order_lowers_like_join_to_inline_ranges() {
    let (document, _report) = import("multispan.gb");
    // order(1..6,16..24): the importer represents order() with the same
    // multi-Range shape as join(); the join/order distinction is not
    // carried into SBOL 3.
    assert_eq!(
        ranges_of(&document, "ordered"),
        vec![(1, 6, INLINE.to_owned()), (16, 24, INLINE.to_owned())]
    );
}

#[test]
fn between_site_is_dropped_with_a_lossy_warning() {
    let (document, report) = import("multispan.gb");
    // A `7^8` between-residues site has no SBOL 3 Range equivalent; the
    // feature is skipped and a LossyLocation warning is recorded.
    let identities: Vec<&str> = document
        .sequence_features()
        .filter_map(|sf| sf.identity.as_iri().map(|i| i.as_str()))
        .collect();
    assert!(
        !identities.contains(&"https://example.org/lab/MULTISPAN/between_site"),
        "the between-site feature must be dropped"
    );
    assert!(
        report.warnings.iter().any(|w| matches!(
            w,
            ImportWarning::LossyLocation { feature, reason }
                if feature == "between_site" && reason.contains("Between")
        )),
        "expected a LossyLocation warning for the between-site: {:?}",
        report.warnings
    );
    // The five well-formed features still import.
    assert_eq!(report.features, 5);
}

#[test]
fn gap_and_external_locations_are_lossy_but_do_not_abort_the_record() {
    let (document, report) = import("lossy_locations.gb");
    assert_no_validation_errors(&document, "lossy_locations");
    // gap(20) and an external cross-reference both lower to nothing;
    // each records a LossyLocation warning naming the offending shape.
    let reasons: Vec<&str> = report
        .warnings
        .iter()
        .filter_map(|w| match w {
            ImportWarning::LossyLocation { feature, reason } => {
                Some((feature.as_str(), reason.as_str()))
            }
            _ => None,
        })
        .map(|(_, reason)| reason)
        .collect();
    assert!(
        reasons.iter().any(|r| r.contains("Gap")),
        "expected a Gap lossy warning: {reasons:?}"
    );
    assert!(
        reasons.iter().any(|r| r.contains("External")),
        "expected an External lossy warning: {reasons:?}"
    );
    // The well-formed control CDS still imports as a single inline span.
    assert_eq!(report.features, 1);
    let target = "https://example.org/lab/LOSSY/ok_control";
    let ok = document
        .sequence_features()
        .find(|sf| sf.identity.as_iri().map(|i| i.as_str()) == Some(target))
        .expect("control feature missing");
    let ranges: Vec<_> = ok
        .locations(&document)
        .filter_map(|loc| match loc {
            LocationRef::Range(r) => Some((r.start.unwrap(), r.end.unwrap())),
            _ => None,
        })
        .collect();
    assert_eq!(ranges, vec![(1, 6)]);
}

#[test]
fn multi_span_document_round_trips_through_turtle() {
    let (document, _report) = import("multispan.gb");
    let turtle = document.write(RdfFormat::Turtle).expect("write turtle");
    let parsed = sbol3::Document::read_turtle(&turtle).expect("re-read turtle");
    // Every Range survives the round-trip: 2 + 1 + 2 + 2 + 2 = 9.
    assert_eq!(parsed.ranges().count(), 9);
    assert_eq!(parsed.ranges().count(), document.ranges().count());
    assert_eq!(
        parsed.sequence_features().count(),
        document.sequence_features().count()
    );
}
