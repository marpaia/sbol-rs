//! Enumerated-value rewrites for the SBOL 2 → SBOL 3 upgrade.
//!
//! Every function here is a pure mapping from SBOL 2 IRI strings to SBOL 3 IRI
//! strings. Unknown inputs are returned unchanged — the conversion is lossless
//! for vocabulary we don't recognize.
#![allow(dead_code)]

use crate::sbol2_vocab as v2;
use crate::vocab as v3;

/// Maps an SBOL 2 orientation IRI to its SBOL 3 equivalent. Returns `None` if
/// the input is not a recognized SBOL 2 orientation.
pub(super) fn map_orientation(iri: &str) -> Option<&'static str> {
    match iri {
        v2::SBOL2_ORIENTATION_INLINE => Some(v3::SBOL_INLINE),
        v2::SBOL2_ORIENTATION_REVERSE_COMPLEMENT => Some(v3::SBOL_REVERSE_COMPLEMENT),
        _ => None,
    }
}

/// Maps an SBOL 2 sequence encoding IRI (legacy IUPAC / opensmiles pages) to
/// the SBOL 3 EDAM equivalent. Returns `None` for unrecognized encodings.
pub(super) fn map_encoding(iri: &str) -> Option<&'static str> {
    match iri {
        v2::SBOL2_ENCODING_IUPAC_DNA => Some(v3::EDAM_IUPAC_DNA_RNA_ENCODING),
        v2::SBOL2_ENCODING_IUPAC_PROTEIN => Some(v3::EDAM_IUPAC_PROTEIN_ENCODING),
        v2::SBOL2_ENCODING_SMILES => Some("https://identifiers.org/edam:format_1196"),
        v2::SBOL2_ENCODING_INCHI => Some("https://identifiers.org/edam:format_1197"),
        _ => None,
    }
}

/// Maps a BioPAX type IRI (as used by SBOL 2 ComponentDefinitions) to the
/// equivalent SBO term used by SBOL 3 Components.
pub(super) fn map_biopax_type(iri: &str) -> Option<&'static str> {
    match iri {
        v2::BIOPAX_DNA | v2::BIOPAX_DNA_REGION => Some("https://identifiers.org/SBO:0000251"),
        v2::BIOPAX_RNA | v2::BIOPAX_RNA_REGION => Some("https://identifiers.org/SBO:0000250"),
        v2::BIOPAX_PROTEIN => Some("https://identifiers.org/SBO:0000252"),
        v2::BIOPAX_SMALL_MOLECULE => Some("https://identifiers.org/SBO:0000247"),
        v2::BIOPAX_COMPLEX => Some("https://identifiers.org/SBO:0000253"),
        _ => None,
    }
}

/// Maps an SBOL 2 MapsTo refinement to an SBOL 3 Constraint restriction.
///
/// `merge` resolves to `sbol3:replaces` because the SBOL 3.1.0 spec
/// (§10.2) directs converters to treat `sbol2:merge` as a synonym for
/// `sbol2:useRemote` — the merge refinement was never well defined and
/// has been removed from SBOL 3.
pub(super) fn map_refinement(iri: &str) -> Option<&'static str> {
    match iri {
        v2::SBOL2_REFINEMENT_VERIFY_IDENTICAL => Some(v3::SBOL_VERIFY_IDENTICAL),
        v2::SBOL2_REFINEMENT_USE_LOCAL => Some(v3::SBOL_REPLACES),
        v2::SBOL2_REFINEMENT_USE_REMOTE => Some(v3::SBOL_REPLACES),
        v2::SBOL2_REFINEMENT_MERGE => Some(v3::SBOL_REPLACES),
        _ => None,
    }
}

/// Maps an SBOL 2 SequenceConstraint restriction IRI to its SBOL 3 equivalent.
/// SBOL 2 hosted these values under `http://sbols.org/v2#…` and SBOL 3 mirrors
/// the same local names under `http://sbols.org/v3#…`. Unrecognized
/// restrictions return `None` and pass through unchanged so callers don't
/// silently lose data.
pub(super) fn map_restriction(iri: &str) -> Option<&'static str> {
    let suffix = iri.strip_prefix("http://sbols.org/v2#")?;
    match suffix {
        "precedes" => Some(v3::SBOL_PRECEDES),
        "strictlyPrecedes" => Some(v3::SBOL_STRICTLY_PRECEDES),
        "meets" => Some(v3::SBOL_MEETS),
        "isDisjointFrom" => Some(v3::SBOL_IS_DISJOINT_FROM),
        "contains" => Some(v3::SBOL_CONTAINS),
        "strictlyContains" => Some(v3::SBOL_STRICTLY_CONTAINS),
        "equals" => Some(v3::SBOL_EQUALS),
        "covers" => Some(v3::SBOL_COVERS),
        "overlaps" => Some(v3::SBOL_OVERLAPS),
        "finishes" => Some(v3::SBOL_FINISHES),
        "starts" => Some(v3::SBOL_STARTS),
        "sameOrientationAs" => Some(v3::SBOL_SAME_ORIENTATION_AS),
        "oppositeOrientationAs" => Some(v3::SBOL_OPPOSITE_ORIENTATION_AS),
        "differentFrom" => Some(v3::SBOL_DIFFERENT_FROM),
        "verifyIdentical" => Some(v3::SBOL_VERIFY_IDENTICAL),
        "replaces" => Some(v3::SBOL_REPLACES),
        _ => None,
    }
}

