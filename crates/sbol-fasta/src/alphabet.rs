//! Detect whether a FASTA sequence is DNA, RNA, or protein.
//!
//! FASTA carries no biological-type metadata, so the importer has to
//! infer it from the sequence alphabet. The rules below match the
//! conventions every bioinformatics tool uses:
//!
//! - **RNA** if any character is `U` or `u`.
//! - **Protein** if any character is one of the protein-only letters
//!   (`E F I L P Q Z` and their lowercase variants).
//! - **DNA** otherwise (the default — this covers the ambiguous case
//!   of sequences that happen to use only the letters `A C G T`,
//!   which are also valid protein letters but overwhelmingly mean
//!   DNA in practice).
//!
//! Ambiguity codes (`N`, `K`, `S`, `Y`, etc.), gap symbols (`-`,
//! `.`), and stop codons (`*`) are all accepted; the detection logic
//! treats them as compatible with either nucleic acid or protein.

/// Biological type of a FASTA sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alphabet {
    Dna,
    Rna,
    Protein,
}

impl Alphabet {
    /// Detects the alphabet from the sequence text. See the module
    /// docstring for the precise rules. Empty input defaults to DNA.
    pub fn detect(sequence: &str) -> Self {
        let mut saw_u = false;
        let mut saw_protein_only = false;
        for ch in sequence.chars() {
            match ch.to_ascii_uppercase() {
                'U' => saw_u = true,
                // Letters that appear in protein FASTA but never in
                // the standard nucleic-acid alphabets. Used as a
                // strong signal that the record is a protein.
                'E' | 'F' | 'I' | 'L' | 'P' | 'Q' | 'Z' => saw_protein_only = true,
                _ => {}
            }
        }
        if saw_protein_only {
            Alphabet::Protein
        } else if saw_u {
            Alphabet::Rna
        } else {
            Alphabet::Dna
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_defaults_to_dna() {
        assert_eq!(Alphabet::detect(""), Alphabet::Dna);
    }

    #[test]
    fn pure_acgt_is_dna() {
        assert_eq!(Alphabet::detect("ACGTACGT"), Alphabet::Dna);
    }

    #[test]
    fn lowercase_acgt_is_dna() {
        assert_eq!(Alphabet::detect("acgtacgt"), Alphabet::Dna);
    }

    #[test]
    fn dna_with_ambiguity_codes_is_dna() {
        assert_eq!(Alphabet::detect("ACGTNRYKMSWBDH-"), Alphabet::Dna);
    }

    #[test]
    fn any_u_means_rna() {
        assert_eq!(Alphabet::detect("ACGUACGU"), Alphabet::Rna);
        assert_eq!(Alphabet::detect("acgu"), Alphabet::Rna);
    }

    #[test]
    fn protein_letters_override_u() {
        // A sequence with both U and protein-only letters is protein
        // (selenocysteine maps to U in protein, but if there are F/E/L
        // around it's overwhelmingly more likely a protein).
        assert_eq!(Alphabet::detect("MULTILINE"), Alphabet::Protein);
    }

    #[test]
    fn protein_only_letters_detected() {
        assert_eq!(Alphabet::detect("MVSKGEEL"), Alphabet::Protein);
        assert_eq!(Alphabet::detect("FILPQ"), Alphabet::Protein);
    }

    #[test]
    fn stop_codon_in_protein_accepted() {
        assert_eq!(Alphabet::detect("MVSKEEL*"), Alphabet::Protein);
    }
}
