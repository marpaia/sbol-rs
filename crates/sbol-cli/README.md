# sbol-cli

Command-line tool for SBOL 3 documents. Ships the `sbol` binary.

```sh
cargo install sbol-cli
```

## `sbol validate`

Validate an SBOL 3 document. The serialization format is inferred from the
file extension: `.ttl` (Turtle), `.rdf` (RDF/XML), `.jsonld` (JSON-LD), or
`.nt` (N-Triples):

```sh
sbol validate design.ttl
sbol validate design.rdf
sbol validate design.jsonld
sbol validate design.nt
```

Exit codes:

| Code | Meaning                                                                |
| ---- | ---------------------------------------------------------------------- |
| `0`  | Document parses and validates with no errors.                          |
| `1`  | Validation errors found (printed with rule IDs).                       |
| `2`  | I/O failure, unsupported / missing filename extension, or bad CLI usage. |
| `3`  | `--treat-partial-as-errors` and at least one rule is partially applied. |

Pass `--treat-warnings-as-errors` to make `sbol validate` exit `1` on
validation warnings as well as errors.

## `sbol convert`

Cross-serialize an SBOL 3 document between Turtle, RDF/XML, JSON-LD, and
N-Triples. The input format is inferred from the source extension; the
target format is taken from `--to` or inferred from `--output`:

```sh
sbol convert design.ttl --output design.jsonld
sbol convert design.ttl --to ntriples > design.nt
```

## `sbol rules list`

Dump the built-in validation rule catalog so you know what `--allow` /
`--deny` / `--warn` can target. Output is tab-separated with a header
row; pass `--format json` for machine consumption, or `--status` to
filter:

```sh
sbol rules list
sbol rules list --format json
sbol rules list --status error
```

## Backing library

The CLI is a thin wrapper around the [`sbol`](https://crates.io/crates/sbol)
library; the same validation, parsing, and serialization machinery is
available programmatically.

## License

Dual-licensed under MIT or Apache-2.0, at your option. See the
[workspace README](https://github.com/marpaia/sbol-rs#readme) for project
context.
