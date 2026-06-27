//! `sbol3-128xx` — Attachment size/hash/hashAlgorithm and format checks.

use super::{PositiveCase, RuleCase};
use sbol::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "Attachment negative size",
            rule: "sbol3-12804",
            severity: Error,
            body: r#":attachment a sbol:Attachment;
    sbol:displayId "attachment";
    sbol:hasNamespace <https://example.org>;
    sbol:size "-1";
    sbol:source <https://example.org/file.txt> .
"#,
        },
        RuleCase {
            name: "Attachment hash is not hexadecimal",
            rule: "sbol3-12805",
            severity: Error,
            body: r#":attachment a sbol:Attachment;
    sbol:displayId "attachment";
    sbol:hash "not-hex";
    sbol:hashAlgorithm "sha3-256";
    sbol:hasNamespace <https://example.org>;
    sbol:source <https://example.org/file.txt> .
"#,
        },
        RuleCase {
            name: "Attachment hashAlgorithm is not a registry token",
            rule: "sbol3-12806",
            severity: Warning,
            body: r#":attachment a sbol:Attachment;
    sbol:displayId "attachment";
    sbol:hashAlgorithm "!bad";
    sbol:hasNamespace <https://example.org>;
    sbol:source <https://example.org/file.txt> .
"#,
        },
        RuleCase {
            name: "Attachment hashAlgorithm is not recommended",
            rule: "sbol3-12807",
            severity: Warning,
            body: r#":attachment a sbol:Attachment;
    sbol:displayId "attachment";
    sbol:hashAlgorithm "sha256";
    sbol:hasNamespace <https://example.org>;
    sbol:source <https://example.org/file.txt> .
"#,
        },
        RuleCase {
            name: "Attachment format is known non-EDAM term",
            rule: "sbol3-12803",
            severity: Warning,
            body: r#":attachment a sbol:Attachment;
    sbol:displayId "attachment";
    sbol:format SBO:0000251;
    sbol:hasNamespace <https://example.org>;
    sbol:source <https://example.org/file.txt> .
"#,
        },
        RuleCase {
            name: "Attachment hash without hashAlgorithm",
            rule: "sbol3-12808",
            severity: Error,
            body: r#":attachment a sbol:Attachment;
    sbol:displayId "attachment";
    sbol:hash "abcdef";
    sbol:hasNamespace <https://example.org>;
    sbol:source <https://example.org/file.txt> .
"#,
        },
        RuleCase {
            name: "Model language uses http variant of Table 15 URI",
            rule: "sbol3-12503",
            severity: Error,
            body: r#":model a sbol:Model;
    sbol:displayId "model";
    sbol:framework <https://identifiers.org/SBO:0000062>;
    sbol:hasNamespace <https://example.org>;
    sbol:language <http://identifiers.org/edam:format_2585>;
    sbol:source <https://example.org/sbml.xml> .
"#,
        },
        RuleCase {
            name: "Model language is known non-EDAM term",
            rule: "sbol3-12504",
            severity: Warning,
            body: r#":model a sbol:Model;
    sbol:displayId "model";
    sbol:framework <https://identifiers.org/SBO:0000062>;
    sbol:hasNamespace <https://example.org>;
    sbol:language SBO:0000176;
    sbol:source <https://example.org/sbml.xml> .
"#,
        },
        RuleCase {
            name: "Model framework uses http variant of Table 16 URI",
            rule: "sbol3-12506",
            severity: Error,
            body: r#":model a sbol:Model;
    sbol:displayId "model";
    sbol:framework <http://identifiers.org/SBO:0000062>;
    sbol:hasNamespace <https://example.org>;
    sbol:language <https://identifiers.org/edam:format_2585>;
    sbol:source <https://example.org/sbml.xml> .
"#,
        },
        RuleCase {
            name: "Model framework is a known SBO term outside the modelling-framework branch",
            rule: "sbol3-12507",
            severity: Warning,
            body: r#":model a sbol:Model;
    sbol:displayId "model";
    sbol:framework SBO:0000251;
    sbol:hasNamespace <https://example.org>;
    sbol:language <https://identifiers.org/edam:format_2585>;
    sbol:source <https://example.org/sbml.xml> .
"#,
        },
        RuleCase {
            name: "Implementation wasDerivedFrom non-Component in-doc target",
            rule: "sbol3-12301",
            severity: Error,
            body: r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
:implementation a sbol:Implementation;
    sbol:displayId "implementation";
    sbol:hasNamespace <https://example.org>;
    prov:wasDerivedFrom :sequence .
"#,
        },
        RuleCase {
            name: "Implementation wasDerivedFrom Components with conflicting types",
            rule: "sbol3-12302",
            severity: Error,
            body: r#":dna a sbol:Component;
    sbol:displayId "dna";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:protein a sbol:Component;
    sbol:displayId "protein";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000252 .
:implementation a sbol:Implementation;
    sbol:displayId "implementation";
    sbol:hasNamespace <https://example.org>;
    prov:wasDerivedFrom :dna, :protein .
"#,
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "Model language refers to an EDAM ontology term",
            rule: "sbol3-12504",
            body: r#":model a sbol:Model;
    sbol:displayId "model";
    sbol:framework <https://identifiers.org/SBO:0000062>;
    sbol:hasNamespace <https://example.org>;
    sbol:language <https://identifiers.org/edam:format_2585>;
    sbol:source <https://example.org/sbml.xml> .
"#,
        },
        PositiveCase {
            name: "Model framework is in the SBO modeling framework branch",
            rule: "sbol3-12507",
            body: r#":model a sbol:Model;
    sbol:displayId "model";
    sbol:framework SBO:0000062;
    sbol:hasNamespace <https://example.org>;
    sbol:language <https://identifiers.org/edam:format_2585>;
    sbol:source <https://example.org/sbml.xml> .
"#,
        },
        PositiveCase {
            name: "Attachment format refers to an EDAM ontology term",
            rule: "sbol3-12803",
            body: r#":attachment a sbol:Attachment;
    sbol:displayId "attachment";
    sbol:format EDAM:format_1207;
    sbol:hasNamespace <https://example.org>;
    sbol:source <https://example.org/file.txt> .
"#,
        },
        PositiveCase {
            name: "Attachment hashAlgorithm is sha3-256",
            rule: "sbol3-12807",
            body: r#":attachment a sbol:Attachment;
    sbol:displayId "attachment";
    sbol:hash "abcdef";
    sbol:hashAlgorithm "sha3-256";
    sbol:hasNamespace <https://example.org>;
    sbol:source <https://example.org/file.txt> .
"#,
        },
        PositiveCase {
            name: "Attachment hash with hashAlgorithm",
            rule: "sbol3-12808",
            body: r#":attachment a sbol:Attachment;
    sbol:displayId "attachment";
    sbol:hash "abcdef";
    sbol:hashAlgorithm "sha3-256";
    sbol:hasNamespace <https://example.org>;
    sbol:source <https://example.org/file.txt> .
"#,
        },
    ]
}
