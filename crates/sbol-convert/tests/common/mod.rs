//! Shared test helpers. Currently exposes the SBOLTestSuite fixture-cache
//! bootstrap so multiple integration-test binaries can depend on the same
//! on-disk fixtures without racing each other during `cargo test --workspace`.
#![allow(dead_code)]

pub mod corpus;
pub mod downgrade;
pub mod upgrade;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

pub const SBOLTESTSUITE_COMMIT: &str = "0044284331b2f915a6e4b9d50e1cbf3ea2f62dcd";
pub const CACHE_SENTINEL: &str = ".sbol-rs-fixture-cache-version";

const SBOLTESTSUITE_ARCHIVE_URL: &str = "https://github.com/SynBioDex/SBOLTestSuite/archive/0044284331b2f915a6e4b9d50e1cbf3ea2f62dcd.tar.gz";
const DEFAULT_FIXTURE_ROOT: &str = "../../tests/fixtures/sbol3";
const LOCK_DIR: &str = ".sbol-rs-fixture-lock";
const LOCK_WAIT_TIMEOUT: Duration = Duration::from_secs(300);
const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(500);

const LOCAL_EXPERIMENTAL_DATA: &str = r#"BASE   <https://example.org/sbol-rs/fixtures/>
PREFIX :      <https://example.org/sbol-rs/fixtures/>
PREFIX EDAM:  <https://identifiers.org/edam:>
PREFIX sbol:  <http://sbols.org/v3#>

:sample_measurements
        a                  sbol:ExperimentalData;
        sbol:displayId     "sample_measurements";
        sbol:hasAttachment :sample_measurements_csv;
        sbol:hasNamespace  <https://example.org/sbol-rs/fixtures> .

:sample_measurements_csv
        a                  sbol:Attachment;
        sbol:displayId     "sample_measurements_csv";
        sbol:format        EDAM:format_3752;
        sbol:hasNamespace  <https://example.org/sbol-rs/fixtures>;
        sbol:source        <https://example.org/sbol-rs/fixtures/sample_measurements.csv> .

:characterization
        a                  sbol:Experiment;
        sbol:displayId     "characterization";
        sbol:hasNamespace  <https://example.org/sbol-rs/fixtures>;
        sbol:member        :sample_measurements .
"#;

const LOCAL_VARIABLE_FEATURE: &str = r#"BASE   <https://example.org/sbol-rs/fixtures/>
PREFIX :      <https://example.org/sbol-rs/fixtures/>
PREFIX SBO:   <https://identifiers.org/SBO:>
PREFIX SO:    <https://identifiers.org/SO:>
PREFIX sbol:  <http://sbols.org/v3#>

:promoter_a
        a                  sbol:Component;
        sbol:displayId     "promoter_a";
        sbol:hasNamespace  <https://example.org/sbol-rs/fixtures>;
        sbol:role          SO:0000167;
        sbol:type          SBO:0000251 .

:promoter_b
        a                  sbol:Component;
        sbol:displayId     "promoter_b";
        sbol:hasNamespace  <https://example.org/sbol-rs/fixtures>;
        sbol:role          SO:0000167;
        sbol:type          SBO:0000251 .

:promoter_library
        a                  sbol:Collection;
        sbol:displayId     "promoter_library";
        sbol:hasNamespace  <https://example.org/sbol-rs/fixtures>;
        sbol:member        :promoter_a , :promoter_b .

:template_component
        a                  sbol:Component;
        sbol:displayId     "template_component";
        sbol:hasFeature    <template_component/promoter_slot>;
        sbol:hasNamespace  <https://example.org/sbol-rs/fixtures>;
        sbol:role          SO:0000167;
        sbol:type          SBO:0000251 .

<template_component/promoter_slot>
        a                sbol:SubComponent;
        sbol:displayId   "promoter_slot";
        sbol:instanceOf  :promoter_a .

:promoter_derivation
        a                       sbol:CombinatorialDerivation;
        sbol:displayId          "promoter_derivation";
        sbol:hasNamespace       <https://example.org/sbol-rs/fixtures>;
        sbol:hasVariableFeature <promoter_derivation/promoter_slot>;
        sbol:strategy           sbol:enumerate;
        sbol:template           :template_component .

<promoter_derivation/promoter_slot>
        a                      sbol:VariableFeature;
        sbol:cardinality       sbol:one;
        sbol:displayId         "promoter_slot";
        sbol:variable          <template_component/promoter_slot>;
        sbol:variant           :promoter_a , :promoter_b;
        sbol:variantCollection :promoter_library .
"#;

static FIXTURE_ROOT: OnceLock<PathBuf> = OnceLock::new();

