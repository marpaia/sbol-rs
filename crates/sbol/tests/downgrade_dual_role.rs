//! Integration tests for the SBOL 3 Component → SBOL 2
//! ComponentDefinition + ModuleDefinition dual-role split.

mod common;

use common::downgrade::*;

use sbol::{Document, RdfFormat};

#[test]
fn dual_role_interface_direction_lands_on_functional_component_variant() {
    let ttl = r#"
@prefix sbol: <http://sbols.org/v3#> .

<https://example.org/lab/dual> a sbol:Component ;
    sbol:displayId "dual" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> ;
    sbol:hasSequence <https://example.org/lab/dual_seq> ;
    sbol:hasFeature <https://example.org/lab/dual/sub> ;
    sbol:hasInteraction <https://example.org/lab/dual/interaction> ;
    sbol:hasInterface <https://example.org/lab/dual/interface> .

<https://example.org/lab/dual/sub> a sbol:SubComponent ;
    sbol:displayId "sub" ;
    sbol:instanceOf <https://example.org/lab/part> .

<https://example.org/lab/dual/interface> a sbol:Interface ;
    sbol:displayId "interface" ;
    sbol:input <https://example.org/lab/dual/sub> .

<https://example.org/lab/dual/interaction> a sbol:Interaction ;
    sbol:displayId "interaction" ;
    sbol:type <https://identifiers.org/SBO:0000170> .

<https://example.org/lab/dual_seq> a sbol:Sequence ;
    sbol:displayId "dual_seq" ;
    sbol:hasNamespace <https://example.org/lab> .

<https://example.org/lab/part> a sbol:Component ;
    sbol:displayId "part" ;
    sbol:hasNamespace <https://example.org/lab> ;
    sbol:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/dual/sub_fc",
            "http://sbols.org/v2#direction",
            "http://sbols.org/v2#in"
        ),
        "dual-role Interface.input should target the MD-side FunctionalComponent variant"
    );
    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/dual/sub_fc",
            "http://sbols.org/v2#access",
            "http://sbols.org/v2#public"
        ),
        "Interface-visible FunctionalComponent variant should be public"
    );
    assert_eq!(
        graph
            .triples()
            .iter()
            .filter(|t| {
                t.subject.as_iri().map(|i| i.as_str()) == Some("https://example.org/lab/dual/sub")
                    && t.predicate.as_str() == "http://sbols.org/v2#direction"
            })
            .count(),
        0,
        "dual-role CD-side Component variant must not receive direction"
    );
}

#[test]
fn downgrade_honors_split_dual_role_components_false() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/dual_role> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/dual_role_seq> ;
    sbol3:hasInteraction <https://lab/dual_role/some_interaction> .

<https://lab/dual_role/some_interaction> a sbol3:Interaction ;
    sbol3:displayId "some_interaction" ;
    sbol3:type <https://identifiers.org/SBO:0000170> .

<https://lab/dual_role_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role_seq" .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let mut options = sbol::DowngradeOptions::default();
    options.split_dual_role_components = false;
    let (graph, report) = document
        .downgrade_to_sbol2_with(options)
        .expect("downgrade");

    assert!(
        !report
            .warnings()
            .iter()
            .any(|w| matches!(w, sbol::DowngradeWarning::DualRoleComponent { .. })),
        "split_dual_role_components=false should not emit a dual-role split warning: {:?}",
        report.warnings()
    );
    let triples = graph.triples();
    let has_type = |subject: &str, ty: &str| {
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some(subject)
                && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
                && t.object.as_iri().map(|i| i.as_str()) == Some(ty)
        })
    };
    assert!(
        has_type(
            "https://lab/dual_role",
            "http://sbols.org/v2#ModuleDefinition"
        ),
        "dual-role Component should collapse to a single MD when splitting is disabled"
    );
    assert!(
        !has_type(
            "https://lab/dual_role_component",
            "http://sbols.org/v2#ComponentDefinition",
        ),
        "split_dual_role_components=false still emitted the synthesized CD half"
    );
}

