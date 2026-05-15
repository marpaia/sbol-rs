//! Refresh the pinned `sbol3.rdf` fixture used by
//! `crates/sbol/tests/sbol_owl3_conformance.rs`.
//!
//! Fetches the upstream `SynBioDex/sbol-owl3` repository's current `main`
//! commit, downloads `sbol-owl3-gen/sbol3.rdf` at that commit (the
//! canonical OWL serialization of the SBOL 3 data model that backs the
//! published HTML documentation), recomputes the sha256, and rewrites
//! both `sbol3.rdf` and `manifest.toml` in
//! `crates/sbol/tests/fixtures/sbol-owl3/`.
//!
//! Run from the workspace root:
//!
//! ```sh
//! cargo run -p sbol-ontology --bin update-sbol-owl3-fixture
//! ```
//!
//! After running, re-execute `cargo test -p sbol --test sbol_owl3_conformance`
//! and triage any new diffs into `OWL_ONLY_ALLOWLIST` /
//! `RUST_ONLY_ALLOWLIST` in the test file.

use std::path::{Path, PathBuf};

use sbol_ontology::download;
use sha2::{Digest, Sha256};

const COMMIT_API: &str = "https://api.github.com/repos/SynBioDex/sbol-owl3/commits/main";
const RAW_URL_TEMPLATE: &str =
    "https://raw.githubusercontent.com/SynBioDex/sbol-owl3/{sha}/sbol-owl3-gen/sbol3.rdf";

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let fixture_dir = locate_fixture_dir()?;
    let rdf_path = fixture_dir.join("sbol3.rdf");
    let manifest_path = fixture_dir.join("manifest.toml");

    println!("fetching upstream commit metadata: {COMMIT_API}");
    let commit_json =
        download::fetch(COMMIT_API).map_err(|e| format!("fetch commit metadata: {e}"))?;
    let commit_json = std::str::from_utf8(&commit_json)
        .map_err(|e| format!("commit metadata is not UTF-8: {e}"))?;
    let sha = extract_first(commit_json, "\"sha\":\"", "\"")
        .ok_or_else(|| "could not extract commit sha from API response".to_string())?;
    let date = extract_first(commit_json, "\"committer\":{\"name\":", "}")
        .and_then(|chunk| extract_first(chunk, "\"date\":\"", "\""))
        .ok_or_else(|| "could not extract committer date from API response".to_string())?;
    println!("upstream main is at {sha} ({date})");

    let raw_url = RAW_URL_TEMPLATE.replace("{sha}", sha);
    println!("downloading {raw_url}");
    let body = download::fetch(&raw_url).map_err(|e| format!("download fixture: {e}"))?;
    let new_sha256 = hex_lower(Sha256::digest(&body).as_slice());

    let previous_sha256 = previous_sha256(&manifest_path);
    if previous_sha256.as_deref() == Some(new_sha256.as_str()) {
        println!(
            "fixture unchanged (sha256 {new_sha256}); manifest will be refreshed for fetched-at"
        );
    } else if let Some(old) = &previous_sha256 {
        println!("fixture changed:\n  old sha256: {old}\n  new sha256: {new_sha256}");
        println!(
            "Re-run `cargo test -p sbol --test sbol_owl3_conformance` to surface any \
             schema drift; triage diffs into the allowlists."
        );
    } else {
        println!("first pin; sha256 {new_sha256}");
    }

    std::fs::write(&rdf_path, &body).map_err(|e| format!("write {}: {e}", rdf_path.display()))?;

    let today = today_yyyy_mm_dd();
    let manifest = render_manifest(sha, date, &new_sha256, &today, &raw_url);
    std::fs::write(&manifest_path, manifest)
        .map_err(|e| format!("write {}: {e}", manifest_path.display()))?;

    println!("wrote {} and manifest.toml", rdf_path.display());
    Ok(())
}

fn locate_fixture_dir() -> Result<PathBuf, String> {
    // The binary may be invoked from anywhere in the workspace. Walk
    // upward from CARGO_MANIFEST_DIR (the `sbol-ontology` crate) to the
    // workspace root and then descend to the fixture directory.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "could not derive workspace root from CARGO_MANIFEST_DIR".to_string())?
        .to_path_buf();
    let dir = workspace.join("crates/sbol/tests/fixtures/sbol-owl3");
    if !dir.is_dir() {
        return Err(format!(
            "expected fixture directory at {} (run from the workspace root)",
            dir.display()
        ));
    }
    Ok(dir)
}

fn previous_sha256(manifest_path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(manifest_path).ok()?;
    let sha = extract_first(&text, "sha256 = \"", "\"")?;
    Some(sha.to_string())
}

fn extract_first<'a>(haystack: &'a str, prefix: &str, terminator: &str) -> Option<&'a str> {
    let start = haystack.find(prefix)? + prefix.len();
    let rest = &haystack[start..];
    let end = rest.find(terminator)?;
    Some(&rest[..end])
}

fn render_manifest(
    sha: &str,
    committer_date: &str,
    sha256: &str,
    today: &str,
    url: &str,
) -> String {
    format!(
        "[source]\n\
         upstream_repo = \"https://github.com/SynBioDex/sbol-owl3\"\n\
         file = \"sbol3.rdf\"\n\
         url = \"{url}\"\n\
         commit = \"{sha}\"\n\
         committer_date = \"{committer_date}\"\n\
         \n\
         [integrity]\n\
         sha256 = \"{sha256}\"\n\
         \n\
         [fetched]\n\
         at = \"{today}\"\n\
         tool = \"crates/sbol-ontology/src/bin/update-sbol-owl3-fixture.rs\"\n"
    )
}

fn today_yyyy_mm_dd() -> String {
    // Coarse formatting that does not need the `chrono`/`time` crates.
    // Updates land via the CLI; the test only consumes `[integrity]`, so
    // formatting drift here is not load-bearing.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days_since_epoch = secs / 86_400;
    let (y, m, d) = civil_from_days(days_since_epoch as i64);
    format!("{y:04}-{m:02}-{d:02}")
}

// Howard Hinnant's days-from-civil algorithm, inverted. Standard public-
// domain reference implementation; treats March as the first month of an
// internal year to fold leap-day handling.
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year as i32, m as u32, d as u32)
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}
