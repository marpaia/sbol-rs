//! Regenerates `tests/fixtures/cross-impl-sbol2/*.libSBOLj.expected.rdf`
//! by running libSBOLj (the SBOL 2 Java implementation) inside the
//! pinned Docker image. Contributors run this locally after a libSBOLj
//! version bump; CI never invokes it (CI only diffs the committed
//! references via the `cross_impl` integration test).
//!
//! SBOL 2 is exchanged as RDF/XML; libSBOLj's JSON and Turtle writers
//! are not part of the standard interchange, so the harness compares
//! RDF/XML output only.
//!
//! Usage:
//!
//!     docker build -t libsbolj-pinned benches/cross-impl/libsbolj/
//!     cargo run -p sbol2 --bin regenerate-cross-impl-sbol2-expectations
//!
//! Prerequisites:
//!   - Docker daemon reachable on the host
//!
//! Failure modes are fail-loud: missing Docker, image not built, or a
//! non-zero exit from the container all surface as an error message and
//! a non-zero exit.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

// Vendored SBOL 2 RDF/XML fixtures, relative to `tests/fixtures/sbol2`.
// These are the same fixtures the sbol2 round-trip test exercises, so
// sbol-rs already round-trips them; the cross-impl test additionally
// asserts triple-set agreement with libSBOLj.
const FIXTURES: &[(&str, &str)] = &[
    ("bba_i0462", "real/BBa_I0462.xml"),
    ("cd_sa_cut_example", "real/CD_SA_Cut_Example.xml"),
    ("cd_sa_range_example", "real/CD_SA_Range_Example.xml"),
    ("component_definition_output", "real/ComponentDefinitionOutput.xml"),
    (
        "component_definition_output_gl_cd_sa_comp",
        "real/ComponentDefinitionOutput_gl_cd_sa_comp.xml",
    ),
    ("implementation_example", "real/implementation_example.xml"),
    ("module_definition_output", "real/ModuleDefinitionOutput.xml"),
    (
        "module_definition_output_int_md_ann",
        "real/ModuleDefinitionOutput_int_md_ann.xml",
    ),
    ("repression_model", "real/RepressionModel.xml"),
    ("sequence_constraint_output", "real/SequenceConstraintOutput.xml"),
];

const DOCKER_IMAGE: &str = "libsbolj-pinned";

fn main() -> ExitCode {
    let workspace = workspace_root();
    let cross_impl_dir = workspace.join("tests/fixtures/cross-impl-sbol2");
    let fixture_root = workspace.join("tests/fixtures/sbol2");

    if let Err(error) = check_docker_available() {
        eprintln!("docker unavailable: {error}");
        return ExitCode::FAILURE;
    }
    if let Err(error) = check_image_present(DOCKER_IMAGE) {
        eprintln!(
            "docker image `{DOCKER_IMAGE}` missing: {error}\n\
             Build it first:\n\
                 docker build -t {DOCKER_IMAGE} {}",
            workspace.join("benches/cross-impl/libsbolj").display()
        );
        return ExitCode::FAILURE;
    }

    let mut failures = 0usize;
    for (stem, source) in FIXTURES {
        let source_path = fixture_root.join(source);
        if !source_path.is_file() {
            eprintln!("skipping {stem}: source fixture missing at {}", source_path.display());
            failures += 1;
            continue;
        }

        let output_path = cross_impl_dir.join(format!("{stem}.libSBOLj.expected.rdf"));
        match run_libsbolj(&source_path, &output_path) {
            Ok(()) => println!("wrote {}", output_path.display()),
            Err(error) => {
                eprintln!("failed to regenerate {stem}: {error}");
                failures += 1;
            }
        }
    }

    if failures > 0 {
        eprintln!("{failures} fixture(s) failed to regenerate");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
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

fn run_libsbolj(source: &Path, output: &Path) -> std::io::Result<()> {
    let parent = source.parent().ok_or_else(|| {
        std::io::Error::other(format!("source path has no parent: {}", source.display()))
    })?;
    let file_name = source
        .file_name()
        .ok_or_else(|| std::io::Error::other("source path has no file name"))?;

    // Mount the fixture's parent dir read-only inside the container at
    // /work, then run RoundTrip with the matching basename.
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
            "rdfxml",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;
    if !result.status.success() {
        return Err(std::io::Error::other(format!(
            "libSBOLj container exited with status {:?}",
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
