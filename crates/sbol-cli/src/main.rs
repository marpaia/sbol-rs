//! `sbol` — command-line tool for SBOL 3 documents.
//!
//! See `sbol validate --help` for the full surface.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use sbol::{
    Blocker, Document, ExternalValidationMode, FileResolver, NormativeSeverity, RdfFormat,
    ReadError, RuleStatus, Severity, ValidationContext, ValidationIssue, ValidationOptions,
    ValidationReport, ValidationRuleStatus, WriteError, validation_rule_statuses,
};
use sbol_ontology::{KnownOntology, OntologyCache, OntologyDescriptor};
use serde_json::{Value, json};

#[cfg(feature = "http-resolver")]
use sbol::CachingHttpResolver;

#[cfg(feature = "sarif")]
mod sarif;

#[derive(Parser)]
#[command(
    name = "sbol",
    version = env!("SBOL_VERSION_FULL"),
    about = "Command-line tool for SBOL 3 documents",
    propagate_version = true
)]
struct Cli {
    /// When to colorize output. `auto` colorizes the streams that are
    /// TTYs and `NO_COLOR` is unset.
    #[arg(long, value_enum, default_value_t = ColorMode::Auto, global = true)]
    color: ColorMode,

    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy)]
struct Styles {
    stdout: bool,
    stderr: bool,
}

impl Styles {
    fn resolve(mode: ColorMode) -> Self {
        let no_color = env::var_os("NO_COLOR").is_some();
        match mode {
            ColorMode::Always => Self {
                stdout: true,
                stderr: true,
            },
            ColorMode::Never => Self {
                stdout: false,
                stderr: false,
            },
            ColorMode::Auto => Self {
                stdout: !no_color && io::stdout().is_terminal(),
                stderr: !no_color && io::stderr().is_terminal(),
            },
        }
    }

    fn err_label(self) -> &'static str {
        if self.stderr {
            "\x1b[1;31merror\x1b[0m"
        } else {
            "error"
        }
    }
}

fn paint(enabled: bool, code: &str, text: &str) -> String {
    if enabled {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

fn severity_code(severity: Severity) -> Option<&'static str> {
    match severity {
        Severity::Error => Some("1;31"),
        Severity::Warning => Some("1;33"),
        _ => None,
    }
}

fn rule_status_code(status: RuleStatus) -> Option<&'static str> {
    match status {
        RuleStatus::Error => Some("31"),
        RuleStatus::Warning => Some("33"),
        RuleStatus::Configurable => Some("36"),
        RuleStatus::MachineUncheckable => Some("90"),
        RuleStatus::Unimplemented => Some("35"),
        _ => None,
    }
}

#[derive(Subcommand)]
enum Command {
    /// Validate an SBOL 3 document against the spec.
    Validate(ValidateArgs),
    /// Convert an SBOL 3 document between RDF serializations.
    Convert(ConvertArgs),
    /// Inspect the built-in validation rule catalog.
    #[command(subcommand)]
    Rules(RulesCommand),
    /// Manage cached extension ontologies (NCIT and others).
    #[command(subcommand)]
    Ontology(OntologyCommand),
}

#[derive(Subcommand)]
enum RulesCommand {
    /// List validation rules, their implementation status, and spec section.
    List(RulesListArgs),
}

#[derive(Subcommand)]
enum OntologyCommand {
    /// Download and build a named ontology extension into the cache.
    Install(OntologyInstallArgs),
    /// List installed ontology extensions.
    List,
    /// Print the cache directory path.
    Path,
    /// Remove an installed ontology extension.
    Remove(OntologyRemoveArgs),
    /// Re-hash an installed extension's TSV and compare against its
    /// manifest. Errors if the extension is missing or tampered with.
    Verify(OntologyVerifyArgs),
}

#[derive(clap::Args)]
struct OntologyInstallArgs {
    /// Built-in ontology to install. Currently: `ncit`.
    name: String,
    /// Re-download and rebuild even if already installed.
    #[arg(long)]
    force: bool,
}