/// Native SBOL 3 dual-role Component: carries both structural
/// (`sbol3:hasSequence`, biopax type) AND functional
/// (`sbol3:hasInteraction`) data, so it splits on downgrade into a CD
/// holding the structural triples and an MD holding the functional
/// triples, plus a synthesized linking FunctionalComponent.
#[test]
fn dual_role_component_splits_into_cd_and_md() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .
@prefix sbo: <https://identifiers.org/SBO:> .

<https://lab/dual_role> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/dual_role_seq> ;
    sbol3:hasInteraction <https://lab/dual_role/some_interaction> .

<https://lab/dual_role/some_interaction> a sbol3:Interaction ;
    sbol3:displayId "some_interaction" ;
    sbol3:type <https://identifiers.org/SBO:0000170> .

<https://lab/dual_role_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role_seq" .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, report) = document.downgrade_to_sbol2().expect("downgrade");

    assert_eq!(
        report.counts().components_split_into_both,
        1,
        "expected one dual-role split, got counts={:?}",
        report.counts()
    );
    assert!(
        report
            .warnings()
            .iter()
            .any(|w| matches!(w, sbol::DowngradeWarning::DualRoleComponent { .. })),
        "expected a DualRoleComponent warning, got {:?}",
        report.warnings()
    );

    let triples = graph.triples();
    let has_type = |subject: &str, ty: &str| {
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some(subject)
                && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
                && t.object.as_iri().map(|i| i.as_str()) == Some(ty)
        })
    };
    assert!(
        has_type(
            "https://lab/dual_role_component",
            "http://sbols.org/v2#ComponentDefinition",
        ),
        "expected sbol2:ComponentDefinition on the _component half"
    );
    assert!(
        has_type(
            "https://lab/dual_role",
            "http://sbols.org/v2#ModuleDefinition"
        ),
        "expected sbol2:ModuleDefinition on the bare IRI (heuristic: interactions present)"
    );
    assert!(
        has_type(
            "https://lab/dual_role/dual_role",
            "http://sbols.org/v2#FunctionalComponent",
        ),
        "expected synthesized linking FunctionalComponent"
    );

    // The CD carries the structural triples; the MD carries the
    // functional ones; the linking FC carries the SplitComponentComposition
    // marker so a future re-upgrade can detect the split origin.
    let cd_has = |predicate: &str| {
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/dual_role_component")
                && t.predicate.as_str() == predicate
        })
    };
    let md_has = |predicate: &str| {
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/dual_role")
                && t.predicate.as_str() == predicate
        })
    };
    assert!(
        cd_has("http://sbols.org/v2#sequence"),
        "CD missing sbol2:sequence"
    );
    assert!(cd_has("http://sbols.org/v2#type"), "CD missing sbol2:type");
    assert!(
        md_has("http://sbols.org/v2#interaction"),
        "MD missing sbol2:interaction"
    );
    assert!(
        md_has("http://sbols.org/v2#functionalComponent"),
        "MD missing the linking FunctionalComponent"
    );
    let has_split_marker = triples.iter().any(|t| {
        t.predicate.as_str() == "http://sboltools.org/backport#type"
            && t.object.as_iri().map(|i| i.as_str())
                == Some("http://sboltools.org/backport#SplitComponentComposition")
    });
    assert!(
        has_split_marker,
        "expected SplitComponentComposition marker on linking FC"
    );

    // Both halves carry backport:sbol3identity pointing at the original
    // SBOL 3 Component IRI so the inverse direction can re-merge.
    let sbol3id_count = triples
        .iter()
        .filter(|t| {
            t.predicate.as_str() == "http://sboltools.org/backport#sbol3identity"
                && t.object.as_iri().map(|i| i.as_str()) == Some("https://lab/dual_role")
        })
        .count();
    assert_eq!(
        sbol3id_count, 2,
        "expected 2 backport:sbol3identity stamps (one per half)"
    );
}

#[test]
fn split_subjects_preserve_extension_types_and_archive_unknown_sbol3_predicates() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .
@prefix ex: <https://example.org/types#> .

<https://lab/dual_role> a sbol3:Component, ex:CustomDual ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/dual_role_seq> ;
    sbol3:hasFeature <https://lab/dual_role/sc> ;
    sbol3:hasInteraction <https://lab/dual_role/inter> ;
    sbol3:futureThing <https://example.org/value> .

