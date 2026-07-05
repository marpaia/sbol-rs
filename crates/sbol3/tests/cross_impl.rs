//! Cross-implementation conformance vs libSBOLj3.
//!
//! For each `*.libSBOLj3.expected.{ttl,rdf,jsonld,nt}` committed under
//! `tests/fixtures/cross-impl/`, parse the corresponding SBOLTestSuite
//! source fixture with sbol-rs, parse the libSBOLj3 reference in its
//! own format, and compare the two normalized triple sets. The
//! reference files are generated locally by running libSBOLj3 in a
//! pinned Docker image — see `docs/testing.md` and
//! `crates/sbol/src/bin/regenerate-cross-impl-expectations.rs`.
//!
//! A divergence either gets fixed in sbol-rs or allowlisted with a
//! written rationale in `tests/fixtures/cross-impl/allowlist.txt`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

mod common;

/// Splits an expected-output filename of the form
/// `<stem>.libSBOLj3.expected.<ext>` into its stem and `RdfFormat`.
/// Returns `None` for any file that doesn't fit the pattern (including
/// the harness's allowlist/Dockerfile/README sibling files).
fn parse_reference_name(reference_name: &str) -> Option<(&str, sbol3::RdfFormat)> {
    let (stem_with_marker, ext) = reference_name.rsplit_once('.')?;
    let stem = stem_with_marker.strip_suffix(".libSBOLj3.expected")?;
    let format = sbol3::RdfFormat::from_extension(ext)?;
    Some((stem, format))
}

/// Maps a reference-file stem to the SBOLTestSuite source fixture path
/// it was generated from. Extend this table when new fixtures gain
/// reference outputs.
fn source_fixture_for_stem(stem: &str) -> Option<&'static str> {
    Some(match stem {
        "activity" => "SBOLTestSuite/SBOL3/provenance_entity/activity/activity.ttl",
        "agent" => "SBOLTestSuite/SBOL3/provenance_entity/agent/agent.ttl",
        "annotation" => "SBOLTestSuite/SBOL3/entity/annotation/annotation.ttl",
        "attachment" => "SBOLTestSuite/SBOL3/entity/attachment/attachment.ttl",
        "bba_f2620_popsreceiver" => {
            "SBOLTestSuite/SBOL3/BBa_F2620_PoPSReceiver/BBa_F2620_PoPSReceiver.ttl"
        }
        "collection" => "SBOLTestSuite/SBOL3/entity/collection/collection.ttl",
        "combinatorialderivation" => {
            "SBOLTestSuite/SBOL3/entity/combinatorialderivation/combinatorialderivation.ttl"
        }
        "combine2020" => "SBOLTestSuite/SBOL3/combine2020/combine2020.ttl",
        "component" => "SBOLTestSuite/SBOL3/entity/component/component.ttl",
        "component_urn_uri" => "SBOLTestSuite/SBOL3/entity/component_urn_uri/component_urn_uri.ttl",
        "componentreference" => {
            "SBOLTestSuite/SBOL3/entity/componentreference/componentreference.ttl"
        }
        "constraint" => "SBOLTestSuite/SBOL3/entity/constraint/constraint.ttl",
        "cut" => "SBOLTestSuite/SBOL3/entity/cut/cut.ttl",
        "experiment" => "SBOLTestSuite/SBOL3/entity/experiment/experiment.ttl",
        "experimental_data" => "local/experimental_data.ttl",
        "externallydefined" => "SBOLTestSuite/SBOL3/entity/externallydefined/externallydefined.ttl",
        "implementation" => "SBOLTestSuite/SBOL3/entity/implementation/implementation.ttl",
        "interaction" => "SBOLTestSuite/SBOL3/entity/interaction/interaction.ttl",
        "interface" => "SBOLTestSuite/SBOL3/entity/interface/interface.ttl",
        "localsubcomponent" => "SBOLTestSuite/SBOL3/entity/localsubcomponent/localsubcomponent.ttl",
        "measurement" => "SBOLTestSuite/SBOL3/measurement_entity/measurement/measurement.ttl",
        "measurement_using_units_from_om" => {
            "SBOLTestSuite/SBOL3/measurement_entity/measurement_using_units_From_OM/measurement_using_units_From_OM.ttl"
        }
        "model" => "SBOLTestSuite/SBOL3/entity/model/model.ttl",
        "multicellular" => "SBOLTestSuite/SBOL3/multicellular/multicellular.ttl",
        "multicellular_simple" => {
            "SBOLTestSuite/SBOL3/multicellular_simple/multicellular_simple.ttl"
        }
        "participation" => "SBOLTestSuite/SBOL3/entity/participation/participation.ttl",
        "plan" => "SBOLTestSuite/SBOL3/provenance_entity/plan/plan.ttl",
        "range" => "SBOLTestSuite/SBOL3/entity/range/range.ttl",
        "sequencefeature" => "SBOLTestSuite/SBOL3/entity/sequencefeature/sequencefeature.ttl",
        "subcomponent" => "SBOLTestSuite/SBOL3/entity/subcomponent/subcomponent.ttl",
        "toggle_switch" => "SBOLTestSuite/SBOL3/toggle_switch/toggle_switch.ttl",
        "toggle_switch_v2" => "SBOLTestSuite/SBOL3/toggle_switch_v2/toggle_switch_v2.ttl",
        "variable_feature" => "local/variable_feature.ttl",
        _ => return None,
    })
}

