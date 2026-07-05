# Converting between SBOL formats

Most published synthetic biology data lives in formats that predate SBOL 3.1.0:

- **SBOL 2** — what SynBioHub, the iGEM Registry, and JBEI ICE serve; what libSBOLj2 and pySBOL2 produce.
- **GenBank** — sequence-with-annotations files from SnapGene, ApE, Benchling, and NCBI. The lingua franca of plasmid and clone data.
- **FASTA** — bare sequences from NCBI, UniProt, BLAST hits, and alignment output.

`sbol-rs` brings all three into SBOL 3 so you can use the modern toolchain, and reverses the SBOL 3 path back to SBOL 2 when you need to publish to consumers that still speak the older format.

## What you can convert

| You have… | Run… | Get… |
|---|---|---|
| `*.xml` / `*.ttl` / `*.jsonld` from SynBioHub, iGEM Registry, JBEI ICE | `sbol upgrade` or [`sbol_convert::upgrade_from_sbol2`] | SBOL 3 RDF |
| `*.gb` / `*.gbk` from SnapGene, ApE, Benchling, NCBI | `sbol import-genbank` or [`sbol_genbank::GenbankImporter`] | SBOL 3 RDF |
| `*.fasta` / `*.fa` / `*.fna` / `*.faa` from NCBI, UniProt, BLAST | `sbol import-fasta` or [`sbol_fasta::FastaImporter`] | SBOL 3 RDF |
| SBOL 3 to publish to a tool that only consumes SBOL 2 | `sbol downgrade` or [`sbol_convert::downgrade`] | SBOL 2 RDF |

Everything runs in pure Rust — no Docker, no Python sidecar, no network calls.

## Workflows

### Pulling a part down from SynBioHub

You found `BBa_F2620` in the iGEM Registry and want to inspect it, validate it, or build something on top. SynBioHub serves SBOL 2 by default:

```rust,no_run
use sbol::convert::upgrade_from_sbol2_path;
use sbol::v3::SbolIdentified;

let (document, report) = upgrade_from_sbol2_path("BBa_F2620.xml")?;
for warning in report.warnings() {
    eprintln!("{warning:?}");
}
document.check()?;

for component in document.components() {
    println!("{}", component.display_id().unwrap_or("(unnamed)"));
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

On the CLI:

```sh
sbol upgrade BBa_F2620.xml --output BBa_F2620.ttl --validate
```

The result is fully native SBOL 3 — all the typed accessors (`document.components()`, `document.sequences()`, etc.) work, all spec rules apply, and the document round-trips through any of the supported RDF serializations.

**A note on identities.** SBOL 2 embeds versions in IRIs as a trailing path segment: `…/BBa_F2620/1`. SBOL 3 doesn't version IRIs. The upgrade strips trailing version segments (`/1`, `/2.3`, …) so `…/BBa_F2620/1` becomes `…/BBa_F2620`. The original version isn't lost — it's preserved under [the backport namespace](#the-backport-namespace) so a future `sbol downgrade` rebuilds the SBOL 2 identity exactly.

### Importing a GenBank file from SnapGene

You designed a plasmid in SnapGene, exported it as `pBR322.gb`, and want to drive it through SBOL-aware tooling:

```rust,no_run
use sbol_genbank::GenbankImporter;
use sbol::v3::RdfFormat;

let importer = GenbankImporter::new("https://my-lab.example.org")?;
let (document, _report) = importer.read_path("pBR322.gb")?;
document.check()?;
document.write_path("pBR322.ttl", RdfFormat::Turtle)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

On the CLI:

```sh
sbol import-genbank pBR322.gb --namespace https://my-lab.example.org --output pBR322.ttl
```

Each GenBank feature becomes an SBOL 3 SequenceFeature; coordinates become Ranges or Cuts; the LOCUS sequence becomes a Sequence with the appropriate EDAM encoding; molecule type (DNA/RNA/Protein) becomes the Component's SBO type. The `--namespace` flag is required because GenBank carries no IRI concept — you supply the namespace under which the SBOL 3 identities will be rooted.

