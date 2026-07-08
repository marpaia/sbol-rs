//! SBOL 3 → SBOL 2 predicate downgrading, backport metadata emission, and
//! the IRI/type rewriting primitives shared across the conversion pass.

use super::helpers::*;
use super::*;

impl<'a> Engine<'a> {
    pub(super) fn handle_sbol3_predicate(&mut self, triple: &Triple) {
        let predicate = triple.predicate.as_str();

        // Drop predicates that have no SBOL 2 equivalent. `hasNamespace`
        // is the most important one: the namespace is implicit in
        // the restored persistentIdentity / versioned IRI in SBOL 2.
        if predicate == v3::SBOL_HAS_NAMESPACE {
            return;
        }

        // `hasFeature` is context-dependent in SBOL 2: it becomes
        // `component`, `functionalComponent`, `module`, or
        // `sequenceAnnotation` depending on what the feature is and
        // what its parent is. Resolve here using the type maps built
        // in preflight.
        if predicate == v3::SBOL_HAS_FEATURE {
            self.handle_has_feature(triple);
            return;
        }

        // `hasLocation` on a SubComponent that was a collapsed SA's
        // component: drop here; the reconstructed SA emits
        // `sbol2:location` itself in `emit_sa_wrappers`.
        if predicate == v3::SBOL_HAS_LOCATION
            && let Some(subject_iri) = triple.subject.as_iri()
            && self.sa_collapses.contains_key(subject_iri.as_str())
        {
            return;
        }

        // A Component classified as a ModuleDefinition carries neither a
        // `type` nor a `hasSequence` in SBOL 2 (both are ComponentDefinition
        // properties), so those triples are dropped.
        if matches!(predicate, v3::SBOL_TYPE | v3::SBOL_HAS_SEQUENCE)
            && let Some(subject_iri) = triple.subject.as_iri()
            && self
                .component_splits
                .get(subject_iri.as_str())
                .is_some_and(|split| split.shape == ComponentShape::MdOnly)
        {
            return;
        }

        // A Location's `sbol3:hasSequence` maps back to `sbol2:sequence` on
        // the Range / Cut, except when the upgrade recorded that the SBOL 2
        // location had no sequence of its own (`sbol2LocationSequenceNull`):
        // that sequence was inferred from the parent or synthesized, so it is
        // dropped to preserve the original SBOL 2 shape.
        if predicate == v3::SBOL_HAS_SEQUENCE
            && let Some(subject_iri) = triple.subject.as_iri()
        {
            let resolved = self
                .resolved_types
                .get(subject_iri.as_str())
                .map(String::as_str);
            let is_location = matches!(
                resolved,
                Some(v2::SBOL2_RANGE) | Some(v2::SBOL2_CUT) | Some(v2::SBOL2_GENERIC_LOCATION)
            );
            if is_location && self.null_sequence_locations.contains(subject_iri.as_str()) {
                return;
            }
        }

        // SBOL 3 carries orientation on a Feature; SBOL 2 carries it only on a
        // Location. A SubComponent's own orientation therefore has no SBOL 2
        // home and is dropped (the Location keeps its own orientation).
        if predicate == v3::SBOL_ORIENTATION
            && let Some(subject_iri) = triple.subject.as_iri()
        {
            let resolved = self
                .resolved_types
                .get(subject_iri.as_str())
                .map(String::as_str);
            let is_location = matches!(
                resolved,
                Some(v2::SBOL2_RANGE) | Some(v2::SBOL2_CUT) | Some(v2::SBOL2_GENERIC_LOCATION)
            );
            if !is_location {
                return;
            }
        }
        // Skip `sbol3:name` / `sbol3:description` when the same
        // subject already carries the matching `dcterms:` form. The
        // upgrade preserves dcterms metadata alongside the
        // synthesized sbol3:name; emitting both back here would
        // duplicate the triple.
        if predicate == v3::SBOL_NAME || predicate == v3::SBOL_DESCRIPTION {
            let dcterms_predicate = if predicate == v3::SBOL_NAME {
                v2::DCTERMS_TITLE
            } else {
                v2::DCTERMS_DESCRIPTION
            };
            if self.subject_already_has(
                triple.subject.as_iri().map(|i| i.as_str()),
                dcterms_predicate,
                &triple.object,
            ) {
                return;
            }
        }

        let subject_iri = triple.subject.as_iri().map(|i| i.as_str().to_owned());
        let subject = self.rewrite_resource(&triple.subject);
        let object = self.rewrite_term(&triple.object);

        if let Some(renamed) =
            self.map_sbol3_predicate_to_sbol2_for_subject(subject_iri.as_deref(), predicate)
        {
            // Value-level reverse mappings apply to a few predicates
            // (orientation, encoding, type, restriction).
            let rewritten_object =
                self.reverse_value_for_subject(subject_iri.as_deref(), predicate, &object);
            self.output_triples.push(Triple {
                subject,
                predicate: Iri::from_static(renamed),
                object: rewritten_object,
            });
            return;
        }

        self.archive_unknown_sbol3_predicate(triple);
    }

