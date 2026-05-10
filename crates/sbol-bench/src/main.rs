//! Cross-implementation performance bench.
//!
//! Times `(parse, serialize)` round trips against sbol-rs (in-process),
//! pySBOL3, libSBOLj3, and sboljs (each pinned in Docker). Inputs are a
//! handful of representative SBOLTestSuite fixtures pre-converted by
//! sbol-rs to every RDF serialization on disk so each implementation
//! sees the same byte-for-byte input. Each bench script reports per
//! iteration nanoseconds in JSON; this binary aggregates and prints
//! min/median/mean/p99/stddev for every (impl × fixture × format)
//! combination, falling back to a "skipped" row when a tool refuses or
//! fails.
//!
//! Methodology notes worth knowing before reading the numbers:
//!
//! - Wall-clock time only; no allocator or memory measurements. JVM and
//!   Node startup costs are excluded by running warmup iterations
//!   inside each container.
//! - The default workload (3 warmup + 20 measured iters) is meant for
//!   relative comparison, not absolute publication-grade reporting.
//!   Override via `SBOL_BENCH_WARMUP` / `SBOL_BENCH_ITERS` for tighter
//!   error bars.
//! - sboljs's underlying rdfoo parses only N-Triples and RDF/XML and
//!   serializes only RDF/XML, so the sboljs row only appears in those
//!   format combinations. This is a real ecosystem fact, not a harness
//!   bug.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use sbol::{Document, RdfFormat};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Implementation {
    SbolRs,
    Pysbol3,
    Libsbolj3,
    Sboljs,
}

impl Implementation {
    fn id(self) -> &'static str {
        match self {
            Self::SbolRs => "sbol-rs",
            Self::Pysbol3 => "pysbol3",
            Self::Libsbolj3 => "libsbolj3",
            Self::Sboljs => "sboljs",
        }
    }

    fn docker_image(self) -> &'static str {
        match self {
            Self::SbolRs => "sbol-rs-bench",
            Self::Pysbol3 => "pysbol3-bench",
            Self::Libsbolj3 => "libsbolj3-bench",
            Self::Sboljs => "sboljs3-bench",
        }
    }

    // Build command hint surfaced when an image is missing. The sbol-rs
    // image is built from the workspace root with `-f` because its
    // Dockerfile needs to see every workspace member, unlike the
    // foreign images whose context is their own directory.
    fn docker_build_command(self) -> &'static str {
        match self {
            Self::SbolRs => {
                "docker build -t sbol-rs-bench -f benches/cross-impl/sbol-rs/Dockerfile ."
            }
            Self::Pysbol3 => "docker build -t pysbol3-bench benches/cross-impl/pysbol3/",
            Self::Libsbolj3 => "docker build -t libsbolj3-bench benches/cross-impl/libsbolj3/",
            Self::Sboljs => "docker build -t sboljs3-bench benches/cross-impl/sboljs3/",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Fixture {
    stem: &'static str,
    source: &'static str,
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        stem: "component",
        source: "SBOLTestSuite/SBOL3/entity/component/component.ttl",
    },
    Fixture {
        stem: "multicellular_simple",
        source: "SBOLTestSuite/SBOL3/multicellular_simple/multicellular_simple.ttl",
    },
    Fixture {
        stem: "bba_f2620_popsreceiver",
        source: "SBOLTestSuite/SBOL3/BBa_F2620_PoPSReceiver/BBa_F2620_PoPSReceiver.ttl",
    },
    Fixture {
        stem: "toggle_switch_v2",
        source: "SBOLTestSuite/SBOL3/toggle_switch_v2/toggle_switch_v2.ttl",
    },
];

#[derive(Clone, Copy, Debug)]
struct BenchCase {
    implementation: Implementation,
    parse_format: RdfFormat,
    serialize_format: RdfFormat,
}

