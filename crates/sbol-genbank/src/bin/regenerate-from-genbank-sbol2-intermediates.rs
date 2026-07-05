//! Regenerate the `tests/fixtures/sbol2/real/from_genbank/*.xml`
//! intermediates from their `.gb` sources in pure Rust.
//!
//! Pipeline:
//!
//! ```text
//!   tests/fixtures/genbank/{name}.gb
//!         │   sbol_genbank::GenbankImporter
//!         ▼
//!   sbol::Document  (SBOL 3)
//!         │   sbol::downgrade::downgrade(&document)
//!         ▼
//!   sbol_rdf::Graph  (SBOL 2)
//!         │   graph.write(RdfFormat::RdfXml)
//!         ▼
//!   tests/fixtures/sbol2/real/from_genbank/{name}.xml
//! ```
//!
//! No Docker, no Python, no Node. Run after a `.gb` corpus change or
//! after intentional changes to either converter.
//!
//! Note: `pUC19.gbk` is intentionally NOT pulled into the SBOL 2
//! upgrade conformance corpus — it's part of the GenBank import
//! conformance harness instead. Only the iGEM `.gb` fixtures
//! participate in the upgrade harness, because they were originally
//! committed there for that purpose.

use std::path::PathBuf;
use std::process::ExitCode;

use sbol::RdfFormat;
use sbol_genbank::GenbankImporter;

const FIXTURES: &[&str] = &["BBa_B0034", "BBa_E0040", "BBa_F2620", "BBa_R0010"];

fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path
}

fn main() -> ExitCode {
    let root = workspace_root();
    let genbank_dir = root.join("tests/fixtures/genbank");
    let target_dir = root.join("tests/fixtures/sbol2/real/from_genbank");

    if let Err(err) = std::fs::create_dir_all(&target_dir) {
        eprintln!(
            "failed to create target dir {}: {err}",
            target_dir.display()
        );
        return ExitCode::from(2);
    }

    let mut failures = 0usize;
    for name in FIXTURES {
        let gb_path = genbank_dir.join(format!("{name}.gb"));
        let xml_path = target_dir.join(format!("{name}.xml"));
        let namespace = format!("https://sbol-rs.example.org/genbank/{name}");

        let importer = match GenbankImporter::new(&namespace) {
            Ok(importer) => importer,
            Err(err) => {
                eprintln!("[FAIL] {name}: namespace: {err}");
                failures += 1;
                continue;
            }
        };
        let (document, _ireport) = match importer.read_path(&gb_path) {
            Ok(pair) => pair,
            Err(err) => {
                eprintln!("[FAIL] {name}: import: {err}");
                failures += 1;
                continue;
            }
        };
        let (sbol2_graph, _dreport) = match sbol::downgrade::downgrade(&document) {
            Ok(pair) => pair,
            Err(err) => {
                eprintln!("[FAIL] {name}: downgrade: {err}");
                failures += 1;
                continue;
            }
        };
        let payload = match sbol2_graph.write(RdfFormat::RdfXml) {
            Ok(payload) => payload,
            Err(err) => {
                eprintln!("[FAIL] {name}: serialize: {err}");
                failures += 1;
                continue;
            }
        };
        if let Err(err) = std::fs::write(&xml_path, payload) {
            eprintln!("[FAIL] {name}: write {}: {err}", xml_path.display());
            failures += 1;
            continue;
        }
        println!("regenerated {name} ({})", xml_path.display());
    }

    println!(
        "{}/{} fixtures regenerated",
        FIXTURES.len() - failures,
        FIXTURES.len()
    );
    if failures > 0 {
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}
