# FASTA source corpus

Real FASTA files driving the [GenBank/FASTA import conformance
gates](../../../docs/fasta-import-conformance.md). Each file is
publicly redistributable.

## Provenance

| File | Provenance | Description |
|---|---|---|
| `pUC19.fasta` | NCBI accession `M77789.2` via `efetch.fcgi?db=nuccore&id=M77789.2&rettype=fasta` | The canonical cloning vector — DNA, 2.7 kb. |
| `pBR322.fasta` | NCBI accession `J01749.1` via `efetch.fcgi?db=nuccore&id=J01749.1&rettype=fasta` | The other canonical cloning vector — DNA, 4.4 kb. |
| `GFP_protein.fasta` | UniProt `P42212` via `https://www.uniprot.org/uniprotkb/P42212.fasta` | GFP (Aequorea victoria) — protein, 238 aa. Single-record. |
| `multi_protein.fasta` | UniProt `P42212` + `P03023` via `rest.uniprot.org/uniprotkb/search?query=accession:P42212+OR+accession:P03023&format=fasta` | Two proteins (GFP + LacI) — exercises multi-record handling. |

## Refresh

```sh
curl -fsSL --max-time 30 \
  'https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=nuccore&id=M77789.2&rettype=fasta&retmode=text' \
  -o tests/fixtures/fasta/pUC19.fasta

curl -fsSL --max-time 30 \
  'https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=nuccore&id=J01749.1&rettype=fasta&retmode=text' \
  -o tests/fixtures/fasta/pBR322.fasta

curl -fsSL --max-time 30 \
  'https://www.uniprot.org/uniprotkb/P42212.fasta' \
  -o tests/fixtures/fasta/GFP_protein.fasta

curl -fsSL --max-time 30 \
  'https://rest.uniprot.org/uniprotkb/search?query=accession:P42212+OR+accession:P03023&format=fasta' \
  -o tests/fixtures/fasta/multi_protein.fasta

cargo run -p sbol-fasta --bin regenerate-fasta-import-snapshots
```

## Why FASTA matters

FASTA is the input format for almost every sequence-only tool —
BLAST, alignment programs, primer designers, codon optimizers, and
the bulk of genome / proteome downloads. Supporting it natively
means `sbol-rs` can ingest data from those tools without an
intermediate conversion step.
