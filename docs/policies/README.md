# Validation Policy ADRs

Each file in this directory is an Architecture Decision Record for a
single SBOL 3.1.0 validation rule whose semantics the spec leaves under-
specified. The validator's `PolicyOptions` (in
`crates/sbol3/src/validation/options.rs`) carries one tunable per theme,
and each tunable's variants map onto the decisions captured here.

## Format

Each ADR follows the same 5-section structure:

1. **Spec wording**: verbatim quote from the SBOL 3.1.0 specification.
2. **Ambiguity**: what the spec leaves unspecified.
3. **Decision**: the rule we apply in code, defaulting to `Conservative`.
4. **Rationale**: why this default; what changes under `Strict`/`Lenient`.
5. **Examples**: concise illustration of each mode.

## Theme clusters

| Theme | PolicyOptions knob | Rules |
|---|---|---|
| Conflict resolution | `conflict_resolution` | sbol3-10617, sbol3-11705, sbol3-11706 |
| Combinatorial provenance | `derivation_provenance` | sbol3-12104, sbol3-12105, sbol3-12107, sbol3-12108, sbol3-12109, sbol3-12110, sbol3-12111, sbol3-12112, sbol3-12114, sbol3-12115 |
| Derived collection membership | `derived_collection_membership` | sbol3-12106 |
| Hash algorithm registry | `hash_algorithm_registry` | sbol3-12806 |
| Workflow recommendations | (no knob; coverage-only) | sbol3-10205, sbol3-12901, sbol3-12902, sbol3-13001, sbol3-13401 |
| Faithful built check | (no knob; ▲ Deferred) | sbol3-12303 |
| Unknown SBOL predicates | `unknown_sbol_predicates` | sbol3-10105 |
| Topology completeness | `topology_completeness` | sbol3-10606, sbol3-11006, sbol3-11106 |

## Build-script gate

`crates/sbol3/build.rs` enforces that every rule with `blocker = "Policy"`
has a corresponding `docs/policies/<rule-id>.md` file. Drift fails the
build with the offending rule ID, not a generic missing-file error.