const BENCH_CASES: &[BenchCase] = &[
    BenchCase {
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::Turtle,
    },
    BenchCase {
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
    },
    BenchCase {
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::JsonLd,
        serialize_format: RdfFormat::JsonLd,
    },
    BenchCase {
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::NTriples,
        serialize_format: RdfFormat::NTriples,
    },
    BenchCase {
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::Turtle,
    },
    BenchCase {
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
    },
    BenchCase {
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::JsonLd,
        serialize_format: RdfFormat::JsonLd,
    },
    BenchCase {
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::NTriples,
        serialize_format: RdfFormat::NTriples,
    },
    BenchCase {
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::Turtle,
    },
    BenchCase {
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
    },
    BenchCase {
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::JsonLd,
        serialize_format: RdfFormat::JsonLd,
    },
    BenchCase {
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::NTriples,
        serialize_format: RdfFormat::NTriples,
    },
    // sboljs only: rdfoo's N-Triples parser stack is broken (an
    // `@rdfjs/sink-map` version skew that throws "parser.import is not
    // a function" before any triple is even produced), so sboljs's
    // only working path is RDF/XML in, RDF/XML out. The RDF/XML row
    // uses libSBOLj3's committed reference output as input (see
    // prepare_fixture_in_every_format) so every impl parses the same
    // bytes.
    BenchCase {
        implementation: Implementation::Sboljs,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
    },
];

// 20 warmup iterations is enough for the JVM's tiered JIT to reach
// steady state on a loop this small; 100 measured iterations gives
// stable p50s and acceptable p99s for every implementation in the
// matrix. The numbers in benches/cross-impl/README.md are captured at
// these defaults.
const DEFAULT_WARMUP: usize = 20;
const DEFAULT_ITERS: usize = 100;

#[derive(Debug, Deserialize)]
struct BenchSamples {
    #[serde(default)]
    version: Option<String>,
    parse_ns: Vec<u64>,
    serialize_ns: Vec<u64>,
    #[serde(default)]
    serialized_bytes: u64,
}

// Deserialization surface for whatever the bench scripts emit. The
// `impl` field is read by serde to ignore it (we already know which
// implementation produced the JSON from the case we dispatched);
// `version` is what shows up in the report.
#[derive(Deserialize)]
struct ForeignSamples {
    #[serde(default)]
    version: Option<String>,
    parse_ns: Vec<u64>,
    serialize_ns: Vec<u64>,
    #[serde(default)]
    serialized_bytes: u64,
}

#[derive(Debug)]
struct CaseOutcome {
    case: BenchCase,
    fixture: Fixture,
    state: CaseState,
}

#[derive(Debug)]
enum CaseState {
    Ok {
        version: String,
        samples: BenchSamples,
    },
    Skipped {
        reason: String,
    },
}