<https://lab/dual_role/sc> a sbol3:SubComponent, ex:CustomSub ;
    sbol3:displayId "sc" ;
    sbol3:instanceOf <https://lab/target> ;
    sbol3:futureSub <https://example.org/subvalue> .

<https://lab/target> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "target" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .

<https://lab/dual_role/inter> a sbol3:Interaction ;
    sbol3:displayId "inter" ;
    sbol3:type <https://identifiers.org/SBO:0000170> .

<https://lab/dual_role_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role_seq" .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, report) = document.downgrade_to_sbol2().expect("downgrade");
    let dual_warnings = report
        .warnings()
        .iter()
        .filter(|w| matches!(w, sbol::DowngradeWarning::DualRoleComponent { .. }))
        .count();
    assert_eq!(report.counts().components_split_into_both, 1);
    assert_eq!(dual_warnings, 1, "dual-role warning should not duplicate");

    assert_eq!(
        count_triples(
            &graph,
            "https://lab/dual_role_component",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v2#ComponentDefinition",
        ),
        1,
        "CD split type should be emitted once"
    );
    assert_eq!(
        count_triples(
            &graph,
            "https://lab/dual_role",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v2#ModuleDefinition",
        ),
        1,
        "MD split type should be emitted once"
    );
    assert_eq!(
        count_triples(
            &graph,
            "https://lab/dual_role",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "https://example.org/types#CustomDual",
        ),
        1,
        "dual-role Component extension rdf:type should survive on the bare half"
    );
    assert_eq!(
        count_triples(
            &graph,
            "https://lab/dual_role/sc",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v2#Component",
        ),
        1,
        "SubComponent Component split type should be emitted once"
    );
    assert_eq!(
        count_triples(
            &graph,
            "https://lab/dual_role/sc_fc",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "http://sbols.org/v2#FunctionalComponent",
        ),
        1,
        "SubComponent FunctionalComponent split type should be emitted once"
    );
    assert_eq!(
        count_triples(
            &graph,
            "https://lab/dual_role/sc",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "https://example.org/types#CustomSub",
        ),
        1,
        "split SubComponent extension rdf:type should survive on the bare variant"
    );
    assert!(
        has_triple(
            &graph,
            "https://lab/dual_role",
            "http://sboltools.org/backport#sbol3_futureThing",
            "https://example.org/value",
        ),
        "unknown SBOL 3 predicate on dual-role Component should be archived"
    );
    assert!(
        has_triple(
            &graph,
            "https://lab/dual_role/sc",
            "http://sboltools.org/backport#sbol3_futureSub",
            "https://example.org/subvalue",
        ),
        "unknown SBOL 3 predicate on split SubComponent should be archived"
    );
}

/// SubComponents under a dual-role parent triple into three SBOL 2
/// variants: an `sbol2:Component` under the CD half, an
/// `sbol2:FunctionalComponent` under the MD half, and (when the target
/// is an MD-shaped Component) an `sbol2:Module` under the MD half.
/// Each variant gets its own `sbol2:definition` plus identified
/// properties so all three are valid SBOL 2 objects.
#[test]
fn dual_role_subcomponent_triples_into_three_variants() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/dual_role> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/seq> ;
    sbol3:hasFeature <https://lab/dual_role/sc> ;
    sbol3:hasInteraction <https://lab/dual_role/inter> .

<https://lab/dual_role/sc> a sbol3:SubComponent ;
    sbol3:displayId "sc" ;
    sbol3:instanceOf <https://lab/target> .

<https://lab/target> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "target" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .

<https://lab/dual_role/inter> a sbol3:Interaction ;
    sbol3:displayId "inter" ;
    sbol3:type <https://identifiers.org/SBO:0000170> .

