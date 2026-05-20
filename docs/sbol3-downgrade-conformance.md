# SBOL 3 → SBOL 2 downgrade conformance

How the `sbol::downgrade` module and the `sbol downgrade` subcommand
are gated in CI. For the conversion model itself — what gets preserved,
what intentionally diverges, how dual-role Components split — see
[`conversion.md`](conversion.md).

## Gates

### Round-trip gate (the high-leverage check, pure Rust)

For every committed SBOL 2 fixture in `tests/fixtures/sbol2/`:

```text
SBOL 2 (committed) ──upgrade──> SBOL 3 ──downgrade──> SBOL 2 (reconstructed)
                                                         │
                                                         ▼
                          diff against original (modulo expected loss)
```

If both directions are correct, the SBOL 2 we produce matches the
SBOL 2 we started with. Triples that intentionally don't survive a
round-trip (see [Known intentional divergences in `conversion.md`](conversion.md#known-intentional-divergences))
are documented and allow-listed by the fixture diff.

This is a stronger gate than self-snapshot alone — it tests both
directions simultaneously. A change in either that breaks symmetry
shows up here.

### Self-snapshot gate (critical, pure Rust)

[`crates/sbol/tests/downgrade.rs`](../crates/sbol/tests/downgrade.rs)
fires every round-trip and identity-restoration check, plus the
native-SBOL-3 dual-role-split tests that exercise the
`sbol::downgrade` machinery without an SBOL 2 source.

## Fixture corpus

The downgrade reuses the SBOL 2 fixture corpus described in
[`sbol2-upgrade-conformance.md`](sbol2-upgrade-conformance.md):
SBOLTestSuite, SynBioHub, and GenBank-derived intermediates. The
round-trip is meaningful only for cases where the SBOL 2 source
exists, so no separate fixture set is needed.

Native-SBOL-3-only behavior (dual-role Component splits, Collection
membership duplication, SubComponent triple-emit, Interface routing, and
ComponentInstance default synthesis) is exercised by the
unit tests at `crates/sbol/tests/downgrade.rs` rather than via fixtures,
because the corpus would have to invent SBOL 3 designs from scratch.

## Refreshing the GenBank → SBOL 2 intermediates

The `tests/fixtures/sbol2/real/from_genbank/*.xml` intermediates are
regenerated entirely in Rust:

```sh
cargo run -p sbol-genbank --bin regenerate-from-genbank-sbol2-intermediates
```

Pipeline:

```text
tests/fixtures/genbank/{name}.gb
      │   sbol_genbank::GenbankImporter
      ▼
SBOL 3 Document
      │   sbol::downgrade::sbol3_to_sbol2
      ▼
SBOL 2 RDF/XML
      ▼
tests/fixtures/sbol2/real/from_genbank/{name}.xml
```

After regenerating, refresh the upgrade conformance snapshots so the
harness gates on the new shapes:

```sh
cargo run -p sbol --bin regenerate-sbol2-upgrade-snapshots
```

## Empirical round-trip coverage

[`sbol3-round-trip-report.md`](sbol3-round-trip-report.md) — generated
by `cargo run -p sbol --bin generate-round-trip-report` — runs the full
`upgrade → downgrade → re-upgrade` pipeline against every committed
real-world fixture and diffs the triple sets. The current
headline (21 / 21 clean) covers SBOL 2 → 3 → 2 over the SBOL 2 corpus;
native-SBOL-3 → 2 dual-role behavior is exercised by unit tests, not by
this report.

Use the report to decide whether a structural case actually appears in
real data before investing in deeper coverage.

## `--validate` semantics

`sbol-rs` doesn't bundle an SBOL 2 validator. The `--validate` flag on
`sbol downgrade` runs a **round-trip check** instead: the downgraded
SBOL 2 is upgraded back through `sbol::upgrade`, and the resulting
SBOL 3 document is run through `Document::validate`. This proves
structural correctness — if the downgrade preserved enough information
for the upgrade to rebuild a valid SBOL 3 document, the SBOL 2 itself
is well-formed enough for any SBOL 2 consumer.

For strict SBOL 2 spec compliance, serve the file to libSBOLj2 or
pySBOL2 externally.

## When the conformance test fails

- **Round-trip drift on a simple CD** → either the upgrade or the
  downgrade lost a triple. Read the diff; fix the side that's wrong.
  Add an allowlist entry only if the divergence is intentional and
  documented in [`conversion.md`](conversion.md#known-intentional-divergences)
  (e.g. BIOPAX `Dna` ↔ `DnaRegion`).
- **`DualRoleComponent` warning fires on a fixture that should be
  single-shape** → the classifier in `crates/sbol/src/downgrade/mod.rs`
  is over-counting structural or functional signals. Check whether the
  fixture triggers a signal that should be filtered (e.g. a
  ComponentReference that's actually a MapsTo back-half).
- **`OrphanComponentReference` warning** → the downgrade saw a
  ComponentReference without a paired Constraint. Either the source
  document was already malformed or the upgrade emitted an unpaired
  CRef.
- **`UnsupportedSbol3Type` warning** → a subject's `rdf:type` was an
  SBOL 3-only class with no SBOL 2 equivalent. `ComponentReference`
  and `Interface` are normally folded into SBOL 2 `MapsTo` and
  `sbol2:direction` triples by the structural re-synthesis passes;
  warnings only fire for orphans the pairing couldn't reverse.

## File layout

```
crates/sbol/src/downgrade/
├── mod.rs                # engine: preflight, dispatch, classifier, emit passes
└── values.rs             # reverse enumerated-value maps

crates/sbol/tests/
└── downgrade.rs          # round-trip + identity-restoration + dual-role gates

crates/sbol-genbank/src/bin/
└── regenerate-from-genbank-sbol2-intermediates.rs   # pure-Rust GenBank → SBOL 2 pipeline
```

## Related

- [Conversion guide](conversion.md) — user-facing reference for the
  conversion model: workflows, the backport namespace, dual-role
  Component splits, known divergences, known limitations.
- [SBOL 2 → SBOL 3 upgrade conformance](sbol2-upgrade-conformance.md)
  — the inverse direction's CI gate. The two share the
  `crates/sbol/src/sbol2_vocab.rs` vocabulary and the
  `http://sboltools.org/backport#` namespace.
- [GenBank → SBOL 3 import conformance](genbank-import-conformance.md)
  — pure-Rust GenBank reader; feeds the GenBank-derived round-trip
  fixtures.
- [Validation system overview](validation.md) — the post-round-trip
  spec-compliance gate behind `sbol downgrade --validate`.
