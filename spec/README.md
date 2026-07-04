# SBOL Specification

If you are looking for the SBOL 3.1.0 data model specification, start with
[SBOL3.1.0.md](SBOL3.1.0.md). SBOL defines the core model for representing synthetic biology
designs, including identifiers, components, sequences, features, interactions, provenance,
serialization, and validation rules. The specification was originally published as a PDF, which is
kept here as the authoritative source artifact, but the Markdown version is the repo-friendly copy to
read, search, link to, and review while working on this crate.

The SBOL 2.3 data model is also kept here for reference. SBOL 3 is a new data model rather than a
revision of SBOL 2, so the two specifications describe different classes (for example, SBOL 2's
`ComponentDefinition`/`ModuleDefinition` split versus SBOL 3's unified `Component`). Consult
[SBOL2.3.0.md](SBOL2.3.0.md) when working with SBOL 2 documents or mapping between the two versions.

This directory keeps the specifications in Markdown and PDF form:

- [SBOL3.1.0.md](SBOL3.1.0.md): the curated Markdown copy of the SBOL 3.1.0 data model.
- [SBOL3.1.0.pdf](SBOL3.1.0.pdf): the official SBOL 3.1.0 PDF source artifact.
- [SBOL2.3.0.md](SBOL2.3.0.md): the curated Markdown copy of the SBOL 2.3 data model.
- [SBOL2.3.0.pdf](SBOL2.3.0.pdf): the official SBOL 2.3 PDF source artifact.

The PDF is the authoritative source. The `spec/` directory is excluded from the published crate
package so the crates.io artifact stays small while the repository retains the implementation
reference material.

### SBOL 3.1.0 Specification Provenance

- Version: 3.1.0
- Source: <https://sbolstandard.org/docs/SBOL3.1.0.pdf>
- Canonical page: <https://sbolstandard.org/datamodel-specification/version-3.1.0/>
- Upstream LaTeX source: <https://github.com/SynBioDex/SBOL-specification/tree/v3.1.0>
- Upstream tag: `v3.1.0` (`caf079d9b7e82a96994b8f33160eaf0d436e6731`)
- Published: October 26, 2022
- License: Creative Commons Attribution 4.0 International Public License
- PDF SHA-256: `7c1ef88f83b8fff98acd07c742b377bbb8618508684b7dab17032396667f0b2c`

`SBOL3.1.0.md` is a hand-curated Markdown transcription of the PDF. It was initially extracted from
the PDF, but ongoing cleanup should be made directly in the Markdown and checked against the
official PDF or tagged LaTeX source when fidelity is in question.

### SBOL 2.3.0 Specification Provenance

- Version: 2.3
- Source: <https://sbolstandard.org/docs/SBOL2.3.0.pdf>
- Canonical page: <https://sbolstandard.org/datamodel-specification/version-2-3-0/>
- Upstream LaTeX source: <https://github.com/SynBioDex/SBOL-specification/tree/v2.3.0>
- Upstream tag: `v2.3.0` (`9833489734af1742bd23d87eefa1bb247bdb34cb`)
- Published: April 1, 2019
- License: Creative Commons Attribution 4.0 International Public License
- PDF SHA-256: `6ad000f596a26517d1d0a4cf8f53af87538d8703befea35d2383b5ade7558c88`

`SBOL2.3.0.md` is a Markdown transcription generated from the tagged LaTeX source and reconciled
against the official PDF. Figures are rendered from the upstream vector sources into
`figures/sbol2.3.0/`. Ongoing cleanup should be made directly in the Markdown and checked against
the official PDF or tagged LaTeX source when fidelity is in question.

### 3.1.0 Change Notes

These notes track implementation-relevant differences from 3.0.1.

- Identifier terminology shifts from URI toward IRI, with URLs used for SBOL namespace terms,
  compliant object identifiers, and controlled ontology/resource terms. `TopLevel.hasNamespace` is
  now typed as URL in the property cardinality table.
- Section 5.1 is now "Internationalized Resource Identifiers" and Section 5.2 is now "SBOL URLs".
- Recommended best practices now describe compliant object construction in terms of URLs while
  preserving IRIs for general object identity and external references.
- Recommended external ontology guidance now says ontological-term IRIs SHOULD be identifiers.org
  URLs, with purl.org terms allowed as alternatives when RDF tooling needs compliant QNames.
- Host-context recommendations changed several terms: Cell is `NCIT:C12508`, Growth Medium is
  `NCIT:C85504`, Organism Strain is `NCIT:C14419`, and cell type examples now use Cell Ontology
  `CL:0000000`.
- Mapping guidance between SBOL 2 and SBOL 3 now explicitly covers IRI/URI conversion,
  `persistentIdentity` to `identity`, retained SBOL 2 version metadata, SBOL 2 identity construction
  from SBOL 3 URLs, and `hasNamespace` retention.
- Validation rule IDs remain the same count as 3.0.1, but rule text now reflects IRI/URL
  terminology and the Component modeling recommendation `sbol3-10604` now points to the physical
  entity representation branch of SBO rather than simply Table 2.
- Figure and table captions were refreshed from the 3.1.0 LaTeX source, with URL labels used for
  controlled term tables and IRI labels retained for general object-property types.

### Maintenance Conventions

- Keep section headings aligned with the PDF, using dotted numeric labels such as `## 1. Purpose`.
- Keep the table of contents at the top of the document linked to Markdown section anchors.
- Use the tagged LaTeX source as an audit reference for section structure, captions, footnotes,
  tables, and validation rule IDs.
- Format front-matter role labels as bold text, and format people as `Name: *Institution*`.
- Prefer readable Markdown paragraphs over PDF-extracted hard wraps or merged paragraphs.
- Use descriptive figure alt text and keep figure captions italic with a bold-italic label, e.g.,
  `***Figure 1:*** *Caption text.*`
- Use the same caption strategy for tables, e.g., `***Table 1:*** *Caption text.*`
- Treat literal ontology and namespace IRIs/URLs in prose as code spans unless they are intentionally
  presented as navigational links.
