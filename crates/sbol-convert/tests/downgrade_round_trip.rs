//! Integration tests for SBOL 2 → SBOL 3 → SBOL 2 round-trip
//! preservation, plus IRI-collision and version-synthesis behavior.

mod common;

use common::downgrade::*;

use sbol3::RdfFormat;

/// The killer test: take an SBOL 2 fixture, upgrade to SBOL 3,
/// downgrade back to SBOL 2, and diff against the original. Anything
/// that doesn't survive the round trip is either a bug in one
/// direction or an intentional documented divergence.
fn round_trip_diff(fixture: &str) -> (Vec<String>, Vec<String>) {
    let input = std::fs::read_to_string(workspace_fixture(fixture)).unwrap();
    let original_graph = sbol3::RdfGraph::parse(&input, RdfFormat::Turtle).expect("parse original");
    let original: Vec<String> = canonicalize(&original_graph);

    let original: Vec<String> = original.iter().map(|l| normalize_biopax(l)).collect();

    let (upgraded, _ureport) =
        sbol_convert::upgrade_from_sbol2(&input, RdfFormat::Turtle).expect("upgrade");
    let (downgraded_graph, _dreport) = sbol_convert::downgrade(&upgraded).expect("downgrade");
    let downgraded: Vec<String> = strip_backport(&canonicalize(&downgraded_graph))
        .iter()
        .map(|l| normalize_biopax(l))
        .collect();

    let only_in_original: Vec<String> = original
        .iter()
        .filter(|t| !downgraded.contains(t))
        .cloned()
        .collect();
    let only_in_downgraded: Vec<String> = downgraded
        .iter()
        .filter(|t| !original.contains(t))
        .cloned()
        .collect();
    (only_in_original, only_in_downgraded)
}

fn canonicalize(graph: &sbol3::RdfGraph) -> Vec<String> {
    let mut lines: Vec<String> = graph
        .normalized_triples()
        .iter()
        .map(sbol_convert::canonical_nt_line)
        .collect();
    lines.sort();
    lines.dedup();
    lines
}

/// Collapses the BioPAX `Dna`/`Rna` type variants onto their `*Region`
/// forms. The SBOL 2 → SBOL 3 type map sends both `biopax:Dna` and
/// `biopax:DnaRegion` to the same SBO term, and the SBOL 3 → SBOL 2 direction
/// canonicalizes to the `*Region` form, so a round-trip rewrites `Dna` →
/// `DnaRegion`. Normalizing both sides lets the diff ignore that.
fn normalize_biopax(line: &str) -> String {
    let line = line
        .replace(
            "<http://www.biopax.org/release/biopax-level3.owl#Dna>",
            "<http://www.biopax.org/release/biopax-level3.owl#DnaRegion>",
        )
        .replace(
            "<http://www.biopax.org/release/biopax-level3.owl#Rna>",
            "<http://www.biopax.org/release/biopax-level3.owl#RnaRegion>",
        );
    normalize_ontology(&line)
}

/// Folds the two spellings of an ontology term URI (SBOL 2 `http://…/so/SO:`
/// and SBOL 3 `https://…/SO:`) onto one form. The conversion rewrites the
/// spelling between models, so normalizing both sides lets the diff ignore a
/// difference that is semantically identical.
fn normalize_ontology(line: &str) -> String {
    line.replace(
        "http://identifiers.org/so/SO:",
        "https://identifiers.org/SO:",
    )
    .replace(
        "http://identifiers.org/biomodels.sbo/SBO:",
        "https://identifiers.org/biomodels.sbo/SBO:",
    )
    .replace(
        "http://identifiers.org/edam/",
        "https://identifiers.org/edam:",
    )
}

/// Drops the `https://sbols.org/backport/2_3#` provenance annotations. These
/// are conversion metadata the upgrade stamps (e.g. `sbol2OriginalURI`), never
/// source data, so a round-trip diff of genuine content ignores them.
fn strip_backport(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .filter(|line| !line.contains("https://sbols.org/backport/2_3#"))
        .cloned()
        .collect()
}

