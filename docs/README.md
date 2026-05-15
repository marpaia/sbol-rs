# sbol-rs documentation

Entry points to the project's documentation, organized by topic.

## Getting oriented

- **[Crate guide](crate-guide.md)**: architectural tour covering
  workspace layout, data model, document lifecycle, validation
  pipeline, key decision points. **Start here if you're new to the
  codebase.**

## Validation

The sbol-rs validator covers SBOL 3.1.0 with structured diagnostics and
a coverage signal. Read these in order:

- **[Validation system overview](validation.md)**: narrative entry
  point. What the validator does, how to use it, how to read its
  output, where the trust boundaries are.
- [Conformance report](conformance.md): generated per-rule status
  grid for all 149 spec rules. Headline: 109/109 machine-checkable
  rules implemented (100%). Regenerate with
  `cargo run -p sbol --bin generate-conformance-report`.
- [JSON v1 output schema](validation-output.md): reference for
  downstream tools consuming `sbol validate --format json`. Includes
  the SARIF v2.1.0 mapping.
- [Policy ADRs](policies/): committed design decisions for rules
  where the spec is ambiguous. One ADR per `Policy`-blocked rule.

## I/O

- [RDF I/O](rdf-io.md): Turtle, RDF/XML, JSON-LD, N-Triples support;
  format inference; round-trip guarantees.

## Testing and contribution

- [Test architecture](testing.md): regression cases, fixtures, fuzz
  targets, property tests, cross-implementation conformance harness.
- [SBOL schema conformance regression](ontology-conformance.md):
  pinned `sbol-owl3` fixture, the IRI-level cross-check against
  `vocab.rs`, and how to refresh the pin.
- [`sbol-owl3` conformance report](sbol-owl3-conformance.md):
  generated per-IRI coverage table. Regenerate with
  `cargo run -p sbol --bin generate-sbol-owl3-conformance-report`.
