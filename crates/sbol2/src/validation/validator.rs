//! The SBOL 2 validation engine. libSBOLj runs a fixed set of always-on
//! structural and whole-document checks, then three gated families selected
//! by the validation-mode flags: compliant-URI, completeness, and
//! best-practice. This engine mirrors that dispatch.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use sbol_core::syntax;
use sbol_core::validation::compute_coverage;

use crate::object::ObjectClasses;
use crate::schema::{Cardinality, property_specs_for};
use crate::validation::helpers::{is_sbol_object, object_matches_target, value_matches_kind};
use crate::validation::options::ValidationOptions;
use crate::validation::spec::{is_catalog_rule, validation_rule_statuses};
use crate::validation::{Severity, ValidationIssue, ValidationReport};
use crate::vocab::*;
use crate::{Document, Iri, Object, Resource, Sbol2Class};

use sbol_core::validation::report::AppliedOptions;
use sbol_core::validation::rule_status::ValidationGate;

pub(crate) struct Validator<'a> {
    document: &'a Document,
    options: ValidationOptions,
    issues: Vec<ValidationIssue>,
}

impl<'a> Validator<'a> {
    pub(crate) fn new(document: &'a Document, options: ValidationOptions) -> Self {
        Self {
            document,
            options,
            issues: Vec::new(),
        }
    }

    pub(crate) fn validate(&mut self) {
        let config = self.options.config;

        // Always: per-object structural and identity checks. Extension
        // objects (non-SBOL RDF types) are not governed by SBOL rules.
        for object in self.document.objects().values() {
            if !is_sbol_object(object) {
                continue;
            }
            self.validate_table_rules(object);
            self.validate_identity_syntax(object);
            self.validate_closed_property_set(object);
            self.validate_controlled_values(object);
            self.validate_value_bounds(object);
            self.validate_derivation_cycles(object);
            self.validate_containment(object);
            self.validate_child_semantics(object);
            self.validate_sequence_encoding(object);
        }

        // Always: whole-document checks.
        self.validate_uri_uniqueness();
        self.validate_persistent_identity_uniqueness();
        self.validate_document_namespace();
        self.validate_instance_graph_cycles();
        self.validate_generation_cycles();
        self.validate_maps_to_use_remote_uniqueness();

        // Gated families, dispatched exactly as SBOLValidate does.
        if config.compliant {
            let parents = self.build_parent_map();
            for object in self.document.objects().values() {
                if is_sbol_object(object) {
                    self.validate_compliance(object);
                    self.validate_child_compliance(object, &parents);
                }
            }
        }
        if config.complete {
            let resolvable = self.resolvable_uris();
            for object in self.document.objects().values() {
                if is_sbol_object(object) {
                    self.validate_completeness(object, &resolvable);
                    self.validate_reference_classes(object);
                }
            }
        }
        if config.best_practice {
            for object in self.document.objects().values() {
                if is_sbol_object(object) {
                    self.validate_best_practices(object);
                }
            }
        }
    }

    pub(crate) fn finish(self) -> ValidationReport {
        let coverage = compute_coverage(
            validation_rule_statuses(),
            sbol_core::validation::ExternalValidationMode::Off,
            self.options.config,
        );
        let mut options_summary = AppliedOptions::default();
        options_summary.overridden_rules = self.options.overrides().collect();
        options_summary.severity_floor = self.options.severity_floor();
        options_summary.severity_ceiling = self.options.severity_ceiling();
        ValidationReport::from_parts(self.issues, coverage, options_summary)
    }

    // --- structural engine (Always) ------------------------------------

    fn validate_table_rules(&mut self, object: &Object) {
        let specs = property_specs_for(object);
        for spec in specs.values() {
            if !is_catalog_rule(spec.rule) {
                // No SBOL rule governs this property (om: ontology classes,
                // shared abstract descriptors); the structural engine skips it.
                continue;
            }
            // Count distinct values: RDF/XML may serialize the same triple
            // more than once (an object described in two places), and those
            // identical triples are one value for cardinality purposes.
            let values = distinct_terms(object.values(spec.predicate));
            match spec.cardinality {
                Cardinality::ExactlyOne if values.len() != 1 => self.error(
                    spec.rule,
                    object,
                    Some(spec.predicate),
                    format!(
                        "property `{}` must have exactly one value; found {}",
                        spec.predicate,
                        values.len()
                    ),
                ),
                Cardinality::ZeroOrOne if values.len() > 1 => self.error(
                    spec.rule,
                    object,
                    Some(spec.predicate),
                    format!(
                        "property `{}` must have zero or one value; found {}",
                        spec.predicate,
                        values.len()
                    ),
                ),
                Cardinality::OneOrMore if values.is_empty() => self.error(
                    spec.rule,
                    object,
                    Some(spec.predicate),
                    format!("property `{}` must have one or more values", spec.predicate),
                ),
                _ => {}
            }

            for value in values {
                if !value_matches_kind(value, spec.value_kind) {
                    self.error(
                        spec.rule,
                        object,
                        Some(spec.predicate),
                        format!(
                            "property `{}` value does not match expected {:?}",
                            spec.predicate, spec.value_kind
                        ),
                    );
                    continue;
                }
                // Local (composite child) references must resolve in-document
                // and match the expected class; these are always required.
                let Some(reference) = spec.reference else {
                    continue;
                };
                if !reference.require_local {
                    continue;
                }
                let Some(resource) = value.as_resource() else {
                    continue;
                };
                match self.document.get(resource) {
                    Some(target) if !object_matches_target(target, reference.target) => self.error(
                        spec.rule,
                        object,
                        Some(spec.predicate),
                        format!(
                            "property `{}` refers to `{resource}`, which is not the expected class",
                            spec.predicate
                        ),
                    ),
                    None => self.error(
                        spec.rule,
                        object,
                        Some(spec.predicate),
                        format!(
                            "property `{}` refers to missing local child `{resource}`",
                            spec.predicate
                        ),
                    ),
                    _ => {}
                }
            }
        }
    }

    fn validate_identity_syntax(&mut self, object: &Object) {
        // 10201: identity must be a URI (an IRI resource, not a blank node).
        if object.identity().as_iri().is_none() {
            self.error(
                "sbol2-10201",
                object,
                None,
                "the identity of an Identified object must be a URI",
            );
        }
        // 10204: displayId must be alphanumeric/underscore, not leading digit.
        if let Some(display_id) = &object.identified().display_id
            && !syntax::is_valid_display_id(display_id)
        {
            self.error(
                "sbol2-10204",
                object,
                Some(SBOL2_DISPLAY_ID),
                format!(
                    "displayId `{display_id}` must contain only alphanumeric or underscore \
                         characters and must not begin with a digit"
                ),
            );
        }
        // 10206: version must be alphanumeric/underscore/hyphen/period, leading digit.
        if let Some(version) = self.literal(object, SBOL2_VERSION)
            && !is_valid_version(&version)
        {
            self.error(
                "sbol2-10206",
                object,
                Some(SBOL2_VERSION),
                format!("version `{version}` is not a valid SBOL 2 version string"),
            );
        }
        // 10228: at most one rdfType in each of the SBOL 2 and PROV namespaces.
        let sbol_types = object
            .rdf_types()
            .iter()
            .filter(|t| t.as_str().starts_with(SBOL2_NS))
            .count();
        let prov_types = object
            .rdf_types()
            .iter()
            .filter(|t| t.as_str().starts_with(PROV_NS))
            .count();
        if sbol_types > 1 || prov_types > 1 {
            self.error(
                "sbol2-10228",
                object,
                None,
                "an Identified object must have no more than one rdfType in each of the \
                 SBOL 2 and PROV namespaces",
            );
        }
    }

    /// A property in the SBOL 2 namespace that is not a declared field of the
    /// object's class is a disallowed extra property. Each class has a distinct
    /// rule id for this ("MUST NOT have properties other than ...").
    fn validate_closed_property_set(&mut self, object: &Object) {
        let Some(rule) = closed_property_rule(object) else {
            return;
        };
        let allowed = property_specs_for(object);
        for predicate in object.properties().keys() {
            let predicate = predicate.as_str();
            if predicate.starts_with(SBOL2_NS) && !allowed.contains_key(predicate) {
                self.error(
                    rule,
                    object,
                    None,
                    format!("property `{predicate}` is not permitted on this object's class"),
                );
            }
        }
    }

    /// Enumerated properties whose value must come from a fixed vocabulary
    /// table, plus the `rdfType` namespace restriction on GenericTopLevel.
    fn validate_controlled_values(&mut self, object: &Object) {
        // 10607: access from Table 4 (public, private).
        self.check_enum(
            object,
            SBOL2_ACCESS,
            "sbol2-10607",
            &[SBOL2_PUBLIC, SBOL2_PRIVATE],
        );
        // 10708: roleIntegration from Table 6 (mergeRoles, overrideRoles).
        self.check_enum(
            object,
            SBOL2_ROLE_INTEGRATION,
            "sbol2-10708",
            &[SBOL2_MERGE_ROLES, SBOL2_OVERRIDE_ROLES],
        );
        // 10810: refinement from Table 5.
        self.check_enum(
            object,
            SBOL2_REFINEMENT,
            "sbol2-10810",
            &[
                SBOL2_VERIFY_IDENTICAL,
                SBOL2_USE_LOCAL,
                SBOL2_USE_REMOTE,
                SBOL2_MERGE,
            ],
        );
        // 12902: strategy from Table 15 (enumerate, sample).
        self.check_enum(
            object,
            SBOL2_STRATEGY,
            "sbol2-12902",
            &[SBOL2_ENUMERATE, SBOL2_SAMPLE],
        );
        // 13003: operator from Table 16.
        self.check_enum(
            object,
            SBOL2_OPERATOR,
            "sbol2-13003",
            &[
                SBOL2_OP_ZERO_OR_ONE,
                SBOL2_OP_ONE,
                SBOL2_OP_ZERO_OR_MORE,
                SBOL2_OP_ONE_OR_MORE,
            ],
        );

        // 10709: a Component with one or more roles requires a roleIntegration.
        if object.has_class(Sbol2Class::Component)
            && !object.values(SBOL2_ROLE).is_empty()
            && object.values(SBOL2_ROLE_INTEGRATION).is_empty()
        {
            self.error(
                "sbol2-10709",
                object,
                Some(SBOL2_ROLE_INTEGRATION),
                "a Component that specifies roles must also specify a roleIntegration",
            );
        }

        // 12303: a GenericTopLevel rdfType must not lie in the SBOL 2 namespace.
        if object.has_class(Sbol2Class::GenericTopLevel) {
            for value in object.values(SBOL2_RDF_TYPE) {
                if value
                    .as_iri()
                    .is_some_and(|iri| iri.as_str().starts_with(SBOL2_NS))
                {
                    self.error(
                        "sbol2-12303",
                        object,
                        Some(SBOL2_RDF_TYPE),
                        "the rdfType of a GenericTopLevel must not be in the SBOL 2 namespace",
                    );
                }
            }
        }
    }

    fn check_enum(
        &mut self,
        object: &Object,
        predicate: &'static str,
        rule: &'static str,
        allowed: &[&str],
    ) {
        for value in object.values(predicate) {
            let Some(iri) = value.as_iri() else {
                continue;
            };
            if !allowed.contains(&iri.as_str()) {
                self.error(
                    rule,
                    object,
                    Some(predicate),
                    format!(
                        "value `{}` of `{predicate}` is not an allowed term",
                        iri.as_str()
                    ),
                );
            }
        }
    }

    /// Integer-valued positional bounds on Range and Cut.
    fn validate_value_bounds(&mut self, object: &Object) {
        if object.has_class(Sbol2Class::Range) {
            let start = self.integer(object, SBOL2_START);
            let end = self.integer(object, SBOL2_END);
            // 11102: start must be an integer greater than zero.
            if let Some(start) = start
                && start < 1
            {
                self.error(
                    "sbol2-11102",
                    object,
                    Some(SBOL2_START),
                    "the start of a Range must be greater than zero",
                );
            }
            // 11103: end must be an integer greater than zero.
            if let Some(end) = end
                && end < 1
            {
                self.error(
                    "sbol2-11103",
                    object,
                    Some(SBOL2_END),
                    "the end of a Range must be greater than zero",
                );
            }
            // 11104: end must be greater than or equal to start.
            if let (Some(start), Some(end)) = (start, end)
                && end < start
            {
                self.error(
                    "sbol2-11104",
                    object,
                    Some(SBOL2_END),
                    "the end of a Range must be greater than or equal to its start",
                );
            }
        }
        if object.has_class(Sbol2Class::Cut) {
            // 11202: at must be an integer greater than or equal to zero.
            if let Some(at) = self.integer(object, SBOL2_AT)
                && at < 0
            {
                self.error(
                    "sbol2-11202",
                    object,
                    Some(SBOL2_AT),
                    "the at position of a Cut must be greater than or equal to zero",
                );
            }
        }
    }

    /// Self-reference and cycle checks on the properties that chain objects to
    /// objects of the same kind: wasDerivedFrom on TopLevels, and definition on
    /// component/module instances.
    fn validate_derivation_cycles(&mut self, object: &Object) {
        let Some(identity) = object.identity().as_iri() else {
            return;
        };
        let identity = identity.as_str();
        if object.is_top_level() {
            // 10303: a TopLevel must not appear in its own wasDerivedFrom set.
            for derived in object.resources(PROV_WAS_DERIVED_FROM) {
                if derived.as_iri().is_some_and(|iri| iri.as_str() == identity) {
                    self.error(
                        "sbol2-10303",
                        object,
                        Some(PROV_WAS_DERIVED_FROM),
                        "a TopLevel must not derive from itself",
                    );
                }
            }
            // 10304: TopLevels must not form a wasDerivedFrom cycle.
            if self.derives_cycle(object, identity, &mut std::collections::BTreeSet::new()) {
                self.error(
                    "sbol2-10304",
                    object,
                    Some(PROV_WAS_DERIVED_FROM),
                    "TopLevel objects must not form a circular wasDerivedFrom chain",
                );
            }
        }
        // 10603 / 11704: a ComponentInstance/Module definition must not refer to
        // the definition object that contains the instance. The containing
        // definition is the object whose composite property lists this instance.
        if object.has_class(Sbol2Class::Component) || object.has_class(Sbol2Class::Module) {
            if let (Some(def), Some(parent)) = (
                object
                    .first_resource(SBOL2_DEFINITION)
                    .and_then(Resource::as_iri),
                self.parent_definition(identity),
            ) && def.as_str() == parent
            {
                let rule = if object.has_class(Sbol2Class::Module) {
                    "sbol2-11704"
                } else {
                    "sbol2-10603"
                };
                self.error(
                    rule,
                    object,
                    Some(SBOL2_DEFINITION),
                    "a component or module must not refer to its own containing definition",
                );
            }
        }
    }