fn cross_impl_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root reachable")
        .join("tests/fixtures/cross-impl")
}

fn sboltestsuite_root() -> PathBuf {
    common::fixture_root().to_path_buf()
}

fn load_allowlist() -> BTreeSet<String> {
    let path = cross_impl_root().join("allowlist.txt");
    fs::read_to_string(&path)
        .map(|content| {
            content
                .lines()
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(|line| line.trim().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn sbol_rs_output_matches_libsbolj3_reference() {
    let root = cross_impl_root();
    let Ok(entries) = fs::read_dir(&root) else {
        return;
    };
    let allowlist = load_allowlist();

    let mut compared = 0usize;
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        let Some((stem, format)) = parse_reference_name(name) else {
            continue;
        };
        let Some(source) = source_fixture_for_stem(stem) else {
            panic!(
                "{name} has no source-fixture mapping in cross_impl.rs::source_fixture_for_stem; \
                 add an entry to the table when committing the reference"
            );
        };
        if allowlist.contains(name) {
            continue;
        }

        let source_path = sboltestsuite_root().join(source);
        let source_text = fs::read_to_string(&source_path).unwrap_or_else(|error| {
            panic!(
                "source fixture {source} for cross-impl reference {name} could not be read \
                 from {}: {error} (run the sbol3_fixtures test once to populate the cache)",
                source_path.display()
            )
        });

        let sbol_doc = sbol3::Document::read_turtle(&source_text)
            .unwrap_or_else(|error| panic!("{source}: sbol-rs failed to parse: {error}"));
        // Round-trip through the same format libSBOLj3 emitted in,
        // so format-specific bugs show up in the comparison rather than
        // being masked by an intermediate Turtle hop.
        let sbol_serialized = sbol_doc.write(format).unwrap_or_else(|error| {
            panic!("{source}: sbol-rs failed to serialize as {format}: {error}")
        });
        let sbol_reparsed =
            sbol3::Document::read(&sbol_serialized, format).unwrap_or_else(|error| {
                panic!("{source}: sbol-rs failed to reparse its own {format} output: {error}")
            });

        let reference_path = entry.path();
        let reference_doc = sbol3::Document::read_path(&reference_path).unwrap_or_else(|error| {
            panic!("{name}: libSBOLj3 {format} reference did not parse: {error}")
        });

        let sbol_triples = sbol_reparsed.rdf_graph().normalized_triples();
        let ref_triples = reference_doc.rdf_graph().normalized_triples();
        assert_eq!(
            sbol_triples, ref_triples,
            "{source} produces a different normalized triple set than libSBOLj3 in {format} \
             (reference: {name}). To allowlist a known-compliant divergence, add {name} to \
             tests/fixtures/cross-impl/allowlist.txt with a rationale."
        );
        compared += 1;
    }

    // It's fine for `compared` to be zero — no committed references
    // means the harness is wired but inactive. We log the count so
    // contributors can spot the empty state.
    eprintln!("cross-impl: compared {compared} fixture(s) against libSBOLj3 reference output");
}
