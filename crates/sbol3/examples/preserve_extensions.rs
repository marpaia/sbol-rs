//! Attach a non-SBOL annotation predicate to a typed Component, round-trip
//! the document through Turtle, and confirm the extension triple survives.
//!
//! Run with `cargo run -p sbol --example preserve_extensions`.

use sbol3::constants::SBO_DNA;
use sbol3::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let component = Component::builder("https://example.org/lab", "c")?
        .types([SBO_DNA])
        .extension(
            Iri::from_static("https://lab.example.org/authoredBy"),
            Term::Literal(Literal::simple("alice")),
        )
        .build()?;

    let document = Document::from_objects(vec![SbolObject::Component(component)])?;
    let serialized = document.write_turtle()?;
    println!("--- Written Turtle ---\n{serialized}");

    let reparsed = Document::read_turtle(&serialized)?;
    let component = reparsed
        .components()
        .next()
        .expect("round-tripped Component must be present");
    for extension in &component.identified.extensions {
        println!(
            "extension: {} -> {:?}",
            extension.predicate.as_str(),
            extension.object
        );
    }
    Ok(())
}
