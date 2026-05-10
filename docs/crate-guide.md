# sbol-rs crate guide

This guide orients a newcomer to the codebase. It complements:

- The [README](../README.md): quick-start code and stability policy.
- [`validation.md`](validation.md): the validator in depth.
- [`rdf-io.md`](rdf-io.md): I/O semantics and format inference.
- The crate's `cargo doc` landing page: API reference.

Read this first to know *where* things live and *why* the workspace
is shaped the way it is. Read the others when you need depth.

## Workspace layout

Four published crates:

| Crate          | Purpose                                                                                  |
| -------------- | ---------------------------------------------------------------------------------------- |
| `sbol`         | Public API. Document, typed objects, builders, validation, reference resolution.         |
| `sbol-rdf`     | RDF primitives and multi-format I/O (Turtle, RDF/XML, JSON-LD, N-Triples).                |
| `sbol-ontology`| Offline ontology facts (EDAM, SBO, SO, GO, ChEBI, CL) plus a runtime cache for opt-in extensions (NCIT, custom). See [`ontology-extensions.md`](ontology-extensions.md).|
| `sbol-cli`     | Command-line validator. Ships the `sbol` binary.                                          |

Internally the `sbol` crate is split into private modules; only
`constants`, `identity`, `prelude`, and `schema` are public modules.
Everything else is re-exported at the crate root.

The boundary between `sbol` and `sbol-rdf` is deliberate. SBOL code
depends on owned RDF primitives (`Iri`, `Literal`, `Resource`,
`Term`, `Triple`) and graph traits, not on the underlying parser
implementation. Swapping parsers is a one-crate change.

## Data model

SBOL describes designed biology as a typed object graph. Three
concepts cover most of it:

**TopLevel objects** are the things you put in a document. They have
stable IRIs and exist independently:

- `Component`: the central design unit. DNA, RNA, protein,
  complexes, functional descriptions. A `Component` has a `type`
  set (SBO terms like `SBO_DNA`), a `role` set (SO terms like
  `SO_PROMOTER`), and may contain features and interactions.
- `Sequence`: sequence elements (DNA/RNA/protein/SMILES) with an
  `encoding` IRI.
- `Collection`: a typed grouping of TopLevels.
- `CombinatorialDerivation`: describes a family of related
  `Component` variants.
- `Implementation`, `ExperimentalData`, `Experiment`, `Model`,
  `Attachment`: wet-lab and modeling artifacts.
- PROV-O classes: `Activity`, `Plan`, `Agent`.
- OM (units): `Unit` (and its subclasses), `Prefix`.

**Owned children** belong to a parent TopLevel and don't exist on
their own:

- Hung off `Component`: `SubComponent`, `LocalSubComponent`,
  `ExternallyDefined`, `ComponentReference`, `SequenceFeature`,
  `Interaction`, `Constraint`, `Interface`.
- Hung off features: `Range`, `Cut`, `EntireSequence` (location
  classes).
- Hung off `Interaction`: `Participation`.
- Hung off `CombinatorialDerivation`: `VariableFeature`.

**References** are typed and IRI-based. A `SubComponent` has an
`instance_of: Iri` pointing at the `Component` it instantiates; a
`Participation` has a `participant: Iri` pointing at a feature. The
reference target's typed class is captured in the schema so the
validator can check the IRI resolves to the right kind of object.

The full schema lives in `crates/sbol/src/schema.rs` as
`FieldDescriptor` entries (predicate IRI, cardinality, value kind,
reference target, governing validation rule). Both serialization and
validation read from these descriptors so they can't drift.

## Document lifecycle

The common flow is read → validate → traverse → mutate → write.

### Read

`Document::read_path` infers the format from the extension (`.ttl`,
`.rdf`, `.jsonld`, `.nt`). `Document::read(input, RdfFormat::*)` is
the explicit in-memory form. Both produce a `Document` even if the
file would fail validation; parsing and validation are separate
concerns.

### Validate

Three entry points, ordered from convenience to strictness:

- `Document::check()` returns `Err(ValidationReport)` if there are
  any errors, ignores warnings. Use this when you just need a pass /
  fail signal.
- `Document::validate()` returns the full `ValidationReport`
  unconditionally. Inspect `errors()`, `warnings()`, and per-rule
  `coverage()`.