#[derive(clap::Args)]
struct OntologyRemoveArgs {
    /// Cache entry name (e.g. `ncit`).
    name: String,
}

#[derive(clap::Args)]
struct OntologyVerifyArgs {
    /// Cache entry name to verify. If omitted, every installed
    /// extension is verified.
    name: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    #[cfg(feature = "sarif")]
    Sarif,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum RulesFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum RuleStatusFilter {
    Error,
    Warning,
    Configurable,
    MachineUncheckable,
    Unimplemented,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum RdfFormatArg {
    Turtle,
    Rdfxml,
    Jsonld,
    Ntriples,
}

impl From<RdfFormatArg> for RdfFormat {
    fn from(value: RdfFormatArg) -> Self {
        match value {
            RdfFormatArg::Turtle => RdfFormat::Turtle,
            RdfFormatArg::Rdfxml => RdfFormat::RdfXml,
            RdfFormatArg::Jsonld => RdfFormat::JsonLd,
            RdfFormatArg::Ntriples => RdfFormat::NTriples,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum SeverityArg {
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum ExternalModeArg {
    Off,
    Provided,
    Allowed,
}

impl From<ExternalModeArg> for ExternalValidationMode {
    fn from(value: ExternalModeArg) -> Self {
        match value {
            ExternalModeArg::Off => ExternalValidationMode::Off,
            ExternalModeArg::Provided => ExternalValidationMode::ProvidedOnly,
            ExternalModeArg::Allowed => ExternalValidationMode::ExternalAllowed,
        }
    }
}

impl From<SeverityArg> for Severity {
    fn from(value: SeverityArg) -> Self {
        match value {
            SeverityArg::Warning => Severity::Warning,
            SeverityArg::Error => Severity::Error,
        }
    }
}

#[derive(clap::Args)]
struct ValidateArgs {
    /// Path to an SBOL 3 document. Format is inferred from the extension —
    /// `.ttl` (Turtle), `.rdf` (RDF/XML), `.jsonld` (JSON-LD), or `.nt`
    /// (N-Triples).
    path: PathBuf,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    /// Destination for output. Use `-` for stdout.
    #[arg(long, default_value = "-")]
    output: String,

    /// Suppress diagnostics for these rule IDs (e.g. `--allow sbol3-10502`).
    #[arg(long = "allow", value_name = "RULE_ID")]
    allow: Vec<String>,

    /// Promote these rule IDs to error severity.
    #[arg(long = "deny", value_name = "RULE_ID")]
    deny: Vec<String>,

    /// Demote these rule IDs to warning severity.
    #[arg(long = "warn", value_name = "RULE_ID")]
    warn: Vec<String>,

    /// Floor on the severity of any emitted issue.
    #[arg(long, value_enum)]
    severity_floor: Option<SeverityArg>,

    /// Ceiling on the severity of any emitted issue.
    #[arg(long, value_enum)]
    severity_ceiling: Option<SeverityArg>,

    /// Treat warnings as errors (alias for `--severity-floor error`).
    #[arg(long)]
    treat_warnings_as_errors: bool,

    /// Use `Document::check_complete` semantics: any rule with partial
    /// coverage causes exit code 3.
    #[arg(long)]
    treat_partial_as_errors: bool,

    /// In text output, print a coverage summary after the issues.
    #[arg(long)]
    show_coverage: bool,

    /// Whether to resolve external documents and content.
    #[arg(long, value_enum, default_value_t = ExternalModeArg::Off)]
    external_mode: ExternalModeArg,

    /// Filesystem roots from which external Attachment / Model / TopLevel
    /// references may be resolved.
    #[arg(long = "resolve-documents", value_name = "DIR")]
    resolve_documents: Vec<PathBuf>,

    /// Filesystem roots for Attachment / Model byte content.
    #[arg(long = "resolve-content", value_name = "DIR")]
    resolve_content: Vec<PathBuf>,

    /// Cache directory required by `--external-mode allowed` when the
    /// `http-resolver` feature is built in.
    #[arg(long)]
    cache_dir: Option<PathBuf>,

    /// Layer an installed runtime ontology extension on top of the bundled
    /// facts for this validation run. Pass the cache entry name (e.g.
    /// `--ontology ncit`). Repeatable; later extensions override earlier
    /// ones on conflict.
    #[arg(long = "ontology", value_name = "NAME")]
    ontology: Vec<String>,
}

#[derive(clap::Args)]
struct ConvertArgs {
    /// Path to an SBOL 3 document. Input format is inferred from the
    /// extension — `.ttl` (Turtle), `.rdf` (RDF/XML), `.jsonld` (JSON-LD),
    /// or `.nt` (N-Triples).
    path: PathBuf,

    /// Target serialization. If omitted, inferred from `--output`'s
    /// extension.
    #[arg(long, value_enum, value_name = "FORMAT")]
    to: Option<RdfFormatArg>,

    /// Destination path. Use `-` (the default) for stdout; in that case
    /// `--to` is required.
    #[arg(long, short = 'o', default_value = "-")]
    output: String,
}

#[derive(clap::Args)]
struct RulesListArgs {
    /// Output format.
    #[arg(long, value_enum, default_value_t = RulesFormat::Text)]
    format: RulesFormat,

    /// Only show rules with this implementation status.
    #[arg(long, value_enum, value_name = "STATUS")]
    status: Option<RuleStatusFilter>,

    /// Show full notes instead of truncating to fit one line per rule.
    #[arg(long)]
    full: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let styles = Styles::resolve(cli.color);
    match cli.command {
        Command::Validate(args) => validate(args, styles),
        Command::Convert(args) => convert(args, styles),
        Command::Rules(command) => rules(command, styles),
        Command::Ontology(command) => ontology(command, styles),
    }
}

fn ontology(command: OntologyCommand, styles: Styles) -> ExitCode {
    let cache = OntologyCache::from_default_path();
    match command {
        OntologyCommand::Install(args) => ontology_install(&cache, args, styles),
        OntologyCommand::List => ontology_list(&cache, styles),
        OntologyCommand::Path => {
            println!("{}", cache.path().display());
            ExitCode::SUCCESS
        }
        OntologyCommand::Remove(args) => ontology_remove(&cache, args, styles),
        OntologyCommand::Verify(args) => ontology_verify(&cache, args, styles),
    }
}

fn known_ontology_by_name(name: &str) -> Option<KnownOntology> {
    match name.to_ascii_lowercase().as_str() {
        "ncit" => Some(KnownOntology::Ncit),
        _ => None,
    }
}

fn ontology_install(cache: &OntologyCache, args: OntologyInstallArgs, styles: Styles) -> ExitCode {
    let Some(known) = known_ontology_by_name(&args.name) else {
        eprintln!(
            "{}: unknown ontology `{}` — try one of: ncit",
            styles.err_label(),
            args.name
        );
        return ExitCode::from(2);
    };
    let descriptor: &OntologyDescriptor = known.descriptor();
    let result = if args.force {
        cache.install(descriptor)
    } else {
        cache.ensure_installed(descriptor)
    };
    match result {
        Ok(installed) => {
            println!(
                "{} `{}` from {}\n  fact sha256: {}",
                paint(styles.stdout, "1;32", "installed"),
                installed.name,
                installed.source_url,
                installed.fact_sha256,
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}: ontology install failed: {error}", styles.err_label());
            ExitCode::from(2)
        }
    }
}

fn ontology_list(cache: &OntologyCache, styles: Styles) -> ExitCode {
    match cache.list() {
        Ok(installed) => {
            if installed.is_empty() {
                println!("(no extensions installed)");
                return ExitCode::SUCCESS;
            }
            for entry in installed {
                println!(
                    "{name}\t{url}\tsha256={hash}\tinstalled_at={installed_at}",
                    name = entry.name,
                    url = entry.source_url,
                    hash = entry.fact_sha256,
                    installed_at = entry.installed_at,
                );
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}: cache list failed: {error}", styles.err_label());
            ExitCode::from(2)
        }
    }
}

fn ontology_remove(cache: &OntologyCache, args: OntologyRemoveArgs, styles: Styles) -> ExitCode {
    match cache.remove(&args.name) {
        Ok(true) => {
            println!(
                "{} `{}`",
                paint(styles.stdout, "1;32", "removed"),
                args.name
            );
            ExitCode::SUCCESS
        }
        Ok(false) => {
            eprintln!("{}: `{}` is not installed", styles.err_label(), args.name);
            ExitCode::from(2)
        }
        Err(error) => {
            eprintln!("{}: cache remove failed: {error}", styles.err_label());
            ExitCode::from(2)
        }
    }
}

fn ontology_verify(cache: &OntologyCache, args: OntologyVerifyArgs, styles: Styles) -> ExitCode {
    let names = match args.name {
        Some(name) => vec![name],
        None => match cache.list() {
            Ok(installed) => installed.into_iter().map(|m| m.name).collect(),
            Err(error) => {
                eprintln!("{}: cache list failed: {error}", styles.err_label());
                return ExitCode::from(2);
            }
        },
    };
    if names.is_empty() {
        println!("(no extensions installed)");
        return ExitCode::SUCCESS;
    }
    let mut had_failure = false;
    for name in names {
        match cache.verify(&name) {
            Ok(_) => println!("{}\t{name}", paint(styles.stdout, "32", "ok")),
            Err(error) => {
                eprintln!("{}\t{name}: {error}", paint(styles.stderr, "1;31", "FAIL"));
                had_failure = true;
            }
        }
    }
    if had_failure {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn validate(args: ValidateArgs, styles: Styles) -> ExitCode {
    let options = match build_options(&args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{}: {message}", styles.err_label());
            return ExitCode::from(2);
        }
    };

    if args.external_mode == ExternalModeArg::Allowed && !cfg!(feature = "http-resolver") {
        eprintln!(
            "{}: --external-mode allowed requires the `http-resolver` feature \
             (rebuild sbol-cli with --features http-resolver)",
            styles.err_label()
        );
        return ExitCode::from(2);
    }
    if args.external_mode == ExternalModeArg::Allowed && args.cache_dir.is_none() {
        eprintln!(
            "{}: --external-mode allowed requires --cache-dir (so HTTP fetches stay deterministic)",
            styles.err_label()
        );
        return ExitCode::from(2);
    }

    let document = match read_document(&args.path, styles) {
        Ok(document) => document,
        Err(code) => return code,
    };

    let document_resolver = build_document_resolver(&args);
    let content_resolver = build_content_resolver(&args);
    #[cfg(feature = "http-resolver")]
    let caching_http = args
        .cache_dir
        .as_ref()
        .filter(|_| args.external_mode == ExternalModeArg::Allowed)
        .map(|dir| CachingHttpResolver::new(dir.clone()));

    let mut context =
        ValidationContext::with_options(options).with_external_mode(args.external_mode.into());
    if let Some(resolver) = &document_resolver {
        context = context.with_document_resolver(resolver);
    }
    if let Some(resolver) = &content_resolver {
        context = context.with_content_resolver(resolver);
    }
    #[cfg(feature = "http-resolver")]
    if let Some(resolver) = &caching_http {
        let doc_ref: &dyn sbol::DocumentResolver = resolver;
        let content_ref: &dyn sbol::ContentResolver = resolver;
        context = context
            .with_content_resolver(content_ref)
            .with_document_resolver(doc_ref);
    }

    let report = document.validate_with_context(context);

    if let Err(message) = render_output(&args, &report, styles) {
        eprintln!("{}: failed to write output: {message}", styles.err_label());
        return ExitCode::from(2);
    }

    let has_errors = report.has_errors();
    let has_partial = !report.coverage().partially_applied.is_empty();
    if has_errors {
        ExitCode::from(1)
    } else if args.treat_partial_as_errors && has_partial {
        ExitCode::from(3)
    } else {
        ExitCode::SUCCESS
    }
}

fn build_document_resolver(args: &ValidateArgs) -> Option<FileResolver> {
    if args.resolve_documents.is_empty() {
        return None;
    }
    let mut resolver = FileResolver::new();
    for root in &args.resolve_documents {
        resolver.add_root(root.clone());
    }
    Some(resolver)
}

fn build_content_resolver(args: &ValidateArgs) -> Option<FileResolver> {
    if args.resolve_content.is_empty() {
        return None;
    }
    let mut resolver = FileResolver::new();
    for root in &args.resolve_content {
        resolver.add_root(root.clone());
    }
    Some(resolver)
}

fn build_options(args: &ValidateArgs) -> Result<ValidationOptions, String> {
    let mut options = ValidationOptions::default();
    let mut configured: BTreeSet<&str> = BTreeSet::new();

    for rule in &args.allow {
        check_first_use(&mut configured, rule)?;
        options = options.allow(rule).map_err(|err| err.to_string())?;
    }
    for rule in &args.deny {
        check_first_use(&mut configured, rule)?;
        options = options.deny(rule).map_err(|err| err.to_string())?;
    }
    for rule in &args.warn {
        check_first_use(&mut configured, rule)?;
        options = options.warn(rule).map_err(|err| err.to_string())?;
    }

    if let Some(floor) = args.severity_floor {
        options = options.with_severity_floor(floor.into());
    }
    if let Some(ceiling) = args.severity_ceiling {
        options = options.with_severity_ceiling(ceiling.into());
    }
    if args.treat_warnings_as_errors {
        options = options.with_severity_floor(Severity::Error);
    }

    if !args.ontology.is_empty() {
        let cache = OntologyCache::from_default_path();
        for name in &args.ontology {
            let extension = cache.load(name).map_err(|error| {
                if known_ontology_by_name(name).is_some() {
                    format!(
                        "failed to load ontology extension `{name}` from {}: {error}. \
                         Install it first with `sbol ontology install {name}`.",
                        cache.path().display(),
                    )
                } else {
                    format!(
                        "unknown ontology extension `{name}` — try one of: ncit \
                         (run `sbol ontology install <name>` to install it)"
                    )
                }
            })?;
            options = options.with_ontology_extension(extension);
        }
    }
    Ok(options)
}

fn check_first_use<'a>(configured: &mut BTreeSet<&'a str>, rule: &'a str) -> Result<(), String> {
    if !configured.insert(rule) {
        return Err(format!(
            "rule `{rule}` is given more than one override on the command line"
        ));
    }
    Ok(())
}

fn render_output(args: &ValidateArgs, report: &ValidationReport, styles: Styles) -> io::Result<()> {
    let writing_to_stdout = args.output == "-";
    let payload = match args.format {
        OutputFormat::Text => {
            let color = styles.stdout && writing_to_stdout;
            format_text(args, report, color)
        }
        OutputFormat::Json => sbol::to_json(report),
        #[cfg(feature = "sarif")]
        OutputFormat::Sarif => sarif::to_sarif(report, &args.path),
    };

    if writing_to_stdout {
        let mut stdout = io::stdout().lock();
        stdout.write_all(payload.as_bytes())?;
        if !payload.ends_with('\n') {
            stdout.write_all(b"\n")?;
        }
        Ok(())
    } else {
        fs::write(&args.output, payload)
    }
}

fn format_text(args: &ValidateArgs, report: &ValidationReport, color: bool) -> String {
    let mut out = String::new();
    for issue in report.issues() {
        out.push_str(&format_issue(issue, &args.path, color));
        out.push('\n');
    }

    let errors = report.errors().count();
    let warnings = report.warnings().count();
    out.push_str(&format!(
        "{}: {errors} error{}, {warnings} warning{}",
        args.path.display(),
        plural(errors),
        plural(warnings),
    ));
    if errors == 0 && warnings == 0 {
        out.push_str(&format!(" {}", paint(color, "1;32", "— OK")));
    }
    out.push('\n');

    if args.show_coverage {
        let coverage = report.coverage();
        let line = format!(
            "coverage: {} fully applied, {} partially applied, {} not applied\n",
            coverage.fully_applied.len(),
            coverage.partially_applied.len(),
            coverage.not_applied.len(),
        );
        out.push_str(&paint(color, "2", &line));
    }

    out
}

fn format_issue(issue: &ValidationIssue, path: &Path, color: bool) -> String {
    let severity_label = match issue.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        _ => "issue",
    };
    let severity_painted = match severity_code(issue.severity) {
        Some(code) => paint(color, code, severity_label),
        None => severity_label.to_string(),
    };
    let property = issue
        .property
        .map(|property| format!(" <{property}>"))
        .unwrap_or_default();
    format!(
        "{}: {severity_painted}[{}] [{}]{property}: {}",
        path.display(),
        issue.rule,
        issue.subject,
        issue.message,
    )
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn read_document(path: &Path, styles: Styles) -> Result<Document, ExitCode> {
    match Document::read_path(path) {
        Ok(document) => Ok(document),
        Err(ReadError::Io { source, .. }) => {
            eprintln!(
                "{}: failed to read {}: {source}",
                styles.err_label(),
                path.display()
            );
            Err(ExitCode::from(2))
        }
        Err(ReadError::UnknownFormat { extension, .. }) => {
            let ext = extension.as_deref().unwrap_or("<none>");
            eprintln!(
                "{}: unsupported extension `{ext}` for {} — supported: .ttl, .rdf, .jsonld, .nt",
                styles.err_label(),
                path.display()
            );
            Err(ExitCode::from(2))
        }
        Err(error) => {
            eprintln!(
                "{}: failed to parse {}: {error}",
                styles.err_label(),
                path.display()
            );
            Err(ExitCode::from(2))
        }
    }
}

fn convert(args: ConvertArgs, styles: Styles) -> ExitCode {
    let writing_to_stdout = args.output == "-";
    let target_format = match args.to {
        Some(format) => RdfFormat::from(format),
        None => {
            if writing_to_stdout {
                eprintln!(
                    "{}: --to is required when writing to stdout; \
                     pass --to <FORMAT> or --output <PATH>",
                    styles.err_label()
                );
                return ExitCode::from(2);
            }
            match RdfFormat::from_path(Path::new(&args.output)) {
                Some(format) => format,
                None => {
                    eprintln!(
                        "{}: cannot infer target format from `{}` — pass --to <FORMAT> \
                         (one of: turtle, rdfxml, jsonld, ntriples)",
                        styles.err_label(),
                        args.output
                    );
                    return ExitCode::from(2);
                }
            }
        }
    };

    let document = match read_document(&args.path, styles) {
        Ok(document) => document,
        Err(code) => return code,
    };

    let payload = match document.write(target_format) {
        Ok(payload) => payload,
        Err(WriteError::Io { source, .. }) => {
            eprintln!(
                "{}: failed to serialize as {}: {source}",
                styles.err_label(),
                target_format
            );
            return ExitCode::from(2);
        }
        Err(error) => {
            eprintln!(
                "{}: failed to serialize as {}: {error}",
                styles.err_label(),
                target_format
            );
            return ExitCode::from(2);
        }
    };

    if writing_to_stdout {
        let mut stdout = io::stdout().lock();
        if let Err(error) = stdout.write_all(payload.as_bytes()) {
            eprintln!("{}: failed to write output: {error}", styles.err_label());
            return ExitCode::from(2);
        }
        if !payload.ends_with('\n') && stdout.write_all(b"\n").is_err() {
            return ExitCode::from(2);
        }
    } else if let Err(error) = fs::write(&args.output, payload) {
        eprintln!(
            "{}: failed to write {}: {error}",
            styles.err_label(),
            args.output
        );
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}

fn rules(command: RulesCommand, styles: Styles) -> ExitCode {
    match command {
        RulesCommand::List(args) => rules_list(args, styles),
    }
}

fn rules_list(args: RulesListArgs, styles: Styles) -> ExitCode {
    let statuses: Vec<&ValidationRuleStatus> = validation_rule_statuses()
        .iter()
        .filter(|status| status_matches_filter(status.status, args.status))
        .collect();

    match args.format {
        RulesFormat::Text => {
            print!("{}", format_rules_text(&statuses, styles.stdout, args.full));
        }
        RulesFormat::Json => {
            let payload = format_rules_json(&statuses);
            println!("{payload}");
        }
    }
    ExitCode::SUCCESS
}

fn status_matches_filter(status: RuleStatus, filter: Option<RuleStatusFilter>) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    matches!(
        (filter, status),
        (RuleStatusFilter::Error, RuleStatus::Error)
            | (RuleStatusFilter::Warning, RuleStatus::Warning)
            | (RuleStatusFilter::Configurable, RuleStatus::Configurable)
            | (
                RuleStatusFilter::MachineUncheckable,
                RuleStatus::MachineUncheckable,
            )
            | (RuleStatusFilter::Unimplemented, RuleStatus::Unimplemented)
    )
}

const FALLBACK_TERMINAL_COLS: usize = 100;
const COLUMN_SEPARATOR_WIDTH: usize = 2;
const MIN_NOTE_WIDTH: usize = 10;

fn format_rules_text(statuses: &[&ValidationRuleStatus], color: bool, full: bool) -> String {
    if statuses.is_empty() {
        return String::from("(no rules match)\n");
    }

    let rule_w = column_width("rule", statuses.iter().map(|s| s.rule));
    let status_w = column_width(
        "status",
        statuses.iter().map(|s| rule_status_label(s.status)),
    );
    let normative_w = column_width(
        "normative",
        statuses
            .iter()
            .map(|s| normative_severity_label(s.normative_severity)),
    );
    let section_w = column_width("section", statuses.iter().map(|s| s.spec_section));
    let blocker_w = column_width(
        "blocker",
        statuses
            .iter()
            .map(|s| s.blocker.map(blocker_label).unwrap_or("-")),
    );

    let note_truncate = if full {
        None
    } else {
        let fixed =
            rule_w + status_w + normative_w + section_w + blocker_w + COLUMN_SEPARATOR_WIDTH * 5;
        let total = detect_terminal_cols();
        let remaining = total.saturating_sub(fixed);
        Some(remaining.max(MIN_NOTE_WIDTH))
    };

    let mut out = String::new();
    let header = format!(
        "{rule:<rule_w$}  {status:<status_w$}  {normative:<normative_w$}  {section:<section_w$}  {blocker:<blocker_w$}  {note}\n",
        rule = "rule",
        status = "status",
        normative = "normative",
        section = "section",
        blocker = "blocker",
        note = "note",
    );
    out.push_str(&paint(color, "1", &header));

    let mut counts = StatusCounts::default();
    for status in statuses {
        counts.tally(status.status);
        let status_label = rule_status_label(status.status);
        let status_col = paint_padded(
            status_label,
            status_w,
            rule_status_code(status.status).filter(|_| color),
        );
        let blocker = status.blocker.map(blocker_label).unwrap_or("-");
        let note = match note_truncate {
            Some(max) => truncate(status.note, max),
            None => status.note.to_string(),
        };
        out.push_str(&format!(
            "{rule:<rule_w$}  {status_col}  {normative:<normative_w$}  {section:<section_w$}  {blocker:<blocker_w$}  {note}\n",
            rule = status.rule,
            normative = normative_severity_label(status.normative_severity),
            section = status.spec_section,
        ));
    }

    let summary = format!("\n{} rules{}\n", statuses.len(), counts.summary());
    out.push_str(&paint(color, "2", &summary));
    out
}

fn detect_terminal_cols() -> usize {
    if let Some((width, _)) = terminal_size::terminal_size() {
        return width.0 as usize;
    }
    if let Some(cols) = env::var("COLUMNS").ok().and_then(|s| s.parse().ok()) {
        return cols;
    }
    FALLBACK_TERMINAL_COLS
}

#[derive(Default)]
struct StatusCounts {
    error: usize,
    warning: usize,
    configurable: usize,
    machine_uncheckable: usize,
    unimplemented: usize,
}

impl StatusCounts {
    fn tally(&mut self, status: RuleStatus) {
        match status {
            RuleStatus::Error => self.error += 1,
            RuleStatus::Warning => self.warning += 1,
            RuleStatus::Configurable => self.configurable += 1,
            RuleStatus::MachineUncheckable => self.machine_uncheckable += 1,
            RuleStatus::Unimplemented => self.unimplemented += 1,
            _ => {}
        }
    }

    fn summary(&self) -> String {
        let parts: Vec<String> = [
            ("Error", self.error),
            ("Warning", self.warning),
            ("Configurable", self.configurable),
            ("MachineUncheckable", self.machine_uncheckable),
            ("Unimplemented", self.unimplemented),
        ]
        .into_iter()
        .filter(|(_, n)| *n > 0)
        .map(|(label, n)| format!("{n} {label}"))
        .collect();
        if parts.is_empty() {
            String::new()
        } else {
            format!(" — {}", parts.join(", "))
        }
    }
}

fn column_width<'a>(header: &str, values: impl Iterator<Item = &'a str>) -> usize {
    let mut width = header.chars().count();
    for value in values {
        let n = value.chars().count();
        if n > width {
            width = n;
        }
    }
    width
}

fn paint_padded(label: &str, width: usize, code: Option<&str>) -> String {
    let pad = width.saturating_sub(label.chars().count());
    let painted = match code {
        Some(code) => paint(true, code, label),
        None => label.to_string(),
    };
    format!("{painted}{}", " ".repeat(pad))
}

fn truncate(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        text.to_string()
    } else {
        let mut out: String = text.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn format_rules_json(statuses: &[&ValidationRuleStatus]) -> String {
    let entries: Vec<Value> = statuses
        .iter()
        .map(|status| {
            json!({
                "rule": status.rule,
                "status": rule_status_label(status.status),
                "normative_severity": normative_severity_label(status.normative_severity),
                "spec_section": status.spec_section,
                "blocker": status.blocker.map(blocker_label),
                "note": status.note,
                "validator_function": status.validator_function,
            })
        })
        .collect();
    serde_json::to_string(&Value::Array(entries)).expect("rule-catalog JSON is always serializable")
}

fn rule_status_label(status: RuleStatus) -> &'static str {
    match status {
        RuleStatus::Error => "Error",
        RuleStatus::Warning => "Warning",
        RuleStatus::Configurable => "Configurable",
        RuleStatus::MachineUncheckable => "MachineUncheckable",
        RuleStatus::Unimplemented => "Unimplemented",
        _ => "Unknown",
    }
}

fn normative_severity_label(severity: NormativeSeverity) -> &'static str {
    match severity {
        NormativeSeverity::Must => "MUST",
        NormativeSeverity::Should => "SHOULD",
        NormativeSeverity::May => "MAY",
        _ => "UNKNOWN",
    }
}

fn blocker_label(blocker: Blocker) -> &'static str {
    match blocker {
        Blocker::Ontology => "Ontology",
        Blocker::Resolver => "Resolver",
        Blocker::StrictDatatype => "StrictDatatype",
        Blocker::Policy => "Policy",
        Blocker::External => "External",
        _ => "Unknown",
    }
}