#[test]
fn round_trip_simple_component_definition() {
    let input =
        std::fs::read_to_string(workspace_fixture("tests/fixtures/sbol2/single_cd.ttl")).unwrap();
    let (upgraded, _ureport) = sbol_convert::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();

    let (downgraded_graph, dreport) = sbol_convert::downgrade(&upgraded).expect("downgrade");
    assert!(
        dreport.is_clean(),
        "downgrade emitted warnings: {:?}",
        dreport.warnings()
    );

    // The downgraded graph should contain an sbol2:ComponentDefinition
    // typed subject and the canonical SBOL 2 persistentIdentity /
    // version triples.
    let triples = downgraded_graph.triples();
    let has_cd_type = triples.iter().any(|t| {
        t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            && t.object.as_iri().map(|i| i.as_str())
                == Some("http://sbols.org/v2#ComponentDefinition")
    });
    assert!(has_cd_type, "expected sbol2:ComponentDefinition rdf:type");

    let has_persistent_identity = triples
        .iter()
        .any(|t| t.predicate.as_str() == "http://sbols.org/v2#persistentIdentity");
    assert!(has_persistent_identity, "expected sbol2:persistentIdentity");

    let has_version = triples
        .iter()
        .any(|t| t.predicate.as_str() == "http://sbols.org/v2#version");
    assert!(has_version, "expected sbol2:version");
}

#[test]
fn identity_is_versioned_after_downgrade() {
    let input =
        std::fs::read_to_string(workspace_fixture("tests/fixtures/sbol2/single_cd.ttl")).unwrap();
    let (upgraded, _) = sbol_convert::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let (graph, _) = sbol_convert::downgrade(&upgraded).unwrap();

    // The original fixture has identity `<https://example.org/lab/J23100/1>`
    // (versioned). The upgrade strips `/1`; the downgrade should
    // restore it.
    let triples = graph.triples();
    let found_versioned_subject = triples.iter().any(|t| {
        t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/J23100/1")
    });
    assert!(
        found_versioned_subject,
        "expected `/1` version suffix restored on top-level identity"
    );
}

#[test]
fn downgrade_emits_well_formed_rdfxml() {
    let input =
        std::fs::read_to_string(workspace_fixture("tests/fixtures/sbol2/single_cd.ttl")).unwrap();
    let (upgraded, _) = sbol_convert::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let (graph, _) = sbol_convert::downgrade(&upgraded).unwrap();

    // Both Turtle and RDF/XML should serialize cleanly.
    let turtle = graph.write(RdfFormat::Turtle).expect("write turtle");
    assert!(turtle.contains("http://sbols.org/v2#ComponentDefinition"));
    let xml = graph.write(RdfFormat::RdfXml).expect("write rdfxml");
    assert!(xml.contains("ComponentDefinition") || xml.contains("v2#"));
}

#[test]
fn round_trip_single_cd() {
    // The fixture encodes `biopax:Dna`, which collapses to the same SBO term
    // as `biopax:DnaRegion` and downgrades to the `*Region` form; the diff
    // folds that canonicalization (see `normalize_biopax`).
    let (lost, gained) = round_trip_diff("tests/fixtures/sbol2/single_cd.ttl");
    assert!(
        lost.is_empty() && gained.is_empty(),
        "round-trip drift on single_cd.ttl\nonly in original:\n{}\nonly in downgrade:\n{}",
        lost.join("\n"),
        gained.join("\n")
    );
}

#[test]
fn round_trip_cd_with_subparts() {
    let (lost, gained) = round_trip_diff("tests/fixtures/sbol2/cd_with_subparts.ttl");
    assert!(
        lost.is_empty() && gained.is_empty(),
        "round-trip drift on cd_with_subparts.ttl\nonly in original:\n{}\nonly in downgrade:\n{}",
        lost.join("\n"),
        gained.join("\n")
    );
}

