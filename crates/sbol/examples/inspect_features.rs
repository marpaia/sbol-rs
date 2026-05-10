//! Parse a Turtle document and walk a Component's child features.
//!
//! Run with `cargo run -p sbol --example inspect_features`.

use sbol::prelude::*;

const TURTLE: &str = r#"
PREFIX sbol: <http://sbols.org/v3#>
PREFIX SBO: <https://identifiers.org/SBO:>

<https://example.org/lab/c> a sbol:Component;
    sbol:displayId "c";
    sbol:hasNamespace <https://example.org/lab>;
    sbol:type SBO:0000251;
    sbol:hasFeature <https://example.org/lab/c/sub1>, <https://example.org/lab/c/sub2> .

<https://example.org/lab/c/sub1> a sbol:LocalSubComponent;
    sbol:displayId "sub1";
    sbol:type SBO:0000251 .

<https://example.org/lab/c/sub2> a sbol:LocalSubComponent;
    sbol:displayId "sub2";
    sbol:type SBO:0000251 .
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let document = Document::read_turtle(TURTLE)?;

    for component in document.components() {
        let display_id = component.display_id().unwrap_or("?");
        println!("Component {display_id}:");
        for feature in component.features(&document) {
            match feature {
                FeatureRef::LocalSubComponent(sub) => {
                    println!("  LocalSubComponent {}", sub.display_id().unwrap_or("?"))
                }
                FeatureRef::SubComponent(sub) => {
                    println!("  SubComponent {}", sub.display_id().unwrap_or("?"))
                }
                FeatureRef::SequenceFeature(sf) => {
                    println!("  SequenceFeature {}", sf.display_id().unwrap_or("?"))
                }
                FeatureRef::ComponentReference(_) | FeatureRef::ExternallyDefined(_) => {
                    println!("  (other feature kind)")
                }
                _ => println!("  (unknown feature variant)"),
            }
        }
    }
    Ok(())
}