    /// Follows the wasDerivedFrom chain among TopLevels, reporting whether a
    /// cycle returns to `origin`.
    fn derives_cycle(
        &self,
        object: &Object,
        origin: &str,
        seen: &mut std::collections::BTreeSet<String>,
    ) -> bool {
        for derived in object.resources(PROV_WAS_DERIVED_FROM) {
            let Some(target) = derived.as_iri().map(|iri| iri.as_str().to_owned()) else {
                continue;
            };
            let Some(next) = self.document.get(derived) else {
                continue;
            };
            if !next.is_top_level() {
                continue;
            }
            if target == origin {
                return true;
            }
            if seen.insert(target) && self.derives_cycle(next, origin, seen) {
                return true;
            }
        }
        false
    }

    /// The identity of the ComponentDefinition or ModuleDefinition that lists
    /// `child` in a composite property (component/functionalComponent/module).
    fn parent_definition(&self, child: &str) -> Option<&str> {
        for object in self.document.objects().values() {
            let is_def = object.has_class(Sbol2Class::ComponentDefinition)
                || object.has_class(Sbol2Class::ModuleDefinition);
            if !is_def {
                continue;
            }
            for predicate in [SBOL2_COMPONENT, SBOL2_FUNCTIONAL_COMPONENT, SBOL2_MODULE] {
                for value in object.resources(predicate) {
                    if value.as_iri().is_some_and(|iri| iri.as_str() == child) {
                        return object.identity().as_iri().map(|iri| iri.as_str());
                    }
                }
            }
        }
        None
    }

    /// Always-on structural rules over an object's own children: the class of
    /// the measures it carries, and the uniqueness of Component references
    /// across a ComponentDefinition's SequenceAnnotations.
    fn validate_child_semantics(&mut self, object: &Object) {
        // 10608 / 11707 / 12008: every measure must resolve to an om:Measure.
        let measure_rule = if object.has_class(Sbol2Class::Module) {
            Some("sbol2-11707")
        } else if object.has_class(Sbol2Class::Component)
            || object.has_class(Sbol2Class::FunctionalComponent)
        {
            Some("sbol2-10608")
        } else if object.has_class(Sbol2Class::Participation) {
            Some("sbol2-12008")
        } else if object.has_class(Sbol2Class::Interaction) {
            // 11908: every measure of an Interaction must resolve to an om:Measure.
            Some("sbol2-11908")
        } else {
            None
        };
        if let Some(rule) = measure_rule {
            let mut bad = Vec::new();
            for value in object.resources(SBOL2_MEASURE) {
                let Some(iri) = value.as_iri() else { continue };
                if let Some(target) = self.resolve(iri.as_str())
                    && !target.has_class(Sbol2Class::OmMeasure)
                {
                    bad.push(iri.as_str().to_owned());
                }
            }
            for iri in bad {
                self.error(
                    rule,
                    object,
                    Some(SBOL2_MEASURE),
                    format!("measure refers to `{iri}`, which is not an om:Measure"),
                );
            }
        }

        // 12903: an enumerate CombinatorialDerivation must not contain a
        // VariableComponent whose operator is zeroOrMore or oneOrMore.
        if object.has_class(Sbol2Class::CombinatorialDerivation)
            && object.resources(SBOL2_STRATEGY).any(|r| {
                r.as_iri()
                    .is_some_and(|iri| iri.as_str() == SBOL2_ENUMERATE)
            })
        {
            let mut offending = false;
            for vc_ref in object.resources(SBOL2_VARIABLE_COMPONENT) {
                let Some(vc) = self.document.get(vc_ref) else {
                    continue;
                };
                if vc.resources(SBOL2_OPERATOR).any(|r| {
                    r.as_iri().is_some_and(|iri| {
                        matches!(iri.as_str(), SBOL2_OP_ZERO_OR_MORE | SBOL2_OP_ONE_OR_MORE)
                    })
                }) {
                    offending = true;
                }
            }
            if offending {
                self.error(
                    "sbol2-12903",
                    object,
                    Some(SBOL2_STRATEGY),
                    "an enumerate CombinatorialDerivation must not use a zeroOrMore or oneOrMore operator",
                );
            }
        }

        // 12907: no two VariableComponents of a CombinatorialDerivation may
        // carry the same variable.
        if object.has_class(Sbol2Class::CombinatorialDerivation) {
            let mut variables = std::collections::BTreeSet::new();
            let mut duplicate = false;
            for vc_ref in object.resources(SBOL2_VARIABLE_COMPONENT) {
                let Some(vc) = self.document.get(vc_ref) else {
                    continue;
                };
                if let Some(variable) = vc.first_resource(SBOL2_VARIABLE).and_then(Resource::as_iri)
                    && !variables.insert(variable.as_str().to_owned())
                {
                    duplicate = true;
                }
            }
            if duplicate {
                self.error(
                    "sbol2-12907",
                    object,
                    Some(SBOL2_VARIABLE_COMPONENT),
                    "two VariableComponents of a CombinatorialDerivation must not share a variable",
                );
            }
        }

        // 10522: no two SequenceAnnotations of a ComponentDefinition may refer
        // to the same Component.
        if object.has_class(Sbol2Class::ComponentDefinition) {
            // Map each distinct SequenceAnnotation to the Component it targets;
            // RDF/XML may serialize one SA in two places, so dedupe by SA
            // identity before looking for two SAs sharing a Component.
            let mut by_sa: BTreeMap<String, String> = BTreeMap::new();
            for sa_ref in object.resources(SBOL2_SEQUENCE_ANNOTATION) {
                let Some(sa_id) = sa_ref.as_iri().map(|iri| iri.as_str().to_owned()) else {
                    continue;
                };
                let Some(sa) = self.document.get(sa_ref) else {
                    continue;
                };
                if let Some(component) = sa
                    .first_resource(SBOL2_COMPONENT)
                    .and_then(Resource::as_iri)
                {
                    by_sa.insert(sa_id, component.as_str().to_owned());
                }
            }
            let mut seen = std::collections::BTreeSet::new();
            let duplicate = by_sa
                .values()
                .any(|component| !seen.insert(component.clone()));
            if duplicate {
                self.error(
                    "sbol2-10522",
                    object,
                    Some(SBOL2_SEQUENCE_ANNOTATION),
                    "no two SequenceAnnotations may refer to the same Component",
                );
            }
        }
    }

    /// Cross-object containment: the Component/FunctionalComponent references
    /// carried by SequenceAnnotation, SequenceConstraint, Participation, and
    /// MapsTo must point at instances contained by the appropriate parent
    /// definition.
    fn validate_containment(&mut self, object: &Object) {
        if object.has_class(Sbol2Class::SequenceConstraint) {
            self.validate_sequence_constraint(object);
        }
        if object.has_class(Sbol2Class::SequenceAnnotation) {
            // 10909: a SequenceAnnotation must not carry both a component and roles.
            if !object.values(SBOL2_COMPONENT).is_empty() && !object.values(SBOL2_ROLE).is_empty() {
                self.error(
                    "sbol2-10909",
                    object,
                    Some(SBOL2_ROLE),
                    "a SequenceAnnotation must not include both a component and a roles property",
                );
            }
            // 10905: the referenced Component must belong to the containing CD.
            let missing = match object
                .first_resource(SBOL2_COMPONENT)
                .and_then(Resource::as_iri)
            {
                Some(component) => match self.container_by(object, SBOL2_SEQUENCE_ANNOTATION) {
                    Some(cd) => !self.lists(cd, SBOL2_COMPONENT, component.as_str()),
                    None => false,
                },
                None => false,
            };
            if missing {
                self.error(
                    "sbol2-10905",
                    object,
                    Some(SBOL2_COMPONENT),
                    "the Component of a SequenceAnnotation must be contained by its ComponentDefinition",
                );
            }
        }
        if object.has_class(Sbol2Class::Participation) {
            // 12003: the participant FunctionalComponent must belong to the
            // ModuleDefinition that contains the Interaction of the Participation.
            let missing = match object
                .first_resource(SBOL2_PARTICIPANT)
                .and_then(Resource::as_iri)
            {
                Some(participant) => match self.container_by(object, SBOL2_PARTICIPATION) {
                    Some(interaction) => match self.container_by(interaction, SBOL2_INTERACTION) {
                        Some(md) => {
                            !self.lists(md, SBOL2_FUNCTIONAL_COMPONENT, participant.as_str())
                        }
                        None => false,
                    },
                    None => false,
                },
                None => false,
            };
            if missing {
                self.error(
                    "sbol2-12003",
                    object,
                    Some(SBOL2_PARTICIPANT),
                    "the participant of a Participation must be a FunctionalComponent of the ModuleDefinition",
                );
            }
        }
        if object.has_class(Sbol2Class::MapsTo) {
            self.validate_maps_to_local(object);
        }
    }

    fn validate_sequence_constraint(&mut self, object: &Object) {
        let subject = object
            .first_resource(SBOL2_SUBJECT)
            .and_then(Resource::as_iri);
        let obj = object
            .first_resource(SBOL2_OBJECT)
            .and_then(Resource::as_iri);
        // 11406: subject and object must not be the same Component.
        if let (Some(subject), Some(obj)) = (subject, obj)
            && subject.as_str() == obj.as_str()
        {
            self.error(
                "sbol2-11406",
                object,
                Some(SBOL2_OBJECT),
                "the object of a SequenceConstraint must not be the same Component as its subject",
            );
        }
        let Some(cd) = self.container_by(object, SBOL2_SEQUENCE_CONSTRAINT) else {
            return;
        };
        // 11403 / 11405: subject and object must be Components of the CD.
        let subject_missing = subject
            .map(|s| !self.lists(cd, SBOL2_COMPONENT, s.as_str()))
            .unwrap_or(false);
        let object_missing = obj
            .map(|o| !self.lists(cd, SBOL2_COMPONENT, o.as_str()))
            .unwrap_or(false);
        // 11413: under differentFrom, subject and object Components must not
        // resolve to the same ComponentDefinition.
        let restriction = object
            .first_resource(SBOL2_RESTRICTION)
            .and_then(Resource::as_iri)
            .map(|iri| iri.as_str().to_owned());
        let is_different_from = restriction.as_deref() == Some(SBOL2_DIFFERENT_FROM);
        let same_definition = is_different_from
            && match (subject, obj) {
                (Some(s), Some(o)) => {
                    match (
                        self.definition_of(s.as_str()),
                        self.definition_of(o.as_str()),
                    ) {
                        (Some(sd), Some(od)) => sd == od,
                        _ => false,
                    }
                }
                _ => false,
            };
        // 11414: under differentFrom, the subject and object Components must not
        // resolve to the same ComponentDefinition. Distinct from 11413, which
        // compares the Components' definition URIs literally; 11414 compares the
        // ComponentDefinitions the URIs resolve to, catching identity/
        // persistentIdentity aliases of one definition.
        let same_resolved_definition = is_different_from
            && match (subject, obj) {
                (Some(s), Some(o)) => match (
                    self.definition_of(s.as_str())
                        .and_then(|d| self.resolved_identity(&d)),
                    self.definition_of(o.as_str())
                        .and_then(|d| self.resolved_identity(&d)),
                ) {
                    (Some(sd), Some(od)) => sd == od,
                    _ => false,
                },
                _ => false,
            };
        let cd_id = cd.identity().as_iri().map(|iri| iri.as_str().to_owned());
        if let Some(cd_id) = cd_id {
            self.check_sequence_constraint_positions(
                object,
                &cd_id,
                restriction.as_deref(),
                subject,
                obj,
            );
        }
        if subject_missing {
            self.error(
                "sbol2-11403",
                object,
                Some(SBOL2_SUBJECT),
                "the subject of a SequenceConstraint must be a Component of its ComponentDefinition",
            );
        }
        if object_missing {
            self.error(
                "sbol2-11405",
                object,
                Some(SBOL2_OBJECT),
                "the object of a SequenceConstraint must be a Component of its ComponentDefinition",
            );
        }
        if same_definition {
            self.error(
                "sbol2-11413",
                object,
                None,
                "a differentFrom SequenceConstraint must relate Components with different definitions",
            );
        }
        if same_resolved_definition {
            self.error(
                "sbol2-11414",
                object,
                None,
                "a differentFrom SequenceConstraint must relate Components resolving to different ComponentDefinitions",
            );
        }
    }

    fn validate_maps_to_local(&mut self, object: &Object) {
        let Some(local) = object
            .first_resource(SBOL2_LOCAL)
            .and_then(Resource::as_iri)
        else {
            return;
        };
        let Some(owner) = self.container_by(object, SBOL2_MAPS_TO) else {
            return;
        };
        // The owning instance sits in a ComponentDefinition (as a Component) or a
        // ModuleDefinition (as a FunctionalComponent or Module).
        let owner_id = owner.identity().as_iri().map(|iri| iri.as_str().to_owned());
        let Some(owner_id) = owner_id else {
            return;
        };
        let local = local.as_str().to_owned();
        if let Some(cd) = self.definition_containing(&owner_id, SBOL2_COMPONENT) {
            // 10803: local must refer to another Component in the same CD.
            let missing = !self.lists(cd, SBOL2_COMPONENT, &local);
            if missing {
                self.error(
                    "sbol2-10803",
                    object,
                    Some(SBOL2_LOCAL),
                    "the local of a MapsTo must refer to a Component in the same ComponentDefinition",
                );
            }
        } else if let Some(md) = self
            .definition_containing(&owner_id, SBOL2_FUNCTIONAL_COMPONENT)
            .or_else(|| self.definition_containing(&owner_id, SBOL2_MODULE))
        {
            // 10804: local must refer to a FunctionalComponent in the same MD.
            let missing = !self.lists(md, SBOL2_FUNCTIONAL_COMPONENT, &local);
            if missing {
                self.error(
                    "sbol2-10804",
                    object,
                    Some(SBOL2_LOCAL),
                    "the local of a MapsTo must refer to a FunctionalComponent in the same ModuleDefinition",
                );
            }
        }
    }

    /// The object that lists `child`'s identity in its `predicate` property.
    fn container_by(&self, child: &Object, predicate: &str) -> Option<&Object> {
        let child_id = child.identity().as_iri()?.as_str();
        self.definition_containing(child_id, predicate)
    }

    /// The object whose `predicate` property lists `child_id`.
    fn definition_containing(&self, child_id: &str, predicate: &str) -> Option<&Object> {
        self.document.objects().values().find(|candidate| {
            candidate
                .resources(predicate)
                .any(|r| r.as_iri().is_some_and(|iri| iri.as_str() == child_id))
        })
    }

