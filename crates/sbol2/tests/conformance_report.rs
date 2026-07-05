//! Freshness gate for `docs/sbol2-conformance.md`. Fails CI if the
//! committed report drifts from the rendered output. Run
//! `cargo run -p sbol2 --bin generate-sbol2-conformance-report` to refresh.

use sbol2::render_sbol2_conformance_report;
use sbol2::validation::validation_rule_statuses;

#[test]
fn sbol2_conformance_report_matches_committed_file() {
    let rendered = render_sbol2_conformance_report(validation_rule_statuses());
    let committed = include_str!("../../../docs/sbol2-conformance.md");
    assert_eq!(
        committed, rendered,
        "docs/sbol2-conformance.md is stale. Run `cargo run -p sbol2 --bin generate-sbol2-conformance-report` to refresh."
    );
}
