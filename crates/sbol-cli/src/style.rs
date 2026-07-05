use std::env;
use std::io::{self, IsTerminal};

use sbol::v3::{RuleStatus, Severity};

use crate::cli::ColorMode;

#[derive(Clone, Copy)]
pub(crate) struct Styles {
    pub(crate) stdout: bool,
    pub(crate) stderr: bool,
}

impl Styles {
    pub(crate) fn resolve(mode: ColorMode) -> Self {
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

    pub(crate) fn err_label(self) -> &'static str {
        if self.stderr {
            "\x1b[1;31merror\x1b[0m"
        } else {
            "error"
        }
    }
}

pub(crate) fn paint(enabled: bool, code: &str, text: &str) -> String {
    if enabled {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub(crate) fn severity_code(severity: Severity) -> Option<&'static str> {
    match severity {
        Severity::Error => Some("1;31"),
        Severity::Warning => Some("1;33"),
        _ => None,
    }
}

pub(crate) fn rule_status_code(status: RuleStatus) -> Option<&'static str> {
    match status {
        RuleStatus::Error => Some("31"),
        RuleStatus::Warning => Some("33"),
        RuleStatus::Configurable => Some("36"),
        RuleStatus::MachineUncheckable => Some("90"),
        RuleStatus::Unimplemented => Some("35"),
        _ => None,
    }
}
