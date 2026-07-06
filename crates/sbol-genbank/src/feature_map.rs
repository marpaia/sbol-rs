//! GenBank feature-key → Sequence Ontology IRI mapping table.
//!
//! The table is a curated subset of the community consensus mapping
//! published by `sbol-utilities` as `sbol_utilities/gb2so.csv`. Every
//! entry whose GenBank key also appears in that reference carries the
//! same Sequence Ontology term; the parity test at the bottom of this
//! module pins that agreement against a committed copy of the reference
//! (`tests/fixtures/gb2so.csv`). Keys the crate maps that the reference
//! does not carry are enumerated in that test's allowlist with a
//! rationale.
//!
//! Unrecognized keys are preserved verbatim in the upgrade warning
//! stream so callers can audit anything the table misses, and the
//! importer falls back to `SO:0000110` (sequence_feature).
//!
//! Reference: INSDC Feature Table at
//! <https://www.insdc.org/submitting-standards/feature-table/> and the
//! `sbol_utilities/gb2so.csv` table bundled with `sbol-utilities`.

/// GenBank feature key → Sequence Ontology IRI. Keys use the exact
/// INSDC spelling (including leading-hyphen keys such as `-10_signal`).
/// IRIs are `identifiers.org` SO URLs, the canonical form the rest of
/// the crate emits.
pub(crate) const GENBANK_SO_MAP: &[(&str, &str)] = &[
    // Most common synbio feature keys.
    ("CDS", "https://identifiers.org/SO:0000316"),
    ("gene", "https://identifiers.org/SO:0000704"),
    ("promoter", "https://identifiers.org/SO:0000167"),
    ("terminator", "https://identifiers.org/SO:0000141"),
    ("RBS", "https://identifiers.org/SO:0000139"),
    (
        "ribosome_binding_site",
        "https://identifiers.org/SO:0000139",
    ),
    ("regulatory", "https://identifiers.org/SO:0005836"),
    ("5'UTR", "https://identifiers.org/SO:0000204"),
    ("3'UTR", "https://identifiers.org/SO:0000205"),
    ("mRNA", "https://identifiers.org/SO:0000234"),
    ("tRNA", "https://identifiers.org/SO:0000253"),
    ("rRNA", "https://identifiers.org/SO:0000252"),
    ("ncRNA", "https://identifiers.org/SO:0000655"),
    ("exon", "https://identifiers.org/SO:0000147"),
    ("intron", "https://identifiers.org/SO:0000188"),
    ("operon", "https://identifiers.org/SO:0000178"),
    ("polyA_signal", "https://identifiers.org/SO:0000551"),
    ("polyA_site", "https://identifiers.org/SO:0000553"),
    ("primer_bind", "https://identifiers.org/SO:0005850"),
    ("protein_bind", "https://identifiers.org/SO:0000410"),
    ("misc_binding", "https://identifiers.org/SO:0000409"),
    ("misc_recomb", "https://identifiers.org/SO:0000298"),
    ("misc_signal", "https://identifiers.org/SO:0001411"),
    ("misc_structure", "https://identifiers.org/SO:0000002"),
    ("misc_difference", "https://identifiers.org/SO:0000413"),
    ("misc_feature", "https://identifiers.org/SO:0000001"),
    ("stem_loop", "https://identifiers.org/SO:0000313"),
    ("repeat_region", "https://identifiers.org/SO:0000657"),
    ("rep_origin", "https://identifiers.org/SO:0000296"),
    ("enhancer", "https://identifiers.org/SO:0000165"),
    ("attenuator", "https://identifiers.org/SO:0000140"),
    ("TATA_signal", "https://identifiers.org/SO:0000174"),
    ("-10_signal", "https://identifiers.org/SO:0000175"),
    ("-35_signal", "https://identifiers.org/SO:0000176"),
    ("GC_signal", "https://identifiers.org/SO:0000173"),
    ("CAAT_signal", "https://identifiers.org/SO:0000172"),
    (
        "polyA_secondary_structure",
        "https://identifiers.org/SO:0000553",
    ),
    ("iDNA", "https://identifiers.org/SO:0000723"),
    ("old_sequence", "https://identifiers.org/SO:0000413"),
    ("modified_base", "https://identifiers.org/SO:0000305"),
    ("mat_peptide", "https://identifiers.org/SO:0000419"),
    ("sig_peptide", "https://identifiers.org/SO:0000418"),
    ("transit_peptide", "https://identifiers.org/SO:0000725"),
    ("propeptide", "https://identifiers.org/SO:0001062"),
    ("variation", "https://identifiers.org/SO:0001060"),
    ("S_region", "https://identifiers.org/SO:0001836"),
    ("V_region", "https://identifiers.org/SO:0001833"),
    ("J_segment", "https://identifiers.org/SO:0000470"),
    ("C_region", "https://identifiers.org/SO:0001834"),
    ("D_segment", "https://identifiers.org/SO:0000458"),
    ("centromere", "https://identifiers.org/SO:0000577"),
    ("telomere", "https://identifiers.org/SO:0000624"),
    ("STS", "https://identifiers.org/SO:0000331"),
];

