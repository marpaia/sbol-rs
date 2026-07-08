//! Reverse-direction enumerated-value rewrites for SBOL 3 → SBOL 2.
//!
//! Every function here is the inverse of the corresponding rewrite in
//! [`crate::upgrade::values`]. Unknown inputs are returned unchanged.

use crate::sbol2_vocab as v2;
use sbol3::vocab as v3;

/// Maps an SBOL 3 orientation IRI back to its SBOL 2 equivalent.
pub(super) fn map_orientation(iri: &str) -> Option<&'static str> {
    match iri {
        v3::SBOL_INLINE | v3::SO_INLINE => Some(v2::SBOL2_ORIENTATION_INLINE),
        v3::SBOL_REVERSE_COMPLEMENT | v3::SO_REVERSE_COMPLEMENT => {
            Some(v2::SBOL2_ORIENTATION_REVERSE_COMPLEMENT)
        }
        _ => None,
    }
}

/// Maps an SBOL 3 EDAM sequence encoding IRI back to the legacy IUPAC
/// page URL that SBOL 2 documents carry.
pub(super) fn map_encoding(iri: &str) -> Option<&'static str> {
    match iri {
        v3::EDAM_IUPAC_DNA_RNA_ENCODING => Some(v2::SBOL2_ENCODING_IUPAC_DNA),
        v3::EDAM_IUPAC_PROTEIN_ENCODING => Some(v2::SBOL2_ENCODING_IUPAC_PROTEIN),
        "https://identifiers.org/edam:format_1196" => Some(v2::SBOL2_ENCODING_SMILES),
        "https://identifiers.org/edam:format_1197" => Some(v2::SBOL2_ENCODING_INCHI),
        _ => None,
    }
}

/// Maps an SBOL 3 Component type IRI (SBO term) back to the BioPAX
/// type IRI used by SBOL 2 ComponentDefinitions. The upgrade folded
/// both `BIOPAX_DNA` and `BIOPAX_DNA_REGION` to the same SBO term, so
/// the downgrade picks `*Region`, the more common modern choice in
/// real-world data.
pub(super) fn map_biopax_type(iri: &str) -> Option<&'static str> {
    match iri {
        "https://identifiers.org/SBO:0000251" => Some(v2::BIOPAX_DNA_REGION),
        "https://identifiers.org/SBO:0000250" => Some(v2::BIOPAX_RNA_REGION),
        "https://identifiers.org/SBO:0000252" => Some(v2::BIOPAX_PROTEIN),
        "https://identifiers.org/SBO:0000247" => Some(v2::BIOPAX_SMALL_MOLECULE),
        "https://identifiers.org/SBO:0000253" => Some(v2::BIOPAX_COMPLEX),
        _ => None,
    }
}

/// Maps an SBOL 3 Constraint restriction IRI back to the SBOL 2
/// equivalent. The SBOL 3 IRIs live in `http://sbols.org/v3#`; the
/// SBOL 2 ones live in `http://sbols.org/v2#` under the same local
/// names.
pub(super) fn map_restriction(iri: &str) -> Option<String> {
    let local = iri.strip_prefix("http://sbols.org/v3#")?;
    Some(format!("http://sbols.org/v2#{local}"))
}

/// Maps an SBOL 3 Constraint restriction IRI back to the SBOL 2
/// MapsTo refinement value, given the position of the
/// ComponentReference in the Constraint's `subject` / `object` pair.
///
/// Per SBOL 3.1.0 §10.2 the CRef's position carries the useLocal/
/// useRemote distinction:
///
/// - restriction `replaces`        + CRef in subject → `useRemote`
/// - restriction `replaces`        + CRef in object  → `useLocal`
/// - restriction `verifyIdentical` + CRef in either  → `verifyIdentical`
pub(super) fn map_restriction_to_refinement(
    restriction: &str,
    cref_role: CRefPosition,
) -> Option<&'static str> {
    match (restriction, cref_role) {
        (v3::SBOL_VERIFY_IDENTICAL, _) => Some(v2::SBOL2_REFINEMENT_VERIFY_IDENTICAL),
        (v3::SBOL_REPLACES, CRefPosition::Subject) => Some(v2::SBOL2_REFINEMENT_USE_REMOTE),
        (v3::SBOL_REPLACES, CRefPosition::Object) => Some(v2::SBOL2_REFINEMENT_USE_LOCAL),
        _ => None,
    }
}

/// Which position the ComponentReference occupies in a MapsTo-shaped
/// SBOL 3 Constraint. The CRef always represents the `remote` side of
/// the reconstructed MapsTo; the *other* position holds the `local`
/// SubComponent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CRefPosition {
    /// CRef appears as `sbol3:subject` of the Constraint.
    Subject,
    /// CRef appears as `sbol3:object` of the Constraint.
    Object,
}

/// Maps an SBOL 3 VariableFeature cardinality IRI back to the SBOL 2
/// VariableComponent operator equivalent.
pub(super) fn map_cardinality(iri: &str) -> Option<&'static str> {
    match iri {
        v3::SBOL_ONE => Some(v2::SBOL2_ONE),
        v3::SBOL_ZERO_OR_ONE => Some(v2::SBOL2_ZERO_OR_ONE),
        v3::SBOL_ONE_OR_MORE => Some(v2::SBOL2_ONE_OR_MORE),
        v3::SBOL_ZERO_OR_MORE => Some(v2::SBOL2_ZERO_OR_MORE),
        _ => None,
    }
}

