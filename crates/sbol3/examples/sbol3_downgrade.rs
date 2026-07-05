//! Pure-Rust SBOL 3 → SBOL 2 downgrade end-to-end.
//!
//! ```sh
//! # Defaults to a fixture + RDF/XML to stdout.
//! cargo run -p sbol --example sbol3_downgrade
//!
//! # Point at any local SBOL 3 file and write SBOL 2 RDF/XML alongside it.
//! cargo run -p sbol --example sbol3_downgrade -- \
//!     tests/fixtures/sbol2/real/expected/synbiohub/BBa_F2620.nt \
//!     /tmp/BBa_F2620.xml
//! ```
//!
//! The pipeline is:
//!
//! ```text
//!   SBOL 3 RDF (any format)
//!         │   Document::read_path
//!         ▼
//!   sbol3::Document
//!         │   sbol3::downgrade::downgrade(&document)
//!         ▼
//!   sbol_rdf::Graph (SBOL 2)
//!         │   graph.write(RdfFormat::RdfXml)
//!         ▼
//!   serialized SBOL 2 RDF/XML on disk
//!         │   sbol3::upgrade::upgrade_from_sbol2  (--validate)
//!         ▼
//!   re-upgraded SBOL 3 + validation report
//! ```
//!
//! Stage 4 is the proxy for SBOL 2 spec compliance: if the downgrade
//! preserved enough structure for the upgrade to rebuild a valid
//! SBOL 3 graph, the SBOL 2 we emitted is well-formed.

use std::env;
use std::path::PathBuf;

use sbol3::{Document, RdfFormat, Severity};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = PathBuf::from(env::args().nth(1).unwrap_or_else(|| {
        "tests/fixtures/sbol2/real/expected/synbiohub/BBa_E0040.nt".to_string()
    }));
    let output_arg = env::args().nth(2);

    println!("▸ stage 1: read SBOL 3 document");
    println!("   reading {}", input_path.display());
    let document = Document::read_path(&input_path)?;
    println!(
        "   {} Component(s), {} Sequence(s)",
        document.components().count(),
        document.sequences().count()
    );

    println!("\n▸ stage 2: downgrade to SBOL 2");
    let (sbol2_graph, report) = sbol3::downgrade::downgrade(&document)?;
    let counts = report.counts();
    println!(
        "   {} CD, {} MD, {} SubComponent, {} SequenceFeature, {} backport-restored, {} synthesized",
        counts.components_to_component_definition,
        counts.components_to_module_definition,
        counts.sub_components_emitted,
        counts.sequence_features_emitted,
        counts.identities_restored_from_backport,
        counts.identities_synthesized,
    );
    if !report.warnings().is_empty() {
        for w in report.warnings() {
            println!("   warning: {w:?}");
        }
    }

    println!("\n▸ stage 3: serialize");
    let (output_path, format) = match output_arg.as_deref() {
        None | Some("-") => (PathBuf::from("-"), RdfFormat::RdfXml),
        Some(path) => {
            let path = PathBuf::from(path);
            // SBOL 2 ecosystem convention is `.xml` for RDF/XML;
            // accept that alongside the strict `.rdf`.
            let format = RdfFormat::from_path(&path)
                .or_else(|| {
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    if ext.eq_ignore_ascii_case("xml") {
                        Some(RdfFormat::RdfXml)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    format!(
                        "cannot infer RDF format from `{}` — supported: .ttl .rdf .xml .jsonld .nt",
                        path.display()
                    )
                })?;
            (path, format)
        }
    };
    let payload = sbol2_graph.write(format)?;
    if output_path.to_string_lossy() == "-" {
        print!("\n{payload}");
        println!("   wrote {} bytes to stdout as {format}", payload.len());
    } else {
        std::fs::write(&output_path, &payload)?;
        println!(
            "   wrote {} ({} bytes, {format})",
            output_path.display(),
            payload.len()
        );
    }

    println!("\n▸ stage 4: round-trip validate");
    let turtle = sbol2_graph.write(RdfFormat::Turtle)?;
    match sbol3::upgrade::upgrade_from_sbol2(&turtle, RdfFormat::Turtle) {
        Ok((reupgraded, _)) => {
            let validation = reupgraded.validate();
            let errors = validation
                .issues()
                .iter()
                .filter(|i| matches!(i.severity, Severity::Error))
                .count();
            if errors == 0 {
                println!(
                    "   ✓ round-trip validates: 0 errors, {} warning(s)",
                    validation.issues().len() - errors
                );
            } else {
                println!("   ✗ round-trip produced {errors} validation error(s):");
                for issue in validation.issues().iter().take(5) {
                    println!(
                        "       [{:?}] {} — {}",
                        issue.severity, issue.rule, issue.message
                    );
                }
            }
        }
        Err(err) => {
            println!("   ✗ round-trip failed at re-upgrade: {err}");
        }
    }

    Ok(())
}
