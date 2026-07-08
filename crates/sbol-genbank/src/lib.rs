//! Convert between GenBank flat-file records and SBOL 3 documents.
//!
//! GenBank is the de facto exchange format for plasmids, genes, and
//! parts in molecular biology: SnapGene, ApE, Benchling, the NCBI
//! Nucleotide database, and SynBioHub all speak it natively. This crate
//! moves data both ways, so `sbol-rs` can ingest GenBank and hand it
//! back without leaving Rust.
//!
//! The GenBank grammar is handled by [`gb_io`], a mature MIT-licensed
//! nom-based parser and writer. [`GenbankImporter`] translates each
//! `gb_io::seq::Seq` record into an SBOL 3 [`Component`] + [`Sequence`]
//! pair, with annotated features becoming [`SequenceFeature`]s with
//! [`Range`] locations. [`GenbankExporter`] performs the inverse,
//! rebuilding a `gb_io::seq::Seq` per component and writing it out.
//!
//! Common GenBank feature keys (CDS, promoter, terminator, RBS, …) map
//! to their canonical Sequence Ontology IRIs and back again. A key with
//! no Sequence Ontology term can be preserved verbatim under
//! `http://sboltools.org/backport#genbank_feature_kind`, which the
//! exporter reads to reproduce the original key on the way out.
//!
//! ```no_run
//! use sbol_genbank::{GenbankExporter, GenbankImporter};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let (document, report) =
//!     GenbankImporter::new("https://example.org/lab")?.read_path("plasmid.gb")?;
//! document.check()?;
//! println!("{} components, {} sequences", report.components, report.sequences);
//!
//! let genbank = GenbankExporter::new().to_string(&document)?;
//! println!("{genbank}");
//! # Ok(())
//! # }
//! ```
//!
//! [`Component`]: sbol3::Component
//! [`Sequence`]: sbol3::Sequence
//! [`SequenceFeature`]: sbol3::SequenceFeature
//! [`Range`]: sbol3::Range

#![forbid(unsafe_code)]

mod exporter;
mod feature_map;
mod importer;

pub use exporter::{ExportError, ExportReport, ExportWarning, GenbankExporter};
pub use importer::{GenbankImporter, ImportError, ImportReport, ImportWarning};
