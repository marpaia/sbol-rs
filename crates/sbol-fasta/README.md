# sbol-fasta

Pure-Rust FASTA → SBOL 3 importer for the `sbol-rs` workspace.

FASTA is the lowest-common-denominator sequence exchange format:
NCBI BLAST, UniProt downloads, every genome project, and most
bioinformatics tools either emit or accept it. This crate lets
`sbol-rs` ingest that data with zero new transitive dependencies.

Each `>header` record becomes one `sbol::Component` paired with one
`sbol::Sequence`. The component's biological type (DNA / RNA /
protein) and the sequence's EDAM encoding are auto-detected from the
alphabet of the sequence itself; the detection can be overridden
with `FastaImporter::with_alphabet` when the data is ambiguous.

FASTA carries no feature annotations: what you get back is a
Component with no `SequenceFeature`s. For annotated data, use
[`sbol-genbank`](../sbol-genbank/) instead.

## Quickstart

```rust
use sbol_fasta::FastaImporter;

let (document, report) =
    FastaImporter::new("https://example.org/lab")?.read_path("genome.fasta")?;

println!(
    "{} component(s) ({} DNA, {} RNA, {} protein)",
    report.components, report.dna_records, report.rna_records, report.protein_records
);
document.check()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## CLI

```sh
sbol import-fasta genome.fasta \
  --namespace https://example.org/lab \
  --to turtle \
  -o genome.ttl \
  --validate

# Override alphabet detection for ambiguous sequences:
sbol import-fasta peptide.fasta \
  --namespace https://example.org/lab \
  --alphabet protein \
  --to turtle -o peptide.ttl
```

Accepted extensions: `.fasta`, `.fa`, `.fna`, `.faa`.

## Alphabet detection

| Heuristic | Result |
|---|---|
| Sequence contains `U` or `u` | RNA |
| Sequence contains protein-only letters (`E`, `F`, `I`, `L`, `P`, `Q`, `Z`) | Protein |
| Anything else | DNA |

This handles the ambiguous case of FASTA files whose sequence is
pure `A`/`C`/`G`/`T`: these are also valid protein letters, but in
practice overwhelmingly mean DNA. Override with `--alphabet protein`
on the CLI or `.with_alphabet(Alphabet::Protein)` in the SDK when
the data is genuinely a peptide.

## Dependencies

`sbol-fasta` does not depend on a third-party FASTA parser; the
~100-line parser lives in
[`src/parser.rs`](src/parser.rs). Compared to pulling in
`noodles-fasta`, this saves four transitive dependencies (`bstr`,
`memchr`, `noodles-bgzf`, `noodles-core`) for what amounts to "split
on `>`, concatenate continuation lines."
