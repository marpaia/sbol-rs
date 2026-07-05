use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::process::ExitCode;

use sbol::v3::{FileResolver, Severity, ValidationContext, ValidationOptions, ValidationReport};
use sbol_ontology::OntologyCache;

#[cfg(feature = "http-resolver")]
use sbol::v3::CachingHttpResolver;

use crate::cli::{ExternalModeArg, OutputFormat, ValidateArgs};
use crate::commands::ontology::known_ontology_by_name;
use crate::output::{format_issue, plural, read_document};
use crate::style::{Styles, paint};

pub(crate) fn validate(args: ValidateArgs, styles: Styles) -> ExitCode {
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
        let doc_ref: &dyn sbol::v3::DocumentResolver = resolver;
        let content_ref: &dyn sbol::v3::ContentResolver = resolver;
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
        OutputFormat::Json => sbol::v3::to_json(report),
        #[cfg(feature = "sarif")]
        OutputFormat::Sarif => crate::sarif::to_sarif(report, &args.path),
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