    pub(super) fn map_sbol3_predicate_to_sbol2_for_subject(
        &self,
        subject_iri: Option<&str>,
        predicate: &str,
    ) -> Option<&'static str> {
        if predicate == v3::SBOL_MEMBER
            && subject_iri
                .and_then(|iri| self.resolved_type_sets.get(iri))
                .is_some_and(|types| types.contains(v2::SBOL2_EXPERIMENT))
        {
            return Some(v2::SBOL2_EXPERIMENTAL_DATA_PROP);
        }
        map_sbol3_predicate_to_sbol2(predicate)
    }

    /// SBOL 3 predicates with no SBOL 2 equivalent are dropped. The reference
    /// converter does not carry SBOL 3-only data across to SBOL 2.
    pub(super) fn archive_unknown_sbol3_predicate(&mut self, _triple: &Triple) {}

    /// Returns true if the input document already carries a
    /// `(subject, predicate, object)` triple for one of the indexed
    /// dcterms predicates. Used to avoid duplicating `dcterms:title` /
    /// `dcterms:description` when the upgrade preserved both Dublin
    /// Core and SBOL 3 forms. O(1) via [`Engine::dcterms_index`].
    pub(super) fn subject_already_has(
        &self,
        subject_iri: Option<&str>,
        predicate: &'static str,
        object: &Term,
    ) -> bool {
        let Some(subject_iri) = subject_iri else {
            return false;
        };
        self.dcterms_index
            .get(&(subject_iri.to_owned(), predicate))
            .map(|objects| objects.contains(object))
            .unwrap_or(false)
    }

    /// Routes `sbol3:hasFeature` to the right SBOL 2 predicate based
    /// on the resolved types of the parent (CD vs MD) and the feature
    /// itself (SubComponent, SequenceFeature, ComponentReference).
    ///
    /// Decision table:
    /// - parent = CD, feature = SubComponent → `sbol2:component`
    /// - parent = CD, feature = SequenceFeature → `sbol2:sequenceAnnotation`
    /// - parent = MD, feature = SubComponent → `sbol2:functionalComponent`,
    ///   or `sbol2:module` when the SubComponent's `instanceOf` is an MD
    /// - parent unknown / other shape → `sbol2:component` (safe default)
    pub(super) fn handle_has_feature(&mut self, triple: &Triple) {
        let parent_iri = triple
            .subject
            .as_iri()
            .map(|i| i.as_str().to_owned())
            .unwrap_or_default();
        let feature_iri = triple
            .object
            .as_iri()
            .map(|i| i.as_str().to_owned())
            .unwrap_or_default();

        let parent_type = self.resolved_types.get(&parent_iri).cloned();
        let feature_type = self.resolved_types.get(&feature_iri).cloned();

        let predicate: &'static str = match (parent_type.as_deref(), feature_type.as_deref()) {
            (Some(v2::SBOL2_MODULE_DEFINITION), Some(v2::SBOL2_COMPONENT))
            | (Some(v2::SBOL2_MODULE_DEFINITION), Some(v2::SBOL2_FUNCTIONAL_COMPONENT))
            | (Some(v2::SBOL2_MODULE_DEFINITION), Some(v2::SBOL2_MODULE)) => {
                // SubComponent inside an MD-derived Component. The
                // module-vs-FunctionalComponent distinction depends on
                // what the SubComponent points at via instanceOf.
                let target = self.subcomponent_targets.get(&feature_iri).cloned();
                let target_type = target
                    .as_deref()
                    .and_then(|t| self.resolved_types.get(t).cloned());
                match target_type.as_deref() {
                    Some(v2::SBOL2_MODULE_DEFINITION) => v2::SBOL2_MODULE_PROP,
                    _ => v2::SBOL2_FUNCTIONAL_COMPONENT_PROP,
                }
            }
            (_, Some(v2::SBOL2_SEQUENCE_ANNOTATION)) => v2::SBOL2_SEQUENCE_ANNOTATION_PROP,
            _ => v2::SBOL2_COMPONENT_PROP,
        };

        self.output_triples.push(Triple {
            subject: self.rewrite_resource(&triple.subject),
            predicate: Iri::from_static(predicate),
            object: self.rewrite_term(&triple.object),
        });
    }

    /// Reverses value-level mappings (orientation, encoding, type,
    /// restriction, role, framework, language). `subject` supplies the
    /// subject's resolved SBOL 2 type, which disambiguates the ontology for
    /// the `type`/`role` predicates shared across entity kinds.
    pub(super) fn reverse_value_for_subject(
        &self,
        subject: Option<&str>,
        predicate_str: &str,
        object: &Term,
    ) -> Term {
        let iri = match object.as_iri() {
            Some(iri) => iri.as_str(),
            None => return object.clone(),
        };
        // `type`/`role` are shared across entity kinds, so the subject's SBOL 3
        // type decides the ontology: an Interaction's `type` and a
        // Participation's `role` are SBO terms; a Component's `role` is an SO
        // term and its `type` a BioPAX/SO term.
        // `resolved_type_sets` holds each subject's resolved SBOL 2 class.
        let is_interaction = subject
            .and_then(|s| self.resolved_type_sets.get(s))
            .is_some_and(|set| set.contains(v2::SBOL2_INTERACTION));
        let is_participation = subject
            .and_then(|s| self.resolved_type_sets.get(s))
            .is_some_and(|set| set.contains(v2::SBOL2_PARTICIPATION));
        let mapped: Option<String> = match predicate_str {
            v3::SBOL_ORIENTATION => values::map_orientation(iri).map(String::from),
            v3::SBOL_ENCODING => values::map_encoding(iri).map(String::from),
            v3::SBOL_TYPE if is_interaction => Some(crate::uri::convert_sbo_3_to_2(iri)),
            // A Component's `sbol3:type` is a BioPAX-derived SBO term
            // (mapped to its canonical `*Region` SBOL 2 form) or, when it
            // isn't one, an SO / foreign term canonicalized to its SBOL 2
            // spelling. The SBOL 2 → SBOL 3 map sends both `biopax:Dna` and
            // `biopax:DnaRegion` to the same SBO term, so the reverse always
            // yields the `*Region` form.
            v3::SBOL_TYPE => Some(
                values::map_biopax_type(iri)
                    .map(str::to_owned)
                    .unwrap_or_else(|| crate::uri::component_type_3_to_2(iri)),
            ),
            v3::SBOL_ROLE if is_participation => Some(crate::uri::convert_sbo_3_to_2(iri)),
            v3::SBOL_ROLE => Some(crate::uri::convert_so_3_to_2(iri)),
            v3::SBOL_FRAMEWORK => Some(crate::uri::convert_sbo_3_to_2(iri)),
            v3::SBOL_LANGUAGE => Some(crate::uri::convert_edam_3_to_2(iri)),
            v3::SBOL_RESTRICTION => values::map_restriction(iri),
            v3::SBOL_CARDINALITY => values::map_cardinality(iri).map(String::from),
            v3::SBOL_STRATEGY => values::map_strategy(iri).map(String::from),
            v3::SBOL_ROLE_INTEGRATION => values::map_role_integration(iri).map(String::from),
            _ => None,
        };
        if let Some(sbol2_iri) = mapped {
            return Term::Resource(Resource::Iri(Iri::new_unchecked(sbol2_iri)));
        }
        object.clone()
    }

    pub(super) fn emit_backport_metadata(&mut self) {
        // SBOL 2 requires `persistentIdentity` and `version` on every
        // owned object, not just top-levels. Iterate every subject
        // that received a version suffix during identity restoration
        // (top-levels and their children) and emit both triples.
        //
        // The version for a child is the parent's version, propagated
        // through `iri_rewrites` (see `build_iri_rewrites`).
        let mut entries: Vec<(String, String)> = self
            .iri_rewrites
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        entries.sort();
        for (sbol3_iri, sbol2_iri) in entries {
            // Skip subjects folded into a structural re-synthesis. The
            // synthesizer emits persistentIdentity / version itself
            // (typically at a different IRI). Without this skip the
            // metadata would land on a leftover IRI as an orphan
            // subject in the SBOL 2 output and round-trip back as
            // unwanted backport triples on a CRef/Constraint/Interface.
            if self.mapsto_reconstructions.contains_key(&sbol3_iri)
                || self.mapsto_constraints.contains(&sbol3_iri)
                || self.interface_subjects.contains(&sbol3_iri)
                || self.discarded_subjects.contains(&sbol3_iri)
            {
                continue;
            }
            let subject = Resource::Iri(Iri::new_unchecked(sbol2_iri.clone()));

            // persistentIdentity: the recomputed value for the subject, or
            // the unversioned SBOL 3 IRI itself when none was derived.
            let persistent = self
                .persistent_identities
                .get(&sbol3_iri)
                .cloned()
                .unwrap_or_else(|| sbol3_iri.clone());
            self.output_triples.push(Triple {
                subject: subject.clone(),
                predicate: Iri::from_static(v2::SBOL2_PERSISTENT_IDENTITY),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(persistent))),
            });

            // Version: emit when the subject's owning top-level carries a
            // version (from its IRI) or synthesis is enabled via
            // `default_version`; otherwise skip (SBOL 2 makes it optional).
            if let Some(version) = self.effective_version_for_iri(&sbol3_iri) {
                self.output_triples.push(Triple {
                    subject,
                    predicate: Iri::from_static(v2::SBOL2_VERSION),
                    object: Term::Literal(sbol_rdf::Literal::simple(version)),
                });
            }
        }
    }

    pub(super) fn rewrite_iri<'b>(&'b self, iri: &'b str) -> &'b str {
        self.iri_rewrites
            .get(iri)
            .map(String::as_str)
            .unwrap_or(iri)
    }

    pub(super) fn rewrite_resource(&self, resource: &Resource) -> Resource {
        match resource {
            Resource::Iri(iri) => {
                let new = self.rewrite_iri(iri.as_str());
                if new == iri.as_str() {
                    resource.clone()
                } else {
                    Resource::Iri(Iri::new_unchecked(new))
                }
            }
            _ => resource.clone(),
        }
    }

    pub(super) fn rewrite_term(&self, term: &Term) -> Term {
        match term {
            Term::Resource(resource) => Term::Resource(self.rewrite_resource(resource)),
            _ => term.clone(),
        }
    }

    pub(super) fn rewrite_triple(&self, triple: &Triple) -> Triple {
        Triple {
            subject: self.rewrite_resource(&triple.subject),
            predicate: triple.predicate.clone(),
            object: self.rewrite_term(&triple.object),
        }
    }

    pub(super) fn sbol2_type_for_subject_type(
        &self,
        subject_iri: &str,
        object_iri: &str,
    ) -> Option<String> {
        if object_iri == v3::SBOL_SUB_COMPONENT_CLASS {
            return Some(self.default_subcomponent_type(subject_iri).to_owned());
        }
        if object_iri == v3::SBOL_COMPONENT_CLASS {
            return Some(
                match self
                    .component_splits
                    .get(subject_iri)
                    .map(|split| split.shape)
                {
                    Some(ComponentShape::MdOnly) => v2::SBOL2_MODULE_DEFINITION,
                    _ => v2::SBOL2_COMPONENT_DEFINITION,
                }
                .to_owned(),
            );
        }
        map_sbol3_type_to_sbol2(object_iri).map(str::to_owned)
    }

    pub(super) fn default_subcomponent_type(&self, subject_iri: &str) -> &'static str {
        let parent_type = self
            .feature_parent
            .get(subject_iri)
            .and_then(|parent| self.component_sbol2_type(parent));
        match parent_type {
            Some(v2::SBOL2_MODULE_DEFINITION) => {
                // A SubComponent whose target is a ModuleDefinition, or that
                // the upgrade marked as Module-derived, restores as a Module;
                // otherwise a FunctionalComponent.
                if self.module_origin_subcomponents.contains(subject_iri) {
                    return v2::SBOL2_MODULE;
                }
                let target_type = self
                    .subcomponent_targets
                    .get(subject_iri)
                    .and_then(|target| self.component_sbol2_type(target));
                match target_type {
                    Some(v2::SBOL2_MODULE_DEFINITION) => v2::SBOL2_MODULE,
                    _ => v2::SBOL2_FUNCTIONAL_COMPONENT,
                }
            }
            _ => v2::SBOL2_COMPONENT,
        }
    }

    pub(super) fn component_sbol2_type(&self, component_iri: &str) -> Option<&'static str> {
        self.component_splits
            .get(component_iri)
            .map(|split| match split.shape {
                ComponentShape::MdOnly => v2::SBOL2_MODULE_DEFINITION,
                _ => v2::SBOL2_COMPONENT_DEFINITION,
            })
    }
}