<https://lab/seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "seq" .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    let triples = graph.triples();
    let has_typed_subject = |iri: &str, ty: &str| {
        triples.iter().any(|t| {
            t.subject.as_iri().map(|i| i.as_str()) == Some(iri)
                && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
                && t.object.as_iri().map(|i| i.as_str()) == Some(ty)
        })
    };

    // The target is CD-only (just type+namespace), so its SubComponent
    // doesn't get a Module variant. Component (C) keeps the bare IRI;
    // FunctionalComponent gets the `_fc` suffix.
    assert!(
        has_typed_subject("https://lab/dual_role/sc", "http://sbols.org/v2#Component"),
        "expected sbol2:Component on bare SubComponent IRI"
    );
    assert!(
        has_typed_subject(
            "https://lab/dual_role/sc_fc",
            "http://sbols.org/v2#FunctionalComponent",
        ),
        "expected sbol2:FunctionalComponent on `_fc` variant"
    );

    // Each variant carries its own definition pointing at the target's
    // CD half.
    let target_cd = "https://lab/target";
    let count_definition = |subject: &str| {
        triples
            .iter()
            .filter(|t| {
                t.subject.as_iri().map(|i| i.as_str()) == Some(subject)
                    && t.predicate.as_str() == "http://sbols.org/v2#definition"
                    && t.object.as_iri().map(|i| i.as_str()) == Some(target_cd)
            })
            .count()
    };
    assert_eq!(
        count_definition("https://lab/dual_role/sc"),
        1,
        "C variant missing sbol2:definition → target"
    );
    assert_eq!(
        count_definition("https://lab/dual_role/sc_fc"),
        1,
        "FC variant missing sbol2:definition → target"
    );
}

/// A Collection that lists a dual-role Component as a member must
/// reference BOTH halves of the split in the SBOL 2 output. Otherwise
/// the SBOL 2 Collection only sees the structural OR functional view,
/// losing data.
#[test]
fn collection_membership_duplicates_for_dual_role_split() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/collection> a sbol3:Collection ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "collection" ;
    sbol3:member <https://lab/dual_role> .

<https://lab/dual_role> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/dual_role_seq> ;
    sbol3:hasInteraction <https://lab/dual_role/some_interaction> .

<https://lab/dual_role/some_interaction> a sbol3:Interaction ;
    sbol3:displayId "some_interaction" ;
    sbol3:type <https://identifiers.org/SBO:0000170> .

<https://lab/dual_role_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_role_seq" .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    let members: Vec<&str> = graph
        .triples()
        .iter()
        .filter(|t| {
            t.predicate.as_str() == "http://sbols.org/v2#member"
                && t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/collection")
        })
        .filter_map(|t| t.object.as_iri().map(|i| i.as_str()))
        .collect();
    assert!(
        members.contains(&"https://lab/dual_role"),
        "Collection missing MD-half member, got {members:?}"
    );
    assert!(
        members.contains(&"https://lab/dual_role_component"),
        "Collection missing CD-half member, got {members:?}"
    );
}

/// Native SBOL 3 dual-role Components whose displayId matches a
/// SubComponent's displayId previously emitted the synthesized linking
/// FunctionalComponent at the SAME IRI as the SubComponent — two
/// contradictory rdf:types on one subject, rejected by any compliant
/// SBOL 2 reader. The downgrade now allocates the next available
/// `displayId_N` and propagates it to the FC's displayId so the IRI's
/// last segment matches.
#[test]
fn dual_role_linking_fc_avoids_subcomponent_iri_collision() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/widget> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "widget" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/widget_seq> ;
    sbol3:hasInteraction <https://lab/widget/inhibition> ;
    sbol3:hasFeature <https://lab/widget/widget> .

<https://lab/widget/widget> a sbol3:SubComponent ;
    sbol3:displayId "widget" ;
    sbol3:instanceOf <https://lab/inner> .

<https://lab/widget/inhibition> a sbol3:Interaction ;
    sbol3:displayId "inhibition" ;
    sbol3:type <https://identifiers.org/SBO:0000169> .

<https://lab/widget_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "widget_seq" ;
    sbol3:encoding <https://identifiers.org/edam:format_1207> ;
    sbol3:elements "ACGT" .