fn main() -> ExitCode {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("config error: {error}");
            return ExitCode::FAILURE;
        }
    };

    let workspace = workspace_root();
    let sboltest_root = workspace.join("tests/fixtures/sbol3");
    let run_id = format!(
        "{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    let scratch_root = workspace.join(format!("target/sbol-bench/{run_id}"));
    if let Err(error) = fs::create_dir_all(&scratch_root) {
        eprintln!(
            "failed to create scratch dir {}: {error}",
            scratch_root.display()
        );
        return ExitCode::FAILURE;
    }

    eprintln!("sbol-bench run {run_id}");
    eprintln!("  warmup iters : {}", config.warmup);
    eprintln!("  measured iters: {}", config.iters);
    eprintln!("  fixtures      : {}", config.fixtures.len());
    eprintln!("  scratch       : {}", scratch_root.display());

    let docker_available = if config.skip_docker {
        false
    } else {
        check_docker_available()
    };
    if !docker_available && !config.skip_docker {
        eprintln!(
            "docker not reachable; foreign implementations will be skipped \
             and sbol-rs will fall back to in-process timing \
             (set SBOL_BENCH_DOCKER=0 to silence this warning)"
        );
    }

    let mut outcomes: Vec<CaseOutcome> = Vec::new();
    let mut any_native_failure = false;

    for fixture in &config.fixtures {
        let source = sboltest_root.join(fixture.source);
        if !source.is_file() {
            eprintln!(
                "fixture missing at {}; populate the cache by running the sbol3_fixtures test once",
                source.display()
            );
            for case in BENCH_CASES {
                outcomes.push(CaseOutcome {
                    case: *case,
                    fixture: *fixture,
                    state: CaseState::Skipped {
                        reason: format!("fixture {} not on disk", source.display()),
                    },
                });
            }
            continue;
        }

        let prepared =
            match prepare_fixture_in_every_format(&source, &workspace, &scratch_root, fixture.stem)
            {
                Ok(prepared) => prepared,
                Err(error) => {
                    eprintln!("failed to prepare {}: {error}", fixture.stem);
                    for case in BENCH_CASES {
                        outcomes.push(CaseOutcome {
                            case: *case,
                            fixture: *fixture,
                            state: CaseState::Skipped {
                                reason: format!("could not pre-convert fixture: {error}"),
                            },
                        });
                    }
                    continue;
                }
            };

        for case in BENCH_CASES {
            let input_path = prepared
                .get(&case.parse_format)
                .expect("every format was prepared above");
            let outcome = if docker_available {
                let output_json = scratch_root.join(format!(
                    "{}.{}.{}-{}.json",
                    fixture.stem,
                    case.implementation.id(),
                    case.parse_format.extension(),
                    case.serialize_format.extension(),
                ));
                match run_docker(input_path, &output_json, *case, &config) {
                    Ok(samples) => {
                        let version = samples.version.clone().unwrap_or_default();
                        CaseOutcome {
                            case: *case,
                            fixture: *fixture,
                            state: CaseState::Ok { version, samples },
                        }
                    }
                    Err(error) => CaseOutcome {
                        case: *case,
                        fixture: *fixture,
                        state: CaseState::Skipped {
                            reason: format!("{} bench failed: {error}", case.implementation.id()),
                        },
                    },
                }
            } else if matches!(case.implementation, Implementation::SbolRs) {
                // Docker is unavailable: fall back to in-process Rust so devs
                // can iterate without building the Docker image. The result
                // is not directly comparable to a foreign-impl row (no VM
                // overhead, native macOS vs Linux), so the report tags this
                // case with `(native, not in container)` for clarity.
                match run_native(input_path, *case, &config) {
                    Ok(samples) => CaseOutcome {
                        case: *case,
                        fixture: *fixture,
                        state: CaseState::Ok {
                            version: format!("{} (native, not in container)", rust_impl_version()),
                            samples,
                        },
                    },
                    Err(error) => {
                        any_native_failure = true;
                        CaseOutcome {
                            case: *case,
                            fixture: *fixture,
                            state: CaseState::Skipped {
                                reason: format!("native bench failed: {error}"),
                            },
                        }
                    }
                }
            } else {
                CaseOutcome {
                    case: *case,
                    fixture: *fixture,
                    state: CaseState::Skipped {
                        reason: "docker unavailable".to_owned(),
                    },
                }
            };
            outcomes.push(outcome);
        }
    }

    print_report(&outcomes);

    if let Some(report_path) = config.report_path.as_ref() {
        match write_json_report(report_path, &outcomes, &config) {
            Ok(()) => eprintln!("wrote machine-readable report to {}", report_path.display()),
            Err(error) => eprintln!(
                "failed to write report to {}: {error}",
                report_path.display()
            ),
        }
    }

    if any_native_failure {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

#[derive(Debug)]
struct Config {
    warmup: usize,
    iters: usize,
    fixtures: Vec<Fixture>,
    skip_docker: bool,
    report_path: Option<PathBuf>,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let warmup = parse_env_usize("SBOL_BENCH_WARMUP", DEFAULT_WARMUP)?;
        let iters = parse_env_usize("SBOL_BENCH_ITERS", DEFAULT_ITERS)?;
        if iters == 0 {
            return Err("SBOL_BENCH_ITERS must be > 0".to_owned());
        }
        let skip_docker = matches!(
            std::env::var("SBOL_BENCH_DOCKER").ok().as_deref(),
            Some("0") | Some("false") | Some("off")
        );
        let fixtures = match std::env::var("SBOL_BENCH_FIXTURES") {
            Ok(list) => {
                let mut chosen = Vec::new();
                for stem in list.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                    let fixture = FIXTURES
                        .iter()
                        .find(|f| f.stem == stem)
                        .ok_or_else(|| format!("unknown fixture stem: {stem}"))?;
                    chosen.push(*fixture);
                }
                if chosen.is_empty() {
                    FIXTURES.to_vec()
                } else {
                    chosen
                }
            }
            Err(_) => FIXTURES.to_vec(),
        };
        let report_path = std::env::var_os("SBOL_BENCH_REPORT").map(PathBuf::from);
        Ok(Self {
            warmup,
            iters,
            fixtures,
            skip_docker,
            report_path,
        })
    }
}

