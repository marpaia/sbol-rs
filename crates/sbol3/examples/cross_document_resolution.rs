//! Walk SubComponent.instanceOf across two SBOL documents.
//!
//! Two documents are constructed in memory: a design document holding an
//! outer Component with a SubComponent, and a parts-library document
//! holding the Component the SubComponent points at. A `DocumentSet`
//! composes both, and `SubComponent::definition(&scope)` resolves the
//! cross-document reference.
//!
//! Run with `cargo run -p sbol --example cross_document_resolution`.

use sbol3::constants::{SBO_DNA, SO_PROMOTER};
use sbol3::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let design_ns = "https://example.org/design";
    let parts_ns = "https://example.org/parts";

    // Parts library document: a Component the design will reference.
    let promoter = Component::builder(parts_ns, "j23119")?
        .types([SBO_DNA])
        .add_component_role(SO_PROMOTER)
        .name("J23119 constitutive promoter")
        .build()?;
    let parts = Document::from_objects(vec![SbolObject::Component(promoter.clone())])?;

    // Design document: outer Component with a SubComponent whose
    // instanceOf points into the parts library.
    let mut device = Component::builder(design_ns, "device")?
        .types([SBO_DNA])
        .name("Promoter-driven device")
        .build()?;
    let sub = SubComponent::new(&device.identity, "promoter", promoter.identity.clone())?;
    device.features.push(sub.identity.clone());

    let design = Document::from_objects(vec![
        SbolObject::Component(device),
        SbolObject::SubComponent(sub.clone()),
    ])?;

    // Resolve against just the design — the part lives elsewhere, so
    // this is expected to fail with `NotFound`.
    match sub.definition(&design) {
        Err(ReferenceError::NotFound(iri)) => {
            println!("within design alone: NotFound({iri}) — expected\n");
        }
        other => panic!("unexpected single-document outcome: {other:?}"),
    }

    // Compose both documents and resolve against the union.
    let scope = DocumentSet::from_documents([&design, &parts])?;
    let definition = sub.definition(&scope)?;

    println!(
        "design SubComponent `{}` resolves to Component `{}` from the parts library",
        sub.identity, definition.identity,
    );
    if let Some(name) = definition.name() {
        println!("  name: {name}");
    }
    println!("  types: {:?}", definition.types);
    println!("  roles: {:?}", definition.roles);

    Ok(())
}
