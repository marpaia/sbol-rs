//! Captures git metadata (commit, tag, dirty flag) at compile time and
//! exposes it to `main.rs` via `SBOL_VERSION_FULL`.
//!
//! Tarball / crates.io / no-git builds: every git probe fails silently
//! and the version string collapses to `CARGO_PKG_VERSION`.
//!
//! Reproducible builds (Nix, Bazel, package managers without `.git/`):
//! set `SBOL_BUILD_GIT_SHA`, `SBOL_BUILD_GIT_TAG`, and
//! `SBOL_BUILD_GIT_DIRTY=1` to inject values without invoking `git`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

fn main() {
    let pkg_version = env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION set by cargo");

    let sha = env_override("SBOL_BUILD_GIT_SHA").or_else(|| git_output(&["rev-parse", "HEAD"]));
    let tag = env_override("SBOL_BUILD_GIT_TAG")
        .or_else(|| git_output(&["describe", "--tags", "--exact-match", "HEAD"]));
    let dirty = match env_override("SBOL_BUILD_GIT_DIRTY") {
        Some(v) => matches!(v.as_str(), "1" | "true" | "yes"),
        None => git_is_dirty(),
    };

    let mut parts: Vec<String> = Vec::new();
    if let Some(sha) = sha.as_deref() {
        let short = &sha[..sha.len().min(7)];
        if dirty {
            parts.push(format!("commit {short}+dirty"));
        } else {
            parts.push(format!("commit {short}"));
        }
    } else if dirty {
        parts.push("dirty".to_string());
    }
    if let Some(tag) = tag.as_deref() {
        parts.push(format!("tag {tag}"));
    }

    let version_full = if parts.is_empty() {
        pkg_version
    } else {
        format!("{pkg_version} ({})", parts.join(", "))
    };
    println!("cargo:rustc-env=SBOL_VERSION_FULL={version_full}");

    println!("cargo:rerun-if-env-changed=SBOL_BUILD_GIT_SHA");
    println!("cargo:rerun-if-env-changed=SBOL_BUILD_GIT_TAG");
    println!("cargo:rerun-if-env-changed=SBOL_BUILD_GIT_DIRTY");
    println!("cargo:rerun-if-changed=build.rs");

    if let Some(git_dir) = find_git_dir() {
        let head = git_dir.join("HEAD");
        if head.exists() {
            println!("cargo:rerun-if-changed={}", head.display());
            if let Ok(contents) = fs::read_to_string(&head) {
                if let Some(ref_path) = contents.strip_prefix("ref: ").map(str::trim) {
                    let ref_file = git_dir.join(ref_path);
                    if ref_file.exists() {
                        println!("cargo:rerun-if-changed={}", ref_file.display());
                    }
                    let packed_refs = git_dir.join("packed-refs");
                    if packed_refs.exists() {
                        println!("cargo:rerun-if-changed={}", packed_refs.display());
                    }
                }
            }
        }
    }
}

fn env_override(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

fn git_output(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn git_is_dirty() -> bool {
    match Command::new("git").args(["status", "--porcelain"]).output() {
        Ok(out) if out.status.success() => !out.stdout.is_empty(),
        _ => false,
    }
}

fn find_git_dir() -> Option<PathBuf> {
    let start = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR")?);
    let mut cur: &Path = &start;
    loop {
        let candidate = cur.join(".git");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if candidate.is_file() {
            // git worktree: `.git` is a file pointing at the real gitdir.
            if let Ok(contents) = fs::read_to_string(&candidate) {
                if let Some(path) = contents.strip_prefix("gitdir: ").map(str::trim) {
                    let resolved = cur.join(path);
                    if resolved.exists() {
                        return Some(resolved);
                    }
                }
            }
        }
        cur = cur.parent()?;
    }
}
