#![no_main]

//! Fuzzes parse → check → serialize → reparse → check. Asserts the
//! issue rule-id set is identical across the round-trip, catching
//! validator output that depends on map iteration order, a real
//! class of bug in HashMap-driven validators.

use std::collections::BTreeSet;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(document) = sbol3::Document::read_turtle(text) else {
        return;
    };

    let report_a = document.validate();
    let rules_a: BTreeSet<&str> = report_a.issues().iter().map(|issue| issue.rule).collect();

    let Ok(serialized) = document.write_turtle() else {
        return;
    };
    let Ok(document_b) = sbol3::Document::read_turtle(&serialized) else {
        return;
    };
    let report_b = document_b.validate();
    let rules_b: BTreeSet<&str> = report_b.issues().iter().map(|issue| issue.rule).collect();

    assert_eq!(
        rules_a, rules_b,
        "round-trip validation produced different rule-id sets"
    );
});
