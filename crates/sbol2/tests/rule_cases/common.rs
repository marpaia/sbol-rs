//! Harness for the per-spec-area SBOL 2 validation rule cases. Each
//! sub-module exposes `cases()` (negative fixtures that MUST report a rule)
//! and `positives()` (valid instances that MUST NOT report it); the top-level
//! `all_rule_cases`/`all_positive_cases` concatenate them for the coverage
//! cross-check in `tests/validation_rules.rs`.
//!
//! Both fixture kinds are minimal, hermetic, in-memory SBOL 2 documents and
//! are evaluated under `ValidationConfig::all_on()` so every rule family — the
//! compliant, complete, and best-practice gates — is active. A negative is
//! asserted to report its rule at the catalog severity (MUST rules as errors,
//! SHOULD rules as warnings); a positive is asserted not to report it.

use sbol2::validation::{Severity, ValidationConfig, ValidationReport};
use sbol2::{Document, RdfFormat};

/// Turtle preamble shared by every fixture body. Bodies name subjects as
/// absolute `<http://ex/...>` IRIs and reference the SBOL 2, PROV, OM, and XSD
/// vocabularies through these prefixes.
pub const PREAMBLE: &str = "\
@prefix sbol: <http://sbols.org/v2#> .\n\
@prefix prov: <http://www.w3.org/ns/prov#> .\n\
@prefix om: <http://www.ontology-of-units-of-measure.org/resource/om-2/> .\n\
@prefix dcterms: <http://purl.org/dc/terms/> .\n\
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";

/// Negative case: a Turtle body that MUST report `rule` at `severity` under
/// `ValidationConfig::all_on()`.
#[derive(Clone, Copy)]
pub struct RuleCase {
    pub name: &'static str,
    pub rule: &'static str,
    pub severity: Severity,
    pub body: &'static str,
}

/// Positive case: a Turtle body that is a valid instance of the construct the
/// rule governs and MUST NOT report `rule`. Other rules may fire (only the
/// named rule is asserted absent).
#[derive(Clone, Copy)]
pub struct PositiveCase {
    pub name: &'static str,
    pub rule: &'static str,
    pub body: &'static str,
}

pub fn read_case(case: RuleCase) -> Document {
    let turtle = format!("{PREAMBLE}{}", case.body);
    Document::read(&turtle, RdfFormat::Turtle)
        .unwrap_or_else(|error| panic!("negative `{}` did not parse: {error}\n{turtle}", case.name))
}

pub fn read_positive_case(case: PositiveCase) -> Document {
    let turtle = format!("{PREAMBLE}{}", case.body);
    Document::read(&turtle, RdfFormat::Turtle)
        .unwrap_or_else(|error| panic!("positive `{}` did not parse: {error}\n{turtle}", case.name))
}

/// The all-flags-on config: compliant + complete + best-practice, so every
/// gated rule family runs.
pub fn all_on() -> ValidationConfig {
    ValidationConfig::all_on()
}

/// Whether `report` carries `rule` at the given severity.
pub fn reports_at(report: &ValidationReport, rule: &str, severity: Severity) -> bool {
    match severity {
        Severity::Warning => report.warnings().any(|issue| issue.rule == rule),
        _ => report.errors().any(|issue| issue.rule == rule),
    }
}

/// Whether `report` carries `rule` at any severity.
pub fn reports_any(report: &ValidationReport, rule: &str) -> bool {
    report
        .errors()
        .chain(report.warnings())
        .any(|issue| issue.rule == rule)
}

pub fn all_rule_cases() -> Vec<RuleCase> {
    let mut cases = Vec::new();
    cases.extend(super::structural::cases());
    cases.extend(super::identity::cases());
    cases.extend(super::sequence::cases());
    cases.extend(super::component::cases());
    cases.extend(super::mapsto::cases());
    cases.extend(super::location::cases());
    cases.extend(super::module::cases());
    cases.extend(super::interaction::cases());
    cases.extend(super::provenance::cases());
    cases.extend(super::combinatorial::cases());
    cases.extend(super::toplevel::cases());
    cases.extend(super::measure::cases());
    cases
}

pub fn all_positive_cases() -> Vec<PositiveCase> {
    let mut cases = Vec::new();
    cases.extend(super::structural::positives());
    cases.extend(super::identity::positives());
    cases.extend(super::sequence::positives());
    cases.extend(super::component::positives());
    cases.extend(super::mapsto::positives());
    cases.extend(super::location::positives());
    cases.extend(super::module::positives());
    cases.extend(super::interaction::positives());
    cases.extend(super::provenance::positives());
    cases.extend(super::combinatorial::positives());
    cases.extend(super::toplevel::positives());
    cases.extend(super::measure::positives());
    cases
}
