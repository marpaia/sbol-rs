//! Read a Turtle document, check it for validation errors, and print any issues.
//!
//! Run with `cargo run -p sbol --example read_validate`.

use sbol::prelude::*;

const TURTLE: &str = r#"
PREFIX sbol: <http://sbols.org/v3#>
PREFIX SBO: <https://identifiers.org/SBO:>

<https://example.org/lab/c> a sbol:Component;
    sbol:displayId "c";
    sbol:hasNamespace <https://example.org/lab>;
    sbol:type SBO:0000251 .
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let document = Document::read_turtle(TURTLE)?;

    let report = document.validate();
    if report.is_valid() {
        println!("document is valid");
    } else {
        println!("validation reported {} issues:", report.issues().len());
        for issue in report.issues() {
            println!(
                "  [{:?}] {} — {}",
                issue.severity, issue.rule, issue.message
            );
        }
    }

    // `check` returns Err carrying the full report when any fully-evaluated rule errored.
    document.check()?;
    Ok(())
}