/// Returns the canonical Sequence Ontology IRI for a GenBank feature
/// key, or `None` if the key isn't in the curated table. Returning
/// `None` is informational. The importer falls back to
/// [`GENERIC_FEATURE`] (`SO:0000110`, sequence_feature) and records the
/// original key for the user.
pub(crate) fn feature_key_to_so(kind: &str) -> Option<&'static str> {
    GENBANK_SO_MAP
        .iter()
        .find(|(key, _)| *key == kind)
        .map(|(_, iri)| *iri)
}

/// `SO:0000110`, the umbrella "sequence_feature" term used as a
/// fallback when a GenBank feature key isn't in our curated mapping.
pub(crate) const GENERIC_FEATURE: &str = "https://identifiers.org/SO:0000110";

#[cfg(test)]
mod parity_tests {
    use super::GENBANK_SO_MAP;
    use std::collections::HashMap;

    /// GenBank feature keys the crate maps that the `sbol-utilities`
    /// `gb2so.csv` reference does not carry as a matching row. Each is
    /// paired with the reason it is a legitimate crate-only extension
    /// rather than a disagreement with the reference.
    const REFERENCE_GAPS: &[(&str, &str)] = &[
        (
            "ribosome_binding_site",
            "INSDC long-form alias for RBS; both spell the same feature and \
             map to the same SO term (SO:0000139). The reference only lists RBS.",
        ),
        (
            "misc_recomb",
            "The reference row `misc_recom` is a misspelling with an empty SO \
             cell; the crate maps the correct INSDC key misc_recomb to \
             SO:0000298 (recombination_feature).",
        ),
        (
            "old_sequence",
            "Valid INSDC key absent from the reference; mapped to SO:0000413 \
             (sequence_difference), matching the sibling misc_difference key.",
        ),
        (
            "polyA_secondary_structure",
            "Crate-only key absent from the reference; retained for callers \
             that emit it, sharing the polyA_site SO term (SO:0000553).",
        ),
    ];

    fn load_reference() -> HashMap<String, String> {
        let csv = include_str!("../tests/fixtures/gb2so.csv");
        let mut map = HashMap::new();
        for (lineno, line) in csv.lines().enumerate() {
            let line = line.trim();
            let _ = lineno;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Skip the CSV header row.
            if line.starts_with("GenBank_Ontology") {
                continue;
            }
            let (key, so) = line
                .split_once(',')
                .unwrap_or_else(|| panic!("malformed gb2so row: {line:?}"));
            let so = so.trim();
            // Reference rows with an empty SO cell carry no mapping.
            if so.is_empty() {
                continue;
            }
            map.insert(key.trim().to_owned(), so.to_owned());
        }
        map
    }

    /// Converts an `identifiers.org` SO IRI back to the `SO:NNNNNNN`
    /// CURIE the reference CSV uses.
    fn iri_to_curie(iri: &str) -> String {
        iri.strip_prefix("https://identifiers.org/")
            .unwrap_or(iri)
            .to_owned()
    }

    #[test]
    fn every_mapping_entry_agrees_with_the_gb2so_reference() {
        let reference = load_reference();
        let allowlist: HashMap<&str, &str> = REFERENCE_GAPS.iter().copied().collect();

        let mut disagreements = Vec::new();
        let mut unexpected_gaps = Vec::new();

        for (key, iri) in GENBANK_SO_MAP {
            let curie = iri_to_curie(iri);
            match reference.get(*key) {
                Some(reference_curie) => {
                    if reference_curie != &curie {
                        disagreements.push(format!(
                            "{key}: crate maps {curie}, gb2so reference says {reference_curie}"
                        ));
                    }
                }
                None => {
                    if !allowlist.contains_key(*key) {
                        unexpected_gaps.push(format!(
                            "{key}: not present in the gb2so reference and not in the \
                             reviewed allowlist"
                        ));
                    }
                }
            }
        }

        assert!(
            disagreements.is_empty(),
            "SO mapping disagrees with the sbol-utilities gb2so reference:\n  {}",
            disagreements.join("\n  ")
        );
        assert!(
            unexpected_gaps.is_empty(),
            "crate maps keys absent from the gb2so reference without an allowlist \
             entry (add a reviewed rationale to REFERENCE_GAPS or correct the key):\n  {}",
            unexpected_gaps.join("\n  ")
        );
    }

    #[test]
    fn allowlist_entries_are_actually_absent_from_the_reference() {
        // Guards against a stale allowlist: if the reference later adds
        // one of these keys, the rationale is no longer needed and the
        // entry should move into the checked set.
        let reference = load_reference();
        let mapped: HashMap<&str, &str> = GENBANK_SO_MAP.iter().copied().collect();
        for (key, _reason) in REFERENCE_GAPS {
            assert!(
                mapped.contains_key(*key),
                "allowlist names `{key}`, which the crate no longer maps"
            );
            assert!(
                !reference.contains_key(*key),
                "allowlist names `{key}`, but the gb2so reference now carries it; \
                 remove the allowlist entry so the mapping is checked directly"
            );
        }
    }
}