    /// Whether `container`'s `predicate` property lists `target`.
    fn lists(&self, container: &Object, predicate: &str, target: &str) -> bool {
        container
            .resources(predicate)
            .any(|r| r.as_iri().is_some_and(|iri| iri.as_str() == target))
    }

    /// The definition URI of the ComponentInstance identified by `instance`.
    fn definition_of(&self, instance: &str) -> Option<String> {
        let object = self.document.objects().values().find(|o| {
            o.identity()
                .as_iri()
                .is_some_and(|iri| iri.as_str() == instance)
        })?;
        object
            .first_resource(SBOL2_DEFINITION)
            .and_then(Resource::as_iri)
            .map(|iri| iri.as_str().to_owned())
    }

    fn integer(&self, object: &Object, predicate: &str) -> Option<i64> {
        object.values(predicate).iter().find_map(|term| {
            term.as_literal()
                .and_then(|l| l.value().parse::<i64>().ok())
        })
    }

    fn validate_uri_uniqueness(&mut self) {
        // 10202: identity globally unique; checked for document-local collisions.
        let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
        for object in self.document.objects().values() {
            if let Some(iri) = object.identity().as_iri() {
                *seen.entry(iri.as_str()).or_insert(0) += 1;
            }
        }
        let dupes: Vec<String> = seen
            .iter()
            .filter(|(_, count)| **count > 1)
            .map(|(iri, _)| (*iri).to_owned())
            .collect();
        for object in self.document.objects().values() {
            if let Some(iri) = object.identity().as_iri()
                && dupes.iter().any(|d| d == iri.as_str())
            {
                self.error(
                    "sbol2-10202",
                    object,
                    None,
                    format!(
                        "identity `{}` is not unique within the document",
                        iri.as_str()
                    ),
                );
            }
        }
    }

    fn validate_persistent_identity_uniqueness(&mut self) {
        // 10220: objects with the same persistentIdentity must be the same class.
        let mut classes_by_pid: BTreeMap<String, std::collections::BTreeSet<String>> =
            BTreeMap::new();
        for object in self.document.objects().values() {
            let Some(pid) = object
                .first_resource(SBOL2_PERSISTENT_IDENTITY)
                .and_then(Resource::as_iri)
            else {
                continue;
            };
            let types: String = object
                .rdf_types()
                .iter()
                .map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join(",");
            classes_by_pid
                .entry(pid.as_str().to_owned())
                .or_default()
                .insert(types);
        }
        for object in self.document.objects().values() {
            let Some(pid) = object
                .first_resource(SBOL2_PERSISTENT_IDENTITY)
                .and_then(Resource::as_iri)
            else {
                continue;
            };
            if classes_by_pid
                .get(pid.as_str())
                .is_some_and(|set| set.len() > 1)
            {
                self.error(
                    "sbol2-10220",
                    object,
                    Some(SBOL2_PERSISTENT_IDENTITY),
                    format!(
                        "objects sharing persistentIdentity `{}` must be instances of the same class",
                        pid.as_str()
                    ),
                );
            }
        }
    }

    // --- compliant-URI family (gate: compliant) ------------------------

    fn validate_compliance(&mut self, object: &Object) {
        let display_id = object.identified().display_id.clone();
        // 10215: a compliant Identified object requires a displayId.
        let Some(display_id) = display_id else {
            self.error(
                "sbol2-10215",
                object,
                Some(SBOL2_DISPLAY_ID),
                "a compliant Identified object requires a displayId",
            );
            return;
        };

        let persistent = object
            .first_resource(SBOL2_PERSISTENT_IDENTITY)
            .and_then(Resource::as_iri)
            .map(|iri| iri.as_str().to_owned());
        let identity = object
            .identity()
            .as_iri()
            .map(|iri| iri.as_str().to_owned());
        let version = self.literal(object, SBOL2_VERSION);

        if object.is_top_level() {
            // 10216: a compliant TopLevel requires a persistentIdentity that
            // ends with a delimiter followed by its displayId.
            match &persistent {
                None => self.error(
                    "sbol2-10216",
                    object,
                    Some(SBOL2_PERSISTENT_IDENTITY),
                    "a compliant TopLevel requires a persistentIdentity",
                ),
                Some(pid) if !ends_with_delimited(pid, &display_id) => self.error(
                    "sbol2-10216",
                    object,
                    Some(SBOL2_PERSISTENT_IDENTITY),
                    format!(
                        "compliant TopLevel persistentIdentity `{pid}` must end with a \
                         delimiter followed by displayId `{display_id}`"
                    ),
                ),
                _ => {}
            }
        }

        // 10218: identity relates to persistentIdentity and version.
        if let (Some(pid), Some(id)) = (&persistent, &identity) {
            let ok = match &version {
                None => id == pid,
                Some(v) => id == pid || is_delimited_suffix(id, pid, v),
            };
            if !ok {
                self.error(
                    "sbol2-10218",
                    object,
                    None,
                    format!(
                        "compliant identity `{id}` must equal persistentIdentity `{pid}` \
                         (optionally followed by a delimiter and version)"
                    ),
                );
            }
        }
    }

