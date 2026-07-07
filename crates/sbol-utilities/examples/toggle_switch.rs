//! The canonical genetic toggle switch: two transcriptional units, each an
//! engineered region whose parts are chained head-to-tail with `meets`
//! constraints. An idiomatic translation of the `sbol_utilities` toggle-switch
//! script — no IRIs, no `SbolObject`, no manual document assembly.
//!
//! Run with `cargo run -p sbol-utilities --example toggle_switch`.

use sbol_utilities::prelude::*;
use sbol3::design::Design;
use sbol3::prelude::RdfFormat;

const NS: &str = "https://github.com/DRAGGON-Lab";

const PLAC: &str = "caatacgcaaaccgcctctccccgcgcgttggccgattcattaatgcagctggcacgacaggtttcccgactggaaagcgggcagtgagcgcaacgcaattaatgtgagttagctcactcattaggcaccccaggctttacactttatgcttccggctcgtatgttgtgtggaattgtgagcggataacaatttcacaca";
const RBS_B0034: &str = "ttgaacaccgtcTCAGGTAAGTATCAGTTGTAAatcacacaggacta";
const TETR: &str = "GTCCatggtgaatgtgaaaccagtaacgttatacgatgtcgcagagtatgccggtgtctcttatcagaccgtttcccgcgtggtgaaccaggccagccacgtttctgcgaaaacgcgggaaaaagtggaagcggcgatggcggagctgaattacattcccaaccgcgtggcacaacaactggcgggcaaacagtcgttgctgattggcgttgccacctccagtctggccctgcacgcgccgtcgcaaattgtcgcggcgattaaatctcgcgccgatcaactgggtgccagcgtggtggtgtcgatggtagaacgaagcggcgtcgaagcctgtaaagcggcggtgcacaatcttctcgcgcaacgcgtcagtgggctgatcattaactatccgctggatgaccaggatgccattgctgtggaagctgcctgcactaatgttccggcgttatttcttgatgtctctgaccagacacccatcaacagtattattttctcccatgaagacggtacgcgactgggcgtggagcatctggtcgcattgggtcaccagcaaatcgcgctgttagcgggcccattaagttctgtctcggcgcgtctgcgtctggctggctggcataaatatctcactcgcaatcaaattcagccgatagcggaacgggaaggcgactggagtgccatgtccggttttcaacaaaccatgcaaatgctgaatgagggcatcgttcccactgcgatgctggttgccaacgatcagatggcgctgggcgcaatgcgcgccattaccgagtccgggctgcgcgttggtgcggatatctcggtagtgggatacgacgataccgaagacagctcatgttatatcccgccgttaaccaccatcaaacaggattttcgcctgctggggcaaaccagcgtggaccgcttgctgcaactctctcagggccaggcggtgaagggcaatcagctgttgcccgtctcactggtgaaaagaaaaaccaccctggcgcccaatacgcaaaccgcctctccccgcgcgttggccgattcattaatgcagctggcacgacaggtttcccgactggaaagcgggcagGGCTCG";
const TER_B0015: &str = "GTCCatttgtcctactcaggagagcgttcaccgacaaacaacagataaaacgaaaggcccagtctttcgactgagcctttcgttttatttgTAAGGCTCG";

const PTET: &str = "tccctatcagtgatagagattgacatccctatcagtgatagagatactgagcac";
const RBS_B0064: &str = "AAAGAGGGGAAA";
const LACI: &str = "atggtgaatgtgaaaccagtaacgttatacgatgtcgcagagtatgccggtgtctcttatcagaccgtttcccgcgtggtgaaccaggccagccacgtttctgcgaaaacgcgggaaaaagtggaagcggcgatggcggagctgaattacattcccaaccgcgtggcacaacaactggcgggcaaacagtcgttgctgattggcgttgccacctccagtctggccctgcacgcgccgtcgcaaattgtcgcggcgattaaatctcgcgccgatcaactgggtgccagcgtggtggtgtcgatggtagaacgaagcggcgtcgaagcctgtaaagcggcggtgcacaatcttctcgcgcaacgcgtcagtgggctgatcattaactatccgctggatgaccaggatgccattgctgtggaagctgcctgcactaatgttccggcgttatttcttgatgtctctgaccagacacccatcaacagtattattttctcccatgaagacggtacgcgactgggcgtggagcatctggtcgcattgggtcaccagcaaatcgcgctgttagcgggcccattaagttctgtctcggcgcgtctgcgtctggctggctggcataaatatctcactcgcaatcaaattcagccgatagcggaacgggaaggcgactggagtgccatgtccggttttcaacaaaccatgcaaatgctgaatgagggcatcgttcccactgcgatgctggttgccaacgatcagatggcgctgggcgcaatgcgcgccattaccgagtccgggctgcgcgttggtgcggatatctcggtagtgggatacgacgataccgaagacagctcatgttatatcccgccgttaaccaccatcaaacaggattttcgcctgctggggcaaaccagcgtggaccgcttgctgcaactctctcagggccaggcggtgaagggcaatcagctgttgcccgtctcactggtgaaaagaaaaaccaccctggcgcccaatacgcaaaccgcctctccccgcgcgttggccgattcattaatgcagctggcacgacaggtttcccgactggaaagcgggcaggctgcaaacgacgaaaactacgctttagtagcttaataactctgatagtgctagtgtagatctc";
const RBS_BD14: &str =
    "gggcccaagttcacttaaaaaggagatcaacaatgaaagcaattttcgtactgaaacatcttaatcatgcggtggagggtttcta";
