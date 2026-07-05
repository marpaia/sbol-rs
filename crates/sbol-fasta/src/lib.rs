//! Convert FASTA records to SBOL 3 documents.
//!
//! FASTA is the lowest-common-denominator sequence exchange format —
//! NCBI BLAST, the UniProt downloads, every genome project, and most
//! bioinformatics tools either emit or accept it. This crate lets
//! `sbol-rs` ingest that data with no external dependencies.
//!
//! Each `>header` record becomes one [`sbol3::Component`] paired with
//! one [`sbol3::Sequence`]. The component's biological type
//! (`SBO_DNA` / `SBO_RNA` / `SBO_PROTEIN`) and the sequence's EDAM
//! encoding are auto-detected from the alphabet of the sequence
//! itself — the caller can override the detection with
//! [`FastaImporter::with_alphabet`] when the data is ambiguous (e.g. a
//! very short DNA-looking sequence that's actually a protein).
//!
//! ```no_run
//! use sbol_fasta::FastaImporter;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let (document, report) = FastaImporter::new("https://example.org/lab")?
//!     .read_path("genome.fasta")?;
//! document.check()?;
//! println!("{} component(s), {} sequence(s)", report.components, report.sequences);
//! # Ok(())
//! # }
//! ```
//!
//! FASTA carries no feature annotations — what you get back is a
//! Component with no `SequenceFeature`s. If you need annotated data,
//! reach for [`sbol-genbank`](https://docs.rs/sbol-genbank) instead.

#![forbid(unsafe_code)]

mod alphabet;
mod importer;
mod parser;

pub use alphabet::Alphabet;
pub use importer::{FastaImporter, ImportError, ImportReport, ImportWarning};