fn parse_env_usize(name: &str, default: usize) -> Result<usize, String> {
    match std::env::var(name) {
        Ok(value) => value
            .parse::<usize>()
            .map_err(|error| format!("invalid {name}={value:?}: {error}")),
        Err(_) => Ok(default),
    }
}

fn prepare_fixture_in_every_format(
    source: &Path,
    workspace_root: &Path,
    scratch_root: &Path,
    stem: &str,
) -> Result<BTreeMap<RdfFormat, PathBuf>, String> {
    let source_text = fs::read_to_string(source)
        .map_err(|error| format!("read {}: {error}", source.display()))?;
    let document = Document::read_turtle(&source_text)
        .map_err(|error| format!("parse {}: {error}", source.display()))?;

    let mut paths = BTreeMap::new();
    for &format in RdfFormat::ALL {
        let output_path = scratch_root.join(format!("{stem}.{}", format.extension()));

        // RDF/XML gets special handling: sbol-rs emits inline `xmlns`
        // on every child element, which is valid RDF/XML 1.1 but
        // sboljs's rdf-parser-rdfxml mis-parses the inner literal
        // elements as blank-node subjects. libSBOLj3 emits standard
        // prefix-style RDF/XML that every parser including
        // rdf-parser-rdfxml handles correctly, so we use libSBOLj3's
        // committed reference output for the RDF/XML input row across
        // every impl. The triple set is the same (the conformance
        // tests guarantee this), so the bench stays apples-to-apples.
        let serialized = if matches!(format, RdfFormat::RdfXml) {
            let normalized = workspace_root
                .join("tests/fixtures/cross-impl")
                .join(format!("{stem}.libSBOLj3.expected.rdf"));
            fs::read_to_string(&normalized).map_err(|error| {
                format!(
                    "read libSBOLj3 reference rdfxml {} \
                     (needed for cross-impl parity in the RDF/XML row): {error}",
                    normalized.display()
                )
            })?
        } else {
            document
                .write(format)
                .map_err(|error| format!("serialize {} as {format}: {error}", source.display()))?
        };

        fs::write(&output_path, &serialized).map_err(|error| {
            format!(
                "write pre-converted fixture {}: {error}",
                output_path.display()
            )
        })?;
        paths.insert(format, output_path);
    }
    Ok(paths)
}

fn run_native(input_path: &Path, case: BenchCase, config: &Config) -> Result<BenchSamples, String> {
    let rdf_text = fs::read_to_string(input_path)
        .map_err(|error| format!("read {}: {error}", input_path.display()))?;

    for _ in 0..config.warmup {
        let doc = Document::read(&rdf_text, case.parse_format)
            .map_err(|error| format!("warmup parse: {error}"))?;
        let _ = doc
            .write(case.serialize_format)
            .map_err(|error| format!("warmup serialize: {error}"))?;
    }

    let mut parse_ns = Vec::with_capacity(config.iters);
    let mut serialize_ns = Vec::with_capacity(config.iters);
    let mut last_bytes = 0u64;
    for _ in 0..config.iters {
        let t0 = Instant::now();
        let doc = Document::read(&rdf_text, case.parse_format)
            .map_err(|error| format!("measured parse: {error}"))?;
        let t1 = Instant::now();
        let out = doc
            .write(case.serialize_format)
            .map_err(|error| format!("measured serialize: {error}"))?;
        let t2 = Instant::now();
        parse_ns.push(t1.duration_since(t0).as_nanos() as u64);
        serialize_ns.push(t2.duration_since(t1).as_nanos() as u64);
        last_bytes = out.len() as u64;
    }

    Ok(BenchSamples {
        version: Some(rust_impl_version().to_owned()),
        parse_ns,
        serialize_ns,
        serialized_bytes: last_bytes,
    })
}

