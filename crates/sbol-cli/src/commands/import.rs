use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use sbol::RdfFormat;

use crate::cli::{ImportFastaArgs, ImportGenbankArgs};
use crate::output::format_issue;
use crate::style::Styles;

pub(crate) fn import_genbank(args: ImportGenbankArgs, styles: Styles) -> ExitCode {
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

    let importer = match sbol_genbank::GenbankImporter::new(&args.namespace) {
        Ok(importer) => importer,
        Err(err) => {
            eprintln!(
                "{}: invalid --namespace `{}`: {err}",
                styles.err_label(),
                args.namespace
            );
            return ExitCode::from(2);
        }
    };

    let (document, report) = match importer.read_path(&args.path) {
        Ok(pair) => pair,
        Err(err) => {
            eprintln!(
                "{}: failed to import {}: {err}",
                styles.err_label(),
                args.path.display()
            );
            return ExitCode::from(2);
        }
    };

    let payload = match document.write(target_format) {
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

    // Always print the import summary to stderr — it's the user's
    // signal that the conversion actually picked up the right number of
    // Components, Sequences, and Features. Mirrors the `sbol upgrade`
    // summary line.
    eprintln!(
        "imported: {} Component(s), {} Sequence(s), {} SequenceFeature(s){}",
        report.components,
        report.sequences,
        report.features,
        if report.warnings.is_empty() {
            String::new()
        } else {
            format!(", {} warning(s)", report.warnings.len())
        }
    );
    for warning in &report.warnings {
        eprintln!("  warning: {}", format_import_warning(warning));
    }

    if args.strict && !report.is_clean() {
        return ExitCode::from(1);
    }

    if args.validate {
        let validation = document.validate();
        if validation.has_errors() {
            for issue in validation.issues() {
                eprintln!("{}", format_issue(issue, &args.path, styles.stderr));
            }
            return ExitCode::from(1);
        }
    }
    ExitCode::SUCCESS
}

pub(crate) fn import_fasta(args: ImportFastaArgs, styles: Styles) -> ExitCode {
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

    let mut importer = match sbol_fasta::FastaImporter::new(&args.namespace) {
        Ok(importer) => importer,
        Err(err) => {
            eprintln!(
                "{}: invalid --namespace `{}`: {err}",
                styles.err_label(),
                args.namespace
            );
            return ExitCode::from(2);
        }
    };
    if let Some(alphabet) = args.alphabet {
        importer = importer.with_alphabet(alphabet.into());
    }

    let (document, report) = match importer.read_path(&args.path) {
        Ok(pair) => pair,
        Err(err) => {
            eprintln!(
                "{}: failed to import {}: {err}",
                styles.err_label(),
                args.path.display()
            );
            return ExitCode::from(2);
        }
    };

    let payload = match document.write(target_format) {
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

    eprintln!(
        "imported: {} Component(s), {} Sequence(s) ({} DNA, {} RNA, {} protein){}",
        report.components,
        report.sequences,
        report.dna_records,
        report.rna_records,
        report.protein_records,
        if report.warnings.is_empty() {
            String::new()
        } else {
            format!(", {} warning(s)", report.warnings.len())
        }
    );
    for warning in &report.warnings {
        eprintln!("  warning: {}", format_fasta_import_warning(warning));
    }

    if args.strict && !report.is_clean() {
        return ExitCode::from(1);
    }

    if args.validate {
        let validation = document.validate();
        if validation.has_errors() {
            for issue in validation.issues() {
                eprintln!("{}", format_issue(issue, &args.path, styles.stderr));
            }
            return ExitCode::from(1);
        }
    }
    ExitCode::SUCCESS
}

fn format_fasta_import_warning(warning: &sbol_fasta::ImportWarning) -> String {
    match warning {
        sbol_fasta::ImportWarning::EmptyRecord { record_id } => {
            format!("record `{record_id}` has no sequence body")
        }
        _ => "unrecognized fasta import warning".to_string(),
    }
}

fn format_import_warning(warning: &sbol_genbank::ImportWarning) -> String {
    match warning {
        sbol_genbank::ImportWarning::UnknownFeatureKey { kind } => {
            format!("unrecognized GenBank feature key `{kind}` — fell back to SO:0000110")
        }
        sbol_genbank::ImportWarning::LossyLocation { feature, reason } => {
            format!("feature `{feature}`: lossy location — {reason}")
        }
        sbol_genbank::ImportWarning::SynthesizedIdentifier => {
            "GenBank record had no ACCESSION or LOCUS name; synthesized `imported_record`"
                .to_string()
        }
        _ => "unrecognized import warning".to_string(),
    }
}