- `Document::check_complete()` is the strictest variant. Fails if any
  rule produced a `PartialApplication` (e.g. an external reference
  the configured resolver couldn't reach). Use when partial coverage
  is a hard failure.

For cross-document checks, configure a `ValidationContext` with a
`DocumentResolver` / `ContentResolver` and call
`Document::validate_with(...)` (or the matching `check_with`).

### Traverse

Typed accessors live on `Document`:

```rust,ignore
for component in document.components() {  /* ... */ }
for sequence  in document.sequences()  {  /* ... */ }
```

For reference traversal, methods that follow IRIs live on the typed
structs themselves and take anything implementing `ObjectGraph`:

```rust,ignore
let definition = sub_component.definition(&document)?;
let parts = ComponentReference::trace(&document_set)?;
let variants = combinatorial_derivation.variants(&document_set)?;
```

`Document` implements `ObjectGraph` for single-file work;
`DocumentSet` composes multiple parsed documents into one resolution
scope so references can cross document boundaries.

The reverse direction (given a child, find its parent) has
ergonomic helpers on the typed structs:

```rust,ignore
sub_component.parent_component(&document)        // Option<&Component>
sequence_feature.parent_component(&document)     // Option<&Component>
participation.parent_interaction(&document)      // Option<&Interaction>
component.parent_collections(&document)          // Vec<&Collection>
```

These iterate the document to locate the owner; they're linear in
the relevant typed-iterator length and suitable for typical-sized
documents.

### Mutate

Builders ([`Component::builder`], [`Sequence::builder`], etc.)
return owned objects. Add them to a document via
`Document::from_objects([...])` or by mutating
`document.objects_mut()`.

### Write

`Document::write(RdfFormat::*)` is the explicit form; the four
shortcut methods (`write_turtle`, `write_rdf_xml`, `write_jsonld`,
`write_ntriples`) cover the common cases. Round-trip preserves
unknown extension triples. Extension data downstream tools don't
recognize is held in `IdentifiedExtension` rather than dropped.

## Validation pipeline at a glance

The validator is at 148/149 SBOL 3.1.0 rules. The pipeline is:

1. **Parse to RDF triples** (`sbol-rdf` crate). Format-agnostic
   after this point.
2. **Typed object graph** (`crates/sbol/src/client/from_rdf.rs`).
   Triples → owned typed structs.
3. **Rule firings** (`crates/sbol/src/validation/rules/`). One file
   per spec area (`component.rs`, `sequence.rs`, etc.). Each rule
   has a stable `sbol3-*` identifier matched to a row in
   `crates/sbol/rules.toml`.
4. **Report assembly** (`crates/sbol/src/validation/report.rs`).
   Errors, warnings, hints, per-rule coverage, and partial-application
   data are all surfaced.

Rules fall into five status categories tracked in `rules.toml`:

- **Error**: violates the spec; fails `check()`.
- **Warning**: surfaced but doesn't fail `check()`.
- **Configurable**: the spec leaves the choice to implementations;
  default is conservative, callable via `ValidationOptions`.
- **MachineUncheckable (▲)**: the spec rule isn't algorithmically
  verifiable (e.g., requires biological judgment).
- **Unimplemented**: a known gap. The current outstanding one is
  `sbol3-10204` (cross-document `prov:wasGeneratedBy` cycles).

Severity for a particular rule on a particular run can be lifted or
suppressed via `ValidationOptions::override_*`. The CLI exposes
this as `--allow`, `--deny`, and `--warn` flags.

Rules that consult ontology data (Component type / role, Sequence
encoding, Interaction type, Participation role, Model framework, and
the EDAM / SBO / SO branch checks) go through the `OntologyRegistry`
held by `ValidationContext`. By default the registry contains only
the bundled snapshot (EDAM, SBO, SO, GO, ChEBI, CL). Library callers
that want to recognize terms from larger ontologies (NCIT, custom
domain ontologies) layer them on with
`ValidationOptions::with_ontology_extension(Ontology)`. Bundled facts
always win on conflicts; extensions can add new terms and parent
links but cannot rewrite bundled ones. See
[`ontology-extensions.md`](ontology-extensions.md) for the install
workflow and the stable TSV contract.

For the full system overview, read [`validation.md`](validation.md).
For per-rule status, read [`conformance.md`](conformance.md) (auto-
generated from `rules.toml`).

## Key decision points

These are the choices a newcomer hits first.

### `Document` vs `DocumentSet`

A `Document` is one parsed file. A `DocumentSet` composes multiple
documents into a single resolution scope.

Use a `Document` when references stay inside one file. Use a
`DocumentSet` when you have a design that pulls in a separate parts
library: `SubComponent::definition()` in the design needs to
resolve against `Component` definitions in the library.

### `check` vs `validate` vs `check_complete`

- `check()`: pass/fail. Errors fail, warnings ignored.
- `validate()`: gives you the report no matter what.
- `check_complete()`: like `check()`, but also fails on partial
  application (rules that couldn't fully apply because an external
  resolver returned nothing).

If you're loading user input and want to reject malformed designs,
`check()` is right. If you're surfacing diagnostics in a UI,
`validate()` and walk `report.issues()`. If you're in CI and want
the strongest signal, `check_complete()`.

### Accessor traits vs field access

The shared metadata (`displayId`, `name`, `description`, namespace, PROV
links, OM measures, extensions) lives in `IdentifiedData` and
`TopLevelData` fields nested inside every typed object. Two traits in
the prelude (`SbolIdentified` and `SbolTopLevel`) expose that
metadata directly:

```rust,ignore
component.display_id()          // Option<&str>
component.name()                // Option<&str>
component.description()         // Option<&str>
component.namespace()           // Option<&Iri>     (TopLevel only)
component.attachments()         // &[Resource]      (TopLevel only)
component.derived_from()        // &[Resource]
component.generated_by()        // &[Resource]
```

Prefer the trait methods in user-facing code. The raw nested fields
(`component.identified.name`, `component.top_level.namespace`) remain
public for cases where you need to construct or mutate the data
directly, but the accessor methods are the canonical read path.

### Bundled vs extension ontology

The validator consults two tiers of ontology data:

- **Bundled** snapshot, compiled into `sbol-ontology`: EDAM, SBO, SO,
  GO, ChEBI, CL. Always available; zero IO; deterministic across
  releases (the snapshot is pinned per `sbol-ontology` version).
- **Extension** snapshots, loaded from a TSV at runtime: NCIT, plus
  any lab-specific ontology you ship. Installed once via
  `OntologyCache::ensure_installed(...)` or
  `sbol ontology install <name>`, then opted into per validation via
  `ValidationOptions::with_ontology_extension(...)` or
  `--ontology=<name>`.

Use the bundled set when the term you care about is in one of the
six bundled ontologies; the validator already recognizes it. Use an
extension when you need terms outside that set (e.g. NCIT cell
lines, organism strains, reagents) or when a lab-specific ontology
needs to participate in validation. Validation itself never touches
the network; the cache is the only IO surface, populated explicitly.

Two users on the same `sbol` version can see different reports when
their caches contain different extension snapshots. For deterministic
CI, commit a `*.tsv` to the repo and load it with
`Ontology::from_tsv_path` instead of relying on the user-local
cache.

### Builders vs `from_rdf`

Builders construct new objects from typed inputs and validate at
build time (`BuildError` fires immediately for invalid `displayId`,
malformed namespace, etc.).

`from_rdf` (via `Document::read*`) is for ingesting existing RDF
serializations. Errors at read time mean the bytes weren't valid
RDF; SBOL-rule violations surface later through `validate()`.

You shouldn't normally call `from_rdf` directly; read through
`Document`.

### `Iri` vs `DisplayId` vs `Namespace`

- `Iri`: full IRI. Cheap to clone (Arc<str> internally).
- `DisplayId`: validated `sbol:displayId` (matches the lexical
  rule sbol3-10201). Builders take `&str` and construct one
  internally.
- `Namespace`: validated SBOL namespace IRI.

Builder APIs take the cheap forms (`&str` for both display ID and
namespace) and validate at construction.

## RDF extension triples and round-tripping

(For runtime *ontology* extensions (NCIT, lab ontologies layered on
top of the bundled snapshot), see
[`ontology-extensions.md`](ontology-extensions.md) and the "Bundled
vs extension ontology" decision point above. This section is about
the orthogonal concern: arbitrary RDF triples emitted by downstream
tools.)

SBOL is an open RDF vocabulary. Real documents commonly carry
extension triples from downstream tools: annotations, custom
predicates, application-specific vocabularies. The crate preserves
these:

- Unknown triples attached to a known SBOL object are held in that
  object's extension property bag.
- Subjects typed only as `sbol:Identified` (extension TopLevels that
  the spec doesn't promote to a typed class) are surfaced as
  `IdentifiedExtension`.

The round-trip guarantee: parse + write produces the same triples
modulo blank node renaming and predicate ordering. See
[`rdf-io.md`](rdf-io.md) for the full I/O reference and the
cross-implementation conformance harness against libSBOLj3.

## CLI

`sbol-cli` ships a `sbol` binary:

```sh
cargo install sbol-cli           # installs the `sbol` binary
sbol validate design.rdf         # text output
sbol validate design.rdf --format json --treat-partial-as-errors
sbol validate design.rdf --allow sbol3-10502 --deny sbol3-12807

sbol ontology install ncit       # install NCIT into the runtime cache
sbol ontology list               # show installed extensions
sbol validate design.rdf --ontology ncit  # opt into the cached extension
```

Exit codes are documented in
[`validation-output.md`](validation-output.md). The CLI uses the
same validator as the library; there's no separate code path.

## Testing

Regression cases live in `crates/sbol/tests/rule_cases/`, one
module per spec area. Each `RuleCase` exercises a specific rule
identifier with either a fixture that should fire it (negative case)
or one that should pass while still touching the predicate (positive
case).

Cross-implementation conformance lives at
`tests/fixtures/cross-impl/`: 33 fixtures × 4 formats validated
against libSBOLj3 outputs. Property-based and fuzz coverage are
described in [`testing.md`](testing.md).
