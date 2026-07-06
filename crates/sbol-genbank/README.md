# sbol-genbank

Pure-Rust GenBank → SBOL 3 importer for the `sbol-rs` workspace.

GenBank is the de facto exchange format for plasmids, parts, and genes
in molecular biology: SnapGene, ApE, Benchling, and the NCBI Nucleotide
database all emit `.gb` / `.gbk` files natively. This crate lets
`sbol-rs` ingest that data without a Python or Node detour.

Parsing is delegated to [`gb-io`], a mature MIT-licensed nom-based
GenBank parser. The mapping layer in this crate translates each
`gb_io::seq::Seq` record into a native `sbol::Document`:

- Each GenBank record becomes one `sbol::Component` + one
  `sbol::Sequence`.
- Each annotated feature becomes a `sbol::SequenceFeature` with a
  `sbol::Range` per location segment.
- Common GenBank feature keys (`CDS`, `promoter`, `terminator`, `RBS`,
  `5'UTR`, …) are mapped to their canonical Sequence Ontology IRIs.
  Unrecognized keys fall back to `SO:0000110` (`sequence_feature`) and
  surface in the [`ImportReport`] warnings so nothing is silently lost.
- Topology (`linear` / `circular`) is added to `Component.type` per
  SBOL 3 best practice.
- The molecule type (`DNA` / `RNA` / `AA`) chooses the SBO component
  type (`SBO:0000251` / `SBO:0000250` / `SBO:0000252`) and the EDAM
  sequence encoding.
- Real-world dialect quirks are tolerated: SynBioHub emits mixed-case
  month names on the LOCUS line (`20-May-2026`), which `gb-io`'s strict
  parser would otherwise reject; this crate normalizes them upfront.

## Quickstart

```rust
use sbol_genbank::GenbankImporter;

let (document, report) =
    GenbankImporter::new("https://example.org/lab")?.read_path("plasmid.gb")?;

println!(
    "{} components, {} sequences, {} features ({} warnings)",
    report.components, report.sequences, report.features, report.warnings.len()
);

document.check()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## CLI

The same logic is wired into the `sbol` CLI:

```sh
sbol import-genbank plasmid.gb \
  --namespace https://example.org/lab \
  --to turtle \
  -o plasmid.ttl \
  --validate
```

## Runnable example

```sh
cargo run -p sbol-genbank --example genbank_to_sbol3 \
  -- tests/fixtures/genbank/BBa_F2620.gb
```

prints a structural summary of an iGEM Registry composite design as
SBOL 3.

## Scope

This crate handles the read direction (GenBank → SBOL 3). The reverse
direction (SBOL 3 → GenBank) is not implemented; for that, route
through `sbol-utilities` or libSBOLj. End-to-end SBOL 3 round-trips
within `sbol-rs` work via the `sbol` crate's RDF I/O.

[`gb-io`]: https://crates.io/crates/gb-io
[`ImportReport`]: https://docs.rs/sbol-genbank/latest/sbol_genbank/struct.ImportReport.html