/// Maps an SBOL 3 CombinatorialDerivation strategy IRI back to its SBOL 2
/// equivalent.
pub(super) fn map_strategy(iri: &str) -> Option<&'static str> {
    match iri {
        v3::SBOL_ENUMERATE => Some(v2::SBOL2_ENUMERATE),
        v3::SBOL_SAMPLE => Some(v2::SBOL2_SAMPLE),
        _ => None,
    }
}

/// Maps an SBOL 3 roleIntegration IRI back to its SBOL 2 equivalent.
pub(super) fn map_role_integration(iri: &str) -> Option<&'static str> {
    match iri {
        v3::SBOL_MERGE_ROLES => Some(v2::SBOL2_MERGE_ROLES),
        v3::SBOL_OVERRIDE_ROLES => Some(v2::SBOL2_OVERRIDE_ROLES),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orientation_round_trips() {
        assert_eq!(
            map_orientation(v3::SBOL_INLINE),
            Some(v2::SBOL2_ORIENTATION_INLINE)
        );
        assert_eq!(
            map_orientation(v3::SBOL_REVERSE_COMPLEMENT),
            Some(v2::SBOL2_ORIENTATION_REVERSE_COMPLEMENT)
        );
        assert_eq!(map_orientation("http://example.org/other"), None);
    }

    #[test]
    fn encoding_round_trips() {
        assert_eq!(
            map_encoding(v3::EDAM_IUPAC_DNA_RNA_ENCODING),
            Some(v2::SBOL2_ENCODING_IUPAC_DNA)
        );
        assert_eq!(
            map_encoding(v3::EDAM_IUPAC_PROTEIN_ENCODING),
            Some(v2::SBOL2_ENCODING_IUPAC_PROTEIN)
        );
    }

    #[test]
    fn sbo_to_biopax_prefers_region_variant() {
        // The upgrade collapses both BIOPAX_DNA and BIOPAX_DNA_REGION
        // to SBO_DNA, so the downgrade has to pick one. We pick the
        // `*Region` variant because it's the modern norm.
        assert_eq!(
            map_biopax_type("https://identifiers.org/SBO:0000251"),
            Some(v2::BIOPAX_DNA_REGION)
        );
    }

    #[test]
    fn restriction_namespace_shift() {
        assert_eq!(
            map_restriction("http://sbols.org/v3#precedes").as_deref(),
            Some("http://sbols.org/v2#precedes")
        );
        assert_eq!(
            map_restriction("http://sbols.org/v3#verifyIdentical").as_deref(),
            Some("http://sbols.org/v2#verifyIdentical")
        );
        assert_eq!(map_restriction("http://example.org/other"), None);
    }

    #[test]
    fn refinement_inference_is_position_aware() {
        // replaces + subject=CR → useRemote
        assert_eq!(
            map_restriction_to_refinement(v3::SBOL_REPLACES, CRefPosition::Subject),
            Some(v2::SBOL2_REFINEMENT_USE_REMOTE)
        );
        // replaces + object=CR → useLocal
        assert_eq!(
            map_restriction_to_refinement(v3::SBOL_REPLACES, CRefPosition::Object),
            Some(v2::SBOL2_REFINEMENT_USE_LOCAL)
        );
        // verifyIdentical is position-insensitive
        assert_eq!(
            map_restriction_to_refinement(v3::SBOL_VERIFY_IDENTICAL, CRefPosition::Subject),
            Some(v2::SBOL2_REFINEMENT_VERIFY_IDENTICAL)
        );
        assert_eq!(
            map_restriction_to_refinement(v3::SBOL_VERIFY_IDENTICAL, CRefPosition::Object),
            Some(v2::SBOL2_REFINEMENT_VERIFY_IDENTICAL)
        );
        assert_eq!(
            map_restriction_to_refinement("http://example.org/other", CRefPosition::Subject),
            None
        );
    }

    #[test]
    fn cardinality_to_operator() {
        assert_eq!(map_cardinality(v3::SBOL_ONE), Some(v2::SBOL2_ONE));
        assert_eq!(
            map_cardinality(v3::SBOL_ZERO_OR_ONE),
            Some(v2::SBOL2_ZERO_OR_ONE)
        );
        assert_eq!(
            map_cardinality(v3::SBOL_ONE_OR_MORE),
            Some(v2::SBOL2_ONE_OR_MORE)
        );
        assert_eq!(
            map_cardinality(v3::SBOL_ZERO_OR_MORE),
            Some(v2::SBOL2_ZERO_OR_MORE)
        );
        assert_eq!(map_cardinality("http://example.org/other"), None);
    }

    #[test]
    fn strategy_namespace_shift() {
        assert_eq!(map_strategy(v3::SBOL_ENUMERATE), Some(v2::SBOL2_ENUMERATE));
        assert_eq!(map_strategy(v3::SBOL_SAMPLE), Some(v2::SBOL2_SAMPLE));
        assert_eq!(map_strategy("http://example.org/other"), None);
    }

    #[test]
    fn role_integration_namespace_shift() {
        assert_eq!(
            map_role_integration(v3::SBOL_MERGE_ROLES),
            Some(v2::SBOL2_MERGE_ROLES)
        );
        assert_eq!(
            map_role_integration(v3::SBOL_OVERRIDE_ROLES),
            Some(v2::SBOL2_OVERRIDE_ROLES)
        );
        assert_eq!(map_role_integration("http://example.org/other"), None);
    }
}
