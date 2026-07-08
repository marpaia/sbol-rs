# SBOL-Converter differential conformance

How the `sbol-convert` upgrade and downgrade are checked for **behavioral
parity** with the reference Java converter, [SynBioDex/SBOL-Converter][ref].
This is the strongest converter gate: it verifies sbol-rs makes the same
conversion decisions as the reference on identical input, in both directions.

For the conversion model itself (what is preserved, what intentionally
diverges), see [`conversion.md`](conversion.md). For the pure-Rust round-trip
and self-snapshot gates, see [`sbol2-upgrade-conformance.md`](sbol2-upgrade-conformance.md)
and [`sbol3-downgrade-conformance.md`](sbol3-downgrade-conformance.md).

Test: [`crates/sbol-convert/tests/cross_impl_reference.rs`](../crates/sbol-convert/tests/cross_impl_reference.rs).
CI runs only `cargo test`; no Docker, no JDK.

## The matrix

For each committed fixture the test runs four comparisons:

```text
same-direction agreement (GATED)
  SBOL 2 ──reference──> SBOL 3   vs   SBOL 2 ──sbol-rs──> SBOL 3
  SBOL 3 ──reference──> SBOL 2   vs   SBOL 3 ──sbol-rs──> SBOL 2

cross-tool round-trip
  SBOL 2 ──reference──> SBOL 3 ──sbol-rs──> SBOL 2   vs   original  (GATED)
  SBOL 3 ──reference──> SBOL 2 ──sbol-rs──> SBOL 3   vs   original  (REPORTED)
```

**Same-direction agreement** is the parity metric: given identical input, does
sbol-rs produce the same graph the reference does? Both directions are gated to
logical equivalence (see below).

**Cross-tool round-trips** feed one tool's output through the other and back:

- `ref-up→rs-down` (SBOL 2 → reference SBOL 3 → sbol-rs SBOL 2) is **gated**:
  sbol-rs's downgrade of the reference's exact SBOL 3 must reproduce the
  original SBOL 2.
- `ref-down→rs-up` (SBOL 3 → reference SBOL 2 → sbol-rs SBOL 3) is **reported,
  not gated**. It cannot reach equivalence: SBOL 2 cannot represent a feature's
  orientation or a FunctionalComponent's locations, so the reference's own
  `3 → 2` step discards them (both tools agree on that SBOL 2 — the `3 → 2`
  agreement matches). Re-upgrading cannot restore what the SBOL 2 model does
  not hold; the residual is that inherent model-expressiveness gap, not an
  sbol-rs ↔ reference divergence.

## Logical equivalence, not byte-identity

Comparison normalizes away differences that are semantically irrelevant, so a
real structural difference still fails while cosmetic ones don't:

- **Auto-generated `Constraint<N>` IRIs are content-addressed.** The reference
  numbers the Constraints it mints for MapsTo decompositions in its Java
  `HashSet` iteration order, so which ordinal a given Constraint receives
  carries no meaning. Each is keyed by its `restriction` / `subject` / `object`
  instead, so a permutation of the labels compares equal.
- **Round-trip comparisons also** drop conversion provenance (the
  `https://sbols.org/backport/2_3#` annotations, which are metadata sbol-rs
  stamps, never source data) and collapse version-equivalent references
  (`.../X/1.0` ≡ `.../X`, so a reference to an object's persistent identity
  matches a reference to its single concrete version).

The `logical` and `logical_roundtrip` helpers in the test implement this.

## Fixtures

Committed reference outputs live under
[`tests/fixtures/cross-impl-sbolconverter/expected/`](../tests/fixtures/cross-impl-sbolconverter/expected/):

| Fixture | Source | Direction |
| --- | --- | --- |
| `component_definition_output` | SBOLTestSuite `SBOL2/ComponentDefinitionOutput.xml` | `.to-sbol3.nt` |
| `module_definition_output` | SBOLTestSuite `SBOL2/ModuleDefinitionOutput.xml` | `.to-sbol3.nt` |
| `sequence_constraint_output` | SBOLTestSuite `SBOL2/SequenceConstraintOutput.xml` | `.to-sbol3.nt` |
| `repression_model` | SBOLTestSuite `SBOL2/RepressionModel.xml` | `.to-sbol3.nt` |
| `bba_f2620` | SBOLTestSuite `SBOL3/BBa_F2620_PoPSReceiver` | `.to-sbol2.rdf` |
| `model` | SBOLTestSuite `SBOL3/entity/model` | `.to-sbol2.rdf` |

The `.to-sbol3.nt` files are the reference's SBOL 2 → SBOL 3 output; the
`.to-sbol2.rdf` files are its SBOL 3 → SBOL 2 output. sbol-rs's own conversions
are computed in-process at test time.

## Current status

All gated comparisons are logically equivalent to the reference:
same-direction agreement (both directions, all six fixtures) and
`ref-up→rs-down` (all four SBOL 2 fixtures). The only reported residual is
`ref-down→rs-up` on `bba_f2620` and `model`, which is the inherent SBOL 2
model-expressiveness gap described above.

## Refreshing references after a SBOL-Converter version bump

The goldens are regenerated on a contributor machine (never in CI):

```sh
# build the pinned image (one-time per version bump)
docker build -t sbolconverter-pinned tests/fixtures/cross-impl-sbolconverter/

# regenerate every committed reference output
cargo run -p sbol-convert --bin regenerate-cross-impl-reference
```

The image (`tests/fixtures/cross-impl-sbolconverter/Dockerfile`) pins the
`SBOLCONVERTER_TAG` and pulls the public GitHub Releases fat jar (Java 21
bytecode, so `eclipse-temurin:21-jdk`), avoiding any Maven / GitHub-Packages
authentication. The Java wrapper
(`tests/fixtures/cross-impl-sbolconverter/Convert.java`) is a bidirectional
`<input> <to-sbol3|to-sbol2> <format>` shim with save-time validation disabled
so the reference emits even non-conformant intermediates. Commit the
regenerated `expected/*` files and any `SBOLCONVERTER_TAG` bump together.

[ref]: https://github.com/SynBioDex/SBOL-Converter