const GFP: &str = "atggcatccaagggcgaggagctctttactggcgtagtaccaattctcgtagagctcgatggcgatgtaaatggccataagttttccgtacgcggcgagggcgagggcgatgcaactaacggcaagctcactctcaagtttatttgtactactggcaagctcccagtaccatggccaactctcgtaactactctgacctatggcgtacaatgtttttcccgctatccagatcacatgaagcaacatgatttttttaagtccgcaatgccagagggctatgtacaagagcgcactattagctttaaggatgatggcacctataagactcgcgcagaggtaaagtttgagggcgatactctcgtaaatcgcattgagctcaagggcattgattttaaggaggatggcaatattctcggccataagctggagtataatttcaattcccataatgtatatattaccgcagataagcaaaagaatggcattaaggcgaattttaagattcgccataatgtggaggatggctccgtacaactcgcagatcattatcaacaaaatactccaattggcgatggcccagtactcctcccagataatcattatctctccactcaatccgtgctctccaaagatccaaatgagaagcgcgatcacatggtactcctggagtttgtaactgcagcaggcattactcatggcatggatgagctctataagctcgagcaccaccaccaccaccactga";
const TER_L3S2P21: &str = "CTCGGTACCAAATTCCAGAAAAGAGGCCTCCCGAAAGGGGGGCCTTTTTTCGTTTTGGTCC";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut d = Design::new(NS)?;

    // --- pLac transcriptional unit: LacI-repressible, produces TetR ---
    let plac = d
        .promoter("pLac", PLAC)
        .description("Promoter for LacI repressible expression")
        .add();
    let b0034 = d
        .rbs("B0034", RBS_B0034)
        .description("RBS (Elowitz 1999) -- defines RBS efficiency")
        .add();
    let tetr = d
        .cds("tetR", TETR)
        .description("Coding region for the TetR protein with degradation tag")
        .add();
    let b0015 = d
        .terminator("B0015", TER_B0015)
        .description("Double terminator consisting of BBa_B0010 and BBa_B0012")
        .add();
    d.engineered_region("pLac_tu", [plac, b0034, tetr, b0015])
        .description("Transcriptional unit: pLac, B0034, tetR, B0015 (produces TetR).")
        .add();

    // --- pTet transcriptional unit: TetR-repressible, produces GFP ---
    let ptet = d
        .promoter("pTet", PTET)
        .description("Sequence for pTet inverting regulator driven by the TetR protein")
        .add();
    let b0064 = d
        .rbs("B0064", RBS_B0064)
        .description("This is a single bp change from B0034")
        .add();
    let laci = d
        .cds("lacI", LACI)
        .description("Coding region for the LacI protein with an LVA degradation tail")
        .add();
    let bd14 = d
        .rbs("BD14", RBS_BD14)
        .description("A bicistronic RBS with a leader sequence including the second RBS")
        .add();
    let gfp = d.cds("gfp", GFP).description("superfolder GFP gene").add();
    let l3s2p21 = d
        .terminator("L3S2P21", TER_L3S2P21)
        .description("Strong synthetic transcriptional terminator")
        .add();
    d.engineered_region("pTet_tu", [ptet, b0064, laci, bd14, gfp, l3s2p21])
        .description("Transcriptional unit: pTet, B0064, lacI, BD14, gfp, L3S2P21 (produces GFP).")
        .add();

    let doc = d.finish()?;

    // `typed_objects()` preserves the order objects were added; `objects()` is a
    // BTreeMap sorted by IRI. Neither affects the file — N-Triples is an
    // unordered set, and part order is carried by the `meets` constraints.
    for object in doc.typed_objects() {
        println!("{}", object.identity());
    }

    doc.write_path("toggle_switch.nt", RdfFormat::NTriples)?;
    Ok(())
}
