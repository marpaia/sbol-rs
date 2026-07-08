# Testing architecture

`sbol-rs` is correctness-critical: every claim about SBOL conformance,
for both SBOL 2 and SBOL 3, has to survive `cargo test`. This document
maps the test surface so contributors know what each layer gates, where
to add a new case, and how the fixture corpora are produced.

## Test layers at a glance

| Layer                              | Where                                                   | What it gates                                                                       |
| ---------------------------------- | ------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Unit tests                         | Inline `#[cfg(test)]` mods (minimal; most logic moves to integration tests) | Type-local invariants too small to integrate. |
| SBOL 3 integration tests           | `crates/sbol3/tests/*.rs`                               | Public-API behavior: builders, traversal, parsing, validation entry points.         |
| SBOL 2 integration tests           | `crates/sbol2/tests/*.rs`                               | The SBOL 2 peer: builders, client API, RDF round-trip, validation.                  |
| Validation rule regressions        | `crates/sbol3/tests/rule_cases/{spec-area}.rs`, `crates/sbol2/tests/rule_cases.rs` | One negative case per non-deferred rule ID; positive cases per spec cluster.        |
| Schema consistency                 | `crates/sbol3/tests/schema_consistency.rs`              | `to_rdf` ↔ `from_rdf` ↔ `FieldDescriptor` predicate-set parity across every class.  |
| Property tests                     | `crates/sbol3/tests/properties.rs`                      | Spec-derived invariants under `proptest`-generated input.                           |
| SBOL 3 fixtures                    | `crates/sbol3/tests/sbol3_fixtures.rs`                  | All 33 valid SBOL 3 fixtures parse, validate, and round-trip in every RDF format.   |
| SBOL 2 corpora                     | `crates/sbol2/tests/conformance.rs`, `crates/sbol2/tests/invalid_files.rs` | The four SBOLTestSuite SBOL 2 corpora and the per-rule negative corpus.             |
| Migration round-trip               | `crates/sbol-convert/tests/*.rs`                        | SBOL 2 → 3 upgrade, SBOL 3 → 2 downgrade, 2 → 3 → 2 fixed-point over the full corpus, and a differential gate against the SynBioDex/SBOL-Converter reference. |
| Importer oracles                   | `crates/sbol-fasta/tests/*.rs`, `crates/sbol-genbank/tests/*.rs` | FASTA / GenBank import against SO-mapping and BioPython reference oracles.           |
| Cross-implementation conformance   | `crates/sbol3/tests/cross_impl.rs`, `crates/sbol3/tests/cross_impl_pysbol3.rs`, `crates/sbol2/tests/cross_impl.rs` | Triple-set equivalence against libSBOLj3 1.0.5.2 and pySBOL3 1.2 (132 combos each), plus SBOL 2 RDF/XML equivalence against libSBOLj 2.4.0. |
| Catalog freshness gates            | `crates/sbol3/tests/validation_rules.rs`, `crates/sbol3/tests/conformance_report.rs`, `crates/sbol2/tests/conformance_report.rs` | `rules.toml` invariants and `docs/conformance.md` / `docs/sbol2-conformance.md` freshness. |
| RDF backend                        | `crates/sbol-rdf/tests/basic.rs`                        | Per-format round-trip, cross-format equivalence, extension parsing.                 |
| Fuzz tests                         | `fuzz/fuzz_targets/*.rs`                                | No panics on arbitrary bytes in any of the four RDF formats.                        |
| Doctests                           | `///` examples across the crates                        | README / prelude / `Document::read*` examples compile and run.                      |

Run everything: `cargo test --workspace`. No Docker or JDK is needed; the
cross-implementation references are committed.

## Validation rule architecture

Each version carries its own validation catalog with the same shape;
testing it has structure beyond "a `#[test]` per rule."

### Rule catalog (source of truth)

`crates/sbol3/rules.toml` (149 rules) and `crates/sbol2/rules.toml` (268
rules) are the single sources of truth for the SBOL 3.1.0 and SBOL 2.3.0
validation rules. Each entry carries:

- `id`: the canonical SBOL rule ID (e.g. `sbol3-10101`, `sbol2-10101`).
- `status`: one of `Error`, `Warning`, `Configurable`,
  `MachineUncheckable` (spec ▲), or `Unimplemented`.
- `normative_severity`: RFC 2119 force (`MUST` / `SHOULD` / `MAY`).
- `spec_section`: section reference into the version's spec.
- `blocker` (Configurable / MachineUncheckable / Unimplemented): `Ontology`
  / `Resolver` / `Policy` / `External` / `StrictDatatype`. Names the
  configuration axis or what a fuller check would need.
