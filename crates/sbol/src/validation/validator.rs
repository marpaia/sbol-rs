use sbol_ontology::Ontology;

use crate::validation::context::ValidationContext;
use crate::validation::options::ValidationOptions;
use crate::validation::report::{AppliedOptions, ValidationIssue, ValidationReport};
use crate::validation::resolver::OwnershipIndex;
use crate::validation::spec::validation_rule_statuses;
use crate::{Document, Object};

use sbol_core::validation::compute_coverage;

pub(crate) struct Validator<'a> {
    pub(crate) document: &'a Document,
    pub(crate) ownership: OwnershipIndex,
    pub(crate) context: ValidationContext<'a>,
    pub(crate) issues: Vec<ValidationIssue>,
}

impl<'a> Validator<'a> {
    pub(crate) fn new(document: &'a Document, options: ValidationOptions) -> Self {
        Self::new_with_context(document, ValidationContext::with_options(options))
    }

    pub(crate) fn new_with_context(document: &'a Document, context: ValidationContext<'a>) -> Self {
        Self {
            document,
            ownership: OwnershipIndex::new(document),
            context,
            issues: Vec::new(),
        }
    }

    pub(crate) fn ontology(&self) -> &Ontology {
        self.context.ontology()
    }

    pub(crate) fn validate(&mut self) {
        for object in self.document.objects().values() {
            self.validate_sbol_namespace(object);
            self.validate_sbol_types(object);
            self.validate_table_rules(object);
            self.validate_display_id(object);
            self.validate_top_level(object);
            self.validate_sequence(object);
            self.validate_feature_vocabularies(object);
            self.validate_class_specific_rules(object);
            self.validate_workflow_rules(object);
        }
        self.validate_derived_from_cycles();
        self.validate_was_generated_by_cycles();
        self.validate_component_instance_cycles();
        self.validate_variant_derivation_cycles();
        self.validate_top_level_url_prefixes();
        self.validate_child_url_patterns();
        self.validate_location_sequence_membership();
    }

    pub(crate) fn finish(self) -> ValidationReport {
        let coverage = compute_coverage(validation_rule_statuses(), self.context.external_mode());
        let options = self.context.options();
        let mut options_summary = AppliedOptions::default();
        options_summary.topology_completeness = options.topology_completeness;
        options_summary.external_mode = self.context.external_mode();
        options_summary.document_resolvers =
            self.context.document_resolvers().len() + self.context.documents().len();
        options_summary.content_resolvers = self.context.content_resolvers().len();
        options_summary.overridden_rules = options.overrides().collect();
        options_summary.severity_floor = options.severity_floor();
        options_summary.severity_ceiling = options.severity_ceiling();
        ValidationReport::from_parts(self.issues, coverage, options_summary)
    }

    pub(crate) fn error(
        &mut self,
        rule: &'static str,
        object: &Object,
        property: Option<&'static str>,
        message: impl Into<String>,
    ) {
        self.emit(rule, object, property, message, crate::Severity::Error);
    }

    pub(crate) fn warning(
        &mut self,
        rule: &'static str,
        object: &Object,
        property: Option<&'static str>,
        message: impl Into<String>,
    ) {
        self.emit(rule, object, property, message, crate::Severity::Warning);
    }

    fn emit(
        &mut self,
        rule: &'static str,
        object: &Object,
        property: Option<&'static str>,
        message: impl Into<String>,
        catalog_default: crate::Severity,
    ) {
        let Some(severity) = self
            .context
            .options()
            .resolved_severity(rule, catalog_default)
        else {
            return;
        };
        let issue = match severity {
            crate::Severity::Warning => {
                ValidationIssue::warning(rule, object.identity().clone(), property, message)
            }
            _ => ValidationIssue::error(rule, object.identity().clone(), property, message),
        };
        self.issues.push(issue);
    }

    /// Add pre-built issues to the report after applying per-rule overrides.
    /// Used by rule modules (e.g. combinatorial) that compute issues with a
    /// shared immutable borrow of the validator and append them in bulk.
    pub(crate) fn extend_with_overrides(
        &mut self,
        issues: impl IntoIterator<Item = ValidationIssue>,
    ) {
        let options = self.context.options();
        for mut issue in issues {
            let Some(severity) = options.resolved_severity(issue.rule, issue.severity) else {
                continue;
            };
            issue.severity = severity;
            self.issues.push(issue);
        }
    }
}