    /// Maps each composite child object's identity to the identity of the
    /// object that contains it. Built from the child-bearing properties of the
    /// SBOL 2 data model.
    fn build_parent_map(&self) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        for object in self.document.objects().values() {
            let Some(parent) = object.identity().as_iri() else {
                continue;
            };
            // A composite child is one referenced through a property whose
            // schema reference is local (contained in-document), which
            // distinguishes containment (a CD's `component`) from a same-named
            // cross-reference (a SequenceAnnotation's `component`).
            for spec in property_specs_for(object).values() {
                let owns = spec
                    .reference
                    .is_some_and(|reference| reference.require_local);
                if !owns {
                    continue;
                }
                for child in object.resources(spec.predicate) {
                    if let Some(child) = child.as_iri() {
                        map.insert(child.as_str().to_owned(), parent.as_str().to_owned());
                    }
                }
            }
        }
        map
    }

    /// Compliance rules that relate a child object to its parent: the child's
    /// persistentIdentity must extend the parent's (10217), and their versions
    /// must agree (10219).
    fn validate_child_compliance(&mut self, object: &Object, parents: &BTreeMap<String, String>) {
        if object.is_top_level() {
            return;
        }
        let Some(child_id) = object
            .identity()
            .as_iri()
            .map(|iri| iri.as_str().to_owned())
        else {
            return;
        };
        let Some(parent_id) = parents.get(&child_id) else {
            return;
        };
        let Some(display_id) = object.identified().display_id.clone() else {
            return;
        };
        let child_pid = object
            .first_resource(SBOL2_PERSISTENT_IDENTITY)
            .and_then(Resource::as_iri)
            .map(|iri| iri.as_str().to_owned());
        let child_version = self.literal(object, SBOL2_VERSION);
        let Some(parent) = self.resolve(parent_id) else {
            return;
        };
        let parent_pid = parent
            .first_resource(SBOL2_PERSISTENT_IDENTITY)
            .and_then(Resource::as_iri)
            .map(|iri| iri.as_str().to_owned());
        let parent_version = self.literal(parent, SBOL2_VERSION);

        // 10217: a compliant child requires a persistentIdentity, and it must
        // be the parent's persistentIdentity, a delimiter, then the child's
        // displayId.
        match (&child_pid, &parent_pid) {
            (None, _) => self.error(
                "sbol2-10217",
                object,
                Some(SBOL2_PERSISTENT_IDENTITY),
                "a compliant child object requires a persistentIdentity",
            ),
            (Some(child_pid), Some(parent_pid))
                if !['/', '#', ':']
                    .iter()
                    .any(|d| *child_pid == format!("{parent_pid}{d}{display_id}")) =>
            {
                self.error(
                    "sbol2-10217",
                    object,
                    Some(SBOL2_PERSISTENT_IDENTITY),
                    format!(
                        "compliant child persistentIdentity `{child_pid}` must extend parent \
                         persistentIdentity `{parent_pid}` with displayId `{display_id}`"
                    ),
                );
            }
            _ => {}
        }

        // 10219: a compliant child's version must match its parent's version.
        let version_mismatch = match (&parent_version, &child_version) {
            (Some(p), Some(c)) => p != c,
            (Some(_), None) | (None, Some(_)) => true,
            (None, None) => false,
        };
        if version_mismatch {
            self.error(
                "sbol2-10219",
                object,
                Some(SBOL2_VERSION),
                "a compliant child object must have the same version as its parent",
            );
        }
    }

    // --- completeness family (gate: complete) --------------------------

    /// The set of URIs a reference can resolve to within this document: every
    /// object's identity and its persistentIdentity. libSBOLj resolves
    /// references by either form.
    fn resolvable_uris(&self) -> std::collections::BTreeSet<String> {
        let mut set = std::collections::BTreeSet::new();
        for object in self.document.objects().values() {
            if let Some(iri) = object.identity().as_iri() {
                set.insert(iri.as_str().to_owned());
            }
            if let Some(pid) = object
                .first_resource(SBOL2_PERSISTENT_IDENTITY)
                .and_then(Resource::as_iri)
            {
                set.insert(pid.as_str().to_owned());
            }
        }
        set
    }

    fn validate_completeness(
        &mut self,
        object: &Object,
        resolvable: &std::collections::BTreeSet<String>,
    ) {
        let specs = property_specs_for(object);
        for spec in specs.values() {
            let Some(reference) = spec.reference else {
                continue;
            };
            // Local composite children are checked by the always-on structural
            // pass; completeness governs external references to other objects.
            if reference.require_local {
                continue;
            }
            let Some(rule) = completeness_rule(spec.predicate, reference.target) else {
                continue;
            };
            for value in object.values(spec.predicate) {
                let Some(resource) = value.as_resource() else {
                    continue;
                };
                let resolved = resource
                    .as_iri()
                    .map(|iri| resolvable.contains(iri.as_str()))
                    .unwrap_or(false);
                if !resolved {
                    self.error(
                        rule,
                        object,
                        Some(spec.predicate),
                        format!(
                            "complete document must contain the object `{resource}` referenced by \
                             `{}`",
                            spec.predicate
                        ),
                    );
                }
            }
        }
    }

    /// Reference properties whose target must be a specific class. libSBOLj
    /// resolves each reference and reports when it does not name an object of
    /// the required class (a missing or wrong-class target). Checked under the
    /// completeness family for references resolvable within the document.
    fn validate_reference_classes(&mut self, object: &Object) {
        // (predicate, required classes, rule). A reference fails when it names
        // an in-document object of none of the required classes.
        let wrong_class = |this: &Self, predicate: &str, classes: &[Sbol2Class]| -> Vec<String> {
            let mut bad = Vec::new();
            for value in object.resources(predicate) {
                let Some(iri) = value.as_iri() else { continue };
                let Some(target) = this.resolve(iri.as_str()) else {
                    continue;
                };
                if !classes.iter().any(|c| target.has_class(*c)) {
                    bad.push(iri.as_str().to_owned());
                }
            }
            bad
        };

        // 10604: a Component/FunctionalComponent definition must resolve to a
        // ComponentDefinition. (A Module definition targets a ModuleDefinition,
        // rule 11703, already handled by the completeness engine.)
        if (object.has_class(Sbol2Class::Component)
            || object.has_class(Sbol2Class::FunctionalComponent))
            && !object.has_class(Sbol2Class::Module)
        {
            let bad = wrong_class(self, SBOL2_DEFINITION, &[Sbol2Class::ComponentDefinition]);
            for iri in bad {
                self.error(
                    "sbol2-10604",
                    object,
                    Some(SBOL2_DEFINITION),
                    format!("definition refers to `{iri}`, which is not a ComponentDefinition"),
                );
            }
        }

        let checks: &[(&str, &[Sbol2Class], &str)] = &[
            (
                PROV_WAS_GENERATED_BY,
                &[Sbol2Class::ProvActivity],
                "sbol2-10222",
            ),
            (
                PROV_WAS_INFORMED_BY,
                &[Sbol2Class::ProvActivity],
                "sbol2-12407",
            ),
            (PROV_AGENT, &[Sbol2Class::ProvAgent], "sbol2-12606"),
            (SBOL2_MODEL, &[Sbol2Class::Model], "sbol2-11608"),
            (
                SBOL2_VARIANT_COLLECTION,
                &[Sbol2Class::Collection],
                "sbol2-13010",
            ),
            (
                SBOL2_VARIANT_DERIVATION,
                &[Sbol2Class::CombinatorialDerivation],
                "sbol2-13014",
            ),
            (
                SBOL2_EXPERIMENTAL_DATA,
                &[Sbol2Class::ExperimentalData],
                "sbol2-13404",
            ),
        ];
        for (predicate, classes, rule) in checks {
            let bad = wrong_class(self, predicate, classes);
            for iri in bad {
                self.error(
                    rule,
                    object,
                    Some(predicate),
                    format!("`{predicate}` refers to `{iri}`, which is not the required class"),
                );
            }
        }

        // 13506: the hasUnit of a Measure must refer to an om:Unit (any unit
        // subclass). A missing target is left to the completeness family; a
        // resolved target of a non-unit class fails here.
        if object.has_class(Sbol2Class::OmMeasure) {
            const UNIT_CLASSES: &[Sbol2Class] = &[
                Sbol2Class::OmUnit,
                Sbol2Class::OmSingularUnit,
                Sbol2Class::OmCompoundUnit,
                Sbol2Class::OmUnitMultiplication,
                Sbol2Class::OmUnitDivision,
                Sbol2Class::OmUnitExponentiation,
                Sbol2Class::OmPrefixedUnit,
            ];
            let mut bad = Vec::new();
            for value in object.resources(OM_HAS_UNIT) {
                let Some(iri) = value.as_iri() else { continue };
                if let Some(target) = self.resolve(iri.as_str())
                    && !UNIT_CLASSES.iter().any(|c| target.has_class(*c))
                {
                    bad.push(iri.as_str().to_owned());
                }
            }
            for iri in bad {
                self.error(
                    "sbol2-13506",
                    object,
                    Some(OM_HAS_UNIT),
                    format!("the hasUnit of a Measure `{iri}` must refer to a Unit"),
                );
            }
        }

        // 10807: the ComponentInstance named by a MapsTo remote must have
        // public access.
        if object.has_class(Sbol2Class::MapsTo) {
            let mut bad = Vec::new();
            for value in object.resources(SBOL2_REMOTE) {
                let Some(iri) = value.as_iri() else { continue };
                if let Some(remote) = self.resolve(iri.as_str())
                    && !remote
                        .resources(SBOL2_ACCESS)
                        .any(|r| r.as_iri().is_some_and(|a| a.as_str() == SBOL2_PUBLIC))
                {
                    bad.push(iri.as_str().to_owned());
                }
            }
            for iri in bad {
                self.error(
                    "sbol2-10807",
                    object,
                    Some(SBOL2_REMOTE),
                    format!("the MapsTo remote `{iri}` must have public access"),
                );
            }
        }

        // 13103: a non-empty built reference must resolve to a ComponentDefinition
        // or ModuleDefinition; a missing or wrong-class target fails.
        if object.has_class(Sbol2Class::Implementation) {
            let mut bad = Vec::new();
            for value in object.resources(SBOL2_BUILT) {
                let Some(iri) = value.as_iri() else { continue };
                let ok = self.resolve(iri.as_str()).is_some_and(|target| {
                    target.has_class(Sbol2Class::ComponentDefinition)
                        || target.has_class(Sbol2Class::ModuleDefinition)
                });
                if !ok {
                    bad.push(iri.as_str().to_owned());
                }
            }
            for iri in bad {
                self.error(
                    "sbol2-13103",
                    object,
                    Some(SBOL2_BUILT),
                    format!("built refers to `{iri}`, which is not a ComponentDefinition or ModuleDefinition"),
                );
            }
        }
    }

    /// The in-document object whose identity or persistentIdentity equals `uri`.
    fn resolve(&self, uri: &str) -> Option<&Object> {
        self.document.objects().values().find(|object| {
            object
                .identity()
                .as_iri()
                .is_some_and(|iri| iri.as_str() == uri)
                || object
                    .first_resource(SBOL2_PERSISTENT_IDENTITY)
                    .and_then(Resource::as_iri)
                    .is_some_and(|iri| iri.as_str() == uri)
        })
    }

    // --- best-practice family (gate: best_practice) --------------------

    fn validate_best_practices(&mut self, object: &Object) {
        // 10224-10227: an object generated by an Activity whose Association
        // carries a design/build/test/learn role SHOULD have the matching
        // TopLevel type.
        self.validate_activity_role_usage(object);
        self.validate_activity_usage_role_conflicts(object);
        self.validate_usage_entity_roles(object);
        self.validate_derivation_version_order(object);
        self.validate_ontology_usage(object);
        self.validate_cd_sequences(object);
        self.validate_sequence_annotation_overlaps(object);
        self.validate_component_source_lengths(object);
        self.validate_interaction_participation_roles(object);
        self.validate_combinatorial_best_practices(object);
    }

    /// ComponentDefinition sequence best practices. 10516 (an error gated with
    /// the best-practice family): a definition whose type requires a sequence
    /// category must carry a sequence of that encoding. 10518: sequences of one
    /// category should share a length. 10523: a nucleic annotation's Range/Cut
    /// positions should lie within the sequence. (10520, the implied-sequence
    /// consistency check, requires assembling the definition's nucleic sequence
    /// from its annotations and stays deferred.)
    fn validate_cd_sequences(&mut self, object: &Object) {
        if !object.has_class(Sbol2Class::ComponentDefinition) {
            return;
        }
        let mut sequences: Vec<(String, usize)> = Vec::new();
        for sequence_ref in object.resources(SBOL2_SEQUENCE) {
            let Some(iri) = sequence_ref.as_iri() else {
                continue;
            };
            let Some(sequence) = self.resolve(iri.as_str()) else {
                continue;
            };
            let Some(encoding) = sequence
                .first_resource(SBOL2_ENCODING)
                .and_then(Resource::as_iri)
                .map(|iri| iri.as_str().to_owned())
            else {
                continue;
            };
            let length = self
                .literal(sequence, SBOL2_ELEMENTS)
                .map(|elements| elements.chars().count())
                .unwrap_or(0);
            sequences.push((encoding, length));
        }
        if sequences.is_empty() {
            return;
        }

        let mut nucleic_length: Option<usize> = None;
        let mut protein_length: Option<usize> = None;
        let mut smiles_length: Option<usize> = None;
        let mut length_mismatch = false;
        for (encoding, length) in &sequences {
            let slot = match encoding.as_str() {
                IUPAC_NUCLEIC_ENCODING => &mut nucleic_length,
                IUPAC_PROTEIN_ENCODING => &mut protein_length,
                SMILES_ENCODING => &mut smiles_length,
                _ => continue,
            };
            match slot {
                Some(first) if *first != *length => length_mismatch = true,
                Some(_) => {}
                None => *slot = Some(*length),
            }
        }

        let types: Vec<&str> = object.iris(SBOL2_TYPE).map(|iri| iri.as_str()).collect();
        let wants_nucleic =
            types.contains(&BIOPAX_DNA_REGION) || types.contains(&BIOPAX_RNA_REGION);
        let missing_category = (wants_nucleic && nucleic_length.is_none())
            || (types.contains(&BIOPAX_PROTEIN) && protein_length.is_none())
            || (types.contains(&BIOPAX_SMALL_MOLECULE) && smiles_length.is_none());

        let mut out_of_bounds = false;
        if let Some(length) = nucleic_length {
            let length = length as i64;
            for sa_ref in object.resources(SBOL2_SEQUENCE_ANNOTATION) {
                let Some(sa) = self.document.get(sa_ref) else {
                    continue;
                };
                for location_ref in sa.resources(SBOL2_LOCATION) {
                    let Some(location) = self.document.get(location_ref) else {
                        continue;
                    };
                    if location.has_class(Sbol2Class::Range) {
                        let start = self.integer(location, SBOL2_START).unwrap_or(1);
                        let end = self.integer(location, SBOL2_END).unwrap_or(0);
                        if start <= 0 || end > length {
                            out_of_bounds = true;
                        }
                    } else if location.has_class(Sbol2Class::Cut) {
                        let at = self.integer(location, SBOL2_AT).unwrap_or(0);
                        if at < 0 || at > length {
                            out_of_bounds = true;
                        }
                    }
                }
            }
        }

        // 10520: within a ComponentDefinition-Component hierarchy, a child
        // ComponentDefinition's nucleic Sequence should map into the region its
        // SequenceAnnotation Range spans in the parent. A child sequence whose
        // length differs from its annotated span admits no well-defined mapping.
        let mut hierarchy_inconsistent = false;
        if nucleic_length.is_some() {
            for sa_ref in object.resources(SBOL2_SEQUENCE_ANNOTATION) {
                let Some(sa) = self.document.get(sa_ref) else {
                    continue;
                };
                let Some(component_iri) = sa
                    .first_resource(SBOL2_COMPONENT)
                    .and_then(Resource::as_iri)
                else {
                    continue;
                };
                let Some(span) = self
                    .location_extents(sa)
                    .iter()
                    .find_map(|region| match region {
                        Region::Range(start, end) => Some(end - start + 1),
                        Region::Cut(_) => None,
                    })
                else {
                    continue;
                };
                let Some(definition_iri) = self
                    .resolve(component_iri.as_str())
                    .and_then(|component| {
                        component
                            .first_resource(SBOL2_DEFINITION)
                            .and_then(Resource::as_iri)
                    })
                    .map(|iri| iri.as_str().to_owned())
                else {
                    continue;
                };
                let Some(child_length) = self
                    .resolve(&definition_iri)
                    .and_then(|definition| self.definition_nucleic_length(definition))
                else {
                    continue;
                };
                if child_length != span {
                    hierarchy_inconsistent = true;
                }
            }
        }

        if missing_category {
            self.error(
                "sbol2-10516",
                object,
                Some(SBOL2_SEQUENCE),
                "a ComponentDefinition must carry a Sequence whose encoding matches its type",
            );
        }
        if length_mismatch {
            self.warning(
                "sbol2-10518",
                object,
                Some(SBOL2_SEQUENCE),
                "a ComponentDefinition's Sequences of one encoding should share a length",
            );
        }
        if out_of_bounds {
            self.warning(
                "sbol2-10523",
                object,
                Some(SBOL2_SEQUENCE_ANNOTATION),
                "a SequenceAnnotation position should lie within the ComponentDefinition's sequence",
            );
        }
        if hierarchy_inconsistent {
            self.warning(
                "sbol2-10520",
                object,
                Some(SBOL2_SEQUENCE),
                "Sequences across a ComponentDefinition-Component hierarchy should be consistent",
            );
        }
    }

    /// The length of the IUPAC-nucleic Sequence a ComponentDefinition directly
    /// references, if any.
    fn definition_nucleic_length(&self, definition: &Object) -> Option<i64> {
        for sequence_ref in definition.resources(SBOL2_SEQUENCE) {
            let Some(iri) = sequence_ref.as_iri() else {
                continue;
            };
            let Some(sequence) = self.resolve(iri.as_str()) else {
                continue;
            };
            let is_nucleic = sequence
                .first_resource(SBOL2_ENCODING)
                .and_then(Resource::as_iri)
                .is_some_and(|iri| iri.as_str() == IUPAC_NUCLEIC_ENCODING);
            if is_nucleic && let Some(elements) = self.literal(sequence, SBOL2_ELEMENTS) {
                return Some(elements.chars().count() as i64);
            }
        }
        None
    }

    /// 10903: two Locations of one SequenceAnnotation should not overlap.
    fn validate_sequence_annotation_overlaps(&mut self, object: &Object) {
        if !object.has_class(Sbol2Class::SequenceAnnotation) {
            return;
        }
        let locations = self.location_extents(object);
        if regions_overlap(&locations) {
            self.warning(
                "sbol2-10903",
                object,
                Some(SBOL2_LOCATION),
                "the Locations of a SequenceAnnotation should not overlap",
            );
        }
    }

    /// 10711: a Component's source Locations should not overlap. 10712: the total
    /// length of a Component's source Ranges should equal the total length of the
    /// Ranges annotating it.
    fn validate_component_source_lengths(&mut self, object: &Object) {
        if !object.has_class(Sbol2Class::Component) {
            return;
        }
        let source = self.location_extents_of(object, SBOL2_SOURCE_LOCATION);
        if regions_overlap(&source) {
            self.warning(
                "sbol2-10711",
                object,
                Some(SBOL2_SOURCE_LOCATION),
                "the source Locations of a Component should not overlap",
            );
        }
        let Some(component_id) = object
            .identity()
            .as_iri()
            .map(|iri| iri.as_str().to_owned())
        else {
            return;
        };
        let annotation = self.document.objects().values().find(|candidate| {
            candidate.has_class(Sbol2Class::SequenceAnnotation)
                && candidate
                    .first_resource(SBOL2_COMPONENT)
                    .and_then(Resource::as_iri)
                    .is_some_and(|iri| iri.as_str() == component_id)
        });
        let Some(annotation) = annotation else {
            return;
        };
        let target = self.location_extents(annotation);
        let source_len = range_span(&source);
        let target_len = range_span(&target);
        if let (Some(source_len), Some(target_len)) = (source_len, target_len)
            && source_len != target_len
        {
            self.warning(
                "sbol2-10712",
                object,
                Some(SBOL2_SOURCE_LOCATION),
                "a Component's source Range length should equal its annotation Range length",
            );
        }
    }

    /// The Range/Cut extents of the Locations named by `object`'s `location`
    /// property.
    fn location_extents(&self, object: &Object) -> Vec<Region> {
        self.location_extents_of(object, SBOL2_LOCATION)
    }

    fn location_extents_of(&self, object: &Object, predicate: &str) -> Vec<Region> {
        let mut regions = Vec::new();
        for location_ref in object.resources(predicate) {
            let Some(location) = self.document.get(location_ref) else {
                continue;
            };
            if location.has_class(Sbol2Class::Range) {
                if let (Some(start), Some(end)) = (
                    self.integer(location, SBOL2_START),
                    self.integer(location, SBOL2_END),
                ) {
                    regions.push(Region::Range(start, end));
                }
            } else if location.has_class(Sbol2Class::Cut)
                && let Some(at) = self.integer(location, SBOL2_AT)
            {
                regions.push(Region::Cut(at));
            }
        }
        regions
    }

    /// 11907: an Interaction's type must be compatible with the roles of its
    /// Participations, per the SBO interaction/participant table.
    fn validate_interaction_participation_roles(&mut self, object: &Object) {
        if !object.has_class(Sbol2Class::Interaction) {
            return;
        }
        let ontology = sbol_ontology::Ontology::bundled();
        // The Interaction's single occurring-entity SBO type; 11905 governs the
        // "exactly one" case, so a lone type is used here.
        let types: Vec<String> = object
            .iris(SBOL2_TYPE)
            .filter(|iri| {
                ontology.is_in_branch(
                    ontology_curie(iri.as_str()),
                    SBO_OCCURRING_ENTITY_REPRESENTATION,
                )
            })
            .map(|iri| ontology_curie(iri.as_str()).to_owned())
            .collect();
        let [interaction_type] = types.as_slice() else {
            return;
        };
        let Some(allowed) = participant_roles_for_interaction(interaction_type) else {
            return;
        };
        // Each Participation's single participant-role SBO term.
        let mut offending = false;
        for participation_ref in object.resources(SBOL2_PARTICIPATION) {
            let Some(participation) = self.document.get(participation_ref) else {
                continue;
            };
            let roles: Vec<String> = participation
                .iris(SBOL2_ROLE)
                .filter(|iri| {
                    ontology.is_in_branch(ontology_curie(iri.as_str()), SBO_PARTICIPANT_ROLE)
                })
                .map(|iri| ontology_curie(iri.as_str()).to_owned())
                .collect();
            if let [role] = roles.as_slice()
                && !allowed.contains(&role.as_str())
            {
                offending = true;
            }
        }
        if offending {
            self.warning(
                "sbol2-11907",
                object,
                Some(SBOL2_TYPE),
                "an Interaction's participant roles should be compatible with its SBO type",
            );
        }
    }

    /// CombinatorialDerivation best practices, all computed over in-document
    /// references. 12909/13006 flag empty templates and variable components;
    /// 12910/12911/13018-13022 flag a ComponentDefinition derived from a
    /// CombinatorialDerivation that departs from its template; 12912/12913 flag
    /// a Collection whose wasDerivedFrom set disagrees with its members'.
    fn validate_combinatorial_best_practices(&mut self, object: &Object) {
        let mut findings: Vec<(&'static str, Resource, &'static str)> = Vec::new();
        if object.has_class(Sbol2Class::CombinatorialDerivation) {
            // 12909: the template ComponentDefinition should have Components.
            if let Some(template) = object
                .first_resource(SBOL2_TEMPLATE)
                .and_then(Resource::as_iri)
                .and_then(|iri| self.resolve(iri.as_str()))
                && template.resources(SBOL2_COMPONENT).next().is_none()
            {
                findings.push(("sbol2-12909", object.identity().clone(), SBOL2_TEMPLATE));
            }
            // 13006: a VariableComponent should offer at least one variant source.
            for variable_ref in object.resources(SBOL2_VARIABLE_COMPONENT) {
                let Some(variable) = self.document.get(variable_ref) else {
                    continue;
                };
                let empty = variable.resources(SBOL2_VARIANT).next().is_none()
                    && variable
                        .resources(SBOL2_VARIANT_COLLECTION)
                        .next()
                        .is_none()
                    && variable
                        .resources(SBOL2_VARIANT_DERIVATION)
                        .next()
                        .is_none();
                if empty {
                    findings.push(("sbol2-13006", variable.identity().clone(), SBOL2_VARIANT));
                }
            }
        }
        if object.has_class(Sbol2Class::ComponentDefinition) {
            self.collect_derived_component_findings(object, &mut findings);
        }
        if object.has_class(Sbol2Class::Collection) {
            self.collect_collection_findings(object, &mut findings);
        }
        for (rule, identity, property) in findings {
            self.warning_at(rule, &identity, Some(property), combinatorial_message(rule));
        }
    }

    /// Findings for a ComponentDefinition derived from a CombinatorialDerivation:
    /// type/role agreement with the template (12910/12911/13018) and the
    /// per-variable-component realization counts (13019-13022).
    fn collect_derived_component_findings(
        &self,
        object: &Object,
        findings: &mut Vec<(&'static str, Resource, &'static str)>,
    ) {
        for derived_ref in object.resources(PROV_WAS_DERIVED_FROM) {
            let Some(iri) = derived_ref.as_iri() else {
                continue;
            };
            let Some(derivation) = self.resolve(iri.as_str()) else {
                continue;
            };
            if !derivation.has_class(Sbol2Class::CombinatorialDerivation) {
                continue;
            }
            let Some(template) = derivation
                .first_resource(SBOL2_TEMPLATE)
                .and_then(Resource::as_iri)
                .and_then(|iri| self.resolve(iri.as_str()))
            else {
                continue;
            };
            let identity = object.identity().clone();
            // 12910 / 12911: derived definition should keep the template's types
            // and roles.
            if set_of(object, SBOL2_TYPE) != set_of(template, SBOL2_TYPE) {
                findings.push(("sbol2-12910", identity.clone(), SBOL2_TYPE));
            }
            if set_of(object, SBOL2_ROLE) != set_of(template, SBOL2_ROLE) {
                findings.push(("sbol2-12911", identity.clone(), SBOL2_ROLE));
            }
            // 13018: a derived Component should keep its template Component's roles.
            for component_ref in object.resources(SBOL2_COMPONENT) {
                let Some(component) = self.document.get(component_ref) else {
                    continue;
                };
                for source in component.resources(PROV_WAS_DERIVED_FROM) {
                    let Some(source_iri) = source.as_iri() else {
                        continue;
                    };
                    let Some(template_component) =
                        self.child_by_identity(template, SBOL2_COMPONENT, source_iri.as_str())
                    else {
                        continue;
                    };
                    if set_of(component, SBOL2_ROLE) != set_of(template_component, SBOL2_ROLE) {
                        findings.push(("sbol2-13018", identity.clone(), SBOL2_ROLE));
                    }
                }
            }
            // 13022: each non-replaced template Component should be realized once.
            let replaced: std::collections::BTreeSet<String> = derivation
                .resources(SBOL2_VARIABLE_COMPONENT)
                .filter_map(|vc_ref| self.document.get(vc_ref))
                .filter_map(|vc| vc.first_resource(SBOL2_VARIABLE).and_then(Resource::as_iri))
                .map(|iri| iri.as_str().to_owned())
                .collect();
            for template_component_ref in template.resources(SBOL2_COMPONENT) {
                let Some(template_component_id) = template_component_ref.as_iri() else {
                    continue;
                };
                if replaced.contains(template_component_id.as_str()) {
                    continue;
                }
                let count = self.count_derived_from(object, template_component_id.as_str());
                if count != 1 {
                    findings.push(("sbol2-13022", identity.clone(), SBOL2_COMPONENT));
                }
            }
            // 13019 / 13020 / 13021: realization counts by operator.
            for vc_ref in derivation.resources(SBOL2_VARIABLE_COMPONENT) {
                let Some(vc) = self.document.get(vc_ref) else {
                    continue;
                };
                let Some(variable) = vc.first_resource(SBOL2_VARIABLE).and_then(Resource::as_iri)
                else {
                    continue;
                };
                let operator = vc
                    .first_resource(SBOL2_OPERATOR)
                    .and_then(Resource::as_iri)
                    .map(|iri| iri.as_str().to_owned());
                let count = self.count_derived_from(object, variable.as_str());
                match operator.as_deref() {
                    Some(SBOL2_OP_ZERO_OR_ONE) if count > 1 => {
                        findings.push(("sbol2-13019", identity.clone(), SBOL2_COMPONENT));
                    }
                    Some(SBOL2_OP_ONE) if count != 1 => {
                        findings.push(("sbol2-13020", identity.clone(), SBOL2_COMPONENT));
                    }
                    Some(SBOL2_OP_ONE_OR_MORE) if count == 0 => {
                        findings.push(("sbol2-13021", identity.clone(), SBOL2_COMPONENT));
                    }
                    _ => {}
                }
            }
        }
    }

    /// Findings for a Collection whose members derive from a
    /// CombinatorialDerivation: the Collection and its members should share the
    /// deriving wasDerivedFrom URIs (12912/12913).
    fn collect_collection_findings(
        &self,
        object: &Object,
        findings: &mut Vec<(&'static str, Resource, &'static str)>,
    ) {
        let members: Vec<&Object> = object
            .resources(SBOL2_MEMBER)
            .filter_map(|member_ref| member_ref.as_iri())
            .filter_map(|iri| self.resolve(iri.as_str()))
            .collect();
        let identity = object.identity().clone();
        // 12913: when the Collection derives from a CombinatorialDerivation, each
        // member should record the same derivation.
        for derived in object.resources(PROV_WAS_DERIVED_FROM) {
            let Some(iri) = derived.as_iri() else {
                continue;
            };
            let is_combinatorial = self
                .resolve(iri.as_str())
                .is_some_and(|target| target.has_class(Sbol2Class::CombinatorialDerivation));
            if !is_combinatorial {
                continue;
            }
            for member in &members {
                if !self.lists(member, PROV_WAS_DERIVED_FROM, iri.as_str()) {
                    findings.push(("sbol2-12913", identity.clone(), SBOL2_MEMBER));
                }
            }
        }
        // 12912: when a member derives from a CombinatorialDerivation, the
        // Collection should record the same derivation.
        for member in &members {
            for derived in member.resources(PROV_WAS_DERIVED_FROM) {
                let Some(iri) = derived.as_iri() else {
                    continue;
                };
                let is_combinatorial = self
                    .resolve(iri.as_str())
                    .is_some_and(|target| target.has_class(Sbol2Class::CombinatorialDerivation));
                if is_combinatorial && !self.lists(object, PROV_WAS_DERIVED_FROM, iri.as_str()) {
                    findings.push(("sbol2-12912", identity.clone(), SBOL2_MEMBER));
                }
            }
        }
    }

    /// The child of `parent` listed under `predicate` whose identity is `id`.
    fn child_by_identity(&self, parent: &Object, predicate: &str, id: &str) -> Option<&Object> {
        parent
            .resources(predicate)
            .find(|r| r.as_iri().is_some_and(|iri| iri.as_str() == id))
            .and_then(|r| self.document.get(r))
    }

    /// How many Components of `object` carry `derived_from` in wasDerivedFrom.
    fn count_derived_from(&self, object: &Object, derived_from: &str) -> usize {
        object
            .resources(SBOL2_COMPONENT)
            .filter_map(|component_ref| self.document.get(component_ref))
            .filter(|component| self.lists(component, PROV_WAS_DERIVED_FROM, derived_from))
            .count()
    }

    /// Ontology-recommendation checks: whether the SO/SBO terms a
    /// ComponentDefinition, Component, Interaction, Participation, or Measure
    /// carries fall in the branch the specification recommends for that field.
    fn validate_ontology_usage(&mut self, object: &Object) {
        let ontology = sbol_ontology::Ontology::bundled();
        if object.has_class(Sbol2Class::ComponentDefinition) {
            let types: Vec<&str> = object.iris(SBOL2_TYPE).map(|iri| iri.as_str()).collect();
            let is_dna_or_rna_region = types
                .iter()
                .any(|t| *t == BIOPAX_DNA_REGION || *t == BIOPAX_RNA_REGION);
            let is_dna_or_rna_molecule = types
                .iter()
                .any(|t| *t == BIOPAX_DNA_MOLECULE || *t == BIOPAX_RNA_MOLECULE);

            // 10503 / 10525: exactly one Table 2 (BioPAX) type is recommended;
            // more than one is forbidden.
            let num_biopax = types.iter().filter(|t| BIOPAX_TYPES.contains(t)).count();
            if num_biopax == 0 {
                self.warning(
                    "sbol2-10525",
                    object,
                    Some(SBOL2_TYPE),
                    "a ComponentDefinition should contain a Table 2 type",
                );
            } else if num_biopax > 1 {
                self.error(
                    "sbol2-10503",
                    object,
                    Some(SBOL2_TYPE),
                    "a ComponentDefinition must not contain more than one Table 2 type",
                );
            }

            let num_so = object
                .iris(SBOL2_ROLE)
                .filter(|iri| {
                    ontology.is_in_branch(ontology_curie(iri.as_str()), SO_SEQUENCE_FEATURE)
                })
                .count();
            let num_topo = types
                .iter()
                .filter(|t| ontology.is_in_branch(ontology_curie(t), SO_TOPOLOGY_ATTRIBUTE))
                .count();
            let num_strand = types
                .iter()
                .filter(|t| ontology.is_in_branch(ontology_curie(t), SO_STRAND_ATTRIBUTE))
                .count();

            if is_dna_or_rna_region {
                // 10527: exactly one SO sequence-feature role is recommended.
                if num_so != 1 {
                    self.warning(
                        "sbol2-10527",
                        object,
                        Some(SBOL2_ROLE),
                        "a DNA/RNA ComponentDefinition should contain exactly one sequence-feature role",
                    );
                }
                // 10528: at most one topology-attribute type.
                if num_topo > 1 {
                    self.warning(
                        "sbol2-10528",
                        object,
                        Some(SBOL2_TYPE),
                        "a DNA/RNA ComponentDefinition should contain at most one topology type",
                    );
                }
            } else if !is_dna_or_rna_molecule {
                // 10511: no SO sequence-feature role without a DNA/RNA type.
                if num_so != 0 {
                    self.warning(
                        "sbol2-10511",
                        object,
                        Some(SBOL2_ROLE),
                        "a sequence-feature role should not appear without a DNA or RNA type",
                    );
                }
                // 10529: no topology or strand type without a DNA/RNA type.
                if num_topo != 0 || num_strand != 0 {
                    self.warning(
                        "sbol2-10529",
                        object,
                        Some(SBOL2_TYPE),
                        "a topology or strand type should not appear without a DNA or RNA type",
                    );
                }
            }
        }

        if object.has_class(Sbol2Class::Component) {
            // 10706 / 10707: a Component's sequence-feature roles are recommended
            // only when its definition is a DNA/RNA region.
            if let Some(def_types) = self.definition_types(object) {
                let def_is_dna_rna_region = def_types
                    .iter()
                    .any(|t| t == BIOPAX_DNA_REGION || t == BIOPAX_RNA_REGION);
                let num_so = object
                    .iris(SBOL2_ROLE)
                    .filter(|iri| {
                        ontology.is_in_branch(ontology_curie(iri.as_str()), SO_SEQUENCE_FEATURE)
                    })
                    .count();
                if !def_is_dna_rna_region {
                    if num_so != 0 {
                        self.warning(
                            "sbol2-10706",
                            object,
                            Some(SBOL2_ROLE),
                            "a Component role should be a sequence-feature term only for DNA/RNA definitions",
                        );
                    }
                } else if num_so > 1 {
                    self.warning(
                        "sbol2-10707",
                        object,
                        Some(SBOL2_ROLE),
                        "a DNA/RNA Component should contain at most one sequence-feature role",
                    );
                }
            }
        }

        if object.has_class(Sbol2Class::Interaction) {
            // 11905: exactly one occurring-entity-representation SBO type.
            let num = object
                .iris(SBOL2_TYPE)
                .filter(|iri| {
                    ontology.is_in_branch(
                        ontology_curie(iri.as_str()),
                        SBO_OCCURRING_ENTITY_REPRESENTATION,
                    )
                })
                .count();
            if num != 1 {
                self.warning(
                    "sbol2-11905",
                    object,
                    Some(SBOL2_TYPE),
                    "an Interaction should contain exactly one occurring-entity SBO type",
                );
            }
        }

        if object.has_class(Sbol2Class::Participation) {
            // 12007: exactly one participant-role SBO term.
            let num = object
                .iris(SBOL2_ROLE)
                .filter(|iri| {
                    ontology.is_in_branch(ontology_curie(iri.as_str()), SBO_PARTICIPANT_ROLE)
                })
                .count();
            if num != 1 {
                self.warning(
                    "sbol2-12007",
                    object,
                    Some(SBOL2_ROLE),
                    "a Participation should contain exactly one participant-role SBO term",
                );
            }
        }

        if object.has_class(Sbol2Class::Sequence) {
            // 10407: the encoding of a Sequence should be a Table 1 URI.
            for encoding in object.iris(SBOL2_ENCODING) {
                if !TABLE_1_ENCODINGS.contains(&encoding.as_str()) {
                    self.warning(
                        "sbol2-10407",
                        object,
                        Some(SBOL2_ENCODING),
                        "the encoding of a Sequence should be a URI from Table 1",
                    );
                }
            }
        }

        if object.has_class(Sbol2Class::SequenceConstraint) {
            // 11412: the restriction of a SequenceConstraint should be a Table 7 URI.
            for restriction in object.iris(SBOL2_RESTRICTION) {
                if !TABLE_7_RESTRICTIONS.contains(&restriction.as_str()) {
                    self.warning(
                        "sbol2-11412",
                        object,
                        Some(SBOL2_RESTRICTION),
                        "the restriction of a SequenceConstraint should be a URI from Table 7",
                    );
                }
            }
        }

        if object.has_class(Sbol2Class::Model) {
            // 11507: a Model's language should be an EDAM ontology term.
            for language in object.iris(SBOL2_LANGUAGE) {
                if !is_edam_iri(language.as_str()) {
                    self.warning(
                        "sbol2-11507",
                        object,
                        Some(SBOL2_LANGUAGE),
                        "the language of a Model should refer to a term from the EDAM ontology",
                    );
                }
            }
            // 11511: a Model's framework should be in the SBO modeling-framework branch.
            for framework in object.iris(SBOL2_FRAMEWORK) {
                if !ontology
                    .is_in_branch(ontology_curie(framework.as_str()), SBO_MODELING_FRAMEWORK)
                {
                    self.warning(
                        "sbol2-11511",
                        object,
                        Some(SBOL2_FRAMEWORK),
                        "the framework of a Model should refer to a term from the SBO modeling-framework branch",
                    );
                }
            }
        }

        if object.has_class(Sbol2Class::Attachment) {
            // 13206: an Attachment's format should be an EDAM ontology term.
            for format in object.iris(SBOL2_FORMAT) {
                if !is_edam_iri(format.as_str()) {
                    self.warning(
                        "sbol2-13206",
                        object,
                        Some(SBOL2_FORMAT),
                        "the format of an Attachment should refer to a term from the EDAM ontology",
                    );
                }
            }
        }

        if object.has_class(Sbol2Class::OmMeasure) {
            // 13505: a Measure with types should carry exactly one
            // systems-description-parameter SBO type.
            let types: Vec<&str> = object.iris(SBOL2_TYPE).map(|iri| iri.as_str()).collect();
            if !types.is_empty() {
                let num = types
                    .iter()
                    .filter(|t| {
                        ontology.is_in_branch(ontology_curie(t), SBO_SYSTEMS_DESCRIPTION_PARAMETER)
                    })
                    .count();
                if num != 1 {
                    self.warning(
                        "sbol2-13505",
                        object,
                        Some(SBOL2_TYPE),
                        "a Measure with types should carry exactly one systems-description-parameter SBO type",
                    );
                }
            }
        }
    }

    /// The BioPAX/SO type IRIs of the ComponentDefinition referenced by a
    /// Component's definition property.
    fn definition_types(&self, component: &Object) -> Option<Vec<String>> {
        let def = component
            .first_resource(SBOL2_DEFINITION)
            .and_then(Resource::as_iri)?;
        let cd = self.document.objects().values().find(|o| {
            o.identity()
                .as_iri()
                .is_some_and(|iri| iri.as_str() == def.as_str())
        })?;
        Some(
            cd.iris(SBOL2_TYPE)
                .map(|iri| iri.as_str().to_owned())
                .collect(),
        )
    }

    fn validate_activity_role_usage(&mut self, object: &Object) {
        let is_implementation = object.has_class(Sbol2Class::Implementation);
        let is_experimental_data = object.has_class(Sbol2Class::ExperimentalData);
        let is_top_level = object.is_top_level();
        for activity_ref in object.resources(PROV_WAS_GENERATED_BY) {
            let Some(activity) = self.document.get(activity_ref) else {
                continue;
            };
            for association_ref in activity
                .resources(PROV_QUALIFIED_ASSOCIATION)
                .cloned()
                .collect::<Vec<_>>()
            {
                let Some(association) = self.document.get(&association_ref) else {
                    continue;
                };
                for role in association.values(PROV_HAD_ROLE) {
                    let Some(role) = role.as_iri() else {
                        continue;
                    };
                    match role.as_str() {
                        SBOL2_ROLE_DESIGN if is_implementation || !is_top_level => self.error(
                            "sbol2-10224",
                            object,
                            None,
                            "an object generated by a design Activity SHOULD be a TopLevel other \
                             than Implementation",
                        ),
                        SBOL2_ROLE_BUILD if !is_implementation => self.error(
                            "sbol2-10225",
                            object,
                            None,
                            "an object generated by a build Activity SHOULD be an Implementation",
                        ),
                        SBOL2_ROLE_TEST if !is_experimental_data => self.error(
                            "sbol2-10226",
                            object,
                            None,
                            "an object generated by a test Activity SHOULD be ExperimentalData",
                        ),
                        SBOL2_ROLE_LEARN if is_implementation => self.error(
                            "sbol2-10227",
                            object,
                            None,
                            "an object generated by a learn Activity SHOULD NOT be an Implementation",
                        ),
                        _ => {}
                    }
                }
            }
        }
    }

    /// 12408-12411: an Activity whose Association carries a design/build/test/
    /// learn role should not carry Usage objects with the conflicting roles the
    /// design-build-test-learn cycle forbids.
    fn validate_activity_usage_role_conflicts(&mut self, object: &Object) {
        if !object.has_class(Sbol2Class::ProvActivity) {
            return;
        }
        let association_roles: std::collections::BTreeSet<String> = object
            .resources(PROV_QUALIFIED_ASSOCIATION)
            .filter_map(|assoc_ref| self.document.get(assoc_ref))
            .flat_map(|assoc| assoc.iris(PROV_HAD_ROLE).map(|iri| iri.as_str().to_owned()))
            .collect();
        let usage_roles: std::collections::BTreeSet<String> = object
            .resources(PROV_QUALIFIED_USAGE)
            .filter_map(|usage_ref| self.document.get(usage_ref))
            .flat_map(|usage| usage.iris(PROV_HAD_ROLE).map(|iri| iri.as_str().to_owned()))
            .collect();
        // (association role, rule, forbidden usage roles).
        let table: &[(&str, &str, &[&str])] = &[
            (
                SBOL2_ROLE_DESIGN,
                "sbol2-12408",
                &[SBOL2_ROLE_BUILD, SBOL2_ROLE_TEST],
            ),
            (
                SBOL2_ROLE_BUILD,
                "sbol2-12409",
                &[SBOL2_ROLE_TEST, SBOL2_ROLE_LEARN],
            ),
            (
                SBOL2_ROLE_TEST,
                "sbol2-12410",
                &[SBOL2_ROLE_DESIGN, SBOL2_ROLE_LEARN],
            ),
            (
                SBOL2_ROLE_LEARN,
                "sbol2-12411",
                &[SBOL2_ROLE_DESIGN, SBOL2_ROLE_BUILD],
            ),
        ];
        for (association_role, rule, forbidden) in table {
            if association_roles.contains(*association_role)
                && forbidden.iter().any(|r| usage_roles.contains(*r))
            {
                self.warning(
                    rule,
                    object,
                    Some(PROV_QUALIFIED_USAGE),
                    "an Activity's Usage roles should not conflict with its Association role",
                );
            }
        }
    }

    /// 12504-12507: a Usage carrying a design/build/test/learn role should refer
    /// to an entity of the TopLevel kind the design-build-test-learn cycle
    /// expects. Only decided when the entity resolves in-document.
    fn validate_usage_entity_roles(&mut self, object: &Object) {
        if !object.has_class(Sbol2Class::ProvUsage) {
            return;
        }
        let Some(entity) = object
            .first_resource(PROV_ENTITY)
            .and_then(Resource::as_iri)
            .and_then(|iri| self.resolve(iri.as_str()))
        else {
            return;
        };
        let is_implementation = entity.has_class(Sbol2Class::Implementation);
        let is_attachment_or_collection =
            entity.has_class(Sbol2Class::Attachment) || entity.has_class(Sbol2Class::Collection);
        for role in object.iris(PROV_HAD_ROLE) {
            let (rule, offends): (&str, bool) = match role.as_str() {
                // 12504: design should refer to a TopLevel other than Implementation.
                SBOL2_ROLE_DESIGN => ("sbol2-12504", is_implementation),
                // 12505: build should refer to an Implementation.
                SBOL2_ROLE_BUILD => ("sbol2-12505", !is_implementation),
                // 12506: test should refer to an Attachment or Collection.
                SBOL2_ROLE_TEST => ("sbol2-12506", !is_attachment_or_collection),
                // 12507: learn should not refer to an Implementation.
                SBOL2_ROLE_LEARN => ("sbol2-12507", is_implementation),
                _ => continue,
            };
            if offends {
                self.warning(
                    rule,
                    object,
                    Some(PROV_ENTITY),
                    "a Usage's entity should match the TopLevel kind its role expects",
                );
            }
        }
    }

    /// 10302: when a TopLevel derives from another TopLevel with the same
    /// persistentIdentity and both carry a version, the source version should
    /// precede the deriving version under semantic-versioning order.
    fn validate_derivation_version_order(&mut self, object: &Object) {
        if !object.is_top_level() {
            return;
        }
        let Some(pid) = object
            .first_resource(SBOL2_PERSISTENT_IDENTITY)
            .and_then(Resource::as_iri)
            .map(|iri| iri.as_str().to_owned())
        else {
            return;
        };
        let Some(version) = self.literal(object, SBOL2_VERSION) else {
            return;
        };
        for derived in object.resources(PROV_WAS_DERIVED_FROM) {
            let Some(iri) = derived.as_iri() else {
                continue;
            };
            let Some(source) = self.resolve(iri.as_str()) else {
                continue;
            };
            if !source.is_top_level() {
                continue;
            }
            let source_pid = source
                .first_resource(SBOL2_PERSISTENT_IDENTITY)
                .and_then(Resource::as_iri)
                .map(|iri| iri.as_str().to_owned());
            if source_pid.as_deref() != Some(pid.as_str()) {
                continue;
            }
            if let Some(source_version) = self.literal(source, SBOL2_VERSION)
                && !version_precedes(&source_version, &version)
            {
                self.warning(
                    "sbol2-10302",
                    object,
                    Some(SBOL2_VERSION),
                    "a derived TopLevel's source version should precede its own version",
                );
            }
        }
    }

    // --- document namespace (Always) ----------------------------------

    /// 10101: an SBOL document must use the SBOL 2 namespace. After parsing the
    /// declared namespaces are not retained, but a conforming document names at
    /// least one term in the SBOL 2 namespace; a document whose SBOL prefix is
    /// misbound (so every term lands in a look-alike namespace) uses none.
    fn validate_document_namespace(&mut self) {
        let triples = self.document.rdf_graph().triples();
        // An empty document declares but does not exercise the namespace; only a
        // document with content that never touches the SBOL 2 namespace has
        // misbound its SBOL prefix.
        if triples.is_empty() {
            return;
        }
        // A conforming document names SBOL 2 properties (predicates) or declares
        // SBOL 2 classes (rdf:type objects). Enumerated *values* in the SBOL 2
        // namespace (access, orientation, restriction) do not count: a document
        // with a misbound SBOL prefix still writes those value IRIs literally.
        let uses_sbol2 = triples.iter().any(|triple| {
            triple.predicate.as_str().starts_with(SBOL2_NS)
                || (triple.predicate.as_str() == RDF_TYPE
                    && triple
                        .object
                        .as_iri()
                        .is_some_and(|iri| iri.as_str().starts_with(SBOL2_NS)))
        });
        if !uses_sbol2 {
            let identity = Resource::Iri(Iri::new_unchecked(SBOL2_NS));
            self.error_at(
                "sbol2-10101",
                &identity,
                None,
                "an SBOL document must declare and use the SBOL 2 namespace \
                 `http://sbols.org/v2#`",
            );
        }
    }

    // --- instance-graph cycles (Always) --------------------------------

    /// 10605 / 11705 / 13015: the in-document reference graph among definitions
    /// must be acyclic. A ComponentDefinition reaches others through its
    /// Components' definitions, a ModuleDefinition through its Modules'
    /// definitions, and a CombinatorialDerivation through its VariableComponents'
    /// variantDerivations. Each mirrors libSBOLj's per-definition cycle walk.
    fn validate_instance_graph_cycles(&mut self) {
        let cd = self.cycle_offenders(
            Sbol2Class::ComponentDefinition,
            SBOL2_COMPONENT,
            SBOL2_DEFINITION,
        );
        let md = self.cycle_offenders(Sbol2Class::ModuleDefinition, SBOL2_MODULE, SBOL2_DEFINITION);
        let combo = self.cycle_offenders(
            Sbol2Class::CombinatorialDerivation,
            SBOL2_VARIABLE_COMPONENT,
            SBOL2_VARIANT_DERIVATION,
        );
        for identity in cd {
            self.error_at(
                "sbol2-10605",
                &identity,
                Some(SBOL2_COMPONENT),
                "a ComponentDefinition must not form a cycle through its Components' definitions",
            );
        }
        for identity in md {
            self.error_at(
                "sbol2-11705",
                &identity,
                Some(SBOL2_MODULE),
                "a ModuleDefinition must not form a cycle through its Modules' definitions",
            );
        }
        for identity in combo {
            self.error_at(
                "sbol2-13015",
                &identity,
                Some(SBOL2_VARIABLE_COMPONENT),
                "a CombinatorialDerivation must not form a cycle through its variantDerivations",
            );
        }
    }

    /// Identities of `class` definitions whose reference graph reaches a cycle.
    /// `instance_pred` names the definition's child instances, and each instance's
    /// `link_pred` names the definition it points at (resolved in-document).
    fn cycle_offenders(
        &self,
        class: Sbol2Class,
        instance_pred: &str,
        link_pred: &str,
    ) -> Vec<Resource> {
        let mut graph: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for object in self.document.objects().values() {
            if !object.has_class(class) {
                continue;
            }
            let Some(id) = object.identity().as_iri() else {
                continue;
            };
            let mut successors = Vec::new();
            for instance_ref in object.resources(instance_pred) {
                let Some(instance) = self.document.get(instance_ref) else {
                    continue;
                };
                let Some(link) = instance
                    .first_resource(link_pred)
                    .and_then(Resource::as_iri)
                else {
                    continue;
                };
                if let Some(target) = self.resolve(link.as_str())
                    && let Some(target_id) = target.identity().as_iri()
                {
                    successors.push(target_id.as_str().to_owned());
                }
            }
            graph.insert(id.as_str().to_owned(), successors);
        }
        let mut offenders = Vec::new();
        for object in self.document.objects().values() {
            if !object.has_class(class) {
                continue;
            }
            let Some(id) = object.identity().as_iri() else {
                continue;
            };
            let mut path = std::collections::BTreeSet::new();
            if reaches_cycle(id.as_str(), &graph, &mut path) {
                offenders.push(object.identity().clone());
            }
        }
        offenders
    }

    // --- generation-provenance cycles (Always) -------------------------

    /// 10223: the provenance history formed by wasGeneratedBy (object to
    /// Activity) and the entity references of that Activity's Usages (Activity
    /// back to the objects it used) must not form a cycle. The graph links an
    /// object to each entity used by an Activity that generated it; a walk that
    /// returns to its start is a circular provenance chain.
    fn validate_generation_cycles(&mut self) {
        let mut graph: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for object in self.document.objects().values() {
            let Some(id) = object.identity().as_iri() else {
                continue;
            };
            let mut successors = Vec::new();
            for activity_ref in object.resources(PROV_WAS_GENERATED_BY) {
                let Some(activity) = self.document.get(activity_ref) else {
                    continue;
                };
                for usage_ref in activity.resources(PROV_QUALIFIED_USAGE) {
                    let Some(usage) = self.document.get(usage_ref) else {
                        continue;
                    };
                    if let Some(entity) =
                        usage.first_resource(PROV_ENTITY).and_then(Resource::as_iri)
                    {
                        successors.push(entity.as_str().to_owned());
                    }
                }
            }
            graph.insert(id.as_str().to_owned(), successors);
        }
        let mut offenders = Vec::new();
        for object in self.document.objects().values() {
            let Some(id) = object.identity().as_iri() else {
                continue;
            };
            let mut path = std::collections::BTreeSet::new();
            if reaches_cycle(id.as_str(), &graph, &mut path) {
                offenders.push(object.identity().clone());
            }
        }
        for identity in offenders {
            self.error_at(
                "sbol2-10223",
                &identity,
                Some(PROV_WAS_GENERATED_BY),
                "provenance formed by wasGeneratedBy and Usage entity references must not be circular",
            );
        }
    }

    // --- MapsTo useRemote uniqueness (Always) --------------------------

    /// 10526 / 11609: within one definition, two useRemote MapsTos must not
    /// share a local. A ComponentDefinition collects its Components' MapsTos
    /// (10526); a ModuleDefinition its Modules' and FunctionalComponents' MapsTos
    /// (11609). The comparison is in-document; the cross-document remote-
    /// resolution portion stays deferred.
    fn validate_maps_to_use_remote_uniqueness(&mut self) {
        let cd = self.use_remote_collisions(Sbol2Class::ComponentDefinition, &[SBOL2_COMPONENT]);
        let md = self.use_remote_collisions(
            Sbol2Class::ModuleDefinition,
            &[SBOL2_MODULE, SBOL2_FUNCTIONAL_COMPONENT],
        );
        for identity in cd {
            self.error_at(
                "sbol2-10526",
                &identity,
                Some(SBOL2_MAPS_TO),
                "two useRemote MapsTos of a ComponentDefinition must not share a local",
            );
        }
        for identity in md {
            self.error_at(
                "sbol2-11609",
                &identity,
                Some(SBOL2_MAPS_TO),
                "two useRemote MapsTos of a ModuleDefinition must not share a local",
            );
        }
    }

    /// Definitions of `class` in which two distinct useRemote MapsTos, gathered
    /// from the instances named by `instance_preds`, reference the same local.
    fn use_remote_collisions(&self, class: Sbol2Class, instance_preds: &[&str]) -> Vec<Resource> {
        let mut offenders = Vec::new();
        for object in self.document.objects().values() {
            if !object.has_class(class) {
                continue;
            }
            let mut locals: Vec<String> = Vec::new();
            for pred in instance_preds {
                for instance_ref in object.resources(pred) {
                    let Some(instance) = self.document.get(instance_ref) else {
                        continue;
                    };
                    for maps_to_ref in instance.resources(SBOL2_MAPS_TO) {
                        let Some(maps_to) = self.document.get(maps_to_ref) else {
                            continue;
                        };
                        let use_remote = maps_to
                            .first_resource(SBOL2_REFINEMENT)
                            .and_then(Resource::as_iri)
                            .is_some_and(|iri| iri.as_str() == SBOL2_USE_REMOTE);
                        if !use_remote {
                            continue;
                        }
                        if let Some(local) = maps_to
                            .first_resource(SBOL2_LOCAL)
                            .and_then(Resource::as_iri)
                        {
                            locals.push(local.as_str().to_owned());
                        }
                    }
                }
            }
            let mut seen = std::collections::BTreeSet::new();
            if locals.iter().any(|local| !seen.insert(local.clone())) {
                offenders.push(object.identity().clone());
            }
        }
        offenders
    }

    // --- sequence encoding (Always) ------------------------------------

    /// 10405: a Sequence's elements must be consistent with its encoding. The
    /// IUPAC nucleic-acid and protein alphabets are checked locally; SMILES and
    /// unrecognized encodings are accepted (no in-crate parser).
    fn validate_sequence_encoding(&mut self, object: &Object) {
        if !object.has_class(Sbol2Class::Sequence) {
            return;
        }
        let Some(encoding) = object
            .first_resource(SBOL2_ENCODING)
            .and_then(Resource::as_iri)
            .map(|iri| iri.as_str().to_owned())
        else {
            return;
        };
        let Some(elements) = self.literal(object, SBOL2_ELEMENTS) else {
            return;
        };
        let consistent = match encoding.as_str() {
            IUPAC_NUCLEIC_ENCODING => elements
                .to_ascii_uppercase()
                .chars()
                .all(|c| IUPAC_NUCLEIC_ALPHABET.contains(c)),
            IUPAC_PROTEIN_ENCODING => elements.chars().all(|c| c.is_ascii_alphabetic()),
            _ => true,
        };
        if !consistent {
            self.error(
                "sbol2-10405",
                object,
                Some(SBOL2_ELEMENTS),
                "the elements of a Sequence must be consistent with its encoding",
            );
        }
    }

    // --- sequence-constraint positions (Always) ------------------------

    /// 11409 / 11410 / 11411: a SequenceConstraint's restriction must agree with
    /// the positions and orientations of the SequenceAnnotations that annotate
    /// its subject and object Components, all within the same ComponentDefinition.
    fn check_sequence_constraint_positions(
        &mut self,
        constraint: &Object,
        cd_id: &str,
        restriction: Option<&str>,
        subject: Option<&Iri>,
        object: Option<&Iri>,
    ) {
        let (Some(subject), Some(object)) = (subject, object) else {
            return;
        };
        let Some(cd) = self.resolve(cd_id) else {
            return;
        };
        let Some(sa_subject) = self.annotation_for_component(cd, subject.as_str()) else {
            return;
        };
        let Some(sa_object) = self.annotation_for_component(cd, object.as_str()) else {
            return;
        };
        // Resolve all positional and orientation data before emitting; the
        // annotation borrows tie to `&self` and must end before `emit_at`.
        let subject_position = self.annotation_position(sa_subject);
        let object_position = self.annotation_position(sa_object);
        let subject_orientations = self.annotation_orientations(sa_subject);
        let object_orientations = self.annotation_orientations(sa_object);
        let identity = constraint.identity().clone();
        match restriction {
            Some(SBOL2_PRECEDES) => {
                if let (Some(subject_pos), Some(object_pos)) = (subject_position, object_position)
                    && object_pos < subject_pos
                {
                    self.error_at(
                        "sbol2-11409",
                        &identity,
                        Some(SBOL2_RESTRICTION),
                        "a precedes SequenceConstraint requires the subject to be positioned before the object",
                    );
                }
            }
            Some(SBOL2_SAME_ORIENTATION_AS) => {
                let differs = subject_orientations
                    .iter()
                    .any(|s| object_orientations.iter().any(|o| s != o));
                if differs {
                    self.error_at(
                        "sbol2-11410",
                        &identity,
                        Some(SBOL2_RESTRICTION),
                        "a sameOrientationAs SequenceConstraint requires matching orientations",
                    );
                }
            }
            Some(SBOL2_OPPOSITE_ORIENTATION_AS) => {
                let matches = subject_orientations
                    .iter()
                    .any(|s| object_orientations.iter().any(|o| s == o));
                if matches {
                    self.error_at(
                        "sbol2-11411",
                        &identity,
                        Some(SBOL2_RESTRICTION),
                        "an oppositeOrientationAs SequenceConstraint requires differing orientations",
                    );
                }
            }
            _ => {}
        }
    }

    /// The SequenceAnnotation of `cd` whose component is `component`.
    fn annotation_for_component(&self, cd: &Object, component: &str) -> Option<&Object> {
        for sa_ref in cd.resources(SBOL2_SEQUENCE_ANNOTATION) {
            let Some(sa) = self.document.get(sa_ref) else {
                continue;
            };
            if sa
                .first_resource(SBOL2_COMPONENT)
                .and_then(Resource::as_iri)
                .is_some_and(|iri| iri.as_str() == component)
            {
                return Some(sa);
            }
        }
        None
    }

    /// The `(start, end)` of a SequenceAnnotation's lowest Range/Cut location,
    /// mirroring libSBOLj's sorted-location comparison. `None` when the
    /// annotation carries no positionally comparable location.
    fn annotation_position(&self, sa: &Object) -> Option<(i64, i64)> {
        let mut best: Option<(i64, i64)> = None;
        for location_ref in sa.resources(SBOL2_LOCATION) {
            let Some(location) = self.document.get(location_ref) else {
                continue;
            };
            let position = if location.has_class(Sbol2Class::Range) {
                match (
                    self.integer(location, SBOL2_START),
                    self.integer(location, SBOL2_END),
                ) {
                    (Some(start), Some(end)) => Some((start, end)),
                    _ => None,
                }
            } else if location.has_class(Sbol2Class::Cut) {
                self.integer(location, SBOL2_AT).map(|at| (at, at))
            } else {
                None
            };
            if let Some(position) = position
                && best.is_none_or(|current| position < current)
            {
                best = Some(position);
            }
        }
        best
    }

    /// The orientation of each of a SequenceAnnotation's locations, defaulting to
    /// inline when unset (the SBOL 2 default).
    fn annotation_orientations(&self, sa: &Object) -> Vec<String> {
        let mut orientations = Vec::new();
        for location_ref in sa.resources(SBOL2_LOCATION) {
            let Some(location) = self.document.get(location_ref) else {
                continue;
            };
            let orientation = location
                .first_resource(SBOL2_ORIENTATION)
                .and_then(Resource::as_iri)
                .map(|iri| iri.as_str().to_owned())
                .unwrap_or_else(|| SBOL2_INLINE.to_owned());
            orientations.push(orientation);
        }
        orientations
    }

    /// The identity URI of the object `uri` resolves to in-document.
    fn resolved_identity(&self, uri: &str) -> Option<String> {
        self.resolve(uri)
            .and_then(|object| object.identity().as_iri())
            .map(|iri| iri.as_str().to_owned())
    }

    // --- emission ------------------------------------------------------

    fn error(
        &mut self,
        rule: &'static str,
        object: &Object,
        property: Option<&'static str>,
        message: impl Into<String>,
    ) {
        self.emit(rule, object, property, message, Severity::Error);
    }

    #[allow(dead_code)]
    fn warning(
        &mut self,
        rule: &'static str,
        object: &Object,
        property: Option<&'static str>,
        message: impl Into<String>,
    ) {
        self.emit(rule, object, property, message, Severity::Warning);
    }

    fn emit(
        &mut self,
        rule: &'static str,
        object: &Object,
        property: Option<&'static str>,
        message: impl Into<String>,
        catalog_default: Severity,
    ) {
        self.emit_at(rule, object.identity(), property, message, catalog_default);
    }

    /// Emit against an object identity directly. Whole-document checks report
    /// on the offending object by identity, without a borrow of the object.
    fn error_at(
        &mut self,
        rule: &'static str,
        identity: &Resource,
        property: Option<&'static str>,
        message: impl Into<String>,
    ) {
        self.emit_at(rule, identity, property, message, Severity::Error);
    }

    fn warning_at(
        &mut self,
        rule: &'static str,
        identity: &Resource,
        property: Option<&'static str>,
        message: impl Into<String>,
    ) {
        self.emit_at(rule, identity, property, message, Severity::Warning);
    }

    fn emit_at(
        &mut self,
        rule: &'static str,
        identity: &Resource,
        property: Option<&'static str>,
        message: impl Into<String>,
        catalog_default: Severity,
    ) {
        // A rule only fires when its family is enabled by the run config, and
        // an unknown (non-catalog) rule never fires.
        let Some(gate) = rule_gate(rule) else {
            return;
        };
        if !gate_enabled(gate, self.options.config) {
            return;
        }
        let Some(severity) = self.options.resolved_severity(rule, catalog_default) else {
            return;
        };
        let issue = match severity {
            Severity::Warning => {
                ValidationIssue::warning(rule, identity.clone(), property, message)
            }
            _ => ValidationIssue::error(rule, identity.clone(), property, message),
        };
        self.issues.push(issue);
    }

    fn literal(&self, object: &Object, predicate: &str) -> Option<String> {
        object
            .values(predicate)
            .iter()
            .find_map(|term| term.as_literal().map(|l| l.value().to_owned()))
    }
}

