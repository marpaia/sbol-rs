# sbol-rs documentation

Entry points to the project's documentation, organized by topic.

## Getting oriented

- **[Crate guide](crate-guide.md)**: architectural tour covering
  workspace layout, data model, document lifecycle, validation
  pipeline, conversion machinery, key decision points. **Start here
  if you're new to the codebase.**

## Validation

The validator covers SBOL 3.1.0 with structured diagnostics and a
coverage signal. Read these in order:

- **[Validation system overview](validation.md)** — narrative entry
  point. What the validator does, how to use it, how to read its
  output, where the trust boundaries are.
- [Conformance report](conformance.md) — generated per-rule status
  grid for all 149 spec rules. Headline: 109/109 machine-checkable
  rules implemented. Regenerate with
  `cargo run -p sbol --bin generate-conformance-report`.
- [JSON v1 output schema](validation-output.md) — reference for
  downstream tools consuming `sbol validate --format json`. Includes
  the SARIF v2.1.0 mapping.
- [Policy ADRs](policies/) — committed design decisions for rules
  where the spec is ambiguous. One ADR per `Policy`-blocked rule.

## I/O

- [RDF I/O](rdf-io.md) — Turtle, RDF/XML, JSON-LD, N-Triples support;
  format inference; round-trip guarantees.

## Conversion

`sbol-rs` ingests three flavors of upstream synbio data and converts
them to native SBOL 3, and reverses the SBOL 3 path back to SBOL 2
for publishing to consumers that still consume the older format.

- **[Conversion guide](conversion.md)** — the canonical reference.
  Workflows organized by what you have (SBOL 2, GenBank, FASTA,
  native SBOL 3), the `http://sboltools.org/backport#` namespace and
  what it preserves, the dual-role Component split for designs that
  combine structure and function, known intentional divergences,
  known limitations.

| You have… | Use… | Module / subcommand |
|---|---|---|
| SBOL 2 RDF (SynBioHub, iGEM, JBEI ICE) | `sbol::upgrade` | `sbol upgrade` |
| GenBank `.gb` / `.gbk` (SnapGene, ApE, Benchling, NCBI) | [`sbol-genbank`](../crates/sbol-genbank/) | `sbol import-genbank` |
| FASTA `.fasta` / `.fa` / `.fna` / `.faa` (NCBI, UniProt, BLAST) | [`sbol-fasta`](../crates/sbol-fasta/) | `sbol import-fasta` |
| SBOL 3 to publish to a tool that consumes SBOL 2 | `sbol::downgrade` | `sbol downgrade` |

For maintainers and CI:

- [SBOL 2 → SBOL 3 upgrade conformance](sbol2-upgrade-conformance.md)
  — self-snapshot gate (critical, pure Rust). How the test harness
  detects converter drift; how to refresh snapshots after an
  intentional change.
- [SBOL 3 → SBOL 2 downgrade conformance](sbol3-downgrade-conformance.md)
  — round-trip gate (critical, pure Rust) that pairs with the
  upgrade gate.
- [SBOL 3 round-trip smoke test report](sbol3-round-trip-report.md)
  — generated per-fixture report of what survives upgrade →
  downgrade → re-upgrade across the full SBOL 2 fixture corpus.
  Regenerate with `cargo run -p sbol --bin generate-round-trip-report`.
- [GenBank → SBOL 3 import conformance](genbank-import-conformance.md)
  — self-snapshot gate for `sbol-genbank`.
- [FASTA → SBOL 3 import conformance](fasta-import-conformance.md)
  — self-snapshot gate for `sbol-fasta`.

## Ontology extensions

- [Ontology extensions](ontology-extensions.md) — the bundled
  snapshot (EDAM, SBO, SO, GO, ChEBI, CL), the runtime cache for
  opt-in extensions (NCIT, custom domain ontologies), and the install
  workflow.

## Testing and contribution

- [Test architecture](testing.md) — regression cases, fixtures, fuzz
  targets, property tests, cross-implementation conformance harness.
- [SBOL schema conformance regression](ontology-conformance.md) —
  pinned `sbol-owl3` fixture, the IRI-level cross-check against
  `vocab.rs`, and how to refresh the pin.
- [`sbol-owl3` conformance report](sbol-owl3-conformance.md) —
  generated per-IRI coverage table. Regenerate with
  `cargo run -p sbol --bin generate-sbol-owl3-conformance-report`.
