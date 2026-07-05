//! Common ontology IRIs for building SBOL documents.
//!
//! These mirror pySBOL3's `sbol3.constants` module — the constants are the
//! canonical IRIs from the Systems Biology Ontology (SBO), Sequence Ontology
//! (SO), EDAM, and the SBOL vocabulary itself. Each constant is a [`pub const
//! Iri`](crate::Iri) built via [`Iri::from_static`](crate::Iri::from_static),
//! so they cost nothing to use as builder arguments.
//!
//! ```
//! use sbol3::constants::{SBO_DNA, SO_PROMOTER};
//! let types = [SBO_DNA];
//! let roles = [SO_PROMOTER];
//! # let _ = (types, roles);
//! ```

use crate::Iri;
use crate::vocab;

// === Systems Biology Ontology (SBO) — Component.type values ===

pub const SBO_DNA: Iri = Iri::from_static("https://identifiers.org/SBO:0000251");
pub const SBO_RNA: Iri = Iri::from_static("https://identifiers.org/SBO:0000250");
pub const SBO_PROTEIN: Iri = Iri::from_static("https://identifiers.org/SBO:0000252");
pub const SBO_SIMPLE_CHEMICAL: Iri = Iri::from_static("https://identifiers.org/SBO:0000247");
pub const SBO_NON_COVALENT_COMPLEX: Iri = Iri::from_static("https://identifiers.org/SBO:0000253");
pub const SBO_FUNCTIONAL_ENTITY: Iri = Iri::from_static("https://identifiers.org/SBO:0000241");

// === Systems Biology Ontology (SBO) — Interaction.type values ===

pub const SBO_INHIBITION: Iri = Iri::from_static("https://identifiers.org/SBO:0000169");
pub const SBO_STIMULATION: Iri = Iri::from_static("https://identifiers.org/SBO:0000170");
pub const SBO_BIOCHEMICAL_REACTION: Iri = Iri::from_static("https://identifiers.org/SBO:0000176");
pub const SBO_NON_COVALENT_BINDING: Iri = Iri::from_static("https://identifiers.org/SBO:0000177");
pub const SBO_GENETIC_PRODUCTION: Iri = Iri::from_static("https://identifiers.org/SBO:0000589");
pub const SBO_CONTROL: Iri = Iri::from_static("https://identifiers.org/SBO:0000168");
pub const SBO_DEGRADATION: Iri = Iri::from_static("https://identifiers.org/SBO:0000179");

// === Systems Biology Ontology (SBO) — Participation.role values ===

pub const SBO_INHIBITOR: Iri = Iri::from_static("https://identifiers.org/SBO:0000020");
pub const SBO_STIMULATOR: Iri = Iri::from_static("https://identifiers.org/SBO:0000459");
pub const SBO_REACTANT: Iri = Iri::from_static("https://identifiers.org/SBO:0000010");
pub const SBO_PRODUCT: Iri = Iri::from_static("https://identifiers.org/SBO:0000011");
pub const SBO_PROMOTER_PARTICIPATION: Iri = Iri::from_static("https://identifiers.org/SBO:0000598");
pub const SBO_TEMPLATE: Iri = Iri::from_static("https://identifiers.org/SBO:0000645");
pub const SBO_MODIFIER: Iri = Iri::from_static("https://identifiers.org/SBO:0000019");
pub const SBO_MODIFIED: Iri = Iri::from_static("https://identifiers.org/SBO:0000644");

// === Sequence Ontology (SO) — Component.role values ===

pub const SO_PROMOTER: Iri = Iri::from_static("https://identifiers.org/SO:0000167");
pub const SO_CDS: Iri = Iri::from_static("https://identifiers.org/SO:0000316");
pub const SO_RBS: Iri = Iri::from_static("https://identifiers.org/SO:0000139");
pub const SO_TERMINATOR: Iri = Iri::from_static("https://identifiers.org/SO:0000141");
pub const SO_OPERATOR: Iri = Iri::from_static("https://identifiers.org/SO:0000057");
pub const SO_GENE: Iri = Iri::from_static("https://identifiers.org/SO:0000704");
pub const SO_MRNA: Iri = Iri::from_static("https://identifiers.org/SO:0000234");
pub const SO_ENGINEERED_REGION: Iri = Iri::from_static("https://identifiers.org/SO:0000804");
pub const SO_ENGINEERED_GENE: Iri = Iri::from_static("https://identifiers.org/SO:0000280");

// === Sequence Ontology (SO) — topology values ===

pub const SO_CIRCULAR: Iri = Iri::from_static("https://identifiers.org/SO:0000988");
pub const SO_LINEAR: Iri = Iri::from_static("https://identifiers.org/SO:0000987");

// === EDAM — Sequence.encoding values ===

pub const EDAM_IUPAC_DNA: Iri = Iri::from_static("https://identifiers.org/edam:format_1207");
pub const EDAM_IUPAC_PROTEIN: Iri = Iri::from_static("https://identifiers.org/edam:format_1208");
pub const EDAM_SMILES: Iri = Iri::from_static("https://identifiers.org/edam:format_1196");
pub const EDAM_INCHI: Iri = Iri::from_static("https://identifiers.org/edam:format_1197");

