//! End-to-end pure-Rust pipeline from a FASTA file to a usable SBOL 3
//! RDF document.
//!
//! ```sh
//! # Defaults to a fixture + Turtle to stdout.
//! cargo run -p sbol-fasta --example fasta_to_sbol3
//!
//! # Point at any local FASTA file and write Turtle alongside it.
//! cargo run -p sbol-fasta --example fasta_to_sbol3 -- \
//!     tests/fixtures/fasta/GFP_protein.fasta /tmp/GFP.ttl
//!
//! # Multi-record protein FASTA with N-Triples output and a custom namespace.
//! cargo run -p sbol-fasta --example fasta_to_sbol3 -- \
//!     tests/fixtures/fasta/multi_protein.fasta /tmp/proteins.nt \
//!     https://mylab.example.org/parts
//! ```
//!
//! No network, no Python, no Docker. The pipeline is:
//!
//! ```text
//!   FASTA (.fasta / .fa / .fna / .faa)
//!         │   sbol_fasta::FastaImporter (hand-rolled parser + alphabet detection)
//!         ▼
//!   sbol3::Document  ←─────────── native SBOL 3 graph
//!     │     │
//!     │     │   document.validate()  (sbol3-* spec rules)
//!     │     ▼
//!     │   ValidationReport
//!     │
//!     │   document.write(RdfFormat::Turtle|RdfXml|JsonLd|NTriples)
//!     ▼
//!   serialized RDF on disk ─── round-tripped back through
//!                              Document::read_path to confirm
//!                              the graph is stable.
//! ```

use std::env;
use std::path::{Path, PathBuf};

use sbol3::{Document, RdfFormat, SbolIdentified, Severity};
use sbol_fasta::{FastaImporter, ImportReport, ImportWarning};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = PathBuf::from(
        env::args()
            .nth(1)
            .unwrap_or_else(|| "tests/fixtures/fasta/pUC19.fasta".to_string()),
    );
    let output_arg = env::args().nth(2);
    let namespace = env::args()
        .nth(3)
        .unwrap_or_else(|| "https://example.org/lab".to_string());

    println!("▸ stage 1: parse + import");
    println!("   reading   {}", input_path.display());
    println!("   namespace {namespace}");
    let (document, report) = FastaImporter::new(&namespace)?.read_path(&input_path)?;
    print_import_report(&report);

    println!("\n▸ stage 2: validate against the SBOL 3.1.0 spec");
    let validation = document.validate();
    let errors = validation
        .issues()
        .iter()
        .filter(|i| matches!(i.severity, Severity::Error))
        .count();
    let warnings = validation
        .issues()
        .iter()
        .filter(|i| matches!(i.severity, Severity::Warning))
        .count();
    println!("   {errors} error(s), {warnings} warning(s)");
    for issue in validation.issues().iter().take(5) {
        println!(
            "     [{:?}] {} — {}",
            issue.severity, issue.rule, issue.message
        );
    }

    println!("\n▸ stage 3: serialize");
    let (output_path, format) = resolve_output(&input_path, output_arg.as_deref())?;
    let payload = document.write(format)?;
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

    println!("\n▸ stage 4: round-trip — re-read the serialized RDF");
    if output_path.to_string_lossy() != "-" {
        let reparsed = Document::read_path(&output_path)?;
        if reparsed.components().count() == document.components().count()
            && reparsed.sequences().count() == document.sequences().count()
        {
            println!("   ✓ Component and Sequence counts preserved on re-read");
        } else {
            println!(
                "   ✗ count drift: imported Components={} Sequences={}, reread Components={} Sequences={}",
                document.components().count(),
                document.sequences().count(),
                reparsed.components().count(),
                reparsed.sequences().count()
            );
        }
    } else {
        println!("   (skipped — round-trip needs a file on disk)");
    }

    println!("\n▸ summary");
    for component in document.components() {
        let identity = component
            .identity
            .as_iri()
            .map(|i| i.as_str())
            .unwrap_or("?");
        let display_id = component.display_id().unwrap_or("?");
        let name = component.name().unwrap_or("(no name)");
        println!("\n   • {display_id} <{identity}>");
        println!("       name:        {name}");
        if let Some(description) = component.description() {
            let short = if description.len() > 80 {
                format!("{}…", &description[..80])
            } else {
                description.to_string()
            };
            println!("       description: {short}");
        }
        for t in &component.types {
            println!("       type:        {}", short_iri(t.as_str()));
        }
        println!("       hasSequence: {}", component.sequences.len());
    }

    Ok(())
}

fn resolve_output(
    input: &Path,
    arg: Option<&str>,
) -> Result<(PathBuf, RdfFormat), Box<dyn std::error::Error>> {
    match arg {
        None | Some("-") => Ok((PathBuf::from("-"), RdfFormat::Turtle)),
        Some(path) => {
            let path = PathBuf::from(path);
            let format = RdfFormat::from_path(&path).ok_or_else(|| {
                format!(
                    "cannot infer RDF format from `{}` — supported: .ttl .rdf .jsonld .nt",
                    path.display()
                )
            })?;
            let _ = input;
            Ok((path, format))
        }
    }
}

fn print_import_report(report: &ImportReport) {
    println!(
        "   imported  {} Component(s), {} Sequence(s) ({} DNA, {} RNA, {} protein)",
        report.components,
        report.sequences,
        report.dna_records,
        report.rna_records,
        report.protein_records
    );
    if !report.warnings.is_empty() {
        for w in &report.warnings {
            println!("   warning   {}", describe_warning(w));
        }
    }
}

fn describe_warning(warning: &ImportWarning) -> String {
    match warning {
        ImportWarning::EmptyRecord { record_id } => {
            format!("record `{record_id}` has no sequence body")
        }
        _ => "unrecognized warning".to_string(),
    }
}

fn short_iri(iri: &str) -> String {
    if let Some(suffix) = iri.strip_prefix("https://identifiers.org/") {
        suffix.to_string()
    } else if let Some(suffix) = iri.strip_prefix("http://sbols.org/v3#") {
        format!("sbol3:{suffix}")
    } else {
        iri.to_string()
    }
}