<https://lab/inner> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "inner" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");

    let collision_iri = "https://lab/widget/widget";
    let types_at = |iri: &str| -> Vec<String> {
        graph
            .triples()
            .iter()
            .filter(|t| {
                t.subject.as_iri().map(|i| i.as_str()) == Some(iri)
                    && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            })
            .filter_map(|t| t.object.as_iri().map(|i| i.as_str().to_owned()))
            .collect()
    };
    let types = types_at(collision_iri);
    assert!(
        !(types
            .iter()
            .any(|t| t == "http://sbols.org/v2#FunctionalComponent")
            && types.iter().any(|t| t == "http://sbols.org/v2#Component")),
        "linking FC and SubComponent must not share an IRI, got types at {collision_iri}: {types:?}"
    );

    // The disambiguated linking FC should live at `widget_2`.
    let disambig_iri = "https://lab/widget/widget_2";
    assert!(
        types_at(disambig_iri)
            .iter()
            .any(|t| t == "http://sbols.org/v2#FunctionalComponent"),
        "expected synthesized linking FC at the next-available IRI `{disambig_iri}`"
    );

    // The disambiguated displayId must match the new IRI's last segment
    // so the output passes SBOL 2 compliance (sbol-12302).
    let did = graph.triples().iter().find_map(|t| {
        (t.subject.as_iri().map(|i| i.as_str()) == Some(disambig_iri)
            && t.predicate.as_str() == "http://sbols.org/v2#displayId")
            .then(|| t.object.as_literal().map(|l| l.value().to_owned()))
            .flatten()
    });
    assert_eq!(
        did.as_deref(),
        Some("widget_2"),
        "linking FC displayId must equal its IRI's last segment"
    );
}

/// SBOL 2 sbol-12302 requires `displayId` to equal the last path segment
/// of `persistentIdentity`. When the downgrade triple-splits a
/// SubComponent under a dual-role parent into `_c` / `_fc` / `_m`
/// variants, each variant's displayId must carry the same suffix.
#[test]
fn dual_role_subcomponent_variant_display_ids_match_iri_suffix() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/dual> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/dual_seq> ;
    sbol3:hasInteraction <https://lab/dual/inh> ;
    sbol3:hasFeature <https://lab/dual/inner_sc> .

<https://lab/dual/inner_sc> a sbol3:SubComponent ;
    sbol3:displayId "inner_sc" ;
    sbol3:instanceOf <https://lab/target> .

<https://lab/dual/inh> a sbol3:Interaction ;
    sbol3:displayId "inh" ;
    sbol3:type <https://identifiers.org/SBO:0000169> .

<https://lab/dual_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "dual_seq" ;
    sbol3:encoding <https://identifiers.org/edam:format_1207> ;
    sbol3:elements "ACGT" .

<https://lab/target> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "target" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = document.downgrade_to_sbol2().expect("downgrade");
    let display_id_at = |iri: &str| -> Option<String> {
        graph.triples().iter().find_map(|t| {
            (t.subject.as_iri().map(|i| i.as_str()) == Some(iri)
                && t.predicate.as_str() == "http://sbols.org/v2#displayId")
                .then(|| t.object.as_literal().map(|l| l.value().to_owned()))
                .flatten()
        })
    };
    assert_eq!(
        display_id_at("https://lab/dual/inner_sc").as_deref(),
        Some("inner_sc"),
        "Component variant keeps the bare displayId"
    );
    assert_eq!(
        display_id_at("https://lab/dual/inner_sc_fc").as_deref(),
        Some("inner_sc_fc"),
        "FunctionalComponent variant displayId must carry the `_fc` suffix to match its IRI"
    );
}

/// A dual-role Component whose synthesized CD half lands on a
/// separately-named Component's IRI used to silently merge the two
/// (both `sbol2:ComponentDefinition` rdf:types on a single IRI plus
/// chimeric structural triples). The downgrade now routes the
/// synthesized half through the suffix allocator, so a collision picks
/// up a `_2` tail instead of overwriting the sibling Component.
#[test]
fn dual_role_cd_half_avoids_separately_named_component_iri() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/X> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "X" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/X_seq> ;
    sbol3:hasInteraction <https://lab/X/inter> .

<https://lab/X/inter> a sbol3:Interaction ;
    sbol3:displayId "inter" ;
    sbol3:type <https://identifiers.org/SBO:0000169> .

<https://lab/X_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "X_seq" ;
    sbol3:elements "ACGT" .

