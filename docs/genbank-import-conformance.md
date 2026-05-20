# GenBank → SBOL 3 import conformance

The [`sbol-genbank`](../crates/sbol-genbank/) crate and the
`sbol import-genbank` CLI subcommand read GenBank flat-files (`.gb` /
`.gbk` — the format emitted by SnapGene, ApE, Benchling, NCBI, and
SynBioHub) directly into native SBOL 3 documents. The integration test
in [`crates/sbol-genbank/tests/conformance.rs`](../crates/sbol-genbank/tests/conformance.rs)
gates every change against committed snapshots of the importer's
output.

## Gate

### Self-snapshot gate (critical, pure Rust)

For every `.gb` / `.gbk` file in `tests/fixtures/genbank/`, the test
re-runs the importer with a per-fixture namespace and diffs the
deterministically sorted N-Triples output against the committed
snapshot at `tests/fixtures/genbank/expected/{name}.nt`. Any
unintended change in importer output fails the gate.

Both sides of the comparison come from `sbol-genbank`. The gate
proves the importer is **stable and deterministic** — that the same
GenBank input always produces the same SBOL 3 output, byte for byte.
It runs in pure Rust, with no network, no Docker, and no other
tooling required.

`sbol-genbank` is the canonical Rust GenBank → SBOL 3 implementation
in this workspace. There is no separate "reference engine" for this
path — the self-snapshot gate is the whole conformance story.

## Fixture corpus

`tests/fixtures/genbank/` holds committed GenBank source files. Each
file has provenance recorded in
[`tests/fixtures/genbank/README.md`](../tests/fixtures/genbank/README.md).

Current corpus:

| Fixture | Description |
|---|---|
| `BBa_E0040.gb` | GFP CDS (Aequorea victoria) — canonical iGEM "hello world" |
| `BBa_R0010.gb` | LacI-repressible promoter with multiple annotated features |
| `BBa_B0034.gb` | Elowitz RBS — minimal single-feature design |
| `BBa_F2620.gb` | PoPS receiver — composite design with 30 features, duplicate labels |
| `pUC19.gbk` | NCBI `M77789.2` — the canonical cloning vector. `.gbk` extension exercises the SnapGene/community naming convention (NCBI uses `.gb`, SnapGene uses `.gbk`; both are the same flat-file format). |

Adding a new fixture: drop the `.gb` file under
`tests/fixtures/genbank/`, run the regen binary, and commit both the
source and the resulting `expected/{name}.nt`.

## What the test checks

### `self_snapshot_diff`

For each fixture:

1. Read the input from `tests/fixtures/genbank/{name}.gb`.
2. Call [`GenbankImporter::read_path`](../crates/sbol-genbank/src/importer.rs)
   with namespace `https://sbol-rs.example.org/genbank/{name}`.
3. Sort, dedup, and N-Triples-canonicalize the resulting graph.
4. Diff line-by-line against `tests/fixtures/genbank/expected/{name}.nt`.

Any difference fails the test and prints a per-fixture +/- diff with
the refresh command in the failure message.

## What the test exercises

The current corpus drives every importer code path:

- **Simple records.** `BBa_E0040`, `BBa_B0034` — one Component, one
  Sequence, a single feature with one location. Sanity check that
  the basic pipeline produces validating SBOL 3.
- **Multi-feature designs.** `BBa_R0010` — seven annotated features
  on a single Sequence, each with its own SO role mapping. Exercises
  the feature-key → SO term table.
- **Composite designs.** `BBa_F2620` — 30 features including
  duplicate `/label` qualifiers. Exercises the display-ID
  de-duplication logic and confirms IRI compliance holds at scale.
- **Circular topology + NCBI dialect.** `pUC19.gbk` — circular
  cloning vector from NCBI with uppercase-month LOCUS line and the
  `.gbk` extension. Pins both the SO topology mapping
  (`SO:0000988 circular`) and the `.gbk` file-extension handling.
- **SynBioHub dialect quirks.** The iGEM fixtures all come from
  SynBioHub's GenBank export, which emits mixed-case month names
  on the LOCUS line (`20-May-2026`). Exercises the
  [`normalize_genbank_input`](../crates/sbol-genbank/src/importer.rs)
  pre-pass that tolerates this.

The smaller SDK tests in
[`crates/sbol-genbank/tests/import.rs`](../crates/sbol-genbank/tests/import.rs)
cover individual behaviors (unknown feature keys, Turtle round-trip,
mixed-case month tolerance) with synthetic GenBank inputs.

## Refreshing snapshots

After an intentional importer change:

```sh
cargo run -p sbol-genbank --bin regenerate-genbank-import-snapshots
```

That refreshes every `tests/fixtures/genbank/expected/{name}.nt`.
Pure Rust; no external tooling required.

## When the conformance test fails

- **Self-snapshot drift** → either a regression, or an intentional
  importer change.
  - if unwanted, revert it
  - if intentional, run
    `cargo run -p sbol-genbank --bin regenerate-genbank-import-snapshots`
    and commit the refreshed `expected/*.nt` alongside the code
    change.

The failure message includes a per-fixture +/- diff. Triples
present only in the new output appear as `+ …`; triples present
only in the committed snapshot appear as `- …`. Ordering-only
differences are flagged separately.

## File layout

```
tests/fixtures/genbank/
├── README.md                 # provenance for every committed .gb
├── BBa_E0040.gb              # ┐
├── BBa_R0010.gb              # │  source corpus
├── BBa_B0034.gb              # │
├── BBa_F2620.gb              # ┘
└── expected/                 # self-snapshot references (canonical)
    └── *.nt                  #   one per source fixture
```

## Runnable example

[`crates/sbol-genbank/examples/genbank_to_sbol3.rs`](../crates/sbol-genbank/examples/genbank_to_sbol3.rs)
runs the four-stage pipeline end-to-end — parse the `.gb` file with
`gb-io`, import into a native `sbol::Document`, validate against the
SBOL 3.1.0 spec rules, serialize to disk in the user-chosen RDF
format, and then round-trip that file back through
`Document::read_path` to confirm the graph is stable.

```sh
# Defaults to a fixture + Turtle to stdout.
cargo run -p sbol-genbank --example genbank_to_sbol3

# Point at any GenBank file and write SBOL 3 to disk.
cargo run -p sbol-genbank --example genbank_to_sbol3 \
  -- tests/fixtures/genbank/BBa_F2620.gb /tmp/BBa_F2620.ttl
```

## Related

- [SBOL 2 → SBOL 3 upgrade conformance](sbol2-upgrade-conformance.md)
  — the sibling harness for the SBOL 2 RDF input path.
- [Validation system overview](validation.md) — the post-import
  spec-compliance gate. `sbol import-genbank --validate` composes
  the two.