fn run_docker(
    input_path: &Path,
    output_json: &Path,
    case: BenchCase,
    config: &Config,
) -> Result<BenchSamples, String> {
    let image = case.implementation.docker_image();
    if !image_present(image) {
        return Err(format!(
            "docker image `{image}` not built; run `{}`",
            case.implementation.docker_build_command()
        ));
    }

    // Mount the scratch dir read-write because the script writes the
    // timing JSON next to the input file. Containers and the host see
    // the same path under /work, so we translate.
    let scratch = input_path
        .parent()
        .ok_or_else(|| "input path has no parent".to_owned())?;
    let host_mount = scratch.to_string_lossy().to_string();
    let input_name = path_basename(input_path);
    let output_name = path_basename(output_json);
    let container_input = format!("/work/{input_name}");
    let container_output = format!("/work/{output_name}");
    let mount_arg = format!("{host_mount}:/work");

    let warmup_arg = config.warmup.to_string();
    let iters_arg = config.iters.to_string();
    let parse_arg = format_name(case.parse_format);
    let serialize_arg = format_name(case.serialize_format);

    let result = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-v",
            &mount_arg,
            image,
            &container_input,
            parse_arg,
            serialize_arg,
            &warmup_arg,
            &iters_arg,
            &container_output,
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| format!("spawn docker: {error}"))?;
    if !result.success() {
        return Err(format!("`docker run {image}` exited with status {result}"));
    }

    let text = fs::read_to_string(output_json)
        .map_err(|error| format!("read timings json {}: {error}", output_json.display()))?;
    let raw: ForeignSamples = serde_json::from_str(&text)
        .map_err(|error| format!("parse timings json {}: {error}", output_json.display()))?;
    if raw.parse_ns.len() != config.iters || raw.serialize_ns.len() != config.iters {
        return Err(format!(
            "bench produced {} parse samples and {} serialize samples; expected {}",
            raw.parse_ns.len(),
            raw.serialize_ns.len(),
            config.iters
        ));
    }
    Ok(BenchSamples {
        version: raw.version,
        parse_ns: raw.parse_ns,
        serialize_ns: raw.serialize_ns,
        serialized_bytes: raw.serialized_bytes,
    })
}

fn check_docker_available() -> bool {
    Command::new("docker")
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn image_present(image: &str) -> bool {
    Command::new("docker")
        .args(["image", "inspect", image])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn format_name(format: RdfFormat) -> &'static str {
    match format {
        RdfFormat::Turtle => "turtle",
        RdfFormat::RdfXml => "rdfxml",
        RdfFormat::JsonLd => "jsonld",
        RdfFormat::NTriples => "ntriples",
        _ => panic!("bench harness has no name for format {format}"),
    }
}

fn rust_impl_version() -> &'static str {
    env!("SBOL_CRATE_VERSION")
}

fn path_basename(path: &Path) -> String {
    path.file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("input")
        .to_owned()
}

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(manifest)
}

#[derive(Debug)]
struct Stats {
    median_ns: u64,
    p99_ns: u64,
}

fn stats(samples: &[u64]) -> Option<Stats> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let n = sorted.len();
    let median_ns = if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2
    };
    let p99_idx = ((n as f64 * 0.99).ceil() as usize)
        .saturating_sub(1)
        .min(n - 1);
    let p99_ns = sorted[p99_idx];
    Some(Stats { median_ns, p99_ns })
}