- `gate`: the validation family (`Always`, `Compliant`, `Complete`, or
  `BestPractice`) selecting which `ValidationConfig` flag runs the rule.
- `validator_function` / `note`: the emitting function (where applicable)
  and a short description of what is enforced.

Each crate's `build.rs` parses its TOML at compile time via
[`sbol-rulegen`](../crates/sbol-rulegen/) and emits the
`validation_rule_statuses()` slice into `OUT_DIR`. Adding a rule means
editing the TOML, not Rust. Both catalogs classify every machine-checkable
rule as implemented: 109 of 109 for SBOL 3, 239 of 239 for SBOL 2.

### Drift gates and rule-case suites

`crates/sbol3/tests/validation_rules.rs` locks the SBOL 3 catalog against
silent drift. Its meta-tests include
`implemented_validation_rules_have_regression_cases` (every implemented
rule has a negative case under `tests/rule_cases/`),
`validation_rule_regression_cases_report_expected_rule_ids` (each case
fires its claimed rule ID), `positive_rule_cases_do_not_report_their_rule`,
`every_rule_appears_in_exactly_one_coverage_bucket`,
`every_appendix_b_triangle_rule_is_marked_machine_uncheckable`, and
`every_policy_blocked_rule_has_an_adr_file`. Each spec area
(`component.rs`, `feature.rs`, …) holds a `&[RuleCase]` array declaring a
rule ID, a Turtle document that must fire it, and the expected `Severity`.

The SBOL 2 peer is `crates/sbol2/tests/rule_cases.rs`: hermetic in-memory
documents that isolate a single rule with a positive fixture (must not
report it) and a negative fixture (must). To add coverage, edit the
version's `rules.toml`, add the case, and run
`cargo test -p sbol3 --test validation_rules` (or
`cargo test -p sbol2 --test rule_cases`).

### Conformance report freshness

The per-rule status grids are generated from the catalogs and gated in CI:

```sh
cargo run -p sbol3 --bin generate-conformance-report        # docs/conformance.md
cargo run -p sbol2 --bin generate-sbol2-conformance-report  # docs/sbol2-conformance.md
cargo run -p sbol3 --bin generate-sbol-owl3-conformance-report  # docs/sbol-owl3-conformance.md
```

`crates/sbol3/tests/conformance_report.rs` and
`crates/sbol2/tests/conformance_report.rs` fail if the committed file
drifts from the rendered output.

## SBOL 2 corpus conformance

`crates/sbol2/tests/conformance.rs` validates the four SBOLTestSuite SBOL 2
corpora, each isolating one axis of the shared `ValidationConfig`: the
`SBOL2` corpus (179 files) passes under both the default and `all_on`; the
`SBOL2_ic` corpus (71) fails only under `complete`; `SBOL2_nc` (30) fails
only under `compliant`; `SBOL2_bp` (11) is flagged only under
`best_practice`, at warning severity. Every file parses: **0 parse
failures** across all four corpora.

`crates/sbol2/tests/invalid_files.rs` runs the per-rule negative corpus:
each `InvalidFiles/sbol-NNNNN.xml` violates rule `sbol2-NNNNN` and must be
rejected under `all_on`. The current split is 161 strict (the exact rule
fires), 17 loose (rejected via a related error), 3 parse-rejected, and 50
deferred (resolver / cross-document, spec-▲, or SHOULD-level cases tracked
explicitly in `invalid_files_deferred.in`). The full grid is rendered into
[`sbol2-conformance.md`](sbol2-conformance.md).

## Migration round-trip

`crates/sbol-convert/tests/` gates the conversion pipeline both ways:

- `upgrade_conformance.rs` / `upgrade_fixtures.rs`: SBOL 2 → SBOL 3
  against self-snapshots (see
  [`sbol2-upgrade-conformance.md`](sbol2-upgrade-conformance.md)).
- `downgrade_round_trip.rs` / `downgrade_dual_role.rs` /
  `downgrade_semantic.rs`: SBOL 3 → SBOL 2, including the
  Component → ComponentDefinition / ModuleDefinition classification (see
  [`sbol3-downgrade-conformance.md`](sbol3-downgrade-conformance.md)).
