# Testing architecture

`sbol-rs` is correctness-critical: every claim about SBOL conformance has to
survive `cargo test`. This document maps the test surface so contributors
know what each layer gates, where to add a new case, and how the fixture
corpora are produced.

## Test layers at a glance

| Layer                              | Where                                                   | What it gates                                                                       |
| ---------------------------------- | ------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Unit tests                         | Inline `#[cfg(test)]` mods (minimal; most logic moves to integration tests) | Type-local invariants too small to integrate. |
| Integration tests                  | `crates/sbol/tests/*.rs`                                | Public-API behavior per feature: builders, traversal, parsing, validation entry points. |
| Validation rule regressions        | `crates/sbol/tests/rule_cases/{spec-area}.rs`           | One negative case per non-deferred SBOL rule ID; positive cases per spec cluster.   |
| Schema consistency                 | `crates/sbol/tests/schema_consistency.rs`               | `to_rdf` ↔ `from_rdf` ↔ `FieldDescriptor` predicate-set parity across every class.  |
| Property tests                     | `crates/sbol/tests/properties.rs`                       | Five spec-derived invariants under `proptest`-generated input.                      |
| SBOLTestSuite fixtures             | `crates/sbol/tests/sbol3_fixtures.rs`                   | All 33 valid SBOL 3 fixtures parse, validate, and round-trip in every RDF format.   |
| Cross-implementation conformance   | `crates/sbol/tests/cross_impl.rs`, `crates/sbol/tests/cross_impl_pysbol3.rs` | Triple-set equivalence against libSBOLj3 v1.0.5.2 and pySBOL3 1.2 across 132 fixture × format combos per reference implementation. |
| Catalog freshness gates            | `crates/sbol/tests/validation_rules.rs`                 | `rules.toml` invariants and `docs/conformance.md` freshness.                        |
| RDF backend                        | `crates/sbol-rdf/tests/basic.rs`                        | Per-format round-trip, cross-format equivalence, extension parsing.                 |
| Fuzz tests                         | `fuzz/fuzz_targets/{read_turtle, round_trip}.rs`        | No panics on arbitrary bytes in any of the four RDF formats.                        |
| Doctests                           | `///` examples in `crates/sbol/src/`                    | README / prelude / `Document::read*` examples compile and run.                      |

Run everything: `cargo test --workspace`. Total today: 136 tests across 24
binaries, plus three doctests.

## Validation rule architecture

The validation surface is the largest and most spec-coupled subsystem;
testing it has structure beyond "a `#[test]` per rule."

### Rule catalog (source of truth)

`crates/sbol/rules.toml` is the single source-of-truth for every SBOL 3.1.0
validation rule (149 today). Each entry carries:

- `id`: the canonical SBOL rule ID (e.g. `sbol3-10101`).
- `status`: `ImplementedError` (54) / `ImplementedWarning` (8) / `Partial`
  (68) / `Deferred` (19).
- `normative_severity`: RFC2119 force (`MUST` / `SHOULD`).
- `spec_section`: section reference into `spec/SBOL3.1.0.md`.
- `blocker` (Partial/Deferred only): `Ontology` / `Resolver` / `Policy` /
  `External` / `StrictDatatype`. Names why the rule isn't fully checked.
- `validator_function`: the Rust function that emits the issue.
- `note`: short description of what's actually enforced.

`crates/sbol/build.rs` parses this at compile time and emits the
`validation_rule_statuses()` slice into `OUT_DIR`. Adding a new rule means
editing the TOML, not Rust.

### Drift gates (`tests/validation_rules.rs`)

Three meta-tests lock the catalog against silent drift:

| Test                                                | Asserts                                                                          |
| --------------------------------------------------- | -------------------------------------------------------------------------------- |
| `every_partial_or_deferred_rule_has_blocker`        | `blocker.is_some() ⇔ status ∈ {Partial, Deferred}`                              |
| `implemented_rule_severity_matches_status`          | `ImplementedError ⇒ MUST`, `ImplementedWarning ⇒ SHOULD`                         |
| `implemented_rule_ids_appear_in_validator_function` | Every `validator_function` value names a real function in `crates/sbol/src/validation/rules/`. |
| `implemented_validation_rules_have_regression_cases`| Every non-deferred rule has at least one entry in `tests/rule_cases/`.            |
| `validation_rule_regression_cases_report_expected_rule_ids` | Each rule-case fires its claimed rule ID.                                |
| `positive_rule_cases_do_not_report_their_rule`      | Positive cases don't false-positive on their rule.                                |

