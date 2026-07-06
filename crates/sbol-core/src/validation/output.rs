//! Hand-written JSON v1 emitter for [`ValidationReport`]. The schema is
//! versioned (`schema_version: 1`); breaking field renames or removals
//! bump the integer, additive fields do not.

use std::fmt::Write as _;

use crate::validation::blocker::Blocker;
use crate::validation::options::{ExternalValidationMode, RuleOverride, TopologyCompleteness};
use crate::validation::report::{
    AppliedOptions, CoverageKind, Hint, NotApplied, NotAppliedReason, PartialApplication,
    RuleCoverage, Severity, ValidationIssue, ValidationReport,
};

/// JSON schema version emitted by [`to_json`]. Bumps on any breaking
/// rename/removal of fields.
pub const VALIDATION_OUTPUT_SCHEMA_VERSION: u32 = 1;

/// Serialize a [`ValidationReport`] to JSON v1. `spec_version` names the
/// rule-catalog spec version of the validator that produced the report.
pub fn to_json(report: &ValidationReport, spec_version: &str) -> String {
    let mut out = String::with_capacity(256 + report.issues().len() * 96);
    out.push('{');
    write_field_str(
        &mut out,
        "$schema",
        "https://sbolstandard.org/sbol-rs/validation-report/v1.json",
        true,
    );
    write_field_u32(
        &mut out,
        "schema_version",
        VALIDATION_OUTPUT_SCHEMA_VERSION,
        false,
    );
    write_field_str(&mut out, "spec_version", spec_version, false);

    out.push_str(",\"applied_options\":");
    write_applied_options(&mut out, report.options_summary());

    out.push_str(",\"coverage\":");
    write_coverage(&mut out, report.coverage());

    out.push_str(",\"issues\":");
    write_array(&mut out, report.issues(), write_issue);

    out.push('}');
    out
}

fn write_applied_options(out: &mut String, options: &AppliedOptions) {
    out.push('{');
    out.push_str("\"topology_completeness\":");
    write_string(
        out,
        topology_completeness_label(options.topology_completeness),
    );
    out.push_str(",\"external_mode\":");
    write_string(out, external_mode_label(options.external_mode));
    write_field_u32(
        out,
        "document_resolvers",
        options.document_resolvers as u32,
        false,
    );
    write_field_u32(
        out,
        "content_resolvers",
        options.content_resolvers as u32,
        false,
    );

    out.push_str(",\"severity_floor\":");
    match options.severity_floor {
        Some(severity) => write_string(out, severity_label(severity)),
        None => out.push_str("null"),
    }
    out.push_str(",\"severity_ceiling\":");
    match options.severity_ceiling {
        Some(severity) => write_string(out, severity_label(severity)),
        None => out.push_str("null"),
    }

    out.push_str(",\"overridden_rules\":");
    write_array(out, &options.overridden_rules, |out, (rule, ovr)| {
        out.push('{');
        write_field_str(out, "rule", rule, true);
        out.push_str(",\"override\":");
        write_rule_override(out, *ovr);
        out.push('}');
    });
    out.push('}');
}

fn write_rule_override(out: &mut String, ovr: RuleOverride) {
    match ovr {
        RuleOverride::Suppress => write_string(out, "Suppress"),
        RuleOverride::Severity(severity) => {
            out.push('{');
            out.push_str("\"Severity\":");
            write_string(out, severity_label(severity));
            out.push('}');
        }
    }
}

fn write_coverage(out: &mut String, coverage: &RuleCoverage) {
    out.push('{');
    out.push_str("\"fully_applied\":");
    write_array(out, &coverage.fully_applied, |out, rule| {
        write_string(out, rule);
    });
    out.push_str(",\"partially_applied\":");
    write_array(out, &coverage.partially_applied, write_partial_application);
    out.push_str(",\"not_applied\":");
    write_array(out, &coverage.not_applied, write_not_applied);
    out.push('}');
}

fn write_partial_application(out: &mut String, partial: &PartialApplication) {
    out.push('{');
    write_field_str(out, "rule", partial.rule, true);
    write_field_str(out, "blocker", blocker_label(partial.blocker), false);
    write_field_str(
        out,
        "coverage_kind",
        coverage_kind_label(partial.coverage_kind),
        false,
    );
    out.push('}');
}

