use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use sbol::{
    Document, MapsToSide, NamespaceSource, RdfFormat, UpgradeCounts, UpgradeOptions, UpgradeReport,
    UpgradeWarning,
};
use serde_json::{Value, json};

use crate::cli::{UpgradeArgs, UpgradeReportFormat};
use crate::output::{format_issue, infer_conversion_rdf_format};
use crate::style::Styles;

pub(crate) fn upgrade(args: UpgradeArgs, styles: Styles) -> ExitCode {
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

    let mut options = UpgradeOptions::default();
    if let Some(ns) = args.namespace.as_deref() {
        match sbol::Iri::new(ns) {
            Ok(iri) => options.default_namespace = Some(iri),
            Err(err) => {
                eprintln!("{}: invalid --namespace `{ns}`: {err}", styles.err_label());
                return ExitCode::from(2);
            }
        }
    }

    let format = match args.from {
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
    let input = match fs::read_to_string(&args.path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!(
                "{}: failed to read {}: {err}",
                styles.err_label(),
                args.path.display()
            );
            return ExitCode::from(2);
        }
    };

    let (document, report) = match Document::upgrade_from_sbol2_with(&input, format, options) {
        Ok(pair) => pair,
        Err(err) => {
            eprintln!(
                "{}: failed to upgrade {}: {err}",
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

    emit_upgrade_report(&report, args.report, styles);

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

fn emit_upgrade_report(report: &UpgradeReport, format: UpgradeReportFormat, _styles: Styles) {
    match format {
        UpgradeReportFormat::None => {}
        UpgradeReportFormat::Text => {
            let counts = report.counts();
            eprintln!("{}", format_upgrade_counts(counts));
            for warning in report.warnings() {
                eprintln!("warning: {}", format_upgrade_warning(warning));
            }
        }
        UpgradeReportFormat::Json => {
            let payload = format_upgrade_report_json(report);
            eprintln!("{payload}");
        }
    }
}

fn format_upgrade_counts(counts: &UpgradeCounts) -> String {
    format!(
        "upgrade summary: {} CD→Component, {} MD→Component, {} SubComponent, \
         {} SequenceFeature, {} SA collapsed onto SubComponent, \
         {} MapsTo decomposed, {} Interface synthesized, \
         {} Location.hasSequence inferred",
        counts.component_definitions,
        counts.module_definitions,
        counts.sub_components,
        counts.sequence_features,
        counts.sequence_annotations_collapsed,
        counts.mapstos_decomposed,
        counts.interfaces_synthesized,
        counts.locations_with_inferred_sequence,
    )
}

fn namespace_source_label(source: &NamespaceSource) -> &'static str {
    match source {
        NamespaceSource::UrlOrigin => "derived from URL scheme+host",
        NamespaceSource::DefaultOption => "fell back to --namespace value",
        NamespaceSource::None => "no namespace assigned",
        _ => "unknown source",
    }
}

fn namespace_source_token(source: &NamespaceSource) -> &'static str {
    match source {
        NamespaceSource::UrlOrigin => "url_origin",
        NamespaceSource::DefaultOption => "default_option",
        NamespaceSource::None => "none",
        _ => "unknown",
    }
}

fn mapsto_side_token(side: &MapsToSide) -> &'static str {
    match side {
        MapsToSide::Local => "local",
        MapsToSide::Remote => "remote",
        MapsToSide::Carrier => "carrier",
        _ => "unknown",
    }
}

fn format_upgrade_warning(warning: &UpgradeWarning) -> String {
    match warning {
        UpgradeWarning::NamespaceFallback { subject, source } => format!(
            "namespace fallback for <{subject}>: {}",
            namespace_source_label(source)
        ),
        UpgradeWarning::UnresolvedMapsTo { mapsto, side } => format!(
            "unresolved MapsTo <{mapsto}>: {} side did not resolve",
            mapsto_side_token(side)
        ),
        UpgradeWarning::UnsupportedRefinement { mapsto, refinement } => {
            format!("MapsTo <{mapsto}> uses refinement <{refinement}> with no SBOL 3 equivalent")
        }
        UpgradeWarning::SequenceAnnotationWithComponent { annotation } => format!(
            "SequenceAnnotation <{annotation}> references a Component; \
             upgrade collapsed it onto the referenced SubComponent"
        ),
        UpgradeWarning::UnknownSbol2Type {
            subject,
            sbol2_type,
        } => format!("subject <{subject}> has unrecognized SBOL 2 type <{sbol2_type}>"),
        UpgradeWarning::LocationWithoutSequence {
            location,
            component,
            sequence_count,
        } => format!(
            "location <{location}> on component <{component}> has no inferable sbol3:hasSequence \
             (component owns {sequence_count} sequences — need exactly 1)"
        ),
        UpgradeWarning::IdentityCollision { canonical, sources } => format!(
            "{} distinct SBOL 2 subjects canonicalize to <{canonical}>; the SBOL 3 output \
             merges their triples into a single subject. Sources: {}",
            sources.len(),
            sources
                .iter()
                .map(|s| format!("<{s}>"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => "unrecognized upgrade warning".to_string(),
    }
}

fn format_upgrade_report_json(report: &UpgradeReport) -> String {
    let counts = report.counts();
    let counts_json = json!({
        "component_definitions": counts.component_definitions,
        "module_definitions": counts.module_definitions,
        "sub_components": counts.sub_components,
        "sequence_features": counts.sequence_features,
        "sequence_annotations_collapsed": counts.sequence_annotations_collapsed,
        "mapstos_decomposed": counts.mapstos_decomposed,
        "interfaces_synthesized": counts.interfaces_synthesized,
        "locations_with_inferred_sequence": counts.locations_with_inferred_sequence,
    });
    let warnings: Vec<Value> = report
        .warnings()
        .iter()
        .map(|w| match w {
            UpgradeWarning::NamespaceFallback { subject, source } => json!({
                "kind": "namespace_fallback",
                "subject": subject,
                "source": namespace_source_token(source),
            }),
            UpgradeWarning::UnresolvedMapsTo { mapsto, side } => json!({
                "kind": "unresolved_mapsto",
                "mapsto": mapsto,
                "side": mapsto_side_token(side),
            }),
            UpgradeWarning::UnsupportedRefinement { mapsto, refinement } => json!({
                "kind": "unsupported_refinement",
                "mapsto": mapsto,
                "refinement": refinement,
            }),
            UpgradeWarning::SequenceAnnotationWithComponent { annotation } => json!({
                "kind": "sequence_annotation_with_component",
                "annotation": annotation,
            }),
            UpgradeWarning::UnknownSbol2Type {
                subject,
                sbol2_type,
            } => json!({
                "kind": "unknown_sbol2_type",
                "subject": subject,
                "sbol2_type": sbol2_type,
            }),
            UpgradeWarning::LocationWithoutSequence {
                location,
                component,
                sequence_count,
            } => json!({
                "kind": "location_without_sequence",
                "location": location,
                "component": component,
                "sequence_count": sequence_count,
            }),
            UpgradeWarning::IdentityCollision { canonical, sources } => json!({
                "kind": "identity_collision",
                "canonical": canonical,
                "sources": sources,
            }),
            _ => json!({ "kind": "unknown" }),
        })
        .collect();
    let payload = json!({ "counts": counts_json, "warnings": warnings });
    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
}
