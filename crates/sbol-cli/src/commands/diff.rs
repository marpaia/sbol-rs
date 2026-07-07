use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use sbol::v3::{Diff, Document, ObjectDiff, RdfFormat, Term};
use sbol::{SbolVersion, detect_version};
use serde_json::{Map, Value, json};

use crate::cli::{DiffArgs, DiffFormat, SbolVersionArg};
use crate::output::infer_conversion_rdf_format;
use crate::style::{Styles, paint};

pub(crate) fn diff(args: DiffArgs, styles: Styles) -> ExitCode {
    let old = match read_input(&args.old, styles) {
        Ok(input) => input,
        Err(code) => return code,
    };
    let new = match read_input(&args.new, styles) {
        Ok(input) => input,
        Err(code) => return code,
    };

    let version = match resolve_version(&args, &old, &new, styles) {
        Ok(version) => version,
        Err(code) => return code,
    };

    let diff = match compute(version, &old, &new, styles) {
        Ok(diff) => diff,
        Err(code) => return code,
    };

    let writing_to_stdout = args.output == "-";
    let payload = match args.format {
        DiffFormat::Text => render_text(&diff, styles.stdout && writing_to_stdout),
        DiffFormat::Json => render_json(&diff),
    };

    if let Err(code) = write_output(&args.output, &payload, styles) {
        return code;
    }

    if args.exit_code && !diff.is_empty() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// A document read into memory alongside the RDF format inferred from its path.
struct Input {
    path: std::path::PathBuf,
    text: String,
    format: RdfFormat,
}

fn read_input(path: &Path, styles: Styles) -> Result<Input, ExitCode> {
    let Some(format) = infer_conversion_rdf_format(path) else {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("<none>");
        eprintln!(
            "{}: unsupported extension `{ext}` for {} — supported: .ttl, .rdf, .xml, .jsonld, .nt",
            styles.err_label(),
            path.display()
        );
        return Err(ExitCode::from(2));
    };
    let text = fs::read_to_string(path).map_err(|source| {
        eprintln!(
            "{}: failed to read {}: {source}",
            styles.err_label(),
            path.display()
        );
        ExitCode::from(2)
    })?;
    Ok(Input {
        path: path.to_path_buf(),
        text,
        format,
    })
}

/// Picks the SBOL version to read both documents as. An explicit
/// `--sbol-version` wins; otherwise each document's RDF namespaces decide and
/// the two must agree, since a cross-version comparison is not meaningful.
fn resolve_version(
    args: &DiffArgs,
    old: &Input,
    new: &Input,
    styles: Styles,
) -> Result<SbolVersion, ExitCode> {
    match args.sbol_version {
        SbolVersionArg::V2 => Ok(SbolVersion::V2),
        SbolVersionArg::V3 => Ok(SbolVersion::V3),
        SbolVersionArg::Auto => {
            let old_version = detect_version(&old.text, old.format);
            let new_version = detect_version(&new.text, new.format);
            match (old_version, new_version) {
                (Some(a), Some(b)) if a == b => Ok(a),
                (Some(a), Some(b)) => {
                    eprintln!(
                        "{}: {} is {} and {} is {} — diff compares documents of one version. \
                         Upgrade the SBOL 2 document with `sbol upgrade` and diff the SBOL 3 pair.",
                        styles.err_label(),
                        old.path.display(),
                        version_name(a),
                        new.path.display(),
                        version_name(b),
                    );
                    Err(ExitCode::from(2))
                }
                // Fall back to SBOL 3 when neither carries an SBOL namespace,
                // so its parser reports the more informative error.
                _ => Ok(old_version.or(new_version).unwrap_or(SbolVersion::V3)),
            }
        }
    }
}

fn version_name(version: SbolVersion) -> &'static str {
    match version {
        SbolVersion::V2 => "SBOL 2",
        _ => "SBOL 3",
    }
}

fn compute(
    version: SbolVersion,
    old: &Input,
    new: &Input,
    styles: Styles,
) -> Result<Diff, ExitCode> {
    match version {
        SbolVersion::V2 => {
            let old_doc = read_v2(old, styles)?;
            let new_doc = read_v2(new, styles)?;
            Ok(old_doc.diff(&new_doc))
        }
        _ => {
            let old_doc = read_v3(old, styles)?;
            let new_doc = read_v3(new, styles)?;
            Ok(old_doc.diff(&new_doc))
        }
    }
}

fn read_v3(input: &Input, styles: Styles) -> Result<Document, ExitCode> {
    Document::read(&input.text, input.format).map_err(|error| {
        eprintln!(
            "{}: failed to parse {} as {}: {error}",
            styles.err_label(),
            input.path.display(),
            input.format
        );
        ExitCode::from(2)
    })
}