#[test]
fn round_trip_md_simple_structurally() {
    // md_simple has Interactions and Participations — the downgrade
    // restores them with the correct ModuleDefinition predicates. Check
    // the structural shape (counts of each rdf:type) rather than exact
    // triple equality, since the reconstruction may still have
    // ordering / annotation differences.
    let input =
        std::fs::read_to_string(workspace_fixture("tests/fixtures/sbol2/md_simple.ttl")).unwrap();
    let original = sbol3::RdfGraph::parse(&input, RdfFormat::Turtle).unwrap();
    let (upgraded, _) = sbol_convert::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let (downgraded, _) = sbol_convert::downgrade(&upgraded).unwrap();

    let count_types = |g: &sbol3::RdfGraph| -> std::collections::BTreeMap<String, usize> {
        let mut out = std::collections::BTreeMap::new();
        for t in g.triples() {
            if t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
                && let Some(iri) = t.object.as_iri()
            {
                *out.entry(iri.as_str().to_owned()).or_insert(0) += 1;
            }
        }
        out
    };
    let original_types = count_types(&original);
    let downgraded_types = count_types(&downgraded);
    for (type_iri, expected_count) in &original_types {
        let actual = downgraded_types.get(type_iri).copied().unwrap_or(0);
        assert!(
            actual >= *expected_count,
            "round-trip lost some {type_iri}: original {expected_count}, downgrade {actual}"
        );
    }
}

#[test]
fn round_trip_cd_with_annotation_preserves_range_count() {
    // SequenceAnnotation reconstruction: the upgrade collapses
    // SA-with-component into SubComponent.hasLocation. Phase 3
    // restoration reverses this. For now the Range count should
    // survive even if the SA wrapper hierarchy changes shape.
    let input = std::fs::read_to_string(workspace_fixture(
        "tests/fixtures/sbol2/cd_with_annotation.ttl",
    ))
    .unwrap();
    let original = sbol3::RdfGraph::parse(&input, RdfFormat::Turtle).unwrap();
    let (upgraded, _) = sbol_convert::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let (downgraded, _) = sbol_convert::downgrade(&upgraded).unwrap();

    let count_ranges = |g: &sbol3::RdfGraph| -> usize {
        g.triples()
            .iter()
            .filter(|t| {
                t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
                    && t.object.as_iri().map(|i| i.as_str()) == Some("http://sbols.org/v2#Range")
            })
            .count()
    };
    assert_eq!(
        count_ranges(&downgraded),
        count_ranges(&original),
        "Range count drift in round-trip"
    );
}