If you change a rule, all six must still pass.

### Rule-case regression suite (`tests/rule_cases/`)

One file per spec area (`component.rs`, `feature.rs`, etc.) holds a
`&[RuleCase]` array. Each `RuleCase` declares:

- The rule ID it exercises (`sbol3-XXXXX`).
- A Turtle document that should fire that rule at the given severity.
- The expected `Severity` (`Error` or `Warning`).

To add coverage for a new implemented rule:

1. Move the rule's status in `rules.toml` to `ImplementedError` or
   `ImplementedWarning`.
2. Add a `RuleCase` in the matching `tests/rule_cases/{area}.rs`.
3. (Optional) Add a `PositiveCase` covering a non-violating shape.
4. Run `cargo test -p sbol --test validation_rules`. The drift gates above
   fail loudly if anything is inconsistent.

### Conformance report freshness

`docs/conformance.md` is rendered from `validation_rule_statuses()` by
`crates/sbol/src/bin/generate-conformance-report.rs`. The committed file is
checked in CI by `tests/conformance_report.rs`. After changing rule status
or notes:

```sh
cargo run -p sbol --bin generate-conformance-report
git diff docs/conformance.md
```

CI runs `cargo test -p sbol --test conformance_report` which fails if the
committed file is stale.

## Cross-implementation conformance

`tests/fixtures/cross-impl/` holds libSBOLj3 v1.0.5.2 reference outputs for
every SBOLTestSuite SBOL 3 fixture, one file per format:

```
<stem>.libSBOLj3.expected.ttl
<stem>.libSBOLj3.expected.rdf
<stem>.libSBOLj3.expected.jsonld
<stem>.libSBOLj3.expected.nt
```

`cargo test -p sbol --test cross_impl` parses each reference in its own
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
cargo run -p sbol --bin regenerate-cross-impl-expectations
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
pySBOL3 references. File suffix is `.pySBOL3.expected.*` and the test
file is `crates/sbol/tests/cross_impl_pysbol3.rs`. Building references
on a contributor machine:

```sh
docker build -t pysbol3-pinned tests/fixtures/cross-impl-pysbol3/
cargo run -p sbol --bin regenerate-cross-impl-pysbol3-expectations
```

The harness scaffolding (Dockerfile, `roundtrip.py`, regenerate binary,
test, allowlist) ships in the repo, but reference files are not yet
committed. `cross_impl_pysbol3` runs as a zero-comparison passing
test until the first regeneration. The first Docker-equipped
contributor to run the regenerate binary should commit the resulting
files alongside any allowlist entries needed for known-compliant
divergences.

See [`rdf-io.md`](rdf-io.md) for the user-facing I/O subsystem reference.

## Cross-implementation performance benchmarks

Separate from the correctness harness, `crates/sbol-bench` times
`parse + serialize` round trips against pySBOL3, libSBOLj3, and sboljs
(each pinned in Docker, see `benches/cross-impl/`). This is not a CI
gate — there is no perf-regression check — but it gives contributors a
reproducible way to compare implementations head-to-head on the same
fixtures. Run `cargo run --release -p sbol-bench`; see
[`benches/cross-impl/README.md`](../benches/cross-impl/README.md) for
the format-support matrix, knobs, and limitations.

## Property-based testing (`tests/properties.rs`)

Five spec-derived invariants run under `proptest` at 256 cases per property
in CI:

| Property                           | Asserts                                                                  |
| ---------------------------------- | ------------------------------------------------------------------------ |
| `displayId` lexical agreement      | `DisplayId::new` accepts iff the input matches `sbol3-10201`'s regex.    |
| Compliant URL round trip           | `SbolIdentity` produces identities that parse back to their components.  |
| `Range` bounds                     | `Range::new(start, end)` rejects iff `start > end`.                       |
| Derivation cycle detection         | Validator catches `prov:wasDerivedFrom` cycles regardless of insertion order. |
| Component builder required fields  | `ComponentBuilder::build` returns `MissingRequired` iff `types` is empty. |