fn read_v2(input: &Input, styles: Styles) -> Result<sbol::v2::Document, ExitCode> {
    sbol::v2::Document::read(&input.text, input.format).map_err(|error| {
        eprintln!(
            "{}: failed to parse {} as {}: {error}",
            styles.err_label(),
            input.path.display(),
            input.format
        );
        ExitCode::from(2)
    })
}

fn write_output(output: &str, payload: &str, styles: Styles) -> Result<(), ExitCode> {
    if output == "-" {
        let mut stdout = io::stdout().lock();
        stdout.write_all(payload.as_bytes()).map_err(|error| {
            eprintln!("{}: failed to write output: {error}", styles.err_label());
            ExitCode::from(2)
        })?;
        if !payload.ends_with('\n') {
            let _ = stdout.write_all(b"\n");
        }
        Ok(())
    } else {
        fs::write(output, payload).map_err(|error| {
            eprintln!("{}: failed to write {output}: {error}", styles.err_label());
            ExitCode::from(2)
        })
    }
}

fn render_text(diff: &Diff, color: bool) -> String {
    if diff.is_empty() {
        return "no differences\n".to_string();
    }

    let mut out = String::new();
    for identity in diff.added() {
        out.push_str(&paint(color, "32", &format!("+ {identity}")));
        out.push('\n');
    }
    for identity in diff.removed() {
        out.push_str(&paint(color, "31", &format!("- {identity}")));
        out.push('\n');
    }
    for object in diff.changed() {
        render_object_text(object, color, &mut out);
    }

    out.push_str(&format!(
        "{} added, {} removed, {} changed\n",
        diff.added().len(),
        diff.removed().len(),
        diff.changed().len(),
    ));
    out
}

fn render_object_text(object: &ObjectDiff, color: bool, out: &mut String) {
    out.push_str(&paint(color, "33", &format!("~ {}", object.identity())));
    out.push('\n');
    for iri in object.types_added() {
        out.push_str(&paint(color, "32", &format!("    + type {iri}")));
        out.push('\n');
    }
    for iri in object.types_removed() {
        out.push_str(&paint(color, "31", &format!("    - type {iri}")));
        out.push('\n');
    }
    for (predicate, change) in object.properties() {
        out.push_str(&format!("    {predicate}\n"));
        for term in change.added() {
            out.push_str(&paint(
                color,
                "32",
                &format!("        + {}", display_term(term)),
            ));
            out.push('\n');
        }
        for term in change.removed() {
            out.push_str(&paint(
                color,
                "31",
                &format!("        - {}", display_term(term)),
            ));
            out.push('\n');
        }
    }
}

fn render_json(diff: &Diff) -> String {
    let added: Vec<Value> = diff.added().iter().map(|r| json!(r.to_string())).collect();
    let removed: Vec<Value> = diff
        .removed()
        .iter()
        .map(|r| json!(r.to_string()))
        .collect();
    let changed: Vec<Value> = diff
        .changed()
        .iter()
        .map(|object| {
            let mut properties = Map::new();
            for (predicate, change) in object.properties() {
                properties.insert(
                    predicate.to_string(),
                    json!({
                        "added": change.added().iter().map(term_json).collect::<Vec<_>>(),
                        "removed": change.removed().iter().map(term_json).collect::<Vec<_>>(),
                    }),
                );
            }
            json!({
                "identity": object.identity().to_string(),
                "typesAdded": object.types_added().iter().map(|t| t.to_string()).collect::<Vec<_>>(),
                "typesRemoved": object.types_removed().iter().map(|t| t.to_string()).collect::<Vec<_>>(),
                "properties": Value::Object(properties),
            })
        })
        .collect();

    let value = json!({
        "added": added,
        "removed": removed,
        "changed": changed,
    });
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
}

fn term_json(term: &Term) -> Value {
    match term {
        Term::Resource(resource) => json!({ "resource": resource.to_string() }),
        Term::Literal(literal) => match literal.language() {
            Some(language) => json!({ "literal": literal.value(), "language": language }),
            None => json!({ "literal": literal.value() }),
        },
        _ => json!({ "term": format!("{term:?}") }),
    }
}

/// Renders a term for the human-readable diff: resources as their IRI or blank
/// node, literals as their quoted lexical value.
fn display_term(term: &Term) -> String {
    match term {
        Term::Resource(resource) => resource.to_string(),
        Term::Literal(literal) => {
            let value = format!("{:?}", literal.value());
            match literal.language() {
                Some(language) => format!("{value}@{language}"),
                None => value,
            }
        }
        _ => format!("{term:?}"),
    }
}
