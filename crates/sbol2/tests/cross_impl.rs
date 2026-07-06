//! Cross-implementation conformance vs libSBOLj (the SBOL 2 Java
//! implementation).
//!
//! For each `*.libSBOLj.expected.rdf` committed under
//! `tests/fixtures/cross-impl-sbol2/`, parse the corresponding vendored
//! SBOL 2 source fixture with sbol-rs, parse the libSBOLj reference, and
//! compare the two normalized triple sets. SBOL 2 is exchanged as
//! RDF/XML, so RDF/XML is the only serialization compared. The
//! reference files are generated locally by running libSBOLj in a
//! pinned Docker image — see
//! `tests/fixtures/cross-impl-sbol2/README.md` and
//! `crates/sbol2/src/bin/regenerate-cross-impl-sbol2-expectations.rs`.
//!
//! A divergence either gets fixed in sbol-rs or allowlisted with a
//! written rationale in `tests/fixtures/cross-impl-sbol2/allowlist.txt`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

mod common;

/// Splits an expected-output filename of the form
/// `<stem>.libSBOLj.expected.<ext>` into its stem and `RdfFormat`.
/// Returns `None` for any file that doesn't fit the pattern (including
/// the harness's allowlist / README sibling files).
fn parse_reference_name(reference_name: &str) -> Option<(&str, sbol2::RdfFormat)> {
    let (stem_with_marker, ext) = reference_name.rsplit_once('.')?;
    let stem = stem_with_marker.strip_suffix(".libSBOLj.expected")?;
    let format = sbol2::RdfFormat::from_extension(ext)?;
    Some((stem, format))
}

/// Maps a reference-file stem to the vendored SBOL 2 source fixture path
/// it was generated from, relative to `tests/fixtures/sbol2`. Extend
/// this table when new fixtures gain reference outputs; it mirrors the
/// `FIXTURES` table in the regenerate binary.
fn source_fixture_for_stem(stem: &str) -> Option<&'static str> {
    Some(match stem {
        "bba_i0462" => "real/BBa_I0462.xml",
        "cd_sa_cut_example" => "real/CD_SA_Cut_Example.xml",
        "cd_sa_range_example" => "real/CD_SA_Range_Example.xml",
        "component_definition_output" => "real/ComponentDefinitionOutput.xml",
        "component_definition_output_gl_cd_sa_comp" => {
            "real/ComponentDefinitionOutput_gl_cd_sa_comp.xml"
        }
        "implementation_example" => "real/implementation_example.xml",
        "module_definition_output" => "real/ModuleDefinitionOutput.xml",
        "module_definition_output_int_md_ann" => "real/ModuleDefinitionOutput_int_md_ann.xml",
        "repression_model" => "real/RepressionModel.xml",
        "sequence_constraint_output" => "real/SequenceConstraintOutput.xml",
        _ => return None,
    })
}

fn cross_impl_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root reachable")
        .join("tests/fixtures/cross-impl-sbol2")
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
fn sbol_rs_output_matches_libsbolj_reference() {
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

        let source_path = common::fixture_root().join(source);
        let source_text = fs::read_to_string(&source_path).unwrap_or_else(|error| {
            panic!(
                "source fixture {source} for cross-impl reference {name} could not be read \
                 from {}: {error}",
                source_path.display()
            )
        });

        // SBOL 2 source fixtures are RDF/XML.
        let sbol_doc = sbol2::Document::read(&source_text, sbol2::RdfFormat::RdfXml)
            .unwrap_or_else(|error| panic!("{source}: sbol-rs failed to parse: {error}"));
        // Round-trip through the same format libSBOLj emitted in, so
        // format-specific bugs show up in the comparison rather than
        // being masked by an intermediate hop.
        let sbol_serialized = sbol_doc.write(format).unwrap_or_else(|error| {
            panic!("{source}: sbol-rs failed to serialize as {format}: {error}")
        });
        let sbol_reparsed =
            sbol2::Document::read(&sbol_serialized, format).unwrap_or_else(|error| {
                panic!("{source}: sbol-rs failed to reparse its own {format} output: {error}")
            });

        let reference_path = entry.path();
        let reference_text = fs::read_to_string(&reference_path)
            .unwrap_or_else(|error| panic!("{name}: could not read reference: {error}"));
        let reference_doc =
            sbol2::Document::read(&reference_text, format).unwrap_or_else(|error| {
                panic!("{name}: libSBOLj {format} reference did not parse: {error}")
            });

        let sbol_triples: BTreeSet<_> = sbol_reparsed
            .rdf_graph()
            .normalized_triples()
            .into_iter()
            .collect();
        let ref_triples: BTreeSet<_> = reference_doc
            .rdf_graph()
            .normalized_triples()
            .into_iter()
            .collect();
        assert_eq!(
            sbol_triples, ref_triples,
            "{source} produces a different normalized triple set than libSBOLj in {format} \
             (reference: {name}). To allowlist a known-compliant divergence, add {name} to \
             tests/fixtures/cross-impl-sbol2/allowlist.txt with a rationale."
        );
        compared += 1;
    }

    // It's fine for `compared` to be zero — no committed references
    // means the harness is wired but inactive (the Docker image and
    // regenerate binary produce the references). We log the count so
    // contributors can spot the empty state.
    eprintln!("cross-impl-sbol2: compared {compared} fixture(s) against libSBOLj reference output");
}
