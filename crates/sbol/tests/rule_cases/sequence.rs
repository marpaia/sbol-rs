//! `sbol3-105xx` — Sequence encoding rules.

use super::{PositiveCase, RuleCase};
use sbol::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "Sequence elements without encoding",
            rule: "sbol3-10501",
            severity: Error,
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:hasNamespace <https://example.org> .
"#,
        },
        RuleCase {
            name: "Sequence encoding is known but not a Sequence encoding term",
            rule: "sbol3-10502",
            severity: Error,
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:data_0006;
    sbol:hasNamespace <https://example.org> .
"#,
        },
        RuleCase {
            name: "Sequence elements inconsistent with IUPAC nucleotide encoding",
            rule: "sbol3-10503",
            severity: Error,
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGZ";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
"#,
        },
        RuleCase {
            name: "Sequence encoding uses native EDAM alias instead of Table 1 URI",
            rule: "sbol3-10504",
            severity: Error,
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding <http://edamontology.org/format_1207>;
    sbol:hasNamespace <https://example.org> .
"#,
        },
        RuleCase {
            name: "Sequence encoding outside EDAM textual format branch",
            rule: "sbol3-10505",
            severity: Warning,
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:data_0006;
    sbol:hasNamespace <https://example.org> .
"#,
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "IUPAC DNA encoding accepts canonical bases",
            rule: "sbol3-10503",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ACGTACGT";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
"#,
        },
        PositiveCase {
            name: "IUPAC DNA encoding accepts ambiguity codes",
            rule: "sbol3-10503",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ACGTRYKMSWBDHVN";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
"#,
        },
        PositiveCase {
            name: "IUPAC RNA encoding accepts canonical bases",
            rule: "sbol3-10503",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ACGUACGU";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
"#,
        },
        PositiveCase {
            name: "IUPAC protein encoding accepts standard amino acids",
            rule: "sbol3-10503",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "MSTNRDH";
    sbol:encoding EDAM:format_1208;
    sbol:hasNamespace <https://example.org> .
"#,
        },
        PositiveCase {
            name: "Table 1 encoding URI not flagged as native EDAM alias",
            rule: "sbol3-10504",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
"#,
        },
        PositiveCase {
            name: "Sequence with no elements does not require encoding",
            rule: "sbol3-10501",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:hasNamespace <https://example.org> .
"#,
        },
        PositiveCase {
            name: "EDAM textual format branch encoding does not warn",
            rule: "sbol3-10505",
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
"#,
        },
    ]
}