// === Orientation values ===
//
// SBOL 3.1.0 accepts either the SBOL-vocabulary forms or the equivalent SO
// terms; the validator treats them as interchangeable. The `ORIENTATION_*`
// names point at the SBOL forms because pySBOL3 uses those as its defaults.

pub const ORIENTATION_INLINE: Iri = Iri::from_static(vocab::SBOL_INLINE);
pub const ORIENTATION_REVERSE_COMPLEMENT: Iri = Iri::from_static(vocab::SBOL_REVERSE_COMPLEMENT);

// === SubComponent.roleIntegration values ===

pub const ROLE_INTEGRATION_MERGE_ROLES: Iri = Iri::from_static(vocab::SBOL_MERGE_ROLES);
pub const ROLE_INTEGRATION_OVERRIDE_ROLES: Iri = Iri::from_static(vocab::SBOL_OVERRIDE_ROLES);

// === CombinatorialDerivation.strategy values ===

pub const STRATEGY_ENUMERATE: Iri = Iri::from_static(vocab::SBOL_ENUMERATE);
pub const STRATEGY_SAMPLE: Iri = Iri::from_static(vocab::SBOL_SAMPLE);

// === VariableFeature.cardinality values ===

pub const CARDINALITY_ONE: Iri = Iri::from_static(vocab::SBOL_ONE);
pub const CARDINALITY_ZERO_OR_ONE: Iri = Iri::from_static(vocab::SBOL_ZERO_OR_ONE);
pub const CARDINALITY_ONE_OR_MORE: Iri = Iri::from_static(vocab::SBOL_ONE_OR_MORE);
pub const CARDINALITY_ZERO_OR_MORE: Iri = Iri::from_static(vocab::SBOL_ZERO_OR_MORE);

// === Constraint.restriction values ===

pub const RESTRICTION_VERIFY_IDENTICAL: Iri = Iri::from_static(vocab::SBOL_VERIFY_IDENTICAL);
pub const RESTRICTION_DIFFERENT_FROM: Iri = Iri::from_static(vocab::SBOL_DIFFERENT_FROM);
pub const RESTRICTION_REPLACES: Iri = Iri::from_static(vocab::SBOL_REPLACES);
pub const RESTRICTION_SAME_ORIENTATION_AS: Iri = Iri::from_static(vocab::SBOL_SAME_ORIENTATION_AS);
pub const RESTRICTION_OPPOSITE_ORIENTATION_AS: Iri =
    Iri::from_static(vocab::SBOL_OPPOSITE_ORIENTATION_AS);
pub const RESTRICTION_IS_DISJOINT_FROM: Iri = Iri::from_static(vocab::SBOL_IS_DISJOINT_FROM);
pub const RESTRICTION_STRICTLY_CONTAINS: Iri = Iri::from_static(vocab::SBOL_STRICTLY_CONTAINS);
pub const RESTRICTION_CONTAINS: Iri = Iri::from_static(vocab::SBOL_CONTAINS);
pub const RESTRICTION_EQUALS: Iri = Iri::from_static(vocab::SBOL_EQUALS);
pub const RESTRICTION_MEETS: Iri = Iri::from_static(vocab::SBOL_MEETS);
pub const RESTRICTION_COVERS: Iri = Iri::from_static(vocab::SBOL_COVERS);
pub const RESTRICTION_OVERLAPS: Iri = Iri::from_static(vocab::SBOL_OVERLAPS);
pub const RESTRICTION_PRECEDES: Iri = Iri::from_static(vocab::SBOL_PRECEDES);
pub const RESTRICTION_STRICTLY_PRECEDES: Iri = Iri::from_static(vocab::SBOL_STRICTLY_PRECEDES);
pub const RESTRICTION_FINISHES: Iri = Iri::from_static(vocab::SBOL_FINISHES);
pub const RESTRICTION_STARTS: Iri = Iri::from_static(vocab::SBOL_STARTS);

// === Implementation.experiment association tokens (design-build-test-learn) ===

pub const DBTL_DESIGN: Iri = Iri::from_static(vocab::SBOL_DESIGN);
pub const DBTL_BUILD: Iri = Iri::from_static(vocab::SBOL_BUILD);
pub const DBTL_TEST: Iri = Iri::from_static(vocab::SBOL_TEST);
pub const DBTL_LEARN: Iri = Iri::from_static(vocab::SBOL_LEARN);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iri_values_are_well_formed() {
        for iri in [
            &SBO_DNA,
            &SBO_RNA,
            &SBO_PROTEIN,
            &SBO_SIMPLE_CHEMICAL,
            &SO_PROMOTER,
            &SO_CDS,
            &SO_CIRCULAR,
            &EDAM_IUPAC_DNA,
            &EDAM_SMILES,
        ] {
            let s = iri.as_str();
            assert!(s.starts_with("https://identifiers.org/"), "{s}");
        }
    }

    #[test]
    fn vocab_reexports_match_source() {
        assert_eq!(ORIENTATION_INLINE.as_str(), vocab::SBOL_INLINE);
        assert_eq!(STRATEGY_ENUMERATE.as_str(), vocab::SBOL_ENUMERATE);
        assert_eq!(CARDINALITY_ONE.as_str(), vocab::SBOL_ONE);
        assert_eq!(
            RESTRICTION_VERIFY_IDENTICAL.as_str(),
            vocab::SBOL_VERIFY_IDENTICAL
        );
    }
}