fn gate_enabled(gate: ValidationGate, config: sbol_core::validation::ValidationConfig) -> bool {
    match gate {
        ValidationGate::Always => true,
        ValidationGate::Compliant => config.compliant,
        ValidationGate::Complete => config.complete,
        ValidationGate::BestPractice => config.best_practice,
        _ => true,
    }
}

fn rule_gate(rule: &str) -> Option<ValidationGate> {
    static CACHE: OnceLock<BTreeMap<&'static str, ValidationGate>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            validation_rule_statuses()
                .iter()
                .map(|status| (status.rule, status.gate))
                .collect()
        })
        .get(rule)
        .copied()
}

/// The "MUST NOT have properties other than ..." rule for an object's concrete
/// class. Returns `None` for classes with no such rule (abstract mixins and the
/// om: ontology classes).
fn closed_property_rule(object: &Object) -> Option<&'static str> {
    const MAP: &[(Sbol2Class, &str)] = &[
        (Sbol2Class::Sequence, "sbol2-10401"),
        (Sbol2Class::ComponentDefinition, "sbol2-10501"),
        (Sbol2Class::Component, "sbol2-10701"),
        (Sbol2Class::MapsTo, "sbol2-10801"),
        (Sbol2Class::SequenceAnnotation, "sbol2-10901"),
        (Sbol2Class::Range, "sbol2-11101"),
        (Sbol2Class::Cut, "sbol2-11201"),
        (Sbol2Class::GenericLocation, "sbol2-11301"),
        (Sbol2Class::SequenceConstraint, "sbol2-11401"),
        (Sbol2Class::Model, "sbol2-11501"),
        (Sbol2Class::ModuleDefinition, "sbol2-11601"),
        (Sbol2Class::Module, "sbol2-11701"),
        (Sbol2Class::FunctionalComponent, "sbol2-11801"),
        (Sbol2Class::Interaction, "sbol2-11901"),
        (Sbol2Class::Participation, "sbol2-12001"),
        (Sbol2Class::Collection, "sbol2-12101"),
        (Sbol2Class::GenericTopLevel, "sbol2-12301"),
        (Sbol2Class::ProvActivity, "sbol2-12401"),
        (Sbol2Class::ProvUsage, "sbol2-12501"),
        (Sbol2Class::ProvAssociation, "sbol2-12601"),
        (Sbol2Class::ProvPlan, "sbol2-12701"),
        (Sbol2Class::ProvAgent, "sbol2-12801"),
        (Sbol2Class::CombinatorialDerivation, "sbol2-12901"),
        (Sbol2Class::VariableComponent, "sbol2-13001"),
        (Sbol2Class::Implementation, "sbol2-13101"),
        (Sbol2Class::Attachment, "sbol2-13201"),
        (Sbol2Class::ExperimentalData, "sbol2-13301"),
        (Sbol2Class::Experiment, "sbol2-13401"),
        (Sbol2Class::OmMeasure, "sbol2-13501"),
    ];
    MAP.iter()
        .find(|(class, _)| object.has_class(*class))
        .map(|(_, rule)| *rule)
}

