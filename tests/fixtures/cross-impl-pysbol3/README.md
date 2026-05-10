# pySBOL3 cross-implementation harness

This directory holds the Docker image definition and Python round-trip
script that generate `*.pySBOL3.expected.{ttl,rdf,jsonld,nt}` reference
outputs. The committed expected files are diffed against sbol-rs's
own serialization by `crates/sbol/tests/cross_impl_pysbol3.rs`.

## Regenerating the reference outputs

Build the pinned Docker image (only required once per pySBOL3 version
bump):

```sh
docker build -t pysbol3-pinned tests/fixtures/cross-impl-pysbol3/
```

Then regenerate every `*.pySBOL3.expected.*` reference:

```sh
cargo run -p sbol --bin regenerate-cross-impl-pysbol3-expectations
```

The binary fails loudly if Docker is unreachable or the image is not
built. Commit the regenerated files alongside the version bump.

## Bumping the pySBOL3 version

Edit `Dockerfile` and update the `PYSBOL3_VERSION` ARG. Rebuild,
regenerate, and commit. Update `docs/testing.md` if the new release
changes any normalization behavior worth noting.

## Allowlisting a divergence

When pySBOL3 and sbol-rs produce different but spec-compliant output
for some fixture/format, add the reference filename to
`allowlist.txt` with a `#` rationale block above it. Track the
upstream issue or spec citation that justifies the entry.
