//! `sbol3-114xx`, `sbol3-115xx` — Range bounds and Cut position.

use super::{PositiveCase, RuleCase};
use sbol::Severity::Error;

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "Range start out of bounds",
            rule: "sbol3-11401",
            severity: Error,
            body: range_body("0", "2"),
        },
        RuleCase {
            name: "Range end out of bounds",
            rule: "sbol3-11402",
            severity: Error,
            body: range_body("1", "5"),
        },
        RuleCase {
            name: "Range end before start",
            rule: "sbol3-11403",
            severity: Error,
            body: range_body("3", "2"),
        },
        RuleCase {
            name: "Cut at out of bounds",
            rule: "sbol3-11501",
            severity: Error,
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:cut a sbol:Cut;
    sbol:at "-1";
    sbol:displayId "cut";
    sbol:hasSequence :sequence .
"#,
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "Range at sequence start is valid",
            rule: "sbol3-11401",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:range a sbol:Range;
    sbol:displayId "range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "1" .
"#,
        },
        PositiveCase {
            name: "Range at sequence end is valid",
            rule: "sbol3-11402",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:range a sbol:Range;
    sbol:displayId "range";
    sbol:end "4";
    sbol:hasSequence :sequence;
    sbol:start "3" .
"#,
        },
        PositiveCase {
            name: "Range start equals end (single base) is valid",
            rule: "sbol3-11403",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:range a sbol:Range;
    sbol:displayId "range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "2" .
"#,
        },
        PositiveCase {
            name: "Range covering the entire sequence is valid",
            rule: "sbol3-11402",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:range a sbol:Range;
    sbol:displayId "range";
    sbol:end "4";
    sbol:hasSequence :sequence;
    sbol:start "1" .
"#,
        },
        PositiveCase {
            name: "Range with explicit inline orientation",
            rule: "sbol3-11401",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:range a sbol:Range;
    sbol:displayId "range";
    sbol:end "3";
    sbol:hasSequence :sequence;
    sbol:orientation sbol:inline;
    sbol:start "1" .
"#,
        },
        PositiveCase {
            name: "Cut at sequence start (zero) is valid",
            rule: "sbol3-11501",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:cut a sbol:Cut;
    sbol:at "0";
    sbol:displayId "cut";
    sbol:hasSequence :sequence .
"#,
        },
        PositiveCase {
            name: "Cut at sequence end is valid",
            rule: "sbol3-11501",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:cut a sbol:Cut;
    sbol:at "4";
    sbol:displayId "cut";
    sbol:hasSequence :sequence .
"#,
        },
    ]
}

fn range_body(start: &'static str, end: &'static str) -> &'static str {
    match (start, end) {
        ("0", "2") => {
            r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:range a sbol:Range;
    sbol:displayId "range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "0" .
"#
        }
        ("1", "5") => {
            r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:range a sbol:Range;
    sbol:displayId "range";
    sbol:end "5";
    sbol:hasSequence :sequence;
    sbol:start "1" .
"#
        }
        ("3", "2") => {
            r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:range a sbol:Range;
    sbol:displayId "range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "3" .
"#
        }
        _ => unreachable!(),
    }
}