const BIOPAX_DNA_REGION: &str = "http://www.biopax.org/release/biopax-level3.owl#DnaRegion";
const BIOPAX_RNA_REGION: &str = "http://www.biopax.org/release/biopax-level3.owl#RnaRegion";
const BIOPAX_DNA_MOLECULE: &str = "http://www.biopax.org/release/biopax-level3.owl#Dna";
const BIOPAX_RNA_MOLECULE: &str = "http://www.biopax.org/release/biopax-level3.owl#Rna";
const BIOPAX_PROTEIN: &str = "http://www.biopax.org/release/biopax-level3.owl#Protein";
const BIOPAX_COMPLEX: &str = "http://www.biopax.org/release/biopax-level3.owl#Complex";
const BIOPAX_SMALL_MOLECULE: &str = "http://www.biopax.org/release/biopax-level3.owl#SmallMolecule";

/// The Table 2 (BioPAX) ComponentDefinition types. SBOL 2 draws these from
/// BioPAX rather than an ontology the bundled snapshot indexes, so they are
/// matched by exact IRI.
const BIOPAX_TYPES: &[&str] = &[
    BIOPAX_DNA_REGION,
    BIOPAX_RNA_REGION,
    BIOPAX_DNA_MOLECULE,
    BIOPAX_RNA_MOLECULE,
    BIOPAX_PROTEIN,
    BIOPAX_COMPLEX,
    BIOPAX_SMALL_MOLECULE,
];

