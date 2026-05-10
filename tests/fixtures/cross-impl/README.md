# Cross-implementation conformance fixtures

Reference outputs produced by libSBOLj3 against a curated subset of
SBOLTestSuite fixtures. The committed `.expected.ttl` files are the
source of truth for the `cross_impl` integration test
(`crates/sbol/tests/cross_impl.rs`), which parses each source fixture
with sbol-rs, serializes back to Turtle, and asserts triple-set
equality against the libSBOLj3 reference output.

CI does NOT regenerate these files. CI runs only the diff test. To
refresh the references locally after a libSBOLj3 version bump:

```sh
docker build -t libsbolj3-pinned tests/fixtures/cross-impl/
cargo run -p sbol --bin regenerate-cross-impl-expectations
```

## Pinned libSBOLj3 version

`1.0.5.2` from `SynBioDex/libSBOLj3` GitHub releases. libSBOLj3 is not
on Maven Central as of this version; the Dockerfile pulls the fat
jar directly from the release URL.

## Coverage

All 33 valid SBOL 3 fixtures in `tests/sbol3_fixtures_manifest.tsv` —
31 from the upstream SBOLTestSuite (entity, measurement, provenance,
multicellular, BBa_F2620_PoPSReceiver, COMBINE 2020, toggle switch,
toggle switch v2) and 2 sbol-rs-authored local fixtures (`ExperimentalData`,
`VariableFeature`) used to fill gaps the upstream suite does not cover.

All 33 produce normalized triple sets identical to libSBOLj3 1.0.5.2
output. No divergences require allowlisting at the time of writing.
The source-of-truth lists are `FIXTURES` in
`crates/sbol/src/bin/regenerate-cross-impl-expectations.rs` (used to
regenerate references) and `source_fixture_for` in
`crates/sbol/tests/cross_impl.rs` (used by the diff test to find the
matching source). Both must stay in sync.

## File naming

`<fixture-stem>.libSBOLj3.expected.ttl`

The stem identifies the source fixture via the table in
`crates/sbol/tests/cross_impl.rs::source_fixture_for`.

## Allowlist

`allowlist.txt` is a newline-delimited list of reference filenames for
which a known spec-compliant divergence has been accepted. Each entry
should be accompanied by a `#` comment block explaining the rationale
and pointing at the spec section that permits the divergence. The
cross-impl test skips entries on this list.

Currently empty — sbol-rs and libSBOLj3 agree on the triple-set
semantics of every committed reference.
