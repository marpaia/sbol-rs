# RDF I/O

`sbol-rs` reads and writes SBOL 3 documents in all four spec-listed RDF
serializations:

| Format    | Extension  | Notes                                                          |
| --------- | ---------- | -------------------------------------------------------------- |
| Turtle    | `.ttl`     | Default for pySBOL3; canonical CI conformance format here.     |
| RDF/XML   | `.rdf`     | Default for libSBOLj3.                                         |
| JSON-LD   | `.jsonld`  | Emitted in expanded form (no `@context`) by the oxrdfio backend. |
| N-Triples | `.nt`      | Triples are written in insertion order, not sorted.            |

All four formats share one in-memory model. Parsing produces a normalized
triple set; round-tripping is byte-for-byte triple-equivalent against
libSBOLj3 v1.0.5.2 across every SBOLTestSuite SBOL3 fixture.

## API

The format-agnostic surface lives on [`Document`] and re-exports through
`sbol::prelude`. `RdfFormat` itself is defined in `sbol-rdf` and re-exported
from `sbol`.

```rust
use sbol::prelude::*;
use sbol::RdfFormat;

// Parse from a string.
let document = Document::read(turtle_input, RdfFormat::Turtle)?;

// Parse from a file. Format is inferred from the extension.
let document = Document::read_path("design.rdf")?;

// Serialize to a string in the requested format.
let xml = document.write(RdfFormat::RdfXml)?;

// Serialize to a file. Format is explicit (no extension inference on write;
// `foo.json` writing JSON-LD would be ambiguous, so the caller picks).
document.write_path("design.jsonld", RdfFormat::JsonLd)?;
```

The Turtle-specific shorthands [`Document::read_turtle`] and
[`Document::write_turtle`] remain as one-line wrappers; new code should
prefer the generic `read` / `write` methods.

### Extension inference

`Document::read_path` infers format from the path's extension via
`RdfFormat::from_extension`. The match is case-insensitive (`.TTL`
parses), with a leading dot tolerated. Ambiguous extensions
(`.xml`, `.json`) intentionally return `None`; pass an explicit format
via `Document::read` if your input comes through an ambiguous channel.

### Error handling

`ReadError` and `WriteError` are both `#[non_exhaustive]`:

```rust
use sbol::v3::ReadError;

match Document::read_path(&path) {
    Ok(document) => { /* ... */ }
    Err(ReadError::Io { source, .. })          => { /* fs::read_to_string failed */ }
    Err(ReadError::UnknownFormat { extension, .. }) => { /* path extension wasn't ttl/rdf/jsonld/nt */ }
    Err(ReadError::Rdf(_))                     => { /* malformed RDF; see error source chain */ }
    Err(other)                                 => { /* future-compat */ }
}
```

The CLI distinguishes these: `sbol validate` exits with code `2` on IO or
unknown-format failures and code `1` on validation errors.

## CLI

```sh
sbol validate path/to/design.ttl
sbol validate path/to/design.rdf
sbol validate path/to/design.jsonld
sbol validate path/to/design.nt
```

`sbol validate --help` lists supported extensions. Unknown extensions
produce a clear error rather than a guess.

## Cross-implementation conformance

`tests/fixtures/cross-impl/` holds reference outputs produced by
libSBOLj3 v1.0.5.2 for every SBOLTestSuite SBOL 3 fixture (plus two
sbol-rs-authored local fixtures). For each source fixture there are
four committed reference files:

```
<stem>.libSBOLj3.expected.ttl
<stem>.libSBOLj3.expected.rdf
<stem>.libSBOLj3.expected.jsonld
<stem>.libSBOLj3.expected.nt
```

`cargo test -p sbol3 --test cross_impl` parses each reference in its own
format and asserts the normalized triple set matches what sbol-rs
produces by serializing the same source fixture in that format. CI
needs nothing besides `cargo test`; no Docker, no JDK.

A known-compliant divergence is allowlisted by adding the reference
filename to `tests/fixtures/cross-impl/allowlist.txt` with a rationale.
The allowlist is currently empty: sbol-rs and libSBOLj3 produce
equivalent normalized triple sets for every fixture in every format.

To refresh the references after bumping the libSBOLj3 pin, see the
"Cross-implementation conformance" section of [`testing.md`](testing.md).

## Implementation notes

### Triple-set semantics

`RdfGraph::normalized_triples` returns a sorted, **deduplicated**
`Vec<Triple>` because RDF graphs are conceptually sets, not multisets.
This matters in practice: libSBOLj3's RDF/XML output emits a top-level
`rdf:type` triple twice when a resource appears both inline (as a
nested element) and at the top level. The dedup step collapses those
duplicates, so format-equivalence assertions match RDF semantics rather
than the lexical artifact of the serializer.

`RdfGraph::triples` returns the underlying triple `Vec` without
deduplication. Use that when you need the parser's raw output (e.g.
for debugging a serialization bug). Use `normalized_triples` for
equality checks.

### JSON-LD shape

oxrdfio emits expanded JSON-LD (no top-level `@context`). libSBOLj3
emits a compact form. Both parse back to the same triple set, so the
cross-impl test compares triples rather than the JSON text itself.
sbol-rs does not currently produce a curated `@context` on output; if
that becomes important, the work belongs in a follow-up that defines a
canonical SBOL JSON-LD context.

### RDF/XML quirks

oxrdfio's RDF/XML writer requires absolute IRIs (no `@base`-relative
resolution). All SBOL identities are absolute URLs by construction
(`{namespace}/{local}/{displayId}`), so this constraint is invisible
in practice. If you parse an RDF/XML document that uses relative IRIs,
the underlying parser resolves them during parsing.

### N-Triples determinism

N-Triples output preserves the in-memory `Graph` order (insertion
order from the original parse, or the order yielded by typed
`from_objects`). It is not sorted. If you need diff-friendly N-Triples
output, sort `Graph::normalized_triples()` and re-serialize. This is
not the default because round-tripping a document should be idempotent
at the triple-set level without changing the byte sequence.

### Backend boundary

The format-to-parser mapping lives in
`crates/sbol-rdf/src/backends/oxrdf.rs`. Other backends (e.g. a future
`oxigraph` or `sophia` backend) would implement the same
`pub(crate) trait Backend::{parse, write, validate_iri}` shape, taking
the public `RdfFormat` as a parameter. The wrapper crate exists so
swapping backends doesn't ripple into the public `sbol` API surface.

## What's intentionally not here

- **Streaming / reader APIs.** All four formats today parse from `&str`
  and serialize to `String`. Streaming I/O is post-launch.
- **Format auto-detection from content.** Only filename-extension
  inference. Sniffing leading bytes (`<?xml`, `{`, `@prefix`) is a
  reasonable extension but no caller has needed it yet.
- **Canonical / pretty serialization.** Triple order is insertion
  order; prefix layout follows oxrdfio defaults. Diff-friendly
  canonical Turtle is a separate, post-0.1.0 concern.
- **SBOL 2 ⇄ SBOL 3 conversion.** Out of scope for the I/O layer; it
  lives in [`sbol-convert`](../crates/sbol-convert/) and is documented in
  [`conversion.md`](conversion.md). This page covers the shared RDF
  serialization surface, which both versions use.