pub fn fixture_root() -> &'static Path {
    FIXTURE_ROOT
        .get_or_init(|| {
            let root = env::var_os("SBOL_FIXTURE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    Path::new(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_FIXTURE_ROOT)
                });

            ensure_fixture_cache(&root).unwrap_or_else(|error| {
                panic!(
                    "failed to prepare SBOL 3 fixture cache at {}: {error}",
                    root.display()
                )
            });

            root
        })
        .as_path()
}

fn ensure_fixture_cache(root: &Path) -> io::Result<()> {
    fs::create_dir_all(root)?;
    write_local_fixtures(root)?;

    if sentinel_matches(root) {
        return Ok(());
    }

    // Cross-process mutex: cargo runs integration-test binaries in
    // parallel, and the first one to call `fixture_root()` must download
    // the SBOLTestSuite archive while the others wait. `fs::create_dir`
    // is atomic on POSIX and Windows, so it serves as a simple lock.
    let lock = root.join(LOCK_DIR);
    if fs::create_dir(&lock).is_ok() {
        let result = (|| -> io::Result<()> {
            if sentinel_matches(root) {
                return Ok(());
            }
            fetch_sboltestsuite_fixtures(root)?;
            write_local_fixtures(root)?;
            fs::write(root.join(CACHE_SENTINEL), SBOLTESTSUITE_COMMIT)
        })();
        let _ = fs::remove_dir(&lock);
        return result;
    }

    let deadline = Instant::now() + LOCK_WAIT_TIMEOUT;
    while !sentinel_matches(root) {
        if Instant::now() > deadline {
            return Err(io::Error::other(
                "timed out waiting for another test binary to populate the fixture cache",
            ));
        }
        thread::sleep(LOCK_POLL_INTERVAL);
    }
    Ok(())
}

fn sentinel_matches(root: &Path) -> bool {
    fs::read_to_string(root.join(CACHE_SENTINEL))
        .map(|content| content.trim() == SBOLTESTSUITE_COMMIT)
        .unwrap_or(false)
}

fn write_local_fixtures(root: &Path) -> io::Result<()> {
    let local_root = root.join("local");
    fs::create_dir_all(&local_root)?;
    fs::write(
        local_root.join("experimental_data.ttl"),
        LOCAL_EXPERIMENTAL_DATA,
    )?;
    fs::write(
        local_root.join("variable_feature.ttl"),
        LOCAL_VARIABLE_FEATURE,
    )?;
    Ok(())
}

fn fetch_sboltestsuite_fixtures(root: &Path) -> io::Result<()> {
    let download_root = root.join(".download");
    let extract_root = download_root.join("extract");
    let archive = download_root.join("SBOLTestSuite.tar.gz");

    if extract_root.exists() {
        fs::remove_dir_all(&extract_root)?;
    }
    fs::create_dir_all(&extract_root)?;

    run_command(
        "curl",
        &[
            "-fsSL",
            SBOLTESTSUITE_ARCHIVE_URL,
            "-o",
            archive.to_str().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "non-UTF-8 archive path")
            })?,
        ],
    )?;
    run_command(
        "tar",
        &[
            "-xzf",
            archive.to_str().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "non-UTF-8 archive path")
            })?,
            "-C",
            extract_root.to_str().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "non-UTF-8 extraction path")
            })?,
        ],
    )?;

    let source_root = extracted_source_root(&extract_root)?;
    let sbol3_source = source_root.join("SBOL3");
    let sbol3_target = root.join("SBOLTestSuite/SBOL3");
    copy_turtle_files(&sbol3_source, &sbol3_target, &sbol3_source)?;

    fs::remove_dir_all(extract_root)?;
    Ok(())
}

fn extracted_source_root(extract_root: &Path) -> io::Result<PathBuf> {
    for entry in fs::read_dir(extract_root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            return Ok(entry.path());
        }
    }

    Err(io::Error::other(
        "SBOLTestSuite archive did not contain a source directory",
    ))
}

fn copy_turtle_files(source_dir: &Path, target_dir: &Path, source_root: &Path) -> io::Result<()> {
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();

        if entry.file_type()?.is_dir() {
            copy_turtle_files(&path, target_dir, source_root)?;
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("ttl") {
            continue;
        }

        let relative_path = path.strip_prefix(source_root).map_err(io::Error::other)?;
        let target_path = target_dir.join(relative_path);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(path, target_path)?;
    }

    Ok(())
}

fn run_command(program: &str, args: &[&str]) -> io::Result<()> {
    let output = Command::new(program).args(args).output()?;
    if output.status.success() {
        return Ok(());
    }

    Err(io::Error::other(format!(
        "{program} failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )))
}