- `corpus_round_trip.rs`: the SBOL 2 → 3 → 2 fixed-point cycle over the
  **full** SBOLTestSuite SBOL 2 corpora, asserting triple-for-triple
  equality: **290 fixtures round-trip clean**, 0 unexpected drift, 0 parse
  failures, 0 upgrade-unsupported, with a single documented-lossy fixture
  (a non-compliant-identity `SBOL2_nc` case) allowlisted.
- `cross_impl_reference.rs`: the differential gate against the reference
  Java converter (SynBioDex/SBOL-Converter) — SBOL 2 → SBOL 3 via the
  reference vs sbol-rs and SBOL 3 → SBOL 2 vice versa, plus cross-tool
  round-trips, compared on logical equivalence against committed goldens
  (no JDK in CI). See
  [`sbol-converter-differential.md`](sbol-converter-differential.md).

The informational per-fixture report over the committed real-world
fixtures is [`sbol3-round-trip-report.md`](sbol3-round-trip-report.md),
regenerated with `cargo run -p sbol-convert --bin generate-round-trip-report`.

## Importer oracles

The FASTA and GenBank importers are validated against external references
rather than only self-snapshots:

- **SO-mapping parity**: GenBank feature-key → Sequence Ontology role
  mappings are diffed against the sbol-utilities `gb2so` reference table
  (`crates/sbol-genbank/tests/`); reconciling them exposed and fixed 9 real
  mapping bugs.
- **BioPython feature-parity oracle**: `biopython_oracle.rs` in both
  `sbol-fasta` and `sbol-genbank` checks that parsed records match a
  BioPython reference decomposition.
- **Multi-span locations**: `crates/sbol-genbank/tests/multispan.rs`
  hand-verifies `join` / `order` / `complement` location fixtures.
- **Malformed input**: `crates/sbol-genbank/tests/malformed.rs` and the
  FASTA edge tests confirm graceful rejection of broken input.

See [`fasta-import-conformance.md`](fasta-import-conformance.md) and
[`genbank-import-conformance.md`](genbank-import-conformance.md).

## Cross-implementation conformance

This section covers RDF **serialization** equivalence for the `sbol3` / `sbol2`
crates. The `sbol-convert` **converter** has its own differential against the
SynBioDex/SBOL-Converter reference — see
[`sbol-converter-differential.md`](sbol-converter-differential.md).

`tests/fixtures/cross-impl/` holds libSBOLj3 1.0.5.2 reference outputs for
every SBOLTestSuite SBOL 3 fixture, one file per format:

```
<stem>.libSBOLj3.expected.ttl
<stem>.libSBOLj3.expected.rdf
<stem>.libSBOLj3.expected.jsonld
<stem>.libSBOLj3.expected.nt
```

`cargo test -p sbol3 --test cross_impl` parses each reference in its own
format and asserts the normalized triple set matches what sbol-rs produces
by serializing the same source fixture in that format. CI runs only
`cargo test`; no Docker, no JDK.

A known-compliant divergence is allowlisted by adding the reference
filename to `tests/fixtures/cross-impl/allowlist.txt` with a written
rationale. The allowlist is currently empty.

### Refreshing references after a libSBOLj3 version bump

```sh
# build the pinned Docker image (one-time per version bump)
docker build -t libsbolj3-pinned tests/fixtures/cross-impl/

# regenerate all four format references for every fixture
cargo run -p sbol3 --bin regenerate-cross-impl-expectations
```

The regenerate binary iterates `RdfFormat::ALL` per fixture and is
fail-loud about Docker availability and image existence. Commit the
regenerated `*.expected.{ttl,rdf,jsonld,nt}` files and bump the
`LIBSBOLJ3_VERSION` arg in `tests/fixtures/cross-impl/Dockerfile` in the
same commit.

The Java wrapper at `tests/fixtures/cross-impl/RoundTrip.java` takes the
output format as a CLI arg (`turtle`/`rdfxml`/`jsonld`/`ntriples`). If you
add a new format to `RdfFormat`, extend the `parseFormat` switch there and
the `libsbolj3_format_name` mapping in the regenerate binary.

### pySBOL3 harness

`tests/fixtures/cross-impl-pysbol3/` mirrors the libSBOLj3 layout for
pySBOL3 references. File suffix is `.pySBOL3.expected.*` and the test file
is `crates/sbol3/tests/cross_impl_pysbol3.rs`. The committed references
cover the same 33 fixtures in all four formats (132 combinations),
generated from pySBOL3 1.2, and `cross_impl_pysbol3` diffs sbol-rs's
serialization against them exactly as the libSBOLj3 harness does. Refresh
the references on a contributor machine:

