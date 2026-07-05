//! Convert GenBank flat-file records to SBOL 3 documents.
//!
//! GenBank is the de facto exchange format for plasmids, genes, and
//! parts in molecular biology — SnapGene, ApE, Benchling, the NCBI
//! Nucleotide database, and SynBioHub all speak it natively. This crate
//! lets `sbol-rs` ingest that data without leaving Rust.
//!
//! Parsing is delegated to [`gb_io`], a mature MIT-licensed nom-based
//! GenBank parser. The mapping layer in this crate translates each
//! `gb_io::seq::Seq` record into an SBOL 3 [`Component`] + [`Sequence`]
//! pair, with annotated features becoming [`SequenceFeature`]s with
//! [`Range`] locations. Common GenBank feature keys (CDS, promoter,
//! terminator, RBS, …) are mapped to their canonical Sequence Ontology
//! IRIs; unrecognized keys pass through under
//! `http://sboltools.org/backport#genbank_feature_kind` so the
//! information survives round-trips.
//!
//! ```no_run
//! use sbol_genbank::GenbankImporter;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let (document, report) =
//!     GenbankImporter::new("https://example.org/lab")?.read_path("plasmid.gb")?;
//! document.check()?;
//! println!("{} components, {} sequences", report.components, report.sequences);
//! # Ok(())
//! # }
//! ```
//!
//! [`Component`]: sbol3::Component
//! [`Sequence`]: sbol3::Sequence
//! [`SequenceFeature`]: sbol3::SequenceFeature
//! [`Range`]: sbol3::Range

#![forbid(unsafe_code)]

mod feature_map;
mod importer;

pub use importer::{GenbankImporter, ImportError, ImportReport, ImportWarning};
