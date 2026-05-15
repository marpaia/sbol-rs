# Changelog

All notable changes to this workspace are documented here. The format
is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this workspace follows [Cargo's SemVer for `0.x`](https://doc.rust-lang.org/cargo/reference/semver.html)
crates: breaking changes are permitted between minor versions
(`0.1` → `0.2`) until `1.0`.

## [Unreleased]

### Added

- **`sbol`**: `owl_conformance` module exposing
  `analyze_owl_conformance`, `render_owl_conformance_report`,
  `OwlConformanceReport`, `OwlIdentifiers`, `OwlPinInfo`, and the
  `OWL_ONLY_ALLOWLIST` / `RUST_ONLY_ALLOWLIST` constants. The module
  parses a pinned copy of `sbol-owl3-gen/sbol3.rdf` from
  [SynBioDex/sbol-owl3](https://github.com/SynBioDex/sbol-owl3) — the
  canonical OWL serialization of the SBOL 3 data model — and compares
  its `http://sbols.org/v3#` IRI set against the constants declared in
  `crates/sbol/src/vocab.rs`. Two allowlists document every intentional
  divergence; everything else is treated as actionable drift.
- **`sbol`** (test-only): two integration tests enforce the
  conformance contract on every `cargo test` run.
  `sbol_owl3_conformance.rs` fails CI when the IRI surfaces drift
  outside the allowlists or when either allowlist goes stale.
  `sbol_owl3_conformance_report.rs` fails CI when the committed
  report at `docs/sbol-owl3-conformance.md` is out of date.
- **`sbol`** (tool-only): `generate-sbol-owl3-conformance-report`
  binary that renders the auditable markdown report consumed by the
  freshness test. See
  [`docs/sbol-owl3-conformance.md`](docs/sbol-owl3-conformance.md)
  for the current pinned-state coverage table and
  [`docs/ontology-conformance.md`](docs/ontology-conformance.md) for
  the broader regression-system design.
- **`sbol-ontology`** (tool-only): `update-sbol-owl3-fixture` binary
  that re-pins the fixture against the current upstream `main` commit
  and rewrites `manifest.toml`.

### Fixed

- **`sbol`**: reject `http://sbols.org/v3#zero` as a `VariableFeature`
  cardinality. SBOL 3.1.0 Table 14 enumerates only `zeroOrOne`, `one`,
  `zeroOrMore`, and `oneOrMore`; the previous build accepted `#zero` and
  treated it as "count must be 0" via `cardinality_allows`. Rule
  `sbol3-12201` now reports the value as an unsupported cardinality.
- **`sbol`**: reject `http://sbols.org/v3#none` as a Feature or Location
  `orientation`. SBOL 3.1.0 §6.4.1 Tables 5 and 6 enumerate exactly four
  orientation URIs (`sbol:inline`, `sbol:reverseComplement`,
  `SO:0001030`, `SO:0001031`); the previous build silently accepted
  `sbol:none` because the value was in `ORIENTATION_IRIS`. Rules
  `sbol3-10702` and `sbol3-11301` now reject the value.

### Removed

- **`sbol`**: `sbol::constants::CARDINALITY_ZERO`. The constant pointed
  at an IRI that is not in SBOL 3.1.0 Table 14 and is therefore invalid
  as a `VariableFeature.cardinality` value. Use `CARDINALITY_ZERO_OR_ONE`
  or `CARDINALITY_ZERO_OR_MORE` instead, depending on the intended
  semantics.
- **`sbol`**: `sbol::constants::ORIENTATION_NONE`. The constant pointed
  at `http://sbols.org/v3#none`, an IRI that is not in SBOL 3.1.0 Tables
  5 or 6 and therefore invalid as a Feature or Location `orientation`
  value. To represent "no orientation," omit the `orientation` property
  entirely; the spec marks it ZERO OR ONE.

## [0.1.0]

Initial release. All four workspace crates (`sbol`, `sbol-rdf`,
`sbol-ontology`, `sbol-cli`) ship together at `0.1.0`.

### Added

- **`sbol`**: typed Rust API for reading, constructing, inspecting,
  and rewriting SBOL 3.1.0 documents. Owned typed structs for every
  SBOL class (`Component`, `Sequence`, `SubComponent`, …) with
  validated builders. Reference traversal across single documents and
  composed `DocumentSet`s. `Document::check` / `validate` /
  `check_complete` cover the full SBOL 3.1.0 rule set.
- **`sbol`**: deterministic offline validator covering 109 of 109
  machine-checkable SBOL 3.1.0 rules, with per-rule `sbol3-*`
  identifiers, severity overrides (`--allow` / `--deny` / `--warn`),
  and three output formats (text, JSON v1, SARIF v2.1.0).
- **`sbol-rdf`**: RDF primitives (`Iri`, `Literal`, `Resource`,
  `Term`, `Triple`, `Graph`) and multi-format I/O for Turtle, RDF/XML,
  JSON-LD, and N-Triples, behind a parser-agnostic backend trait.
- **`sbol-ontology`**: bundled offline ontology fact snapshots for
  EDAM, SBO, SO, GO, ChEBI, and Cell Ontology, plus a runtime cache
  for opt-in extensions (NCIT, lab-specific ontologies).
- **`sbol-cli`**: `sbol` binary with `validate`, `ontology install`,
  `ontology list`, `ontology verify`, and `ontology remove`
  subcommands. Documented exit codes (`docs/validation-output.md`).
- **Cross-implementation conformance harness.** Round-trip tests
  against pinned-Docker libSBOLj3 and pysbol3 reference outputs for
  33 fixtures × 4 RDF formats.
- **Loss-conscious round-trip.** Unknown extension triples on known
  SBOL objects and `sbol:Identified`-only subjects are preserved
  through parse → write.

### Stability

`0.x` per Cargo's SemVer: breaking changes between `0.1` → `0.2` are
allowed and will be called out in release notes. Covered surface: the
public Rust API of the workspace crates, the `sbol` CLI exit codes,
and the JSON v1 validation output schema.

[Unreleased]: https://github.com/marpaia/sbol-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/marpaia/sbol-rs/releases/tag/v0.1.0