fn print_report(outcomes: &[CaseOutcome]) {
    let mut by_fixture: BTreeMap<&'static str, Vec<&CaseOutcome>> = BTreeMap::new();
    for outcome in outcomes {
        by_fixture
            .entry(outcome.fixture.stem)
            .or_default()
            .push(outcome);
    }

    for (fixture_stem, fixture_outcomes) in by_fixture {
        println!();
        println!("=== fixture: {fixture_stem} ===");
        println!(
            "{:<11} {:<9} -> {:<9} {:>14} {:>14} {:>14} {:>14} {:>10}",
            "impl",
            "parse",
            "serialize",
            "parse p50 (μs)",
            "parse p99 (μs)",
            "ser. p50 (μs)",
            "ser. p99 (μs)",
            "bytes"
        );
        for outcome in fixture_outcomes {
            let parse_label = format_name(outcome.case.parse_format);
            let serialize_label = format_name(outcome.case.serialize_format);
            match &outcome.state {
                CaseState::Ok { samples, .. } => {
                    let parse = stats(&samples.parse_ns);
                    let ser = stats(&samples.serialize_ns);
                    match (parse, ser) {
                        (Some(parse), Some(ser)) => {
                            println!(
                                "{:<11} {:<9} -> {:<9} {:>14.2} {:>14.2} {:>14.2} {:>14.2} {:>10}",
                                outcome.case.implementation.id(),
                                parse_label,
                                serialize_label,
                                parse.median_ns as f64 / 1_000.0,
                                parse.p99_ns as f64 / 1_000.0,
                                ser.median_ns as f64 / 1_000.0,
                                ser.p99_ns as f64 / 1_000.0,
                                samples.serialized_bytes
                            );
                        }
                        _ => {
                            println!(
                                "{:<11} {:<9} -> {:<9} (no samples)",
                                outcome.case.implementation.id(),
                                parse_label,
                                serialize_label
                            );
                        }
                    }
                }
                CaseState::Skipped { reason } => {
                    println!(
                        "{:<11} {:<9} -> {:<9} skipped: {}",
                        outcome.case.implementation.id(),
                        parse_label,
                        serialize_label,
                        reason
                    );
                }
            }
        }
    }
    println!();
    println!("p50 = median across measured iters; p99 picked from same set. Lower is better.");
}

#[derive(serde::Serialize)]
struct ReportFile<'a> {
    run_id: String,
    warmup_iters: usize,
    measured_iters: usize,
    cases: Vec<ReportCase<'a>>,
}

#[derive(serde::Serialize)]
struct ReportCase<'a> {
    fixture: &'a str,
    implementation: &'a str,
    parse_format: &'a str,
    serialize_format: &'a str,
    state: ReportState<'a>,
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ReportState<'a> {
    Ok {
        version: &'a str,
        serialized_bytes: u64,
        parse_ns: &'a [u64],
        serialize_ns: &'a [u64],
    },
    Skipped {
        reason: &'a str,
    },
}

fn write_json_report(path: &Path, outcomes: &[CaseOutcome], config: &Config) -> Result<(), String> {
    let run_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_owned());
    let cases = outcomes
        .iter()
        .map(|outcome| {
            let state = match &outcome.state {
                CaseState::Ok { version, samples } => ReportState::Ok {
                    version: version.as_str(),
                    serialized_bytes: samples.serialized_bytes,
                    parse_ns: &samples.parse_ns,
                    serialize_ns: &samples.serialize_ns,
                },
                CaseState::Skipped { reason } => ReportState::Skipped {
                    reason: reason.as_str(),
                },
            };
            ReportCase {
                fixture: outcome.fixture.stem,
                implementation: outcome.case.implementation.id(),
                parse_format: format_name(outcome.case.parse_format),
                serialize_format: format_name(outcome.case.serialize_format),
                state,
            }
        })
        .collect();
    let report = ReportFile {
        run_id,
        warmup_iters: config.warmup,
        measured_iters: config.iters,
        cases,
    };
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create report parent dir {}: {error}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("serialize report: {error}"))?;
    fs::write(path, json).map_err(|error| format!("write report {}: {error}", path.display()))?;
    Ok(())
}