fn write_not_applied(out: &mut String, not_applied: &NotApplied) {
    out.push('{');
    write_field_str(out, "rule", not_applied.rule, true);
    out.push_str(",\"reason\":");
    match not_applied.reason {
        NotAppliedReason::MachineUncheckable => write_string(out, "MachineUncheckable"),
        NotAppliedReason::Deferred(blocker) => {
            out.push('{');
            out.push_str("\"Deferred\":");
            write_string(out, blocker_label(blocker));
            out.push('}');
        }
        NotAppliedReason::GatedOff(gate) => {
            out.push('{');
            out.push_str("\"GatedOff\":");
            write_string(out, gate_label(gate));
            out.push('}');
        }
    }
    out.push('}');
}

fn gate_label(gate: crate::validation::rule_status::ValidationGate) -> &'static str {
    use crate::validation::rule_status::ValidationGate;
    match gate {
        ValidationGate::Always => "Always",
        ValidationGate::Compliant => "Compliant",
        ValidationGate::Complete => "Complete",
        ValidationGate::BestPractice => "BestPractice",
    }
}

fn write_issue(out: &mut String, issue: &ValidationIssue) {
    out.push('{');
    write_field_str(out, "severity", severity_label(issue.severity), true);
    write_field_str(out, "rule", issue.rule, false);
    out.push_str(",\"subject\":");
    write_string(out, &issue.subject.to_string());
    out.push_str(",\"property\":");
    match issue.property {
        Some(property) => write_string(out, property),
        None => out.push_str("null"),
    }
    out.push_str(",\"message\":");
    write_string(out, &issue.message);
    out.push_str(",\"hint\":");
    match &issue.hint {
        Some(hint) => write_hint(out, hint),
        None => out.push_str("null"),
    }
    out.push('}');
}

fn write_hint(out: &mut String, hint: &Hint) {
    match hint {
        Hint::SuggestedTerm { table, iri, label } => {
            out.push_str("{\"SuggestedTerm\":{");
            write_field_str(out, "table", table, true);
            write_field_str(out, "iri", iri, false);
            write_field_str(out, "label", label, false);
            out.push_str("}}");
        }
        Hint::UrlPattern { expected } => {
            out.push_str("{\"UrlPattern\":{");
            write_field_str(out, "expected", expected, true);
            out.push_str("}}");
        }
        Hint::PreferredAlias { canonical } => {
            out.push_str("{\"PreferredAlias\":{");
            write_field_str(out, "canonical", canonical, true);
            out.push_str("}}");
        }
        Hint::Note(note) => {
            out.push_str("{\"Note\":");
            write_string(out, note);
            out.push('}');
        }
    }
}

fn write_array<T, F>(out: &mut String, items: &[T], mut writer: F)
where
    F: FnMut(&mut String, &T),
{
    out.push('[');
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        writer(out, item);
    }
    out.push(']');
}

fn write_field_str(out: &mut String, name: &str, value: &str, first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(name);
    out.push_str("\":");
    write_string(out, value);
}

fn write_field_u32(out: &mut String, name: &str, value: u32, first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(name);
    out.push_str("\":");
    let _ = write!(out, "{value}");
}

fn write_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            ch if (ch as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", ch as u32);
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
}

fn topology_completeness_label(value: TopologyCompleteness) -> &'static str {
    match value {
        TopologyCompleteness::Conservative => "Conservative",
        TopologyCompleteness::RequireKnownForNucleicAcids => "RequireKnownForNucleicAcids",
    }
}

fn external_mode_label(value: ExternalValidationMode) -> &'static str {
    match value {
        ExternalValidationMode::Off => "Off",
        ExternalValidationMode::ProvidedOnly => "ProvidedOnly",
        ExternalValidationMode::ExternalAllowed => "ExternalAllowed",
    }
}

fn severity_label(value: Severity) -> &'static str {
    match value {
        Severity::Error => "Error",
        Severity::Warning => "Warning",
    }
}

fn blocker_label(value: Blocker) -> &'static str {
    match value {
        Blocker::Ontology => "Ontology",
        Blocker::Resolver => "Resolver",
        Blocker::StrictDatatype => "StrictDatatype",
        Blocker::Policy => "Policy",
        Blocker::External => "External",
    }
}

fn coverage_kind_label(value: CoverageKind) -> &'static str {
    match value {
        CoverageKind::OntologyKnownTermsOnly => "OntologyKnownTermsOnly",
        CoverageKind::LocalReferencesOnly => "LocalReferencesOnly",
        CoverageKind::LexicalShapeOnly => "LexicalShapeOnly",
        CoverageKind::PolicyDefaultUndecided => "PolicyDefaultUndecided",
    }
}
