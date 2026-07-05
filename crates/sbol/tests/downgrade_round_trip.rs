//! Integration tests for SBOL 2 → SBOL 3 → SBOL 2 round-trip
//! preservation, plus IRI-collision and version-synthesis behavior.

mod common;

use common::downgrade::*;

use sbol::{Document, RdfFormat};

/// The killer test: take an SBOL 2 fixture, upgrade to SBOL 3,
/// downgrade back to SBOL 2, and diff against the original. Anything
/// that doesn't survive the round trip is either a bug in one
/// direction or an intentional documented divergence.
fn round_trip_diff(fixture: &str) -> (Vec<String>, Vec<String>) {
    let input = std::fs::read_to_string(workspace_fixture(fixture)).unwrap();
    let original_graph = sbol::RdfGraph::parse(&input, RdfFormat::Turtle).expect("parse original");
    let original: Vec<String> = canonicalize(&original_graph);

    let (upgraded, _ureport) =
        sbol::upgrade::upgrade_from_sbol2(&input, RdfFormat::Turtle).expect("upgrade");
    let (downgraded_graph, _dreport) = sbol::downgrade::downgrade(&upgraded).expect("downgrade");
    let downgraded: Vec<String> = canonicalize(&downgraded_graph);

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

fn canonicalize(graph: &sbol::RdfGraph) -> Vec<String> {
    let mut lines: Vec<String> = graph
        .normalized_triples()
        .iter()
        .map(sbol::upgrade::canonical_nt_line)
        .collect();
    lines.sort();
    lines.dedup();
    lines
}

#[test]
fn round_trip_simple_component_definition() {
    let input =
        std::fs::read_to_string(workspace_fixture("tests/fixtures/sbol2/single_cd.ttl")).unwrap();
    let (upgraded, _ureport) = sbol::upgrade::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();

    let (downgraded_graph, dreport) = sbol::downgrade::downgrade(&upgraded).expect("downgrade");
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
    let (upgraded, _) = sbol::upgrade::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let (graph, _) = sbol::downgrade::downgrade(&upgraded).unwrap();

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
    let (upgraded, _) = sbol::upgrade::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let (graph, _) = sbol::downgrade::downgrade(&upgraded).unwrap();

    // Both Turtle and RDF/XML should serialize cleanly.
    let turtle = graph.write(RdfFormat::Turtle).expect("write turtle");
    assert!(turtle.contains("http://sbols.org/v2#ComponentDefinition"));
    let xml = graph.write(RdfFormat::RdfXml).expect("write rdfxml");
    assert!(xml.contains("ComponentDefinition") || xml.contains("v2#"));
}

#[test]
fn round_trip_single_cd() {
    // The fixture encodes `biopax:Dna` (bare, not `DnaRegion`). The
    // BioPAX collapse on upgrade is reversed via `backport:biopaxType`,
    // so the round-trip is lossless without filtering.
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
    // md_simple has Interactions and Participations — Phase 3 should
    // restore them with the correct MD predicates. Check the
    // structural shape (counts of each rdf:type) rather than exact
    // triple equality, since the Phase 3 reconstruction may still
    // have ordering / annotation differences.
    let input =
        std::fs::read_to_string(workspace_fixture("tests/fixtures/sbol2/md_simple.ttl")).unwrap();
    let original = sbol::RdfGraph::parse(&input, RdfFormat::Turtle).unwrap();
    let (upgraded, _) = sbol::upgrade::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let (downgraded, _) = sbol::downgrade::downgrade(&upgraded).unwrap();

    let count_types = |g: &sbol::RdfGraph| -> std::collections::BTreeMap<String, usize> {
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
    let original = sbol::RdfGraph::parse(&input, RdfFormat::Turtle).unwrap();
    let (upgraded, _) = sbol::upgrade::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let (downgraded, _) = sbol::downgrade::downgrade(&upgraded).unwrap();

    let count_ranges = |g: &sbol::RdfGraph| -> usize {
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
    let (upgraded, _) = sbol::upgrade::upgrade_from_sbol2(&input, RdfFormat::RdfXml).expect("upgrade");
    let original = canonicalize(upgraded.rdf_graph());
    let (downgraded, _) = sbol::downgrade::downgrade(&upgraded).expect("downgrade");
    let turtle = downgraded.write(RdfFormat::Turtle).expect("write turtle");
    let (reupgraded, _) =
        sbol::upgrade::upgrade_from_sbol2(&turtle, RdfFormat::Turtle).expect("re-upgrade");
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
    let (upgraded, _) = sbol::upgrade::upgrade_from_sbol2(&input, RdfFormat::RdfXml).expect("upgrade");
    let original = canonicalize(upgraded.rdf_graph());
    let (downgraded, dreport) = sbol::downgrade::downgrade(&upgraded).expect("downgrade");
    assert!(
        dreport.counts().maps_to_reconstructed >= 5,
        "expected at least 5 MapsTo reconstructions, got {}",
        dreport.counts().maps_to_reconstructed,
    );
    let turtle = downgraded.write(RdfFormat::Turtle).expect("write turtle");
    let (reupgraded, _) =
        sbol::upgrade::upgrade_from_sbol2(&turtle, RdfFormat::Turtle).expect("re-upgrade");
    let after = canonicalize(reupgraded.rdf_graph());
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
    let (upgraded, _) = sbol::upgrade::upgrade_from_sbol2(&input, RdfFormat::Turtle).unwrap();
    let mut options = sbol::DowngradeOptions::default();
    options.default_version = Some(String::new());
    let err = sbol::downgrade::downgrade_with(&upgraded, options).unwrap_err();
    assert!(matches!(
        err,
        sbol::DowngradeError::InvalidDefaultVersion(_)
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
    let (upgraded, _) = sbol::upgrade::upgrade_from_sbol2(&input, RdfFormat::RdfXml).unwrap();

    // Default `None`: no version triples synthesized.
    let (default_graph, default_report) = sbol::downgrade::downgrade(&upgraded).unwrap();
    assert!(
        default_report
            .warnings()
            .iter()
            .all(|w| !matches!(w, sbol::DowngradeWarning::SynthesizedVersion { .. })),
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
    let mut options = sbol::DowngradeOptions::default();
    options.default_version = Some("1".to_string());
    let (opt_in_graph, opt_in_report) = sbol::downgrade::downgrade_with(&upgraded, options).unwrap();
    let synthesized = opt_in_report
        .warnings()
        .iter()
        .filter(|w| matches!(w, sbol::DowngradeWarning::SynthesizedVersion { .. }))
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

#[test]
fn collapsed_sequence_annotation_metadata_round_trips() {
    let input = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix so: <https://identifiers.org/SO:> .
@prefix ex: <https://example.org/custom#> .

<https://example.org/lab/cd/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna ;
    sbol:sequence <https://example.org/lab/seq/1> ;
    sbol:component <https://example.org/lab/cd/sub/1> ;
    sbol:sequenceAnnotation <https://example.org/lab/cd/ann/1> .

<https://example.org/lab/seq/1>
    a sbol:Sequence ;
    sbol:persistentIdentity <https://example.org/lab/seq> ;
    sbol:displayId "seq" ;
    sbol:version "1" ;
    sbol:elements "ATGC" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .

<https://example.org/lab/part/1>
    a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://example.org/lab/part> ;
    sbol:displayId "part" ;
    sbol:version "1" ;
    sbol:type biopax:Dna .

<https://example.org/lab/cd/sub/1>
    a sbol:Component ;
    sbol:persistentIdentity <https://example.org/lab/cd/sub> ;
    sbol:displayId "sub" ;
    sbol:version "1" ;
    sbol:definition <https://example.org/lab/part/1> ;
    sbol:access sbol:public .

<https://example.org/lab/cd/ann/1>
    a sbol:SequenceAnnotation, ex:SpecialAnnotation ;
    sbol:persistentIdentity <https://example.org/lab/cd/ann> ;
    sbol:displayId "ann" ;
    sbol:version "1" ;
    sbol:component <https://example.org/lab/cd/sub/1> ;
    sbol:location <https://example.org/lab/cd/ann/range/1> ;
    sbol:role so:0000167 ;
    dcterms:title "annotation title" ;
    prov:wasDerivedFrom <https://example.org/source/ann> .

<https://example.org/lab/cd/ann/range/1>
    a sbol:Range ;
    sbol:persistentIdentity <https://example.org/lab/cd/ann/range> ;
    sbol:displayId "range" ;
    sbol:version "1" ;
    sbol:start 1 ;
    sbol:end 4 ;
    sbol:sequence <https://example.org/lab/seq/1> .
"#;
    let (upgraded, _ureport) =
        sbol::upgrade::upgrade_from_sbol2(input, RdfFormat::Turtle).expect("upgrade");
    let (downgraded, _dreport) = sbol::downgrade::downgrade(&upgraded).expect("downgrade");
    let sa = "https://example.org/lab/cd/ann/1";

    assert!(
        has_triple(
            &downgraded,
            sa,
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "https://example.org/custom#SpecialAnnotation"
        ),
        "extra rdf:type on a collapsed SequenceAnnotation should round-trip"
    );
    assert!(
        has_triple(
            &downgraded,
            sa,
            "http://sbols.org/v2#persistentIdentity",
            "https://example.org/lab/cd/ann"
        ),
        "SA persistentIdentity should be restored from preserved metadata"
    );
    assert!(
        has_literal_triple(&downgraded, sa, "http://sbols.org/v2#version", "1"),
        "SA version should be restored from preserved metadata"
    );
    assert!(
        has_triple(
            &downgraded,
            sa,
            "http://sbols.org/v2#role",
            "https://identifiers.org/SO:0000167"
        ),
        "SA role should be restored from preserved metadata"
    );
    assert!(
        has_literal_triple(
            &downgraded,
            sa,
            "http://purl.org/dc/terms/title",
            "annotation title"
        ),
        "SA Dublin Core metadata should be restored from preserved metadata"
    );
    assert!(
        has_triple(
            &downgraded,
            sa,
            "http://www.w3.org/ns/prov#wasDerivedFrom",
            "https://example.org/source/ann"
        ),
        "SA provenance should be restored from preserved metadata"
    );
}

/// A ComponentDefinition carrying both `biopax:Dna` and
/// `biopax:DnaRegion` — two variants that collapse to the *same* SBO
/// term (`SBO:0000251`) on upgrade — must round-trip distinctly: one
/// `sbol2:type biopax:Dna` triple and one `sbol2:type biopax:DnaRegion`
/// triple in the SBOL 2 output. Previously the downgrade picked the
/// first iterator-order variant for every `sbol3:type SBO:0000251`
/// triple, dropping the other variant.
#[test]
fn biopax_dna_plus_dna_region_round_trip_distinctly() {
    let sbol2 = r#"
@prefix sbol: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://lab/cd/1> a sbol:ComponentDefinition ;
    sbol:persistentIdentity <https://lab/cd> ;
    sbol:displayId "cd" ;
    sbol:version "1" ;
    sbol:type biopax:Dna ;
    sbol:type biopax:DnaRegion .
"#;
    let (document, _r) = sbol::upgrade::upgrade_from_sbol2(sbol2, RdfFormat::Turtle).expect("upgrade");
    let (graph, _r) = sbol::downgrade::downgrade(&document).expect("downgrade");

    let biopax_types: std::collections::HashSet<String> = graph
        .triples()
        .iter()
        .filter(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/cd/1")
                && t.predicate.as_str() == "http://sbols.org/v2#type"
        })
        .filter_map(|t| t.object.as_iri().map(|i| i.as_str().to_owned()))
        .collect();
    assert!(
        biopax_types.contains("http://www.biopax.org/release/biopax-level3.owl#Dna"),
        "biopax:Dna lost on round-trip — got {biopax_types:?}"
    );
    assert!(
        biopax_types.contains("http://www.biopax.org/release/biopax-level3.owl#DnaRegion"),
        "biopax:DnaRegion lost on round-trip — got {biopax_types:?}"
    );
}

/// A ComponentDefinition that carried multiple distinct BioPAX types
/// (e.g. both `Dna` and `DnaRegion`) used to collapse to one preserved
/// hint, losing the other variant on round-trip. The preserved set is
/// now multi-valued and the downgrade picks the variant whose SBO
/// target matches each `sbol3:type` triple.
#[test]
fn multi_biopax_types_round_trip_distinctly() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix backport: <http://sboltools.org/backport#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://lab/mixed> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "mixed" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:type <https://identifiers.org/SBO:0000252> ;
    backport:biopaxType biopax:Dna ;
    backport:biopaxType biopax:Protein ;
    backport:sbol2type <http://sbols.org/v2#ComponentDefinition> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = sbol::downgrade::downgrade(&document).expect("downgrade");

    let types_emitted: std::collections::HashSet<String> = graph
        .triples()
        .iter()
        .filter(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/mixed")
                && t.predicate.as_str() == "http://sbols.org/v2#type"
        })
        .filter_map(|t| t.object.as_iri().map(|i| i.as_str().to_owned()))
        .collect();
    assert!(
        types_emitted.contains("http://www.biopax.org/release/biopax-level3.owl#Dna"),
        "expected biopax:Dna to be restored (preserved variant for SBO:0000251), got {types_emitted:?}"
    );
    assert!(
        types_emitted.contains("http://www.biopax.org/release/biopax-level3.owl#Protein"),
        "expected biopax:Protein to be restored (preserved variant for SBO:0000252), got {types_emitted:?}"
    );
}

/// Two distinct SBOL 3 subjects whose `iri_rewrites` rewrite to the
/// same SBOL 2 versioned IRI silently merge into one chimeric
/// SBOL 2 subject. The input is technically non-conformant (the
/// implied SBOL 2 versioned identities should be unique), but the
/// merge is otherwise invisible; the downgrade now emits
/// `DowngradeWarning::IdentityCollision` so callers can audit.
/// Symmetric with `UpgradeWarning::IdentityCollision` on the inverse
/// direction.
#[test]
fn colliding_iri_rewrites_emit_warning() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix backport: <http://sboltools.org/backport#> .

<https://lab/foo> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "foo" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    backport:sbol2type <http://sbols.org/v2#ComponentDefinition> ;
    backport:sbol2version "1/1" .

<https://lab/foo/1> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "foo" ;
    sbol3:type <https://identifiers.org/SBO:0000252> ;
    backport:sbol2type <http://sbols.org/v2#ComponentDefinition> ;
    backport:sbol2version "1" .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (_graph, report) = sbol::downgrade::downgrade(&document).expect("downgrade");
    let collision = report.warnings().iter().find_map(|w| match w {
        sbol::DowngradeWarning::IdentityCollision { canonical, sources } => {
            Some((canonical.clone(), sources.clone()))
        }
        _ => None,
    });
    let (canonical, sources) = collision.expect("expected IdentityCollision warning");
    assert_eq!(canonical, "https://lab/foo/1/1");
    assert_eq!(
        sources,
        vec![
            "https://lab/foo".to_string(),
            "https://lab/foo/1".to_string(),
        ]
    );
}
