//! Location positional bound rule (11104: Range end >= start).

use super::common::{PositiveCase, RuleCase};
use sbol2::validation::Severity::Error;

pub fn cases() -> Vec<RuleCase> {
    vec![RuleCase {
        name: "Range end is less than its start",
        rule: "sbol2-11104",
        severity: Error,
        body: r#"<http://ex/r> a sbol:Range ;
    sbol:displayId "r" ;
    sbol:persistentIdentity <http://ex/r> ;
    sbol:start 10 ;
    sbol:end 5 .
"#,
    }]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![PositiveCase {
        name: "Range end is greater than or equal to its start",
        rule: "sbol2-11104",
        body: r#"<http://ex/r> a sbol:Range ;
    sbol:displayId "r" ;
    sbol:persistentIdentity <http://ex/r> ;
    sbol:start 5 ;
    sbol:end 10 .
"#,
    }]
}
