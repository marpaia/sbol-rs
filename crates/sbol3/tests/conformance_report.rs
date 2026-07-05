//! Freshness gate for `docs/conformance.md`. Fails CI if the
//! committed report drifts from the rendered output. Run
//! `cargo run -p sbol --bin generate-conformance-report` to refresh.

use sbol3::{render_conformance_report, validation_rule_statuses};

#[test]
fn conformance_report_matches_committed_file() {
    let rendered = render_conformance_report(validation_rule_statuses());
    let committed = include_str!("../../../docs/conformance.md");
    assert_eq!(
        committed, rendered,
        "docs/conformance.md is stale. Run `cargo run -p sbol --bin generate-conformance-report` to refresh."
    );
}
