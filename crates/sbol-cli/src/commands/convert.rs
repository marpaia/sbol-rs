use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use sbol::v3::{RdfFormat, WriteError};

use crate::cli::ConvertArgs;
use crate::output::read_document;
use crate::style::Styles;

pub(crate) fn convert(args: ConvertArgs, styles: Styles) -> ExitCode {
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
