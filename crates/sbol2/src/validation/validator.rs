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
use crate::{Document, Object, Resource, Sbol2Class};

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
        }

        // Always: whole-document checks.
        self.validate_uri_uniqueness();
        self.validate_persistent_identity_uniqueness();

        // Gated families, dispatched exactly as SBOLValidate does.
        if config.compliant {
            for object in self.document.objects().values() {
                if is_sbol_object(object) {
                    self.validate_compliance(object);
                }
            }
        }
        if config.complete {
            let resolvable = self.resolvable_uris();
            for object in self.document.objects().values() {
                if is_sbol_object(object) {
                    self.validate_completeness(object, &resolvable);
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
            && !syntax::is_valid_display_id(display_id) {
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
            && !is_valid_version(&version) {
                self.error(
                    "sbol2-10206",
                    object,
                    Some(SBOL2_VERSION),
                    format!("version `{version}` is not a valid SBOL 2 version string"),
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
        self.check_enum(object, SBOL2_ACCESS, "sbol2-10607", &[SBOL2_PUBLIC, SBOL2_PRIVATE]);
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
                    format!("value `{}` of `{predicate}` is not an allowed term", iri.as_str()),
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
                object.first_resource(SBOL2_DEFINITION).and_then(Resource::as_iri),
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

    /// Cross-object containment: the Component/FunctionalComponent references
    /// carried by SequenceAnnotation, SequenceConstraint, Participation, and
    /// MapsTo must point at instances contained by the appropriate parent
    /// definition.
    fn validate_containment(&mut self, object: &Object) {
        if object.has_class(Sbol2Class::SequenceConstraint) {
            self.validate_sequence_constraint(object);
        }
        if object.has_class(Sbol2Class::SequenceAnnotation) {
            // 10905: the referenced Component must belong to the containing CD.
            let missing = match object.first_resource(SBOL2_COMPONENT).and_then(Resource::as_iri) {
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
            let missing = match object.first_resource(SBOL2_PARTICIPANT).and_then(Resource::as_iri) {
                Some(participant) => match self.container_by(object, SBOL2_PARTICIPATION) {
                    Some(interaction) => match self.container_by(interaction, SBOL2_INTERACTION) {
                        Some(md) => !self.lists(md, SBOL2_FUNCTIONAL_COMPONENT, participant.as_str()),
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
        let subject = object.first_resource(SBOL2_SUBJECT).and_then(Resource::as_iri);
        let obj = object.first_resource(SBOL2_OBJECT).and_then(Resource::as_iri);
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
        let subject_missing =
            subject.map(|s| !self.lists(cd, SBOL2_COMPONENT, s.as_str())).unwrap_or(false);
        let object_missing =
            obj.map(|o| !self.lists(cd, SBOL2_COMPONENT, o.as_str())).unwrap_or(false);
        // 11413: under differentFrom, subject and object Components must not
        // resolve to the same ComponentDefinition.
        let is_different_from = object
            .first_resource(SBOL2_RESTRICTION)
            .and_then(Resource::as_iri)
            .is_some_and(|iri| iri.as_str() == SBOL2_DIFFERENT_FROM);
        let same_definition = is_different_from
            && match (subject, obj) {
                (Some(s), Some(o)) => {
                    match (self.definition_of(s.as_str()), self.definition_of(o.as_str())) {
                        (Some(sd), Some(od)) => sd == od,
                        _ => false,
                    }
                }
                _ => false,
            };
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
    }

    fn validate_maps_to_local(&mut self, object: &Object) {
        let Some(local) = object.first_resource(SBOL2_LOCAL).and_then(Resource::as_iri) else {
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
        let object = self
            .document
            .objects()
            .values()
            .find(|o| o.identity().as_iri().is_some_and(|iri| iri.as_str() == instance))?;
        object
            .first_resource(SBOL2_DEFINITION)
            .and_then(Resource::as_iri)
            .map(|iri| iri.as_str().to_owned())
    }

    fn integer(&self, object: &Object, predicate: &str) -> Option<i64> {
        object
            .values(predicate)
            .iter()
            .find_map(|term| term.as_literal().and_then(|l| l.value().parse::<i64>().ok()))
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
                && dupes.iter().any(|d| d == iri.as_str()) {
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
            // 10216: TopLevel persistentIdentity ends with delimiter + displayId.
            if let Some(pid) = &persistent
                && !ends_with_delimited(pid, &display_id) {
                    self.error(
                        "sbol2-10216",
                        object,
                        Some(SBOL2_PERSISTENT_IDENTITY),
                        format!(
                            "compliant TopLevel persistentIdentity `{pid}` must end with a \
                             delimiter followed by displayId `{display_id}`"
                        ),
                    );
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

    // --- best-practice family (gate: best_practice) --------------------

    fn validate_best_practices(&mut self, object: &Object) {
        // 10224-10227: an object generated by an Activity whose Association
        // carries a design/build/test/learn role SHOULD have the matching
        // TopLevel type.
        self.validate_activity_role_usage(object);
        self.validate_ontology_usage(object);
    }

    /// Ontology-recommendation checks: whether the SO/SBO terms a
    /// ComponentDefinition, Component, Interaction, Participation, or Measure
    /// carries fall in the branch the specification recommends for that field.
    fn validate_ontology_usage(&mut self, object: &Object) {
        let ontology = sbol_ontology::Ontology::bundled();
        if object.has_class(Sbol2Class::ComponentDefinition) {
            let types: Vec<&str> = object.iris(SBOL2_TYPE).map(|iri| iri.as_str()).collect();
            let is_dna_or_rna_region =
                types.iter().any(|t| *t == BIOPAX_DNA_REGION || *t == BIOPAX_RNA_REGION);
            let is_dna_or_rna_molecule =
                types.iter().any(|t| *t == BIOPAX_DNA_MOLECULE || *t == BIOPAX_RNA_MOLECULE);

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
                .filter(|iri| ontology.is_in_branch(ontology_curie(iri.as_str()), SO_SEQUENCE_FEATURE))
                .count();
            let num_topo = types.iter().filter(|t| ontology.is_in_branch(ontology_curie(t), SO_TOPOLOGY_ATTRIBUTE)).count();
            let num_strand =
                types.iter().filter(|t| ontology.is_in_branch(ontology_curie(t), SO_STRAND_ATTRIBUTE)).count();

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
                    .filter(|iri| ontology.is_in_branch(ontology_curie(iri.as_str()), SO_SEQUENCE_FEATURE))
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
                .filter(|iri| ontology.is_in_branch(ontology_curie(iri.as_str()), SBO_OCCURRING_ENTITY_REPRESENTATION))
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
                .filter(|iri| ontology.is_in_branch(ontology_curie(iri.as_str()), SBO_PARTICIPANT_ROLE))
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

        if object.has_class(Sbol2Class::OmMeasure) {
            // 13505: a Measure with types should carry exactly one
            // systems-description-parameter SBO type.
            let types: Vec<&str> = object.iris(SBOL2_TYPE).map(|iri| iri.as_str()).collect();
            if !types.is_empty() {
                let num =
                    types.iter().filter(|t| ontology.is_in_branch(ontology_curie(t), SBO_SYSTEMS_DESCRIPTION_PARAMETER)).count();
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
        let def = component.first_resource(SBOL2_DEFINITION).and_then(Resource::as_iri)?;
        let cd = self
            .document
            .objects()
            .values()
            .find(|o| o.identity().as_iri().is_some_and(|iri| iri.as_str() == def.as_str()))?;
        Some(cd.iris(SBOL2_TYPE).map(|iri| iri.as_str().to_owned()).collect())
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
                ValidationIssue::warning(rule, object.identity().clone(), property, message)
            }
            _ => ValidationIssue::error(rule, object.identity().clone(), property, message),
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
const SO_SEQUENCE_FEATURE: &str = "SO:0000110";
const SO_TOPOLOGY_ATTRIBUTE: &str = "SO:0000986";
const SO_STRAND_ATTRIBUTE: &str = "SO:0000983";
const SBO_OCCURRING_ENTITY_REPRESENTATION: &str = "SBO:0000231";
const SBO_PARTICIPANT_ROLE: &str = "SBO:0000003";
const SBO_SYSTEMS_DESCRIPTION_PARAMETER: &str = "SBO:0000545";

const SBOL2_DIFFERENT_FROM: &str = "http://sbols.org/v2#differentFrom";
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
        && let Some(after) = rest.strip_suffix(suffix) {
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
