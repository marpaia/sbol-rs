//! Bootstraps the SBOLTestSuite SBOL 2 conformance corpus for the
//! migration round-trip tests, mirroring the extractor in
//! `crates/sbol2/tests/common/mod.rs`. The four SBOL2* subdirectories are
//! materialized into the repo-root `tests/fixtures/sbol2/SBOLTestSuite`,
//! reusing the archive the SBOL 3 tests already download.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

pub const SBOLTESTSUITE_COMMIT: &str = "0044284331b2f915a6e4b9d50e1cbf3ea2f62dcd";
pub const CACHE_SENTINEL: &str = ".sbol2-fixture-cache-version";

const SBOLTESTSUITE_ARCHIVE_URL: &str = "https://github.com/SynBioDex/SBOLTestSuite/archive/0044284331b2f915a6e4b9d50e1cbf3ea2f62dcd.tar.gz";
const SHARED_ARCHIVE: &str = "../../tests/fixtures/sbol3/.download/SBOLTestSuite.tar.gz";
const CORPUS_FIXTURE_ROOT: &str = "../../tests/fixtures/sbol2";
const LOCK_DIR: &str = ".sbol2-fixture-lock";
const LOCK_WAIT_TIMEOUT: Duration = Duration::from_secs(300);
const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(500);

/// The SBOL 2 corpus subdirectories the round-trip harness iterates.
pub const CORPUS_DIRS: &[&str] = &["SBOL2", "SBOL2_bp", "SBOL2_ic", "SBOL2_nc"];

/// The subdirectories materialized into the shared cache. This set — and
/// the [`CACHE_SENTINEL`] name — match the `sbol2` crate's extractor so the
/// two crates share one cache: whichever test binary populates it first
/// produces a directory tree valid for both. Extracting a strict subset
/// here would let a stale sentinel hide `InvalidFiles` from the `sbol2`
/// crate's tests.
const EXTRACT_DIRS: &[&str] = &["SBOL2", "SBOL2_bp", "SBOL2_ic", "SBOL2_nc", "InvalidFiles"];

static CORPUS_ROOT: OnceLock<PathBuf> = OnceLock::new();

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS_FIXTURE_ROOT)
}

/// The `SBOLTestSuite` directory, bootstrapped on first use.
pub fn corpus_root() -> &'static Path {
    CORPUS_ROOT
        .get_or_init(|| {
            let root = fixture_root();
            ensure_corpus_cache(&root).unwrap_or_else(|error| {
                panic!(
                    "failed to prepare SBOL 2 conformance corpus at {}: {error}",
                    root.display()
                )
            });
            root.join("SBOLTestSuite")
        })
        .as_path()
}

/// The `.xml` corpus files in a subdirectory, sorted by name.
pub fn xml_files(sub: &str) -> Vec<PathBuf> {
    let dir = corpus_root().join(sub);
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("xml"))
        .collect();
    files.sort();
    files
}

fn ensure_corpus_cache(root: &Path) -> io::Result<()> {
    fs::create_dir_all(root)?;
    if sentinel_matches(root) {
        return Ok(());
    }

    // Cross-process mutex: cargo runs integration-test binaries in parallel.
    // `fs::create_dir` is atomic, so it serves as a simple lock.
    let lock = root.join(LOCK_DIR);
    if fs::create_dir(&lock).is_ok() {
        let result = (|| -> io::Result<()> {
            if sentinel_matches(root) {
                return Ok(());
            }
            fetch_fixtures(root)?;
            fs::write(root.join(CACHE_SENTINEL), SBOLTESTSUITE_COMMIT)
        })();
        let _ = fs::remove_dir(&lock);
        return result;
    }

    let deadline = Instant::now() + LOCK_WAIT_TIMEOUT;
    while !sentinel_matches(root) {
        if Instant::now() > deadline {
            return Err(io::Error::other(
                "timed out waiting for another test binary to populate the corpus cache",
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

fn fetch_fixtures(root: &Path) -> io::Result<()> {
    let download_root = root.join(".download");
    let extract_root = download_root.join("extract");
    fs::create_dir_all(&download_root)?;
    if extract_root.exists() {
        fs::remove_dir_all(&extract_root)?;
    }
    fs::create_dir_all(&extract_root)?;

    // Reuse the archive the SBOL 3 tests already downloaded when present.
    let shared = Path::new(env!("CARGO_MANIFEST_DIR")).join(SHARED_ARCHIVE);
    let archive = if shared.is_file() {
        shared
    } else {
        let archive = download_root.join("SBOLTestSuite.tar.gz");
        run_command(
            "curl",
            &[
                "-fsSL",
                SBOLTESTSUITE_ARCHIVE_URL,
                "-o",
                path_str(&archive)?,
            ],
        )?;
        archive
    };

    run_command(
        "tar",
        &["-xzf", path_str(&archive)?, "-C", path_str(&extract_root)?],
    )?;

    let source_root = extracted_source_root(&extract_root)?;
    for sub in EXTRACT_DIRS {
        copy_xml_files(
            &source_root.join(sub),
            &root.join("SBOLTestSuite").join(sub),
        )?;
    }
    fs::remove_dir_all(extract_root)?;
    Ok(())
}

fn path_str(path: &Path) -> io::Result<&str> {
    path.to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "non-UTF-8 path"))
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

fn copy_xml_files(source_dir: &Path, target_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(target_dir)?;
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("xml") {
            continue;
        }
        let name = path.file_name().expect("file has a name");
        fs::copy(&path, target_dir.join(name))?;
    }
    Ok(())
}

fn run_command(program: &str, args: &[&str]) -> io::Result<()> {
    let status = Command::new(program).args(args).status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "`{program}` exited with status {status}"
        )));
    }
    Ok(())
}