```sh
docker build -t pysbol3-pinned tests/fixtures/cross-impl-pysbol3/
cargo run -p sbol3 --bin regenerate-cross-impl-pysbol3-expectations
```

Commit the regenerated `*.pySBOL3.expected.*` files alongside any
allowlist entries needed for known-compliant divergences and bump the
`PYSBOL3_VERSION` arg in the Dockerfile in the same commit.

### SBOL 2 harness (libSBOLj)

`crates/sbol2/tests/cross_impl.rs` gates sbol-rs's SBOL 2 serialization
against libSBOLj 2.4.0, the SBOL 2 Java implementation. SBOL 2 is exchanged
as RDF/XML, so the harness compares RDF/XML output only over 10 committed
fixtures; the reference files live in `tests/fixtures/cross-impl-sbol2/`
with suffix `.libSBOLj.expected.rdf`. Building references on a contributor
machine:

```sh
docker build -t libsbolj-pinned benches/cross-impl/libsbolj/
cargo run -p sbol2 --bin regenerate-cross-impl-sbol2-expectations
```

The `benches/cross-impl/libsbolj/RoundTrip.java` wrapper reads a fixture
and writes RDF/XML to stdout; the regenerate binary captures that as the
reference. Bump the `LIBSBOLJ_VERSION` arg in the Dockerfile when a new
release ships, then regenerate and commit.

See [`rdf-io.md`](rdf-io.md) for the user-facing I/O subsystem reference.

## Cross-implementation performance benchmarks

Separate from the correctness harness, `crates/sbol-bench` times four
phases (`parse`, `serialize`, `convert`, and `validate`) for both
SBOL versions sbol-rs implements. The matrix has an **SBOL 2** side and
an **SBOL 3** side. On the SBOL 3 side it compares sbol-rs against
pySBOL3, libSBOLj3, and sboljs; on the SBOL 2 side it compares sbol-rs
against libSBOLj (the SBOL 2 Java implementation). Every foreign tool
is pinned in its own Docker image (see `benches/cross-impl/`), and
sbol-rs runs in its own image too so every row pays the same Linux-VM
overhead.

The serialize phase covers both same-format round trips
(`turtle -> turtle`) and cross-format conversions (`turtle -> rdfxml`),
so conversion cost is measured directly. The validate phase runs for
the implementations that ship a validator on the relevant version:
sbol-rs on both versions, pySBOL3 and libSBOLj3 on SBOL 3. SBOL 2 is
exchanged as RDF/XML; sbol-rs additionally reads and writes Turtle,
JSON-LD, and N-Triples for SBOL 2, so those formats are benchmarked
for sbol-rs. This is not a CI gate (there is no perf-regression check),
but it gives contributors a reproducible way to compare
implementations head-to-head on the same fixtures. Run
`cargo run --release -p sbol-bench`; see
[`benches/cross-impl/README.md`](../benches/cross-impl/README.md) for
the version-by-version format-support matrix, knobs, and limitations.

## Property-based testing (`crates/sbol3/tests/properties.rs`)

Spec-derived invariants run under `proptest` in CI, including:

| Property                           | Asserts                                                                  |
| ---------------------------------- | ------------------------------------------------------------------------ |
| `displayId` lexical agreement      | `DisplayId::new` accepts iff the input matches `sbol3-10201`'s regex.    |
| Compliant URL round trip           | `SbolIdentity` produces identities that parse back to their components.  |
| `Range` bounds                     | `Range::new(start, end)` rejects iff `start > end`.                       |
| Derivation cycle detection         | Validator catches `prov:wasDerivedFrom` cycles regardless of insertion order. |
| Component builder required fields  | `ComponentBuilder::build` returns `MissingRequired` iff `types` is empty. |
| Catalog coverage partitioning      | Every rule falls into exactly one coverage bucket; overrides compose.    |

### Long-form local runs

```sh
PROPTEST_CASES=10000 cargo test -p sbol3 --test properties
```

A failing seed is persisted to
`crates/sbol3/tests/properties.proptest-regressions` (gitignored). To share
a reproducer, copy the failing seed into a dedicated `#[test]` and commit
it.

### When to add a new property

When the rule under test has *infinite* valid inputs and the cost of
generating them programmatically is lower than enumerating witness cases by
hand. Single-witness facts go in `tests/rule_cases/` instead.

## Fuzz testing

