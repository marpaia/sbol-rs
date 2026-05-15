# Changelog

All notable changes to this workspace are documented here. The format
is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this workspace follows [Cargo's SemVer for `0.x`](https://doc.rust-lang.org/cargo/reference/semver.html)
crates: breaking changes are permitted between minor versions
(`0.1` → `0.2`) until `1.0`.

## [Unreleased]

### Fixed

- **`sbol`**: reject `http://sbols.org/v3#zero` as a `VariableFeature`
  cardinality. SBOL 3.1.0 Table 14 enumerates only `zeroOrOne`, `one`,
  `zeroOrMore`, and `oneOrMore`; the previous build accepted `#zero` and
  treated it as "count must be 0" via `cardinality_allows`. Rule
  `sbol3-12201` now reports the value as an unsupported cardinality.

### Removed

- **`sbol`**: `sbol::constants::CARDINALITY_ZERO`. The constant pointed
  at an IRI that is not in SBOL 3.1.0 Table 14 and is therefore invalid
  as a `VariableFeature.cardinality` value. Use `CARDINALITY_ZERO_OR_ONE`
  or `CARDINALITY_ZERO_OR_MORE` instead, depending on the intended
  semantics.

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