### Importing a FASTA file from NCBI

You have a FASTA dump from BLAST, NCBI, or UniProt:

```rust,no_run
use sbol_fasta::FastaImporter;

let importer = FastaImporter::new("https://my-lab.example.org")?;
let (document, _report) = importer.read_path("GFP_protein.fasta")?;
document.check()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

```sh
sbol import-fasta GFP_protein.fasta --namespace https://my-lab.example.org -o GFP.ttl
```

Each FASTA record becomes a Component plus Sequence. Alphabet auto-detects (DNA / RNA / Protein); pass `--alphabet protein` if a short peptide composed only of A/C/G/T letters would otherwise be misclassified as DNA.

### Publishing an SBOL 3 design to SynBioHub

You authored a design natively in SBOL 3 and want to push it to a SynBioHub instance that consumes SBOL 2:

```rust,no_run
use sbol::convert::downgrade;
use sbol::v3::{Document, RdfFormat};

let document = Document::read_path("design.ttl")?;
let (sbol2_graph, report) = downgrade(&document)?;
let sbol2_xml = sbol2_graph.write(RdfFormat::RdfXml)?;
std::fs::write("design.xml", sbol2_xml)?;

for warning in report.warnings() {
    eprintln!("{warning:?}");
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

```sh
sbol downgrade design.ttl --to rdfxml --output design.xml --validate
```

The `--validate` flag is unusual: there is no SBOL 2 schema validator in this workspace, so it instead **round-trips** the downgrade — running the SBOL 2 output back through `sbol upgrade` and validating the resulting SBOL 3. If the round-trip succeeds and the resulting document validates, the downgrade preserved enough structure for `sbol-rs` to recover a valid SBOL 3 graph. For strict SBOL 2 spec compliance, serve the file to libSBOLj2 or pySBOL2 externally.

### Round-tripping SBOL 2 ↔ SBOL 3

If your workflow is "pull SBOL 2 down, edit in SBOL 3 tools, push back to SBOL 2", the trip is near-lossless:

```sh
sbol upgrade BBa_F2620.xml --output BBa_F2620.ttl
# edit in SBOL 3 tools…
sbol downgrade BBa_F2620.ttl --to rdfxml --output BBa_F2620_updated.xml
```

This works because the upgrade preserves SBOL 2 identities, types, versions, and unmapped predicates under a `http://sboltools.org/backport#` namespace. The downgrade reads those triples to rebuild the original SBOL 2 shape. The `corpus_round_trip` test drives the SBOL 2 → 3 → 2 fixed-point over the **full** SBOLTestSuite SBOL 2 corpora and asserts triple-for-triple equality: **290 fixtures round-trip clean** — 0 drift, 0 parse failures, 0 upgrade-unsupported — with a single documented-lossy fixture allowlisted. The backport namespace is also interoperable with sbol-utilities / sbolgraph: the downgrade emits `backport:sbol3namespace` and FunctionalComponent access via `backport:sbol2_access` byte-for-byte matching the predicate IRIs those tools use, and the upgrade honors them (verified in `crates/sbol-convert/tests/interop_backport.rs`). The [empirical per-fixture report](sbol3-round-trip-report.md) covers the committed SynBioHub, iGEM, and SBOLTestSuite real-world fixtures.

### Authoring a design that combines structure and function

SBOL 3 unified what SBOL 2 split: a single SBOL 3 `Component` can carry both **structural** data (a sequence, sub-parts) AND **functional** data (interactions, an interface). For example: a CRISPR guide RNA with a defined sequence (structural) AND an interaction binding Cas9 (functional). SBOL 2 has no single class for that.

When you downgrade such a "dual-role" Component, `sbol-rs` splits it. See [Dual-role Components](#dual-role-components) below for the full mechanism.

## The backport namespace

`http://sboltools.org/backport#` is a namespace `sbol-rs` reads and writes to preserve SBOL 2 detail that has no SBOL 3 home. After `sbol upgrade`, your SBOL 3 document will contain triples like:

```turtle
<https://example.org/lab/BBa_F2620>
    a sbol3:Component ;
    sbol3:displayId "BBa_F2620" ;
    backport:sbol2type sbol2:ComponentDefinition ;
    backport:sbol2version "1" ;
    backport:sbol2persistentIdentity <https://example.org/lab/BBa_F2620> .
```

These triples are **not** part of SBOL 3 — they're round-trip metadata that downstream SBOL 3 consumers can safely ignore. The validator does ignore them. What they record:

| Backport predicate | What it preserves |
|---|---|
| `sbol2type` | Original SBOL 2 rdf:type (`ComponentDefinition`, `ModuleDefinition`, `FunctionalComponent`, `Module`, etc.) so the downgrade restores the exact class the source used |
| `sbol2version` | Version string SBOL 2 embedded in the trailing IRI segment, so the downgrade rebuilds `…/BBa_F2620/1` from `…/BBa_F2620` |
| `sbol2persistentIdentity` | Unversioned identity the SBOL 2 source used in its `persistentIdentity` triples — usually equals the SBOL 3 IRI, recorded explicitly for cases where it differs |
| `sequenceAnnotationDisplayId` | `displayId` of an SBOL 2 SequenceAnnotation that the upgrade [collapsed into a SubComponent](#reversible-structural-collapses) |
| `sequenceAnnotationPredicate_<hex>` | Non-structural triples from that collapsed SequenceAnnotation shell, with the original predicate hex-encoded so the downgrade can replay it on the rebuilt SA |
| `mapsToRefinement` | Original `sbol2:refinement` of a MapsTo. SBOL 3 encodes `useLocal` / `useRemote` / `verifyIdentical` via the Constraint's subject/object position per SBOL 3.1.0 §10.2, so this hint is strictly needed only to preserve `sbol2:merge` literally (the spec collapses `merge` to `useRemote`) and to round-trip extension refinement IRIs the converter doesn't recognize. It's also emitted for the spec-encodable values as defense-in-depth against tools that mutate the Constraint shape downstream. |
| `mapsToDisplayId` | Original `displayId` of an SBOL 2 MapsTo when the synthesized SBOL 3 ComponentReference had to be renamed to avoid an IRI collision |
| `sbol2_<predicate>` | Any SBOL 2 predicate the upgrade doesn't know how to map (e.g. `sbol2:access`) is preserved verbatim under the `sbol2_` prefix |
| `sbol3identity` | On a split CD/MD pair, the original SBOL 3 Component IRI (so external tools and a future re-merge step can match them up) |
| `type = SplitComponentComposition` | Marks the synthesized FunctionalComponent that links a split CD half to its MD half |

A document with backport triples round-trips through SBOL 2 ⇄ SBOL 3 with no structural loss. A document **without** them (one authored natively in SBOL 3) loses more on the way back to SBOL 2 — the downgrade has nothing to consult for the original class and falls back to heuristics.

## Reversible structural collapses

SBOL 3 made several SBOL 2 idioms more compact. The upgrade folds them down; the downgrade re-emits the SBOL 2 shape from the backport hints.

**SequenceAnnotation that references a component.** SBOL 2 expresses "this sub-part lives at positions 100–200" as a SequenceAnnotation wrapping a Component reference and a Location. SBOL 3 collapses this — the SubComponent itself carries the locations directly. On upgrade, the SA's locations migrate to the SubComponent, the SA's `displayId` is preserved as `backport:sequenceAnnotationDisplayId`, and non-structural SA triples are archived under encoded `backport:sequenceAnnotationPredicate_<hex>` predicates. On downgrade, the SA is rebuilt from those hints.

**MapsTo.** SBOL 2's `MapsTo` (mapping a sub-component's port to a parent context) decomposes in SBOL 3 into a `ComponentReference` (carrying `inChildOf` + `refersTo`) plus a `Constraint` (carrying `subject` + `object` + `restriction`). On downgrade, the pair is found by following the Constraint's `object` back to its ComponentReference; the original `<sbol2:MapsTo>` is rebuilt under the carrier SubComponent.

**Interface synthesis.** SBOL 2 expresses port visibility and direction via `sbol2:access` and, for FunctionalComponents, `sbol2:direction` (`in`/`out`/`inout`/`none`). SBOL 3 hoists directional FunctionalComponents, public direction-`none` FunctionalComponents, and public structural Components into a single `sbol3:Interface` object. On downgrade, FunctionalComponent entries flatten back to `sbol2:direction` plus public access for native SBOL 3 inputs; structural Component entries flatten to `sbol2:access public`. When an upgraded document preserved original SBOL 2 access or direction under backport metadata, that original value wins.

## Dual-role Components

SBOL 2 split design into two top-level classes:

- **`ComponentDefinition`** — structural: types (DNA/RNA/protein), roles (promoter, CDS), sequence, sub-parts, sequence annotations, constraints.
- **`ModuleDefinition`** — functional: interactions, functional components, models, interface.

SBOL 3 unified both into a single `Component`. That means a native SBOL 3 design can carry BOTH structural AND functional data in one object — a CRISPR guide RNA with a sequence AND an interaction binding Cas9, a metabolic enzyme with both a CDS sequence AND a kinetic interaction. SBOL 2 has no way to put both on a single object.

When you downgrade such a dual-role Component, `sbol-rs` splits it:

```
sbol3:Component   →   sbol2:ComponentDefinition  (structural half: type, role, sequence, SAs)
                  +   sbol2:ModuleDefinition     (functional half: interactions, FCs, models)
                  +   sbol2:FunctionalComponent  (synthesized, links the MD half to the CD half)
```

**Which half keeps the bare IRI?** If the source carried `backport:sbol2type` (i.e. the SBOL 3 came from a prior `sbol upgrade`), that half keeps the original IRI and the synthesized half gets a `_component` or `_module` suffix. For native SBOL 3 (no backport hint), the heuristic is: if the Component has interactions, the MD keeps the bare IRI; otherwise the CD keeps it.

**How is dual-role detected?** Per-Component classification:

- **CD-only** — has structural signals only (type, role, hasSequence, hasFeature → SequenceFeature or located SubComponent, hasConstraint) OR a `backport:sbol2type = ComponentDefinition` hint. Most SBOL 2 ComponentDefinitions round-trip through this branch and emit as plain CDs.
- **MD-only** — has functional signals only (hasInteraction, hasInterface, hasModel, `sbol3:type = SBO:functionalEntity`) OR a `backport:sbol2type = ModuleDefinition` hint. Most SBOL 2 ModuleDefinitions round-trip through this branch.
- **Dual-role** — has BOTH structural and functional signals. Triggers the split.

A `DowngradeWarning::DualRoleComponent` is emitted for each split with the original SBOL 3 IRI and both halves' SBOL 2 IRIs so callers can audit the choice.

**Collection members.** When a Collection lists a dual-role Component as a member, the downgrade emits the membership twice — once for each half — so SBOL 2 consumers see both the structural and functional view.

**SubComponents under a dual-role parent.** Each SubComponent triple-emits into three SBOL 2 objects: a `sbol2:Component` under the CD half (`_c` suffix), a `sbol2:FunctionalComponent` under the MD half (`_fc`), and a `sbol2:Module` under the MD half (`_m`, only when the SubComponent's target is itself MD-shaped). Whichever variant matches the SubComponent's `backport:sbol2type` hint keeps the bare IRI. The downgrade allocates the suffixed variants against a pass-wide used-IRI set, so any collision with an existing subject (e.g. siblings named `foo` and `foo_fc` where `foo`'s FC variant would otherwise land on `foo_fc`) gets disambiguated by appending `_2`, `_3`, ….

## Known intentional divergences

Some triples don't survive SBOL 2 → 3 → 2 byte-for-byte. These are documented because the conversion is *correct* — they're irreversible folds, not bugs.

- **`biopax:Dna` vs `biopax:DnaRegion`** — both map to `SBO:DNA` on upgrade. Documents upgraded by `sbol-rs` preserve the original BioPAX value under backport metadata and restore it losslessly; native SBOL 3 inputs without that hint downgrade to the `*Region` convention. Same for `Rna` / `RnaRegion`.
- **Native SBOL 3 ComponentInstance defaults** — native SubComponents have no SBOL 2 `access` field. Downgrade emits private access by default, and emits `direction none` for native FunctionalComponents unless an Interface supplies a direction. Upgraded SBOL 2 inputs keep their original access/direction instead of receiving synthesized defaults.
- **`dcterms:title` ↔ `sbol3:name`** (and `dcterms:description` ↔ `sbol3:description`) — the upgrade preserves the Dublin Core triple as-is AND synthesizes the SBOL 3 form. The downgrade emits a single `dcterms:title` to avoid duplication.
- **`SBO:functionalEntity` on MD-derived Components** — the upgrade adds this so SBOL 3's type cardinality is satisfied (every Component needs ≥1 type). The downgrade drops it when emitting back to `ModuleDefinition`.
- **`sbol3:hasNamespace`** — dropped on downgrade. SBOL 2 has no explicit namespace property; the namespace is implicit in the persistentIdentity.

## Known limitations

- **Native SBOL 3 → SBOL 2 → SBOL 3 re-merge.** When a native (no-backport) SBOL 3 dual-role Component is downgraded to a CD + MD pair, the upgrade currently doesn't re-fuse them: the two halves come back as separate Components. The `backport:sbol3identity` stamps are already in place to enable the re-merge; the upgrade-side detection isn't yet implemented. Documents that originated as SBOL 2 round-trip cleanly because the backport hints disambiguate.
- **LocalSubComponent / ExternallyDefined / ComponentReference under dual-role parents.** Currently treated like SubComponents but without bespoke triple-emit handling. The basic shape works; specialized SBOL 2 surface (e.g. `sbol2:ExternalDefinition`) isn't fully covered.
- **No native SBOL 2 schema validator.** `sbol downgrade --validate` round-trips the output back through SBOL 3 and validates *that* — a proxy for SBOL 2 structural correctness, not an SBOL 2 spec check.

## Where to look next

- **API reference** — the conversion functions live in [`sbol-convert`](https://docs.rs/sbol-convert) (re-exported as `sbol::convert`); GenBank and FASTA importers under [`sbol_genbank`](https://docs.rs/sbol-genbank) and [`sbol_fasta`](https://docs.rs/sbol-fasta).
- **Per-fixture round-trip results** — [`sbol3-round-trip-report.md`](sbol3-round-trip-report.md), regenerated by `cargo run -p sbol-convert --bin generate-round-trip-report` against the committed SBOL 2 fixture corpus.
- **Conformance gates and how the test harness works** — [`sbol2-upgrade-conformance.md`](sbol2-upgrade-conformance.md), [`sbol3-downgrade-conformance.md`](sbol3-downgrade-conformance.md).
- **Import paths in depth** — [`genbank-import-conformance.md`](genbank-import-conformance.md), [`fasta-import-conformance.md`](fasta-import-conformance.md).

[`sbol_convert::upgrade_from_sbol2`]: https://docs.rs/sbol-convert/latest/sbol_convert/fn.upgrade_from_sbol2.html
[`sbol_convert::downgrade`]: https://docs.rs/sbol-convert/latest/sbol_convert/fn.downgrade.html
[`sbol_genbank::GenbankImporter`]: https://docs.rs/sbol-genbank/latest/sbol_genbank/struct.GenbankImporter.html
[`sbol_fasta::FastaImporter`]: https://docs.rs/sbol-fasta/latest/sbol_fasta/struct.FastaImporter.html
