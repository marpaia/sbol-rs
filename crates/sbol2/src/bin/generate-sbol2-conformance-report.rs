//! Generates `docs/sbol2-conformance.md` from the SBOL 2 validation rule
//! catalog. The committed report must stay in sync with
//! `validation_rule_statuses()`; CI re-runs this binary and
//! `git diff --exit-code` flags drift.
//!
//! Usage: `cargo run -p sbol2 --bin generate-sbol2-conformance-report`
//!
//! Pass `--check` to print the rendered report to stdout instead of
//! writing to disk; useful for local diffing before committing.

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sbol2::render_sbol2_conformance_report;
use sbol2::validation::validation_rule_statuses;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let check_only = args.iter().any(|arg| arg == "--check");

    let report = render_sbol2_conformance_report(validation_rule_statuses());

    if check_only {
        let mut stdout = std::io::stdout().lock();
        if stdout.write_all(report.as_bytes()).is_err() {
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    let target = workspace_root().join("docs").join("sbol2-conformance.md");
    if let Err(error) = write_if_changed(&target, &report) {
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
