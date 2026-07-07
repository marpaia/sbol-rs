//! Compare two revisions of a design and print what changed.
//!
//! Run with `cargo run -p sbol3 --example diff_documents`.

use sbol3::constants::{EDAM_IUPAC_DNA, SBO_DNA, SO_ENGINEERED_REGION, SO_PROMOTER};
use sbol3::prelude::*;

const NAMESPACE: &str = "https://example.org/lab";

/// Builds a promoter component sharing an identity across revisions, so a diff
/// matches the two by identity and reports only what moved.
fn promoter(name: &str, roles: &[Iri]) -> Result<Component, BuildError> {
    let mut builder = Component::builder(NAMESPACE, "j23119")?
        .types([SBO_DNA])
        .name(name);
    for role in roles {
        builder = builder.add_component_role(role.clone());
    }
    builder.build()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sequence = Sequence::builder(NAMESPACE, "j23119_seq")?
        .elements("ttgacagctagctcagtcctaggtataatgctagc")
        .encoding(EDAM_IUPAC_DNA)
        .build()?;

    let old = Document::from_objects(vec![
        SbolObject::Component(promoter("J23119", &[SO_PROMOTER])?),
        SbolObject::Sequence(sequence.clone()),
    ])?;

    // The revision renames the promoter and annotates it with a second role.
    let new = Document::from_objects(vec![SbolObject::Component(promoter(
        "J23119 constitutive promoter",
        &[SO_PROMOTER, SO_ENGINEERED_REGION],
    )?)])?;

    let diff = old.diff(&new);
    if diff.is_empty() {
        println!("documents are identical");
    } else {
        print!("{diff}");
    }
    Ok(())
}