`fuzz/` is a separate cargo package (not a workspace member) that wires up
`cargo-fuzz` against the parser and validator across all four RDF formats.
The first byte of the fuzz input selects the format; the remainder is fed
to the parser as UTF-8 text.

Prerequisites:

- Nightly Rust toolchain (`rustup install nightly`)
- `cargo install cargo-fuzz`

Run a target:

```sh
cd fuzz
cargo +nightly fuzz run read_turtle
```

Bound the run to a fixed wall clock (CI smoke uses 60s):

```sh
cargo +nightly fuzz run read_turtle -- -max_total_time=60
```

Available targets:

| Target                | What it stresses                                                                                    |
| --------------------- | ---------------------------------------------------------------------------------------------------- |
| `read_turtle`         | Parses arbitrary byte inputs in a fuzz-selected format. Asserts no panic from the parser or backend. |
| `round_trip`          | Parses, serializes back in the same format, and reparses. Asserts the second parse does not panic.  |
| `validate`            | Parses then validates. Asserts the validator never panics on arbitrary input.                       |
| `round_trip_validate` | Combines the round-trip and validate targets.                                                       |

CI smoke-runs the parser targets for 60s each; the validator targets are
local-only until corpus and budget are tuned.

Corpus and crash artifacts land in `fuzz/corpus/` and `fuzz/artifacts/`
respectively (both gitignored). To minimize a found crash:

```sh
cargo +nightly fuzz tmin read_turtle fuzz/artifacts/read_turtle/<crash-file>
```

## Fixture corpora

### SBOL 3 source fixtures

SBOL 3 source fixtures (Turtle) are cached under `tests/fixtures/sbol3`
(gitignored). The first `cargo test` invocation fetches and populates the
cache from the pinned `SynBioDex/SBOLTestSuite` archive at commit
`0044284331b2f915a6e4b9d50e1cbf3ea2f62dcd`. The checked-in manifest is
[`tests/sbol3_fixtures_manifest.tsv`](../tests/sbol3_fixtures_manifest.tsv).
The valid-input set is 31 upstream fixtures plus 2 local supplements
(`experimental_data.ttl`, `variable_feature.ttl`) that exercise features
the upstream corpus doesn't cover, 33 in total.

### SBOL 2 source fixtures

The SBOLTestSuite SBOL 2 corpora (`SBOL2`, `SBOL2_bp`, `SBOL2_ic`,
`SBOL2_nc`, `InvalidFiles`) bootstrap on demand into
`tests/fixtures/sbol2/SBOLTestSuite` (gitignored), reusing the same pinned
archive the SBOL 3 tests download. The hand-written and real-world SBOL 2
fixtures used by the conversion round-trip are vendored under
`tests/fixtures/sbol2/real/`.

### Cross-impl references

Committed under `tests/fixtures/cross-impl/` (libSBOLj3, ~132 files),
`tests/fixtures/cross-impl-pysbol3/` (pySBOL3, ~132 files), and
`tests/fixtures/cross-impl-sbol2/` (libSBOLj, 10 files). See the
cross-implementation conformance section above.

## Adding a new test

| You want to test…                                              | File / pattern                                              |
| -------------------------------------------------------------- | ----------------------------------------------------------- |
| A new SBOL 3 builder, accessor, or `Document` method           | Existing or new file under `crates/sbol3/tests/*.rs`.       |
| A new SBOL 2 builder, accessor, or `Document` method           | Existing or new file under `crates/sbol2/tests/*.rs`.       |
| A new validation rule's failure case                           | Add a `RuleCase` to the matching `tests/rule_cases/*.rs` (or `crates/sbol2/tests/rule_cases.rs`). |
| A new validation rule's positive case                          | Add a `PositiveCase` to the matching rule-case file.        |
| An invariant over generated input                              | Add to `crates/sbol3/tests/properties.rs`.                  |
| A new SBOL feature against the SBOLTestSuite corpus            | Extend `crates/sbol3/tests/sbol3_fixtures.rs` or `crates/sbol2/tests/conformance.rs`. |
| A conversion behavior                                          | Add to the matching `crates/sbol-convert/tests/*.rs`.       |
| A divergence from a reference implementation                   | Either fix sbol-rs, or allowlist with rationale in the relevant `allowlist.txt`. |
| Backend / format behavior on synthetic triples                 | `crates/sbol-rdf/tests/basic.rs`.                           |
| Parser robustness on arbitrary bytes                           | Already covered by fuzz; no per-input test needed.          |

If `cargo test --workspace` is green and the relevant drift gates fire on
the corresponding rule-case file, the change is shippable.
