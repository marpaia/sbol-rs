//! Regenerates `tests/fixtures/cross-impl-pysbol3/*.pySBOL3.expected.{ttl,rdf,jsonld,nt}`
//! by running pySBOL3 inside the pinned Docker image. For each source
//! fixture every output format is regenerated. Contributors run this
//! locally after a pySBOL3 version bump; CI never invokes it (CI only
//! diffs the committed references via the `cross_impl_pysbol3`
//! integration test).
//!
//! Usage:
//!
//!     docker build -t pysbol3-pinned tests/fixtures/cross-impl-pysbol3/
//!     cargo run -p sbol --bin regenerate-cross-impl-pysbol3-expectations
//!
//! Prerequisites:
//!   - Docker daemon reachable on the host
//!   - The SBOLTestSuite fixture cache populated (run the
//!     `sbol3_fixtures` integration test once if not)
//!
//! Failure modes are fail-loud — missing Docker, image not built, or
//! a non-zero exit from the container all surface as an error message
//! and a non-zero exit.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use sbol3::RdfFormat;

const FIXTURES: &[(&str, &str)] = &[
    (
        "activity",
        "SBOLTestSuite/SBOL3/provenance_entity/activity/activity.ttl",
    ),
    (
        "agent",
        "SBOLTestSuite/SBOL3/provenance_entity/agent/agent.ttl",
    ),
    (
        "annotation",
        "SBOLTestSuite/SBOL3/entity/annotation/annotation.ttl",
    ),
    (
        "attachment",
        "SBOLTestSuite/SBOL3/entity/attachment/attachment.ttl",
    ),
    (
        "bba_f2620_popsreceiver",
        "SBOLTestSuite/SBOL3/BBa_F2620_PoPSReceiver/BBa_F2620_PoPSReceiver.ttl",
    ),
    (
        "collection",
        "SBOLTestSuite/SBOL3/entity/collection/collection.ttl",
    ),
    (
        "combinatorialderivation",
        "SBOLTestSuite/SBOL3/entity/combinatorialderivation/combinatorialderivation.ttl",
    ),
    (
        "combine2020",
        "SBOLTestSuite/SBOL3/combine2020/combine2020.ttl",
    ),
    (
        "component",
        "SBOLTestSuite/SBOL3/entity/component/component.ttl",
    ),
    (
        "component_urn_uri",
        "SBOLTestSuite/SBOL3/entity/component_urn_uri/component_urn_uri.ttl",
    ),
    (
        "componentreference",
        "SBOLTestSuite/SBOL3/entity/componentreference/componentreference.ttl",
    ),
    (
        "constraint",
        "SBOLTestSuite/SBOL3/entity/constraint/constraint.ttl",
    ),
    ("cut", "SBOLTestSuite/SBOL3/entity/cut/cut.ttl"),
    (
        "experiment",
        "SBOLTestSuite/SBOL3/entity/experiment/experiment.ttl",
    ),
    ("experimental_data", "local/experimental_data.ttl"),
    (
        "externallydefined",
        "SBOLTestSuite/SBOL3/entity/externallydefined/externallydefined.ttl",
    ),
    (
        "implementation",
        "SBOLTestSuite/SBOL3/entity/implementation/implementation.ttl",
    ),
    (
        "interaction",
        "SBOLTestSuite/SBOL3/entity/interaction/interaction.ttl",
    ),
    (
        "interface",
        "SBOLTestSuite/SBOL3/entity/interface/interface.ttl",
    ),
    (
        "localsubcomponent",
        "SBOLTestSuite/SBOL3/entity/localsubcomponent/localsubcomponent.ttl",
    ),
    (
        "measurement",
        "SBOLTestSuite/SBOL3/measurement_entity/measurement/measurement.ttl",
    ),
    (
        "measurement_using_units_from_om",
        "SBOLTestSuite/SBOL3/measurement_entity/measurement_using_units_From_OM/measurement_using_units_From_OM.ttl",
    ),
    ("model", "SBOLTestSuite/SBOL3/entity/model/model.ttl"),
    (
        "multicellular",
        "SBOLTestSuite/SBOL3/multicellular/multicellular.ttl",
    ),
    (
        "multicellular_simple",
        "SBOLTestSuite/SBOL3/multicellular_simple/multicellular_simple.ttl",
    ),
    (
        "participation",
        "SBOLTestSuite/SBOL3/entity/participation/participation.ttl",
    ),
    (
        "plan",
        "SBOLTestSuite/SBOL3/provenance_entity/plan/plan.ttl",
    ),
    ("range", "SBOLTestSuite/SBOL3/entity/range/range.ttl"),
    (
        "sequencefeature",
        "SBOLTestSuite/SBOL3/entity/sequencefeature/sequencefeature.ttl",
    ),
    (
        "subcomponent",
        "SBOLTestSuite/SBOL3/entity/subcomponent/subcomponent.ttl",
    ),
    (
        "toggle_switch",
        "SBOLTestSuite/SBOL3/toggle_switch/toggle_switch.ttl",
    ),
    (
        "toggle_switch_v2",
        "SBOLTestSuite/SBOL3/toggle_switch_v2/toggle_switch_v2.ttl",
    ),
    ("variable_feature", "local/variable_feature.ttl"),
];

