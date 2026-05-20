//! GenBank feature-key → Sequence Ontology IRI mapping table.
//!
//! Mirrors the consensus mapping used by `sbol-utilities` / libSBOLj /
//! SynBioHub. The list is intentionally a curated subset of the most
//! common keys; unrecognized keys are preserved verbatim in the upgrade
//! warning stream so callers can audit anything we missed.
//!
//! Reference: INSDC Feature Table at
//! <https://www.insdc.org/submitting-standards/feature-table/> and the
//! `sbol_utilities/gb2so.csv` table bundled with `sbol-utilities`.

/// Returns the canonical Sequence Ontology IRI for a GenBank feature
/// key, or `None` if the key isn't in the curated table. Returning
/// `None` is informational — the importer falls back to
/// `SO:0000110` (sequence_feature) and records the original key for
/// the user.
pub(crate) fn feature_key_to_so(kind: &str) -> Option<&'static str> {
    Some(match kind {
        // Most common synbio feature keys.
        "CDS" => "https://identifiers.org/SO:0000316",
        "gene" => "https://identifiers.org/SO:0000704",
        "promoter" => "https://identifiers.org/SO:0000167",
        "terminator" => "https://identifiers.org/SO:0000141",
        "RBS" | "ribosome_binding_site" => "https://identifiers.org/SO:0000139",
        "regulatory" => "https://identifiers.org/SO:0000167",
        "5'UTR" => "https://identifiers.org/SO:0000204",
        "3'UTR" => "https://identifiers.org/SO:0000205",
        "mRNA" => "https://identifiers.org/SO:0000234",
        "tRNA" => "https://identifiers.org/SO:0000253",
        "rRNA" => "https://identifiers.org/SO:0000252",
        "ncRNA" => "https://identifiers.org/SO:0000655",
        "exon" => "https://identifiers.org/SO:0000147",
        "intron" => "https://identifiers.org/SO:0000188",
        "operon" => "https://identifiers.org/SO:0000178",
        "polyA_signal" => "https://identifiers.org/SO:0000551",
        "polyA_site" => "https://identifiers.org/SO:0000553",
        "primer_bind" => "https://identifiers.org/SO:0005850",
        "protein_bind" => "https://identifiers.org/SO:0000410",
        "misc_binding" => "https://identifiers.org/SO:0001654",
        "misc_recomb" => "https://identifiers.org/SO:0000298",
        "misc_signal" => "https://identifiers.org/SO:0001679",
        "misc_structure" => "https://identifiers.org/SO:0001411",
        "misc_difference" => "https://identifiers.org/SO:0000413",
        "misc_feature" => "https://identifiers.org/SO:0000001",
        "stem_loop" => "https://identifiers.org/SO:0000313",
        "repeat_region" => "https://identifiers.org/SO:0000657",
        "rep_origin" => "https://identifiers.org/SO:0000296",
        "enhancer" => "https://identifiers.org/SO:0000165",
        "attenuator" => "https://identifiers.org/SO:0000140",
        "TATA_signal" => "https://identifiers.org/SO:0000174",
        "minus_10_signal" => "https://identifiers.org/SO:0000175",
        "minus_35_signal" => "https://identifiers.org/SO:0000176",
        "GC_signal" => "https://identifiers.org/SO:0000173",
        "CAAT_signal" => "https://identifiers.org/SO:0000172",
        "polyA_secondary_structure" => "https://identifiers.org/SO:0000553",
        "iDNA" => "https://identifiers.org/SO:0000723",
        "old_sequence" => "https://identifiers.org/SO:0000413",
        "modified_base" => "https://identifiers.org/SO:0000305",
        "mat_peptide" => "https://identifiers.org/SO:0000419",
        "sig_peptide" => "https://identifiers.org/SO:0000418",
        "transit_peptide" => "https://identifiers.org/SO:0000725",
        "propeptide" => "https://identifiers.org/SO:0001062",
        "variation" => "https://identifiers.org/SO:0000109",
        "S_region" => "https://identifiers.org/SO:0001354",
        "V_region" => "https://identifiers.org/SO:0000466",
        "J_segment" => "https://identifiers.org/SO:0000470",
        "C_region" => "https://identifiers.org/SO:0001834",
        "D_segment" => "https://identifiers.org/SO:0000458",
        "centromere" => "https://identifiers.org/SO:0000577",
        "telomere" => "https://identifiers.org/SO:0000624",
        "STS" => "https://identifiers.org/SO:0000331",
        // Fallback used by the importer for unrecognized keys; we still
        // keep it here so the IRI is centralized in one place.
        // (Not used by `feature_key_to_so` — the importer falls back
        // to `GENERIC_FEATURE` directly when this returns None.)
        _ => return None,
    })
}

/// `SO:0000110` — the umbrella "sequence_feature" term used as a
/// fallback when a GenBank feature key isn't in our curated mapping.
pub(crate) const GENERIC_FEATURE: &str = "https://identifiers.org/SO:0000110";
