//! Regenerates the SynBioDex/SBOL-Converter reference conversions under
//! `tests/fixtures/cross-impl-sbolconverter/expected/` by running the pinned
//! reference converter in Docker. For each SBOL 2 fixture it records the
//! reference SBOL 3 output (`<stem>.sbolconverter.to-sbol3.nt`); for each
//! SBOL 3 fixture it records the reference SBOL 2 output
//! (`<stem>.sbolconverter.to-sbol2.rdf`). Contributors run this locally after
//! a reference version bump; CI never invokes it (CI only diffs the committed
//! references via the `cross_impl_reference` integration test).
//!
//! Usage:
//!
//!     docker build -t sbolconverter-pinned tests/fixtures/cross-impl-sbolconverter/
//!     cargo run -p sbol-convert --bin regenerate-cross-impl-reference
//!
//! Failure modes are fail-loud: missing Docker, image not built, or a
//! non-zero exit from the container surface as an error and a non-zero exit.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

const DOCKER_IMAGE: &str = "sbolconverter-pinned";

/// SBOL 2 fixtures whose reference SBOL 3 conversion is recorded. Paths are
/// relative to `tests/fixtures/sbol2/`.
const SBOL2_FIXTURES: &[(&str, &str)] = &[
    (
        "component_definition_output",
        "SBOLTestSuite/SBOL2/ComponentDefinitionOutput.xml",
    ),
    (
        "module_definition_output",
        "SBOLTestSuite/SBOL2/ModuleDefinitionOutput.xml",
    ),
    (
        "sequence_constraint_output",
        "SBOLTestSuite/SBOL2/SequenceConstraintOutput.xml",
    ),
    (
        "repression_model",
        "SBOLTestSuite/SBOL2/RepressionModel.xml",
    ),
];

/// SBOL 3 fixtures whose reference SBOL 2 conversion is recorded. Paths are
/// relative to `tests/fixtures/sbol3/`.
const SBOL3_FIXTURES: &[(&str, &str)] = &[
    (
        "bba_f2620",
        "SBOLTestSuite/SBOL3/BBa_F2620_PoPSReceiver/BBa_F2620_PoPSReceiver.ttl",
    ),
    ("model", "SBOLTestSuite/SBOL3/entity/model/model.ttl"),
];

fn main() -> ExitCode {
    if let Err(err) = check_docker_available() {
        eprintln!("docker is required to regenerate reference conversions: {err}");
        return ExitCode::FAILURE;
    }
    if let Err(err) = check_image_present() {
        eprintln!(
            "image `{DOCKER_IMAGE}` not found: {err}\n\
             build it: docker build -t {DOCKER_IMAGE} tests/fixtures/cross-impl-sbolconverter/"
        );
        return ExitCode::FAILURE;
    }

    let expected = fixture_root().join("cross-impl-sbolconverter/expected");
    if let Err(err) = std::fs::create_dir_all(&expected) {
        eprintln!("cannot create {}: {err}", expected.display());
        return ExitCode::FAILURE;
    }

    let sbol2_root = fixture_root().join("sbol2");
    for (stem, rel) in SBOL2_FIXTURES {
        let src = sbol2_root.join(rel);
        let out = expected.join(format!("{stem}.sbolconverter.to-sbol3.nt"));
        match run_reference(&src, "to-sbol3", "nt", &out) {
            Ok(()) => eprintln!("regenerated {stem}.sbolconverter.to-sbol3.nt"),
            Err(err) => {
                eprintln!("FAILED {stem} (to-sbol3): {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    let sbol3_root = fixture_root().join("sbol3");
    for (stem, rel) in SBOL3_FIXTURES {
        let src = sbol3_root.join(rel);
        if !src.exists() {
            eprintln!(
                "skip {stem}: SBOL 3 fixture not present ({})",
                src.display()
            );
            continue;
        }
        let out = expected.join(format!("{stem}.sbolconverter.to-sbol2.rdf"));
        match run_reference(&src, "to-sbol2", "rdf", &out) {
            Ok(()) => eprintln!("regenerated {stem}.sbolconverter.to-sbol2.rdf"),
            Err(err) => {
                eprintln!("FAILED {stem} (to-sbol2): {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}

fn run_reference(
    source: &Path,
    direction: &str,
    format: &str,
    output: &Path,
) -> std::io::Result<()> {
    let parent = source
        .parent()
        .ok_or_else(|| std::io::Error::other("source has no parent"))?;
    let file_name = source
        .file_name()
        .ok_or_else(|| std::io::Error::other("source has no file name"))?;
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
            direction,
            format,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;
    if !result.status.success() {
        return Err(std::io::Error::other(format!(
            "reference container exited with status {:?}",
            result.status
        )));
    }

    let mut handle = std::fs::File::create(output)?;
    handle.write_all(&result.stdout)?;
    if !result.stdout.ends_with(b"\n") {
        handle.write_all(b"\n")?;
    }
    Ok(())
}

fn check_docker_available() -> std::io::Result<()> {
    let status = Command::new("docker").arg("info").output()?;
    if status.status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("`docker info` failed"))
    }
}

fn check_image_present() -> std::io::Result<()> {
    let status = Command::new("docker")
        .args(["image", "inspect", DOCKER_IMAGE])
        .output()?;
    if status.status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("image not present"))
    }
}

fn fixture_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("tests/fixtures");
    path
}