const DOCKER_IMAGE: &str = "pysbol3-pinned";

fn main() -> ExitCode {
    let workspace = workspace_root();
    let cross_impl_dir = workspace.join("tests/fixtures/cross-impl-pysbol3");
    let sboltest_root = workspace.join("tests/fixtures/sbol3");

    if let Err(error) = check_docker_available() {
        eprintln!("docker unavailable: {error}");
        return ExitCode::FAILURE;
    }
    if let Err(error) = check_image_present(DOCKER_IMAGE) {
        eprintln!(
            "docker image `{DOCKER_IMAGE}` missing: {error}\n\
             Build it first:\n\
                 docker build -t {DOCKER_IMAGE} {}",
            cross_impl_dir.display()
        );
        return ExitCode::FAILURE;
    }

    let mut failures = 0usize;
    for (stem, source) in FIXTURES {
        let source_path = sboltest_root.join(source);

        if !source_path.is_file() {
            eprintln!(
                "skipping {stem}: source fixture missing at {} \
                 (run the sbol3_fixtures test to populate the cache)",
                source_path.display()
            );
            failures += 1;
            continue;
        }

        for &format in RdfFormat::ALL {
            let output_path =
                cross_impl_dir.join(format!("{stem}.pySBOL3.expected.{}", format.extension()));
            match run_pysbol3(&source_path, &output_path, format) {
                Ok(()) => {
                    println!("wrote {}", output_path.display());
                }
                Err(error) => {
                    eprintln!("failed to regenerate {stem} ({format}): {error}");
                    failures += 1;
                }
            }
        }
    }

    if failures > 0 {
        eprintln!("{failures} fixture/format combination(s) failed to regenerate");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn pysbol3_format_name(format: RdfFormat) -> &'static str {
    match format {
        RdfFormat::Turtle => "turtle",
        RdfFormat::RdfXml => "rdfxml",
        RdfFormat::JsonLd => "jsonld",
        RdfFormat::NTriples => "ntriples",
        _ => panic!("regenerate harness has no pySBOL3 mapping for format {format}"),
    }
}

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(manifest)
}

fn check_docker_available() -> std::io::Result<()> {
    Command::new("docker").arg("info").output().map(|_| ())
}

fn check_image_present(image: &str) -> std::io::Result<()> {
    let output = Command::new("docker")
        .args(["image", "inspect", image])
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "`docker image inspect {image}` exited with status {:?}",
            output.status
        )));
    }
    Ok(())
}

fn run_pysbol3(source: &Path, output: &Path, format: RdfFormat) -> std::io::Result<()> {
    let parent = source.parent().ok_or_else(|| {
        std::io::Error::other(format!("source path has no parent: {}", source.display()))
    })?;
    let file_name = source
        .file_name()
        .ok_or_else(|| std::io::Error::other("source path has no file name"))?;

    let mount = format!("{}:/work:ro", parent.display());
    let container_input = format!("/work/{}", Path::new(file_name).display());

    let result = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-v",
            &mount,
            DOCKER_IMAGE,
            &container_input,
            pysbol3_format_name(format),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;
    if !result.status.success() {
        return Err(std::io::Error::other(format!(
            "pySBOL3 container exited with status {:?}",
            result.status
        )));
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut handle = std::fs::File::create(output)?;
    handle.write_all(&result.stdout)?;
    if !result.stdout.ends_with(b"\n") {
        handle.write_all(b"\n")?;
    }
    Ok(())
}
