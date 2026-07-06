//! Sequence semantic rules: elements/encoding consistency (10405) and the
//! Table 1 encoding recommendation (10407).

use super::common::{PositiveCase, RuleCase};
use sbol2::validation::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "nucleic Sequence elements contain non-nucleic characters",
            rule: "sbol2-10405",
            severity: Error,
            body: r#"<http://ex/s> a sbol:Sequence ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/s> ;
    sbol:elements "ACGTFZ" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
"#,
        },
        RuleCase {
            name: "Sequence encoding is not a Table 1 URI",
            rule: "sbol2-10407",
            severity: Warning,
            body: r#"<http://ex/s> a sbol:Sequence ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/s> ;
    sbol:elements "ACGT" ;
    sbol:encoding <http://example.org/custom-encoding> .
"#,
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "nucleic Sequence elements match the nucleic alphabet",
            rule: "sbol2-10405",
            body: r#"<http://ex/s> a sbol:Sequence ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/s> ;
    sbol:elements "ACGTUN" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
"#,
        },
        PositiveCase {
            name: "Sequence encoding is the Table 1 nucleic-acid URI",
            rule: "sbol2-10407",
            body: r#"<http://ex/s> a sbol:Sequence ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/s> ;
    sbol:elements "ACGT" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
"#,
        },
    ]
}