<https://lab/X_component> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "X_component" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _r) = document.downgrade_to_sbol2().expect("downgrade");

    let types_at = |iri: &str| -> Vec<String> {
        graph
            .triples()
            .iter()
            .filter(|t| {
                t.subject.as_iri().map(|i| i.as_str()) == Some(iri)
                    && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            })
            .filter_map(|t| t.object.as_iri().map(|i| i.as_str().to_owned()))
            .collect()
    };

    let at_collision = types_at("https://lab/X_component");
    assert_eq!(
        at_collision.len(),
        1,
        "the separately-named Component should keep its IRI to itself, got {at_collision:?}"
    );

    assert!(
        types_at("https://lab/X_component_2")
            .iter()
            .any(|t| t == "http://sbols.org/v2#ComponentDefinition"),
        "X's CD half should disambiguate to `_component_2` when `_component` is taken"
    );

    // And the displayId of the disambiguated CD half must match its
    // IRI's last segment (SBOL 2 sbol-12302 compliance).
    let did = graph.triples().iter().find_map(|t| {
        (t.subject.as_iri().map(|i| i.as_str()) == Some("https://lab/X_component_2")
            && t.predicate.as_str() == "http://sbols.org/v2#displayId")
            .then(|| t.object.as_literal().map(|l| l.value().to_owned()))
            .flatten()
    });
    assert_eq!(
        did.as_deref(),
        Some("X_component_2"),
        "displayId of the disambiguated half must equal its IRI's last segment"
    );
}

/// Two SubComponents under a dual-role parent named `foo` and `foo_fc`
/// previously produced a collision: when `foo` triple-splits, its FC
/// variant lands at `parent/foo_fc` — the same IRI as the sibling
/// SubComponent named `foo_fc`. The downgrade now allocates the FC
/// variant via `next_available_child_iri` against a shared `used` set,
/// so the variant gets bumped to `foo_fc_2`.
#[test]
fn dual_role_subcomponent_variant_avoids_sibling_iri_collision() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://lab/parent> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "parent" ;
    sbol3:type <https://identifiers.org/SBO:0000251> ;
    sbol3:hasSequence <https://lab/parent_seq> ;
    sbol3:hasInteraction <https://lab/parent/inter> ;
    sbol3:hasFeature <https://lab/parent/foo> ;
    sbol3:hasFeature <https://lab/parent/foo_fc> .

<https://lab/parent/foo> a sbol3:SubComponent ;
    sbol3:displayId "foo" ;
    sbol3:instanceOf <https://lab/target> .

<https://lab/parent/foo_fc> a sbol3:SubComponent ;
    sbol3:displayId "foo_fc" ;
    sbol3:instanceOf <https://lab/target> .

<https://lab/parent/inter> a sbol3:Interaction ;
    sbol3:displayId "inter" ;
    sbol3:type <https://identifiers.org/SBO:0000169> .

<https://lab/parent_seq> a sbol3:Sequence ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "parent_seq" ;
    sbol3:elements "ACGT" .

<https://lab/target> a sbol3:Component ;
    sbol3:hasNamespace <https://lab> ;
    sbol3:displayId "target" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _r) = document.downgrade_to_sbol2().expect("downgrade");
    let types_at = |iri: &str| -> Vec<String> {
        graph
            .triples()
            .iter()
            .filter(|t| {
                t.subject.as_iri().map(|i| i.as_str()) == Some(iri)
                    && t.predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
            })
            .filter_map(|t| t.object.as_iri().map(|i| i.as_str().to_owned()))
            .collect()
    };
    let collision_types = types_at("https://lab/parent/foo_fc");
    let has_sc = collision_types
        .iter()
        .any(|t| t == "http://sbols.org/v2#Component");
    let has_fc = collision_types
        .iter()
        .any(|t| t == "http://sbols.org/v2#FunctionalComponent");
    assert!(
        !(has_sc && has_fc),
        "sibling SubComponent named like another's `_fc` variant must not share an IRI, \
         got types at https://lab/parent/foo_fc: {collision_types:?}"
    );
    assert!(
        types_at("https://lab/parent/foo_fc_2")
            .iter()
            .any(|t| t == "http://sbols.org/v2#FunctionalComponent"),
        "FC variant of `foo` should be allocated at `foo_fc_2`"
    );
}
