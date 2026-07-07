//! Toggle-switch design built with the raw `Design` arena — no biology verbs,
//! no IRI or `SbolObject` handling. The biology-verb layer
//! (`d.promoter(...)`, `d.engineered_region(...)`) collapses this further.
//!
//! Run with `cargo run -p sbol3 --example toggle_switch_arena`.

use sbol3::constants::{SO_CDS, SO_ENGINEERED_REGION, SO_PROMOTER, SO_RBS, SO_TERMINATOR};
use sbol3::design::{ComponentId, Design};
use sbol3::prelude::{Iri, RdfFormat};

const NS: &str = "https://github.com/DRAGGON-Lab";

/// Registers a DNA part (component + its sequence) and returns the component.
fn part(d: &mut Design, id: &str, role: Iri, elements: &str, description: &str) -> ComponentId {
    let seq = d
        .sequence(&format!("{id}_seq"))
        .elements(elements)
        .dna()
        .add();
    d.component(id)
        .dna()
        .role(role)
        .sequence(seq)
        .name(id)
        .description(description)
        .add()
}

/// Wires parts into an engineered region, chaining them head-to-tail.
fn engineered_region(d: &mut Design, id: &str, parts: &[(ComponentId, &[Iri])], description: &str) {
    let tu = d
        .component(id)
        .dna()
        .role(SO_ENGINEERED_REGION)
        .description(description)
        .add();
    let subs: Vec<_> = parts
        .iter()
        .enumerate()
        .map(|(i, (part, roles))| {
            d.sub_component(tu, &format!("sub_{i}"))
                .instance_of(*part)
                .roles(roles.iter().cloned())
                .add()
        })
        .collect();
    for pair in subs.windows(2) {
        d.meets(tu, pair[0], pair[1]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut d = Design::new(NS)?;

    let plac = part(
        &mut d,
        "pLac",
        SO_PROMOTER,
        "caatacg",
        "Promoter for LacI repressible expression",
    );
    let b0034 = part(
        &mut d,
        "B0034",
        SO_RBS,
        "ttgaacaccgtc",
        "RBS (Elowitz 1999)",
    );
    let tetr = part(
        &mut d,
        "tetR",
        SO_CDS,
        "atggtgaat",
        "Coding region for the TetR protein",
    );
    let b0015 = part(
        &mut d,
        "B0015",
        SO_TERMINATOR,
        "GTCCatttg",
        "Double terminator BBa_B0010+B0012",
    );

    engineered_region(
        &mut d,
        "pLac_tu",
        &[
            (plac, &[SO_PROMOTER]),
            (b0034, &[SO_RBS]),
            (tetr, &[SO_CDS]),
            (b0015, &[SO_TERMINATOR]),
        ],
        "Transcriptional unit: pLac, B0034, tetR, B0015 (produces TetR).",
    );

    let doc = d.finish()?;

    // `typed_objects()` preserves insertion order; `objects()` is sorted by IRI.
    for object in doc.typed_objects() {
        println!("{}", object.identity());
    }

    doc.write_path("toggle_switch_arena.nt", RdfFormat::NTriples)?;
    Ok(())
}
