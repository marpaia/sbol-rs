# FASTA → SBOL 3 import conformance

The [`sbol-fasta`](../crates/sbol-fasta/) crate and the
`sbol import-fasta` CLI subcommand read FASTA files (`.fasta` /
`.fa` / `.fna` / `.faa` — the de facto sequence-exchange format used
by NCBI, UniProt, BLAST, and every alignment tool) directly into
native SBOL 3 documents. The integration test in
[`crates/sbol-fasta/tests/conformance.rs`](../crates/sbol-fasta/tests/conformance.rs)
gates every change against committed snapshots of the importer's
output.

## Gate

### Self-snapshot gate (critical, pure Rust)

For every `.fasta` / `.fa` / `.fna` / `.faa` file in
`tests/fixtures/fasta/`, the test re-runs the importer with a
per-fixture namespace and diffs the deterministically sorted
N-Triples output against the committed snapshot at
`tests/fixtures/fasta/expected/{name}.nt`. Any unintended change in
importer output fails the gate.

Both sides of the comparison come from `sbol-fasta`. The gate proves
the importer is **stable and deterministic** — that the same FASTA
input always produces the same SBOL 3 output, byte for byte. It
runs in pure Rust, with no network, no Docker, and no other tooling
required.

`sbol-fasta` is the canonical Rust FASTA → SBOL 3 implementation in
this workspace. There is no separate "reference engine" for this
path — the self-snapshot gate is the whole conformance story.

## Fixture corpus

`tests/fixtures/fasta/` holds committed FASTA source files. Each
file's provenance is recorded in
[`tests/fixtures/fasta/README.md`](../tests/fixtures/fasta/README.md).

Current corpus:

| Fixture | Description |
|---|---|
| `pUC19.fasta` | NCBI `M77789.2` — canonical DNA cloning vector, 2.7 kb |
| `pBR322.fasta` | NCBI `J01749.1` — second canonical DNA cloning vector, 4.4 kb |
| `GFP_protein.fasta` | UniProt `P42212` — GFP protein, single record |
| `multi_protein.fasta` | UniProt `P42212` + `P03023` — multi-record (exercises the record-loop and dedup logic) |

Adding a new fixture: drop the file under
`tests/fixtures/fasta/`, run the regen binary, and commit both the
source and the resulting `expected/{name}.nt`.

## What the test exercises

The current corpus drives every importer code path:

- **DNA records.** `pUC19`, `pBR322` — exercise the default alphabet
  detection (pure `A`/`C`/`G`/`T`/`N`), the `SBO:0000251` mapping,
  and `EDAM:format_1207` (IUPAC DNA) encoding.
- **Protein records.** `GFP_protein` — exercises protein-only-letter
  detection (`F`, `L`, `P`, etc.), the `SBO:0000252` mapping, and
  `EDAM:format_1208` (IUPAC protein) encoding.
- **Multi-record files.** `multi_protein` — multiple records in one
  file, header-id de-duplication, multi-component SBOL 3 output.
- **NCBI / UniProt dialect headers.** Both NCBI (`>accession.version
  description`) and UniProt (`>sp|accession|name OS=…`) defline
  styles are exercised.

The smaller SDK tests in
[`crates/sbol-fasta/tests/import.rs`](../crates/sbol-fasta/tests/import.rs)
cover individual behaviors (RNA detection via `U`, forced alphabet
override, empty-record warning, duplicate-id deduplication, Turtle
round-trip, alternate extensions `.fa` / `.fna` / `.faa`).

## Refreshing snapshots

After an intentional importer change:

```sh
cargo run -p sbol-fasta --bin regenerate-fasta-import-snapshots
```

That refreshes every `tests/fixtures/fasta/expected/{name}.nt`.
Pure Rust; no external tooling required.

## When the conformance test fails

- **Self-snapshot drift** → either a regression, or an intentional
  importer change.
  - if unwanted, revert it
  - if intentional, run the refresh command above and commit the
    refreshed `expected/*.nt` alongside the code change.

The failure message includes a per-fixture +/- diff: triples
present only in the new output appear as `+ …`; triples present
only in the committed snapshot appear as `- …`. Ordering-only
differences are flagged separately.

## File layout

```
tests/fixtures/fasta/
├── README.md             # provenance for every committed FASTA
├── pUC19.fasta           # ┐
├── pBR322.fasta          # │  source corpus
├── GFP_protein.fasta     # │
├── multi_protein.fasta   # ┘
└── expected/             # self-snapshot references (canonical)
    └── *.nt              #   one per source fixture
```

## Related

- [GenBank import conformance](genbank-import-conformance.md) — the
  sibling harness for the GenBank → SBOL 3 path via `sbol-genbank`.
  FASTA and GenBank are different formats (FASTA carries sequence
  only; GenBank carries sequence + annotations), so they have
  separate crates and separate gates.
- [SBOL 2 → SBOL 3 upgrade conformance](sbol2-upgrade-conformance.md)
  — the upgrade path for documents already in SBOL 2 RDF.
- [Validation system overview](validation.md) — the post-import
  spec-compliance gate. `sbol import-fasta --validate` composes
  the two.