/// The compact ontology identifier (`SO:0000110`, `SBO:0000231`, …) an SBOL 2
/// term IRI carries. SBOL 2 writes ontology terms as `identifiers.org` URLs
/// with a lowercase path segment (`.../so/SO:0000110`); the bundled ontology
/// keys on the trailing CURIE, so the last path or fragment segment is taken.
fn ontology_curie(iri: &str) -> &str {
    iri.rsplit(['/', '#']).next().unwrap_or(iri)
}
/// The Table 1 sequence-encoding URIs (IUPAC DNA/RNA share one IRI).
const TABLE_1_ENCODINGS: &[&str] = &[
    IUPAC_NUCLEIC_ENCODING,
    IUPAC_PROTEIN_ENCODING,
    SMILES_ENCODING,
];

/// The Table 7 SequenceConstraint restriction URIs.
const TABLE_7_RESTRICTIONS: &[&str] = &[
    SBOL2_PRECEDES,
    SBOL2_SAME_ORIENTATION_AS,
    SBOL2_OPPOSITE_ORIENTATION_AS,
    SBOL2_DIFFERENT_FROM,
];

/// The SBO modeling-framework branch root (SBO:0000004).
const SBO_MODELING_FRAMEWORK: &str = "SBO:0000004";

/// Whether `iri` names a term from the EDAM ontology. SBOL 2 documents write
/// EDAM terms either as native `edamontology.org` URLs or as `identifiers.org`
/// CURIE URLs (`.../edam:format_1915`).
fn is_edam_iri(iri: &str) -> bool {
    iri.starts_with("http://edamontology.org/")
        || iri.starts_with("https://edamontology.org/")
        || iri.contains("edam:")
        || iri.contains("/edam/")
}

