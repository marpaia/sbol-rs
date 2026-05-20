# sbol-cli

Command-line tool for SBOL 3 documents. Ships the `sbol` binary.

```sh
cargo install sbol-cli
```

Eight subcommands cover the common workflows:

| Subcommand | Use it to… |
|---|---|
| `sbol validate` | Validate an SBOL 3 document against the spec |
| `sbol convert` | Cross-serialize SBOL 3 between Turtle, RDF/XML, JSON-LD, N-Triples |
| `sbol upgrade` | Convert SBOL 2 RDF (SynBioHub, iGEM, JBEI) to SBOL 3 |
| `sbol downgrade` | Convert SBOL 3 back to SBOL 2 for legacy tools |
| `sbol import-genbank` | Convert a GenBank `.gb` / `.gbk` file to SBOL 3 |
| `sbol import-fasta` | Convert a FASTA file to SBOL 3 |
| `sbol rules list` | Inspect the built-in validation rule catalog |
| `sbol ontology install` | Manage cached extension ontologies (NCIT, custom) |

The conversion path is explained in depth in
[docs/conversion.md](https://github.com/marpaia/sbol-rs/blob/master/docs/conversion.md);
this README focuses on the CLI surface itself.

## `sbol validate`

Validate an SBOL 3 document. The serialization format is inferred from the
file extension: `.ttl` (Turtle), `.rdf` (RDF/XML), `.jsonld` (JSON-LD), or
`.nt` (N-Triples):

```sh
sbol validate design.ttl
sbol validate design.rdf --format json --treat-partial-as-errors
sbol validate design.rdf --allow sbol3-10502 --deny sbol3-12807
```

Pass `--treat-warnings-as-errors` to make `sbol validate` exit `1` on
warnings as well as errors.

Exit codes:

| Code | Meaning |
| ---- | ---- |
| `0` | Document parses and validates with no errors |
| `1` | Validation errors found (printed with rule IDs) |
| `2` | I/O failure, unsupported / missing file extension, or bad CLI usage |
| `3` | `--treat-partial-as-errors` and at least one rule is partially applied |

## `sbol convert`

Cross-serialize an SBOL 3 document between RDF formats. The input format is
inferred from the source extension; the target format is taken from `--to`
or inferred from `--output`:

```sh
sbol convert design.ttl --output design.jsonld
sbol convert design.ttl --to ntriples > design.nt
```

## `sbol upgrade`

Convert an SBOL 2 RDF document to SBOL 3. Most published synbio content
predates SBOL 3 — SynBioHub serves SBOL 2 by default, iGEM Registry parts
ship as SBOL 2, JBEI ICE exports SBOL 2. Upgrade once on ingest and use
the modern toolchain:

```sh
sbol upgrade BBa_F2620.xml --output BBa_F2620.ttl
sbol upgrade design.rdf --to turtle > design.ttl
sbol upgrade design.xml --output design.ttl --validate --strict
```

Notable flags:

- `--from <FORMAT>` overrides input-format inference (useful for SBOL 2
  files distributed with non-standard extensions).
- `--namespace <IRI>` supplies a fallback `hasNamespace` for top-level
  objects whose namespace can't be derived from the input. Without it,
  the upgrade falls back to the URL scheme+host, then to omitting
  `hasNamespace` entirely.
- `--report text|json` prints a structured per-construct summary
  (warnings + counts of CDs / MDs / SubComponents / MapsTos / collapses
  rewritten).
- `--validate` runs `Document::check` on the converted document and
  folds the result into the exit code.
- `--strict` exits `1` if any conversion warnings were produced.

The full conversion model — what the upgrade preserves, what it can't,
what triggers warnings — is documented in
[docs/conversion.md](https://github.com/marpaia/sbol-rs/blob/master/docs/conversion.md).

## `sbol downgrade`

Convert an SBOL 3 RDF document back to SBOL 2 — for publishing to
SynBioHub, libSBOLj2, pySBOL2, or any other tool that hasn't migrated.

```sh
sbol downgrade design.ttl --to rdfxml --output design.xml
sbol downgrade design.ttl --to turtle --output design_sbol2.ttl --validate
```

The downgrade is the inverse of `sbol upgrade`. Documents that came
through `sbol upgrade` round-trip with near-zero loss because the
upgrade preserves SBOL 2 identities and types under a
`http://sboltools.org/backport#` namespace; the downgrade reads those
triples back. Native SBOL 3 documents lose more — see
[docs/conversion.md](https://github.com/marpaia/sbol-rs/blob/master/docs/conversion.md)
for the loss model.

Notable flags:

- `--default-version <VERSION>` sets the version string assigned to
  top-level objects whose source didn't carry `backport:sbol2version`.
  Omit it to leave those objects unversioned; pass `--default-version 1`
  to match the SynBioHub / libSBOLj convention.
- `--from <FORMAT>` overrides input-format inference (useful for SBOL 3
  RDF/XML files with `.xml` or non-standard extensions).
- `--validate` — unusual semantics: there is no SBOL 2 schema validator
  in this workspace, so `--validate` instead round-trips the produced
  SBOL 2 back through `sbol upgrade` and validates the resulting SBOL 3.
  If the round-trip succeeds and the resulting document validates, the
  SBOL 2 is structurally sound for any downstream consumer.
- `--strict` exits `1` if any downgrade warnings were produced
  (`DualRoleComponent` splits, `OrphanComponentReference` drops, etc.).

## `sbol import-genbank`

Convert a GenBank flat-file to SBOL 3. The whole pipeline runs in pure
Rust:

```sh
sbol import-genbank pBR322.gb \
    --namespace https://my-lab.example.org \
    --output pBR322.ttl
```

`--namespace` is required because GenBank carries no IRI concept — you
supply the namespace under which the SBOL 3 top-levels will be rooted.

Each GenBank feature becomes an SBOL 3 SequenceFeature; coordinates
become Ranges or Cuts; the LOCUS sequence becomes a Sequence with the
appropriate EDAM encoding; molecule type (DNA / RNA / Protein) becomes
the Component's SBO type. Mixed-case month names in the LOCUS line
(as emitted by SynBioHub) are tolerated.

Notable flags:

- `--to <FORMAT>` chooses the SBOL 3 output serialization; otherwise
  inferred from `--output`'s extension.
- `--validate` runs `Document::check` on the converted document.
- `--strict` exits `1` on any import warnings.

## `sbol import-fasta`

Convert a FASTA file to SBOL 3:

```sh
sbol import-fasta GFP_protein.fasta \
    --namespace https://my-lab.example.org \
    --output GFP.ttl
sbol import-fasta short_peptide.fa \
    --namespace https://my-lab.example.org \
    --alphabet protein
```

Each FASTA record becomes a Component plus Sequence. Alphabet
auto-detects from the residue alphabet (DNA / RNA / Protein); pass
`--alphabet` when a short peptide composed only of A/C/G/T letters
would otherwise be misclassified as DNA.

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

## `sbol ontology`

Manage runtime ontology extensions for the validator. The bundled
snapshot (EDAM, SBO, SO, GO, ChEBI, CL) is always available; this
subcommand handles opt-in extensions (NCIT, lab-specific ontologies):

```sh
sbol ontology install ncit       # download and build NCIT into the cache
sbol ontology list               # show installed extensions
sbol ontology path               # print cache directory
sbol ontology verify             # check every installed extension's hash
sbol ontology remove ncit        # uninstall

sbol validate design.ttl --ontology ncit   # opt into the cached extension
```

The cache is the only IO surface; validation itself never touches the
network. See
[docs/ontology-extensions.md](https://github.com/marpaia/sbol-rs/blob/master/docs/ontology-extensions.md)
for the install workflow and the stable TSV contract.

## Backing library

The CLI is a thin wrapper around the [`sbol`](https://crates.io/crates/sbol)
library; everything is available programmatically. See the workspace
README for the Rust SDK side and the docs directory for in-depth guides.

## License

Dual-licensed under MIT or Apache-2.0, at your option. See the
[workspace README](https://github.com/marpaia/sbol-rs#readme) for project
context.