/// Maps an SBOL 2 VariableComponent operator IRI to its SBOL 3
/// VariableFeature cardinality equivalent.
pub(super) fn map_operator(iri: &str) -> Option<&'static str> {
    match iri {
        v2::SBOL2_ONE => Some(v3::SBOL_ONE),
        v2::SBOL2_ZERO_OR_ONE => Some(v3::SBOL_ZERO_OR_ONE),
        v2::SBOL2_ONE_OR_MORE => Some(v3::SBOL_ONE_OR_MORE),
        v2::SBOL2_ZERO_OR_MORE => Some(v3::SBOL_ZERO_OR_MORE),
        _ => None,
    }
}

/// Maps an SBOL 2 CombinatorialDerivation strategy IRI to its SBOL 3
/// equivalent.
pub(super) fn map_strategy(iri: &str) -> Option<&'static str> {
    match iri {
        v2::SBOL2_ENUMERATE => Some(v3::SBOL_ENUMERATE),
        v2::SBOL2_SAMPLE => Some(v3::SBOL_SAMPLE),
        _ => None,
    }
}

/// Maps an SBOL 2 roleIntegration IRI to its SBOL 3 equivalent.
pub(super) fn map_role_integration(iri: &str) -> Option<&'static str> {
    match iri {
        v2::SBOL2_MERGE_ROLES => Some(v3::SBOL_MERGE_ROLES),
        v2::SBOL2_OVERRIDE_ROLES => Some(v3::SBOL_OVERRIDE_ROLES),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orientation_round_trip() {
        assert_eq!(
            map_orientation(v2::SBOL2_ORIENTATION_INLINE),
            Some(v3::SBOL_INLINE)
        );
        assert_eq!(
            map_orientation(v2::SBOL2_ORIENTATION_REVERSE_COMPLEMENT),
            Some(v3::SBOL_REVERSE_COMPLEMENT),
        );
        assert_eq!(map_orientation("http://example.org/custom"), None);
    }

    #[test]
    fn encoding_round_trip() {
        assert_eq!(
            map_encoding(v2::SBOL2_ENCODING_IUPAC_DNA),
            Some(v3::EDAM_IUPAC_DNA_RNA_ENCODING),
        );
        assert_eq!(
            map_encoding(v2::SBOL2_ENCODING_IUPAC_PROTEIN),
            Some(v3::EDAM_IUPAC_PROTEIN_ENCODING),
        );
        assert_eq!(map_encoding("http://example.org/custom"), None);
    }

    #[test]
    fn biopax_to_sbo() {
        assert_eq!(
            map_biopax_type(v2::BIOPAX_DNA),
            Some("https://identifiers.org/SBO:0000251")
        );
        assert_eq!(
            map_biopax_type(v2::BIOPAX_PROTEIN),
            Some("https://identifiers.org/SBO:0000252")
        );
        assert_eq!(map_biopax_type("http://example.org/custom"), None);
    }

    #[test]
    fn refinement_to_restriction() {
        assert_eq!(
            map_refinement(v2::SBOL2_REFINEMENT_VERIFY_IDENTICAL),
            Some(v3::SBOL_VERIFY_IDENTICAL),
        );
        assert_eq!(
            map_refinement(v2::SBOL2_REFINEMENT_USE_LOCAL),
            Some(v3::SBOL_REPLACES)
        );
        assert_eq!(
            map_refinement(v2::SBOL2_REFINEMENT_USE_REMOTE),
            Some(v3::SBOL_REPLACES)
        );
        // Per SBOL 3.1.0 §10.2, merge collapses to useRemote.
        assert_eq!(
            map_refinement(v2::SBOL2_REFINEMENT_MERGE),
            Some(v3::SBOL_REPLACES)
        );
        assert_eq!(map_refinement("http://example.org/other"), None);
    }

    #[test]
    fn operator_to_cardinality() {
        assert_eq!(map_operator(v2::SBOL2_ONE), Some(v3::SBOL_ONE));
        assert_eq!(
            map_operator(v2::SBOL2_ZERO_OR_ONE),
            Some(v3::SBOL_ZERO_OR_ONE)
        );
        assert_eq!(
            map_operator(v2::SBOL2_ONE_OR_MORE),
            Some(v3::SBOL_ONE_OR_MORE)
        );
        assert_eq!(
            map_operator(v2::SBOL2_ZERO_OR_MORE),
            Some(v3::SBOL_ZERO_OR_MORE)
        );
        assert_eq!(map_operator("http://example.org/custom"), None);
    }

    #[test]
    fn strategy_namespace_shift() {
        assert_eq!(map_strategy(v2::SBOL2_ENUMERATE), Some(v3::SBOL_ENUMERATE));
        assert_eq!(map_strategy(v2::SBOL2_SAMPLE), Some(v3::SBOL_SAMPLE));
        assert_eq!(map_strategy("http://example.org/custom"), None);
    }

    #[test]
    fn role_integration_namespace_shift() {
        assert_eq!(
            map_role_integration(v2::SBOL2_MERGE_ROLES),
            Some(v3::SBOL_MERGE_ROLES)
        );
        assert_eq!(
            map_role_integration(v2::SBOL2_OVERRIDE_ROLES),
            Some(v3::SBOL_OVERRIDE_ROLES)
        );
        assert_eq!(map_role_integration("http://example.org/custom"), None);
    }
}
