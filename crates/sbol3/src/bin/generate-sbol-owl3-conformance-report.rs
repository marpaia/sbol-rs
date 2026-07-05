//! Generates `docs/sbol-owl3-conformance.md` from the pinned OWL fixture
//! and `crates/sbol3/src/vocab.rs`. The committed report must stay in
//! sync with both sources; CI re-runs this binary via the freshness
//! gate in `crates/sbol3/tests/sbol_owl3_conformance_report.rs` and
//! `git diff --exit-code` flags drift.
//!
//! Usage: `cargo run -p sbol3 --bin generate-sbol-owl3-conformance-report`
//!
//! Pass `--check` to print the rendered report to stdout instead of
//! writing to disk; useful for local diffing before committing.

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sbol3::owl_conformance::{OwlPinInfo, analyze_owl_conformance, render_owl_conformance_report};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let check_only = args.iter().any(|arg| arg == "--check");

    let workspace = workspace_root();
    let fixture_dir = workspace.join("crates/sbol3/tests/fixtures/sbol-owl3");
    let fixture_path = fixture_dir.join("sbol3.rdf");
    let manifest_path = fixture_dir.join("manifest.toml");

    let rdf = match fs::read_to_string(&fixture_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("failed to read {}: {error}", fixture_path.display());
            return ExitCode::FAILURE;
        }
    };
    let manifest_text = match fs::read_to_string(&manifest_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("failed to read {}: {error}", manifest_path.display());
            return ExitCode::FAILURE;
        }
    };
    let manifest = match parse_manifest(&manifest_text) {
        Ok(m) => m,
        Err(error) => {
            eprintln!("parse manifest: {error}");
            return ExitCode::FAILURE;
        }
    };

    let report = match analyze_owl_conformance(&rdf) {
        Ok(r) => r,
        Err(error) => {
            eprintln!("analyze conformance: {error}");
            return ExitCode::FAILURE;
        }
    };

    let pin = OwlPinInfo {
        upstream_repo: &manifest.upstream_repo,
        source_url: &manifest.url,
        commit: &manifest.commit,
        committer_date: &manifest.committer_date,
        sha256: &manifest.sha256,
        fetched_at: &manifest.fetched_at,
    };
    let rendered = render_owl_conformance_report(&report, &pin);

    if check_only {
        let mut stdout = std::io::stdout().lock();
        if stdout.write_all(rendered.as_bytes()).is_err() {
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    let target = workspace.join("docs/sbol-owl3-conformance.md");
    if let Err(error) = write_if_changed(&target, &rendered) {
        eprintln!("failed to write {}: {error}", target.display());
        return ExitCode::FAILURE;
    }
    println!("wrote {}", target.display());
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

fn write_if_changed(target: &Path, content: &str) -> std::io::Result<()> {
    if let Ok(existing) = fs::read_to_string(target)
        && existing == content
    {
        return Ok(());
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(target, content)
}

struct ManifestFields {
    upstream_repo: String,
    url: String,
    commit: String,
    committer_date: String,
    sha256: String,
    fetched_at: String,
}

fn parse_manifest(text: &str) -> Result<ManifestFields, String> {
    Ok(ManifestFields {
        upstream_repo: required_field(text, "upstream_repo")?,
        url: required_field(text, "url")?,
        commit: required_field(text, "commit")?,
        committer_date: required_field(text, "committer_date")?,
        sha256: required_field(text, "sha256")?,
        fetched_at: required_field(text, "at")?,
    })
}

fn required_field(text: &str, key: &str) -> Result<String, String> {
    // Manifest entries look like: `key = "value"` on a single line. A
    // hand-rolled extractor avoids pulling toml into the bin and keeps
    // the binary buildable from the published crate (which excludes
    // `tests/`, but src/bin is also excluded so this binary never
    // ships).
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(key)
            && let Some(rest) = rest.trim_start().strip_prefix('=')
        {
            let rest = rest.trim();
            if let Some(rest) = rest.strip_prefix('"')
                && let Some(end) = rest.find('"')
            {
                return Ok(rest[..end].to_string());
            }
        }
    }
    Err(format!("manifest missing required field `{key}`"))
}
