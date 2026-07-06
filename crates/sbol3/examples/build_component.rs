//! Build a typed Component with a Sequence and serialize it to Turtle.
//!
//! Run with `cargo run -p sbol --example build_component`.

use sbol3::constants::{EDAM_IUPAC_DNA, SBO_DNA, SO_PROMOTER};
use sbol3::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let namespace = "https://example.org/lab";

    let sequence = Sequence::builder(namespace, "j23119_seq")?
        .elements("ttgacagctagctcagtcctaggtataatgctagc")
        .encoding(EDAM_IUPAC_DNA)
        .name("J23119 promoter sequence")
        .build()?;

    let component = Component::builder(namespace, "j23119")?
        .types([SBO_DNA])
        .add_component_role(SO_PROMOTER)
        .add_sequence(sequence.identity.clone())
        .name("J23119 constitutive promoter")
        .description("An Anderson family constitutive promoter")
        .build()?;

    let document = Document::from_objects(vec![
        SbolObject::Component(component),
        SbolObject::Sequence(sequence),
    ])?;

    let report = document.validate();
    assert!(
        report.is_valid(),
        "expected a valid document, got {report:?}"
    );

    println!("{}", document.write_turtle()?);
    Ok(())
}
