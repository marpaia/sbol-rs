# libSBOLj (SBOL 2) cross-implementation harness

This directory holds the committed `*.libSBOLj.expected.rdf` reference
outputs that `crates/sbol2/tests/cross_impl.rs` diffs against sbol-rs's
own SBOL 2 serialization. The Docker image definition and Java
round-trip wrapper that generate them live in
`benches/cross-impl/libsbolj/`.

SBOL 2 is exchanged as RDF/XML, so the harness compares RDF/XML output
only; libSBOLj's JSON and Turtle writers are outside the standard
interchange.

## Regenerating the reference outputs

Build the pinned Docker image (only required once per libSBOLj version
bump):

```sh
docker build -t libsbolj-pinned benches/cross-impl/libsbolj/
```

Then regenerate every `*.libSBOLj.expected.rdf` reference:

```sh
cargo run -p sbol2 --bin regenerate-cross-impl-sbol2-expectations
```

The binary fails loudly if Docker is unreachable or the image is not
built. Commit the regenerated files alongside the version bump. When no
references are committed, `cross_impl` runs as a zero-comparison passing
test.

## Bumping the libSBOLj version

Edit `benches/cross-impl/libsbolj/Dockerfile` and update the
`LIBSBOLJ_VERSION` ARG. Rebuild, regenerate, and commit.

## Allowlisting a divergence

When libSBOLj and sbol-rs produce different but spec-compliant output
for some fixture, add the reference filename to `allowlist.txt` with a
`#` rationale block above it. Track the upstream issue or spec citation
that justifies the entry.
