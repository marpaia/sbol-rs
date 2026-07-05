use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use sbol::convert::{DowngradeOptions, DowngradeWarning};
use sbol::v3::RdfFormat;

use crate::cli::DowngradeArgs;
use crate::output::{format_issue, infer_conversion_rdf_format, read_document_with_format};
use crate::style::Styles;

pub(crate) fn downgrade(args: DowngradeArgs, styles: Styles) -> ExitCode {
    let writing_to_stdout = args.output == "-";
    let input_format = match args.from {
        Some(format) => RdfFormat::from(format),
        None => match infer_conversion_rdf_format(&args.path) {
            Some(format) => format,
            None => {
                let ext = args
                    .path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("<none>");
                eprintln!(
                    "{}: unsupported extension `{ext}` for {} — pass --from <FORMAT> \
                     (one of: turtle, rdfxml, jsonld, ntriples)",
                    styles.err_label(),
                    args.path.display()
                );
                return ExitCode::from(2);
            }
        },
    };
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
            match infer_conversion_rdf_format(Path::new(&args.output)) {
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

    let document = match read_document_with_format(&args.path, input_format, styles) {
        Ok(doc) => doc,
        Err(code) => return code,
    };

    let mut options = DowngradeOptions::default();
    options.default_version = args.default_version;
    let (sbol2_graph, report) = match sbol::convert::downgrade_with(&document, options) {
        Ok(pair) => pair,
        Err(err) => {
            eprintln!("{}: downgrade failed: {err}", styles.err_label());
            return ExitCode::from(2);
        }
    };

    let payload = match sbol2_graph.write(target_format) {
        Ok(payload) => payload,
        Err(err) => {
            eprintln!(
                "{}: failed to serialize as {target_format}: {err}",
                styles.err_label()
            );
            return ExitCode::from(2);
        }
    };

    if writing_to_stdout {
        let mut stdout = io::stdout().lock();
        if let Err(err) = stdout.write_all(payload.as_bytes()) {
            eprintln!("{}: failed to write output: {err}", styles.err_label());
            return ExitCode::from(2);
        }
        if !payload.ends_with('\n') && stdout.write_all(b"\n").is_err() {
            return ExitCode::from(2);
        }
    } else if let Err(err) = fs::write(&args.output, payload) {
        eprintln!(
            "{}: failed to write {}: {err}",
            styles.err_label(),
            args.output
        );
        return ExitCode::from(2);
    }

    // Print the conversion summary to stderr — same style as upgrade.
    let counts = report.counts();
    eprintln!(
        "downgraded: {} CD, {} MD, {} split-into-both, {} SubComponent, \
         {} SequenceFeature, {} MapsTo, {} backport-restored, {} synthesized{}",
        counts.components_to_component_definition,
        counts.components_to_module_definition,
        counts.components_split_into_both,
        counts.sub_components_emitted,
        counts.sequence_features_emitted,
        counts.maps_to_reconstructed,
        counts.identities_restored_from_backport,
        counts.identities_synthesized,
        if report.warnings().is_empty() {
            String::new()
        } else {
            format!(", {} warning(s)", report.warnings().len())
        }
    );
    for warning in report.warnings() {
        eprintln!("  warning: {}", format_downgrade_warning(warning));
    }

    if args.strict && !report.is_clean() {
        return ExitCode::from(1);
    }

    if args.validate {
        // Round-trip: upgrade the produced SBOL 2 back to SBOL 3,
        // then run the SBOL 3 validator. This is the closest thing
        // we have to an SBOL 2 validator without bundling one.
        let sbol2_text = match sbol2_graph.write(RdfFormat::Turtle) {
            Ok(t) => t,
            Err(err) => {
                eprintln!(
                    "{}: round-trip serialization failed: {err}",
                    styles.err_label()
                );
                return ExitCode::from(2);
            }
        };
        match sbol::convert::upgrade_from_sbol2(&sbol2_text, RdfFormat::Turtle) {
            Ok((re_upgraded, _)) => {
                let validation = re_upgraded.validate();
                if validation.has_errors() {
                    for issue in validation.issues() {
                        eprintln!("{}", format_issue(issue, &args.path, styles.stderr));
                    }
                    return ExitCode::from(1);
                }
            }
            Err(err) => {
                eprintln!(
                    "{}: --validate round-trip failed at the re-upgrade step: {err}",
                    styles.err_label()
                );
                return ExitCode::from(1);
            }
        }
    }
    ExitCode::SUCCESS
}

fn format_downgrade_warning(warning: &DowngradeWarning) -> String {
    match warning {
        DowngradeWarning::DualRoleComponent {
            component,
            component_definition,
            module_definition,
        } => format!(
            "Component <{component}> carries both structure and function; \
             split into ComponentDefinition <{component_definition}> + \
             ModuleDefinition <{module_definition}>"
        ),
        DowngradeWarning::UnresolvableConstraintToMapsTo { constraint, reason } => {
            format!("Constraint <{constraint}> couldn't fold back into a MapsTo: {reason}")
        }
        DowngradeWarning::OrphanComponentReference {
            component_reference,
        } => format!(
            "ComponentReference <{component_reference}> had no matching Constraint — dropped"
        ),
        DowngradeWarning::UnsupportedSbol3Type {
            subject,
            sbol3_type,
        } => format!(
            "subject <{subject}> has SBOL 3 type <{sbol3_type}> with no SBOL 2 equivalent — dropped"
        ),
        DowngradeWarning::SynthesizedVersion { subject, version } => format!(
            "subject <{subject}> had no backport version; synthesized version \"{version}\""
        ),
        DowngradeWarning::IdentityCollision { canonical, sources } => format!(
            "{} distinct SBOL 3 subjects rewrite to <{canonical}>; the SBOL 2 output \
             merges their triples into a single subject. Sources: {}",
            sources.len(),
            sources
                .iter()
                .map(|s| format!("<{s}>"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => format!("downgrade warning: {other:?}"),
    }
}
