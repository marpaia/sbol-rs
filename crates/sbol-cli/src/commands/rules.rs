use std::env;
use std::process::ExitCode;

use sbol::v3::{
    Blocker, NormativeSeverity, RuleStatus, ValidationRuleStatus, validation_rule_statuses,
};
use serde_json::{Value, json};

use crate::cli::{RuleStatusFilter, RulesCommand, RulesFormat, RulesListArgs};
use crate::style::{Styles, paint, rule_status_code};

pub(crate) fn rules(command: RulesCommand, styles: Styles) -> ExitCode {
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
    }
}
