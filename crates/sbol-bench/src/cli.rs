//! Command-line interface and the resolved run configuration.
//!
//! Every flag also reads from its historical `SBOL_BENCH_*` environment
//! variable, so existing scripts and the numbers captured in
//! `benches/cross-impl/README.md` keep working unchanged.

use std::path::PathBuf;

use clap::Parser;

use crate::matrix::{DEFAULT_ITERS, DEFAULT_WARMUP, FIXTURES, Fixture};

/// Dual-version cross-implementation `(parse, serialize)` performance
/// bench for sbol-rs against libSBOLj (SBOL 2) and pySBOL3, libSBOLj3,
/// and sboljs (SBOL 3).
#[derive(Parser, Debug)]
#[command(
    name = "sbol-bench",
    about = "Dual-version (SBOL 2 + SBOL 3) cross-implementation benchmarks for sbol-rs vs libSBOLj, pySBOL3, libSBOLj3, and sboljs"
)]
pub(crate) struct Cli {
    /// Warmup iterations run (and discarded) before timing, so the JVM and
    /// Node JITs reach steady state.
    #[arg(long, env = "SBOL_BENCH_WARMUP", default_value_t = DEFAULT_WARMUP)]
    pub(crate) warmup: usize,

    /// Measured iterations timed per (impl × fixture × format) case.
    #[arg(long, env = "SBOL_BENCH_ITERS", default_value_t = DEFAULT_ITERS)]
    pub(crate) iters: usize,

    /// Comma-separated fixture stems to run. Defaults to every fixture.
    #[arg(long, env = "SBOL_BENCH_FIXTURES", value_delimiter = ',')]
    pub(crate) fixtures: Vec<String>,

    /// Write a machine-readable JSON report to this path in addition to the
    /// console table.
    #[arg(long, env = "SBOL_BENCH_REPORT")]
    pub(crate) report: Option<PathBuf>,

    /// Skip the Docker-based foreign implementations and time only the
    /// in-process Rust path. Equivalent to the legacy `SBOL_BENCH_DOCKER=0`.
    #[arg(long)]
    pub(crate) no_docker: bool,
}

/// Resolved run configuration derived from the parsed [`Cli`].
#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) warmup: usize,
    pub(crate) iters: usize,
    pub(crate) fixtures: Vec<Fixture>,
    pub(crate) skip_docker: bool,
    pub(crate) report_path: Option<PathBuf>,
}

impl Config {
    pub(crate) fn from_cli(cli: Cli) -> Result<Self, String> {
        if cli.iters == 0 {
            return Err("--iters must be > 0".to_owned());
        }
        let fixtures = resolve_fixtures(&cli.fixtures)?;
        // The `--no-docker` flag and the legacy `SBOL_BENCH_DOCKER=0|false|off`
        // env var both force the in-process fallback.
        let skip_docker = cli.no_docker
            || matches!(
                std::env::var("SBOL_BENCH_DOCKER").ok().as_deref(),
                Some("0") | Some("false") | Some("off")
            );
        Ok(Self {
            warmup: cli.warmup,
            iters: cli.iters,
            fixtures,
            skip_docker,
            report_path: cli.report,
        })
    }
}

fn resolve_fixtures(requested: &[String]) -> Result<Vec<Fixture>, String> {
    let mut chosen = Vec::new();
    for stem in requested.iter().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let fixture = FIXTURES
            .iter()
            .find(|f| f.stem == stem)
            .ok_or_else(|| format!("unknown fixture stem: {stem}"))?;
        chosen.push(*fixture);
    }
    if chosen.is_empty() {
        Ok(FIXTURES.to_vec())
    } else {
        Ok(chosen)
    }
}
