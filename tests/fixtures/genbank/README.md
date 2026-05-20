# GenBank source corpus

Real GenBank files (the lingua franca exported by SnapGene, ApE, Benchling,
and the iGEM/NCBI repositories) used to seed the SBOL 2 → SBOL 3
conformance pipeline. Each `.gb` file is fed through `sbol-genbank` and
`sbol::downgrade` to produce
`tests/fixtures/sbol2/real/from_genbank/<name>.xml` — the SBOL 2 RDF/XML
the conformance gate actually exercises. Both source and intermediate
are committed; the regeneration pipeline runs in pure Rust.

## Provenance

All files are public domain or freely-redistributable. Each entry below
records the upstream URL the regen script used.

| File | Provenance | Description |
|---|---|---|
| `BBa_E0040.gb` | <https://synbiohub.org/public/igem/BBa_E0040/1/gb> | GFP CDS (Aequorea victoria) — the canonical iGEM "hello world" |
| `BBa_R0010.gb` | <https://synbiohub.org/public/igem/BBa_R0010/1/gb> | LacI-repressible promoter |
| `BBa_B0034.gb` | <https://synbiohub.org/public/igem/BBa_B0034/1/gb> | Elowitz RBS |
| `BBa_F2620.gb` | <https://synbiohub.org/public/igem/BBa_F2620/1/gb> | PoPS receiver — composite design exercising sub-parts, annotations |
| `pUC19.gbk` | NCBI accession `M77789.2` via `efetch.fcgi?db=nuccore&id=M77789.2&rettype=gb` | The canonical cloning vector. `.gbk` extension confirms both common GenBank file-naming conventions (NCBI `.gb`, SnapGene/community `.gbk`) round-trip through the importer. |

Refresh by re-fetching from upstream:

```sh
# SynBioHub iGEM parts
for part in BBa_E0040 BBa_R0010 BBa_B0034 BBa_F2620; do
  curl -fsSL -A "sbol-rs-fixture-fetch" \
    "https://synbiohub.org/public/igem/${part}/1/gb" \
    -o "tests/fixtures/genbank/${part}.gb"
done

# NCBI pUC19 — note the .gbk extension for parity with SnapGene-style exports
curl -fsSL --max-time 30 \
  'https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=nuccore&id=M77789.2&rettype=gb&retmode=text' \
  -o tests/fixtures/genbank/pUC19.gbk

cargo run -p sbol-genbank --bin regenerate-genbank-import-snapshots
```

After re-fetching, refresh the SBOL 2 intermediates so the upgrade
conformance gate sees the new sources:

```sh
cargo run -p sbol-genbank --bin regenerate-from-genbank-sbol2-intermediates
cargo run -p sbol --bin regenerate-sbol2-upgrade-snapshots
```

Both commands run in pure Rust.

## Why GenBank input matters

Most synthetic biology designs originate in tools that speak GenBank
(SnapGene, ApE, Benchling, NCBI). Validating only against SBOL 2 fixtures
authored by SBOL-aware tools (SBOLTestSuite, SynBioHub exports) leaves a
blind spot for the conversion path real users take: design in SnapGene
→ export `.gb` → upload to SynBioHub or convert offline → SBOL 2 → SBOL 3.
The GenBank-rooted fixtures here close that gap.

See [`docs/sbol2-upgrade-conformance.md`](../../../docs/sbol2-upgrade-conformance.md)
for the full pipeline design.