const SO_SEQUENCE_FEATURE: &str = "SO:0000110";
const SO_TOPOLOGY_ATTRIBUTE: &str = "SO:0000986";
const SO_STRAND_ATTRIBUTE: &str = "SO:0000983";
const SBO_OCCURRING_ENTITY_REPRESENTATION: &str = "SBO:0000231";
const SBO_PARTICIPANT_ROLE: &str = "SBO:0000003";
const SBO_SYSTEMS_DESCRIPTION_PARAMETER: &str = "SBO:0000545";

const SBOL2_DIFFERENT_FROM: &str = "http://sbols.org/v2#differentFrom";
const SBOL2_PRECEDES: &str = "http://sbols.org/v2#precedes";
const SBOL2_SAME_ORIENTATION_AS: &str = "http://sbols.org/v2#sameOrientationAs";
const SBOL2_OPPOSITE_ORIENTATION_AS: &str = "http://sbols.org/v2#oppositeOrientationAs";

/// The IUPAC nucleic-acid encoding IRI (shared by DNA and RNA), the IUPAC
/// protein encoding IRI, and the alphabets their elements must draw from.
const IUPAC_NUCLEIC_ENCODING: &str = "http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html";
const IUPAC_PROTEIN_ENCODING: &str = "http://www.chem.qmul.ac.uk/iupac/AminoAcid/";
const SMILES_ENCODING: &str = "http://www.opensmiles.org/opensmiles.html";
const IUPAC_NUCLEIC_ALPHABET: &str = "ACGTURYSWKMBDHVN-.";

/// A positionally comparable Location: a Range spanning `[start, end]` or a Cut
/// between bases at `at`.
#[derive(Clone, Copy)]
enum Region {
    Range(i64, i64),
    Cut(i64),
}

/// Whether any two distinct regions overlap, mirroring libSBOLj's pairwise
/// overlap test across Range/Cut combinations.
fn regions_overlap(regions: &[Region]) -> bool {
    for (i, a) in regions.iter().enumerate() {
        for b in &regions[i + 1..] {
            let overlap = match (a, b) {
                (Region::Range(s1, e1), Region::Range(s2, e2)) => {
                    (s1 >= s2 && s1 <= e2) || (s2 >= s1 && s2 <= e1)
                }
                (Region::Range(s, e), Region::Cut(at)) | (Region::Cut(at), Region::Range(s, e)) => {
                    e > at && at >= s
                }
                (Region::Cut(a), Region::Cut(b)) => a == b,
            };
            if overlap {
                return true;
            }
        }
    }
    false
}

/// The summed inclusive length of the Range regions, or `None` when there are
/// none. Cut regions contribute no length.
fn range_span(regions: &[Region]) -> Option<i64> {
    let mut total = 0;
    let mut any = false;
    for region in regions {
        if let Region::Range(start, end) = region {
            total += end - start + 1;
            any = true;
        }
    }
    any.then_some(total)
}

/// The permitted participant-role SBO terms for an interaction-type SBO term,
/// keyed by trailing CURIE, per SBOL 2 Table 11. `None` for types the table
/// does not constrain.
fn participant_roles_for_interaction(interaction_type: &str) -> Option<&'static [&'static str]> {
    // SBO CURIEs: inhibition 0000169, stimulation 0000170, non-covalent binding
    // 0000177, degradation 0000179, biochemical reaction 0000176, genetic
    // production 0000589, control 0000168. Roles: inhibitor 0000020, inhibited
    // 0000642, promoter 0000598, stimulator 0000459, stimulated 0000643,
    // reactant 0000010, product 0000011, modifier 0000019, modified 0000644,
    // template 0000645.
    match interaction_type {
        "SBO:0000169" => Some(&["SBO:0000020", "SBO:0000642", "SBO:0000598"]),
        "SBO:0000170" => Some(&["SBO:0000459", "SBO:0000643", "SBO:0000598"]),
        "SBO:0000177" => Some(&["SBO:0000010", "SBO:0000011"]),
        "SBO:0000179" => Some(&["SBO:0000010"]),
        "SBO:0000176" => Some(&["SBO:0000010", "SBO:0000011", "SBO:0000019"]),
        "SBO:0000589" => Some(&["SBO:0000598", "SBO:0000645", "SBO:0000011"]),
        "SBO:0000168" => Some(&["SBO:0000019", "SBO:0000644"]),
        _ => None,
    }
}

/// The set of IRI values `object` carries under `predicate`.
fn set_of(object: &Object, predicate: &str) -> std::collections::BTreeSet<String> {
    object
        .iris(predicate)
        .map(|iri| iri.as_str().to_owned())
        .collect()
}

/// The diagnostic message for a CombinatorialDerivation best-practice finding.
fn combinatorial_message(rule: &str) -> &'static str {
    match rule {
        "sbol2-12909" => "a CombinatorialDerivation template should contain Components",
        "sbol2-13006" => "a VariableComponent should specify at least one variant source",
        "sbol2-12910" => "a derived ComponentDefinition should keep its template's types",
        "sbol2-12911" => "a derived ComponentDefinition should keep its template's roles",
        "sbol2-13018" => "a derived Component should keep its template Component's roles",
        "sbol2-13019" => "a zeroOrOne VariableComponent should be realized at most once",
        "sbol2-13020" => "a one VariableComponent should be realized exactly once",
        "sbol2-13021" => "a oneOrMore VariableComponent should be realized at least once",
        "sbol2-13022" => "a non-replaced template Component should be realized exactly once",
        "sbol2-12912" => {
            "a Collection should record the CombinatorialDerivation its members derive from"
        }
        "sbol2-12913" => {
            "a Collection's members should derive from the same CombinatorialDerivation"
        }
        _ => "combinatorial derivation best practice",
    }
}

/// Whether `node` reaches a cycle in `graph`: a directed walk from `node` that
/// revisits a node already on the current path. `path` is the recursion stack.
fn reaches_cycle(
    node: &str,
    graph: &BTreeMap<String, Vec<String>>,
    path: &mut std::collections::BTreeSet<String>,
) -> bool {
    if !path.insert(node.to_owned()) {
        return true;
    }
    if let Some(successors) = graph.get(node) {
        for successor in successors {
            if reaches_cycle(successor, graph, path) {
                return true;
            }
        }
    }
    path.remove(node);
    false
}
const SBOL2_MERGE_ROLES: &str = "http://sbols.org/v2#mergeRoles";
const SBOL2_OVERRIDE_ROLES: &str = "http://sbols.org/v2#overrideRoles";
const SBOL2_VERIFY_IDENTICAL: &str = "http://sbols.org/v2#verifyIdentical";
const SBOL2_USE_LOCAL: &str = "http://sbols.org/v2#useLocal";
const SBOL2_USE_REMOTE: &str = "http://sbols.org/v2#useRemote";
const SBOL2_MERGE: &str = "http://sbols.org/v2#merge";
const SBOL2_ENUMERATE: &str = "http://sbols.org/v2#enumerate";
const SBOL2_SAMPLE: &str = "http://sbols.org/v2#sample";
const SBOL2_OP_ZERO_OR_ONE: &str = "http://sbols.org/v2#zeroOrOne";
const SBOL2_OP_ONE: &str = "http://sbols.org/v2#one";
const SBOL2_OP_ZERO_OR_MORE: &str = "http://sbols.org/v2#zeroOrMore";
const SBOL2_OP_ONE_OR_MORE: &str = "http://sbols.org/v2#oneOrMore";

const SBOL2_ROLE_DESIGN: &str = "http://sbols.org/v2#design";
const SBOL2_ROLE_BUILD: &str = "http://sbols.org/v2#build";
const SBOL2_ROLE_TEST: &str = "http://sbols.org/v2#test";
const SBOL2_ROLE_LEARN: &str = "http://sbols.org/v2#learn";

/// The distinct terms in `values`, preserving order. Identical triples that
/// RDF/XML serialized more than once collapse to one value.
fn distinct_terms(values: &[crate::Term]) -> Vec<&crate::Term> {
    let mut out: Vec<&crate::Term> = Vec::with_capacity(values.len());
    for value in values {
        if !out.contains(&value) {
            out.push(value);
        }
    }
    out
}

/// SBOL 2 version lexical form: alphanumeric, underscore, hyphen, or period,
/// beginning with a digit.
fn is_valid_version(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_digit()
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

/// Whether `earlier` strictly precedes `later` under semantic-versioning order:
/// dot-separated components compared numerically where both are numeric, and
/// lexically otherwise. Equal versions do not precede.
fn version_precedes(earlier: &str, later: &str) -> bool {
    let mut left = earlier.split('.');
    let mut right = later.split('.');
    loop {
        match (left.next(), right.next()) {
            (Some(a), Some(b)) => {
                let ordering = match (a.parse::<u64>(), b.parse::<u64>()) {
                    (Ok(a), Ok(b)) => a.cmp(&b),
                    _ => a.cmp(b),
                };
                match ordering {
                    std::cmp::Ordering::Equal => continue,
                    std::cmp::Ordering::Less => return true,
                    std::cmp::Ordering::Greater => return false,
                }
            }
            (None, Some(_)) => return true,
            (Some(_), None) => return false,
            (None, None) => return false,
        }
    }
}

/// Whether `uri` ends with a delimiter (`/`, `#`, or `:`) immediately followed
/// by `tail`.
fn ends_with_delimited(uri: &str, tail: &str) -> bool {
    if let Some(prefix) = uri.strip_suffix(tail) {
        prefix.ends_with(['/', '#', ':'])
    } else {
        false
    }
}

/// Whether `full` equals `base` + delimiter + `suffix`.
fn is_delimited_suffix(full: &str, base: &str, suffix: &str) -> bool {
    if let Some(rest) = full.strip_prefix(base)
        && let Some(after) = rest.strip_suffix(suffix)
    {
        return after == "/" || after == "#" || after == ":";
    }
    false
}

/// The Complete-gated rule that governs presence of a referenced TopLevel for
/// a given reference property, keyed by predicate and target class. Returns
/// `None` when no completeness rule is mapped for the reference.
fn completeness_rule(predicate: &str, target: crate::schema::TargetClass) -> Option<&'static str> {
    use crate::schema::TargetClass;
    let cd = Sbol2Class::ComponentDefinition.iri();
    let md = Sbol2Class::ModuleDefinition.iri();
    match (predicate, target) {
        // ComponentInstance/Module definition -> ComponentDefinition/ModuleDefinition.
        (SBOL2_DEFINITION, TargetClass::Sbol(c)) if c == cd => Some("sbol2-10604"),
        (SBOL2_DEFINITION, TargetClass::Sbol(c)) if c == md => Some("sbol2-11703"),
        (SBOL2_SEQUENCE, _) => Some("sbol2-10513"),
        (SBOL2_MODEL, _) => Some("sbol2-11608"),
        (SBOL2_MEMBER, _) => Some("sbol2-12103"),
        (SBOL2_TEMPLATE, _) => Some("sbol2-12905"),
        (SBOL2_ATTACHMENT, _) => Some("sbol2-10307"),
        // CombinatorialDerivation variant references.
        (SBOL2_VARIANT, _) => Some("sbol2-13008"),
        (SBOL2_VARIANT_COLLECTION, _) => Some("sbol2-13010"),
        (SBOL2_VARIANT_DERIVATION, _) => Some("sbol2-13012"),
        // Provenance references.
        (PROV_WAS_GENERATED_BY, _) => Some("sbol2-10222"),
        (PROV_WAS_INFORMED_BY, _) => Some("sbol2-12407"),
        (PROV_AGENT, _) => Some("sbol2-12604"),
        _ => None,
    }
}
