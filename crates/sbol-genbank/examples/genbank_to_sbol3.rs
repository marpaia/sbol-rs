//! End-to-end pure-Rust pipeline from a real GenBank file to a usable
//! SBOL 3 RDF document.
//!
//! ```sh
//! # Defaults to BBa_E0040 + Turtle to stdout.
//! cargo run -p sbol-genbank --example genbank_to_sbol3
//!
//! # Point at any local `.gb` file and write Turtle alongside it.
//! cargo run -p sbol-genbank --example genbank_to_sbol3 -- \
//!     tests/fixtures/genbank/BBa_F2620.gb /tmp/BBa_F2620.ttl
//!
//! # Composite design with N-Triples output and a custom namespace.
//! cargo run -p sbol-genbank --example genbank_to_sbol3 -- \
//!     tests/fixtures/genbank/BBa_F2620.gb /tmp/BBa_F2620.nt \
//!     https://mylab.example.org/parts
//! ```
//!
//! No network, no Python, no Docker. The pipeline is:
//!
//! ```text
//!   GenBank (.gb / .gbk)
//!         │   gb-io parser (sbol_genbank::GenbankImporter)
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
//!
//! The example demonstrates each stage. For the test-side counterpart
//! (snapshot + canonical-engine subset diffs), see
//! `crates/sbol-genbank/tests/import.rs`.

use std::env;
use std::path::{Path, PathBuf};

use sbol3::{Document, RdfFormat, SbolIdentified, Severity};
use sbol_genbank::{GenbankImporter, ImportReport, ImportWarning};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = PathBuf::from(
        env::args()
            .nth(1)
            .unwrap_or_else(|| "tests/fixtures/genbank/BBa_E0040.gb".to_string()),
    );
    let output_arg = env::args().nth(2);
    let namespace = env::args()
        .nth(3)
        .unwrap_or_else(|| "https://example.org/lab".to_string());

    println!("▸ stage 1: parse + import");
    println!("   reading  {}", input_path.display());
    println!("   namespace {namespace}");
    let (document, report) = GenbankImporter::new(&namespace)?.read_path(&input_path)?;
    print_import_report(&report);

    println!("\n▸ stage 2: validate against the SBOL 3.1.0 spec");
    let validation = document.validate();
    let error_count = validation
        .issues()
        .iter()
        .filter(|i| matches!(i.severity, Severity::Error))
        .count();
    let warning_count = validation
        .issues()
        .iter()
        .filter(|i| matches!(i.severity, Severity::Warning))
        .count();
    println!("   {error_count} error(s), {warning_count} warning(s)");
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
        let same_components = reparsed.components().count() == document.components().count();
        let same_sequences = reparsed.sequences().count() == document.sequences().count();
        let same_features =
            reparsed.sequence_features().count() == document.sequence_features().count();
        let same_ranges = reparsed.ranges().count() == document.ranges().count();
        if same_components && same_sequences && same_features && same_ranges {
            println!("   ✓ all top-level counts preserved on re-read");
        } else {
            println!("   ✗ count drift on re-read:");
            println!(
                "     Components       imported={} reread={}",
                document.components().count(),
                reparsed.components().count()
            );
            println!(
                "     Sequences        imported={} reread={}",
                document.sequences().count(),
                reparsed.sequences().count()
            );
            println!(
                "     SequenceFeatures imported={} reread={}",
                document.sequence_features().count(),
                reparsed.sequence_features().count()
            );
            println!(
                "     Ranges           imported={} reread={}",
                document.ranges().count(),
                reparsed.ranges().count()
            );
        }
    } else {
        println!("   (skipped — round-trip needs a file on disk)");
    }

    println!("\n▸ summary");
    print_document_summary(&document);

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
        "   imported  {} Component(s), {} Sequence(s), {} SequenceFeature(s)",
        report.components, report.sequences, report.features
    );
    if !report.warnings.is_empty() {
        for w in &report.warnings {
            println!("   warning   {}", describe_warning(w));
        }
    }
}

fn print_document_summary(document: &Document) {
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
        for r in &component.roles {
            println!("       role:        {}", short_iri(r.as_str()));
        }
        println!("       hasSequence: {}", component.sequences.len());
        println!("       hasFeature:  {}", component.features.len());
    }

    let mut by_role = std::collections::BTreeMap::<String, usize>::new();
    for feature in document.sequence_features() {
        for role in feature.feature.roles.iter() {
            *by_role.entry(short_iri(role.as_str())).or_insert(0) += 1;
        }
    }
    if !by_role.is_empty() {
        println!("\n   SequenceFeature role distribution:");
        for (role, count) in &by_role {
            println!("       {count:>3} × {role}");
        }
    }
}

fn describe_warning(warning: &ImportWarning) -> String {
    match warning {
        ImportWarning::UnknownFeatureKey { kind } => {
            format!("unrecognized GenBank feature key `{kind}`")
        }
        ImportWarning::LossyLocation { feature, reason } => {
            format!("feature `{feature}`: {reason}")
        }
        ImportWarning::SynthesizedIdentifier => {
            "GenBank record had no ACCESSION / LOCUS name".to_string()
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