/// Round-trips a real iGEM design via SynBioHub. Exercises the SA
/// reconstruction path — every Range originates from an
/// SA-with-component collapse, and the downgrade must rebuild the
/// SequenceAnnotation wrapper so the re-upgrade re-collapses to the
/// same triple set.
#[test]
fn round_trip_real_synbiohub_sa_reconstruction() {
    let input = std::fs::read_to_string(workspace_fixture(
        "tests/fixtures/sbol2/real/synbiohub/BBa_F2620.xml",
    ))
    .unwrap();
    let (upgraded, _) =
        sbol_convert::upgrade_from_sbol2(&input, RdfFormat::RdfXml).expect("upgrade");
    let original = canonicalize(upgraded.rdf_graph());
    let (downgraded, _) = sbol_convert::downgrade(&upgraded).expect("downgrade");
    let turtle = downgraded.write(RdfFormat::Turtle).expect("write turtle");
    let (reupgraded, _) =
        sbol_convert::upgrade_from_sbol2(&turtle, RdfFormat::Turtle).expect("re-upgrade");
    let after = canonicalize(reupgraded.rdf_graph());
    let lost: Vec<&String> = original.iter().filter(|t| !after.contains(t)).collect();
    let gained: Vec<&String> = after.iter().filter(|t| !original.contains(t)).collect();
    assert!(
        lost.is_empty() && gained.is_empty(),
        "BBa_F2620 round-trip drifted: lost {}, gained {}\nFirst lost:\n{}\nFirst gained:\n{}",
        lost.len(),
        gained.len(),
        lost.iter()
            .take(3)
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        gained
            .iter()
            .take(3)
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

/// Round-trips the CRISPR repression model, exercising MapsTo
/// decomposition (ComponentReference + Constraint pairs) AND Interface
/// decomposition (FunctionalComponent directions folded into
/// `nondirectional` lists). Both must reverse cleanly for the SBOL 2
/// triple set to survive a full round trip.
#[test]
fn round_trip_real_repression_model_mapsto_and_interface() {
    let input = std::fs::read_to_string(workspace_fixture(
        "tests/fixtures/sbol2/real/RepressionModel.xml",
    ))
    .unwrap();
    let (upgraded, _) =
        sbol_convert::upgrade_from_sbol2(&input, RdfFormat::RdfXml).expect("upgrade");
    // Backport provenance is excluded from the comparison: `sbol2OriginalURI`
    // records a nested SBOL 2 identity whose version placement the re-upgrade
    // normalizes, so it is not stable across the round trip. The structural
    // content is.
    let original = strip_backport(&canonicalize(upgraded.rdf_graph()));
    let (downgraded, dreport) = sbol_convert::downgrade(&upgraded).expect("downgrade");
    assert!(
        dreport.counts().maps_to_reconstructed >= 5,
        "expected at least 5 MapsTo reconstructions, got {}",
        dreport.counts().maps_to_reconstructed,
    );
    let turtle = downgraded.write(RdfFormat::Turtle).expect("write turtle");
    let (reupgraded, _) =
        sbol_convert::upgrade_from_sbol2(&turtle, RdfFormat::Turtle).expect("re-upgrade");
    let after = strip_backport(&canonicalize(reupgraded.rdf_graph()));
    let lost: Vec<&String> = original.iter().filter(|t| !after.contains(t)).collect();
    let gained: Vec<&String> = after.iter().filter(|t| !original.contains(t)).collect();
    assert!(
        lost.is_empty() && gained.is_empty(),
        "RepressionModel round-trip drifted: lost {}, gained {}\nFirst lost:\n{}\nFirst gained:\n{}",
        lost.len(),
        gained.len(),
        lost.iter()
            .take(3)
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        gained
            .iter()
            .take(3)
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

#[test]
fn empty_default_version_is_rejected() {
    let input =
        std::fs::read_to_string(workspace_fixture("tests/fixtures/sbol2/single_cd.ttl")).unwrap();
    let (upgraded, _) = sbol_convert::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let mut options = sbol_convert::DowngradeOptions::default();
    options.default_version = Some(String::new());
    let err = sbol_convert::downgrade_with(&upgraded, options).unwrap_err();
    assert!(matches!(
        err,
        sbol_convert::DowngradeError::InvalidDefaultVersion(_)
    ));
}

#[test]
fn default_version_some_synthesizes_for_unversioned_sources() {
    // `ModuleDefinitionOutput.xml` ships without `sbol2:version` on any
    // top-level. With `default_version = Some("1")` the downgrade must
    // synthesize that version on every owned subject and emit the
    // matching warning. With the default `None`, no synthesis occurs.
    let path = workspace_fixture("tests/fixtures/sbol2/real/ModuleDefinitionOutput.xml");
    let input = std::fs::read_to_string(&path).unwrap();
    let (upgraded, _) = sbol_convert::upgrade_from_sbol2(&input, RdfFormat::RdfXml).unwrap();

    // Default `None`: no version triples synthesized.
    let (default_graph, default_report) = sbol_convert::downgrade(&upgraded).unwrap();
    assert!(
        default_report
            .warnings()
            .iter()
            .all(|w| !matches!(w, sbol_convert::DowngradeWarning::SynthesizedVersion { .. })),
        "default `None` must not emit SynthesizedVersion warnings"
    );
    let default_versioned = default_graph
        .triples()
        .iter()
        .any(|t| t.predicate.as_str() == "http://sbols.org/v2#version");
    assert!(
        !default_versioned,
        "default `None` must not emit any sbol2:version triples for an unversioned source"
    );

    // Opt-in `Some("1")`: synthesizes version.
    let mut options = sbol_convert::DowngradeOptions::default();
    options.default_version = Some("1".to_string());
    let (opt_in_graph, opt_in_report) = sbol_convert::downgrade_with(&upgraded, options).unwrap();
    let synthesized = opt_in_report
        .warnings()
        .iter()
        .filter(|w| matches!(w, sbol_convert::DowngradeWarning::SynthesizedVersion { .. }))
        .count();
    assert!(
        synthesized > 0,
        "opt-in `Some(\"1\")` must emit at least one SynthesizedVersion warning"
    );
    let any_versioned = opt_in_graph
        .triples()
        .iter()
        .any(|t| t.predicate.as_str() == "http://sbols.org/v2#version");
    assert!(
        any_versioned,
        "opt-in `Some(\"1\")` must emit sbol2:version triples"
    );
}