### Long-form local runs

```sh
PROPTEST_CASES=10000 cargo test -p sbol --test properties
```

A failing seed is persisted to
`crates/sbol/tests/properties.proptest-regressions` (gitignored). To share a
reproducer, copy the failing seed into a dedicated `#[test]` and commit it.

### When to add a new property

When the rule under test has *infinite* valid inputs and the cost of
generating them programmatically is lower than enumerating witness cases by
hand. Single-witness facts go in `tests/rule_cases/` instead.

## Fuzz testing

`fuzz/` is a separate cargo package (not a workspace member) that wires up
`cargo-fuzz` against `Document::read` across all four RDF formats. The
first byte of the fuzz input selects the format; the remainder is fed to
the parser as UTF-8 text.

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

| Target        | What it stresses                                                                                    |
| ------------- | ---------------------------------------------------------------------------------------------------- |
| `read_turtle` | Parses arbitrary byte inputs in a fuzz-selected format. Asserts no panic from the parser or backend. |
| `round_trip`  | Parses, serializes back in the same format, and reparses. Asserts the second parse does not panic.  |

The `read_turtle` target name is historical; both targets cover every
format today.

Corpus and crash artifacts land in `fuzz/corpus/` and `fuzz/artifacts/`
respectively (both gitignored). To minimize a found crash:

```sh
cargo +nightly fuzz tmin read_turtle fuzz/artifacts/read_turtle/<crash-file>
```

## Fixture corpora

### SBOLTestSuite source fixtures

SBOL 3 source fixtures (Turtle) are cached under `tests/fixtures/sbol3`
(gitignored). The first `cargo test` invocation fetches and populates the
cache from the pinned `SynBioDex/SBOLTestSuite` archive at commit
`0044284331b2f915a6e4b9d50e1cbf3ea2f62dcd`. The checked-in manifest is
[`tests/sbol3_fixtures_manifest.tsv`](../tests/sbol3_fixtures_manifest.tsv).

A cache sentinel `.sbol-rs-fixture-cache-version` records the pinned
commit; CI rebuilds the cache when the sentinel doesn't match.

### Local fixtures

Two fixtures live under `tests/fixtures/sbol3/local/` rather than upstream
SBOLTestSuite because they exercise features the upstream corpus doesn't
cover: `experimental_data.ttl` and `variable_feature.ttl`. They're treated
identically to upstream fixtures by the test harness.

### Cross-impl references

`tests/fixtures/cross-impl/*.libSBOLj3.expected.{ttl,rdf,jsonld,nt}`:
committed, ~132 files (33 fixtures × 4 formats). See the
cross-implementation conformance section above.

## Adding a new test

| You want to test…                                              | File / pattern                                              |
| -------------------------------------------------------------- | ----------------------------------------------------------- |
| A new builder, accessor, or `Document` method                  | Existing or new file under `crates/sbol/tests/*.rs`.        |
| A new SBOL validation rule's failure case                      | Add a `RuleCase` to the matching `tests/rule_cases/*.rs`.   |
| A new SBOL validation rule's positive case                     | Add a `PositiveCase` to the matching `tests/rule_cases/*.rs`. |
| An invariant over generated input                              | Add to `tests/properties.rs`.                               |
| A new SBOL feature against the SBOLTestSuite corpus            | Extend `tests/sbol3_fixtures.rs`.                           |
| A divergence from libSBOLj3                                    | Either fix sbol-rs, or allowlist with rationale in `cross-impl/allowlist.txt`. |
| Backend / format behavior on synthetic triples                 | `crates/sbol-rdf/tests/basic.rs`.                           |
| Parser robustness on arbitrary bytes                           | Already covered by fuzz; no per-input test needed.          |

If `cargo test --workspace` is green and the relevant drift gates fire on
the corresponding rule-case file, the change is shippable.
