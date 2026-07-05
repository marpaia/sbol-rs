//! SBOL 3 → SBOL 2 predicate downgrading, backport metadata emission, and
//! the IRI/type rewriting primitives shared across the conversion pass.

use super::helpers::*;
use super::*;

impl<'a> Engine<'a> {
    pub(super) fn handle_sbol3_predicate(&mut self, triple: &Triple) {
        let predicate = triple.predicate.as_str();

        // SubComponent under a dual-role parent: route to the three
        // SBOL 2 variants emitted in `emit_subcomponent_split_types`.
        if let Some(subject_iri) = triple.subject.as_iri() {
            let s = subject_iri.as_str().to_owned();
            if let Some(sc_split) = self.subcomponent_splits.get(&s).cloned() {
                self.handle_subcomponent_split_predicate(triple, &sc_split);
                return;
            }
        }

        // Dual-role Component: every predicate routes to the structural
        // half (CD), the functional half (MD), or both, based on whether
        // it describes structure (sequence, role, type) or function
        // (interaction, model). Identified properties (displayId, name,
        // description) land on both halves.
        if let Some(subject_iri) = triple.subject.as_iri() {
            let s = subject_iri.as_str().to_owned();
            if let Some(split) = self.component_splits.get(&s).cloned()
                && split.shape == ComponentShape::DualRole
            {
                self.handle_dual_role_predicate(triple, &split);
                return;
            }
        }

        // Drop predicates that have no SBOL 2 equivalent. `hasNamespace`
        // is the most important one — the namespace is implicit in
        // the restored persistentIdentity / versioned IRI in SBOL 2.
        if predicate == v3::SBOL_HAS_NAMESPACE {
            return;
        }

        // `hasFeature` is context-dependent in SBOL 2 — it becomes
        // `component`, `functionalComponent`, `module`, or
        // `sequenceAnnotation` depending on what the feature is and
        // what its parent is. Resolve here using the type maps built
        // in preflight.
        if predicate == v3::SBOL_HAS_FEATURE {
            self.handle_has_feature(triple);
            return;
        }

        // `hasLocation` on a SubComponent that was a collapsed SA's
        // component — drop here; the reconstructed SA emits
        // `sbol2:location` itself in `emit_sa_wrappers`.
        if predicate == v3::SBOL_HAS_LOCATION
            && let Some(subject_iri) = triple.subject.as_iri()
            && self.sa_collapses.contains_key(subject_iri.as_str())
        {
            return;
        }

        // `sbol3:type` on an MD-derived Component drops the
        // synthesized `SBO:functionalEntity` term — SBOL 2 MDs don't
        // carry it. The original SBOL 2 type triples (if any) are
        // still emitted because they pass through the same predicate.
        if predicate == v3::SBOL_TYPE
            && let Some(subject_iri) = triple.subject.as_iri()
            && self
                .backport_types
                .get(subject_iri.as_str())
                .map(String::as_str)
                == Some(v2::SBOL2_MODULE_DEFINITION)
            && triple.object.as_iri().map(|i| i.as_str())
                == Some("https://identifiers.org/SBO:0000241")
        {
            return;
        }

        // SBOL 3 requires `hasSequence` on every Range / Cut /
        // Location; SBOL 2 represents that linkage implicitly through
        // the location's parent SequenceAnnotation → ComponentDefinition
        // → sequence chain. Forwarding `hasSequence` here would emit a
        // bare `sbol2:sequence` triple on the Range, which the upgrade
        // round-trip then duplicates against the inferred location
        // sequence. Drop on Location-typed subjects.
        if predicate == v3::SBOL_HAS_SEQUENCE
            && let Some(subject_iri) = triple.subject.as_iri()
        {
            let resolved = self
                .resolved_types
                .get(subject_iri.as_str())
                .map(String::as_str);
            if matches!(
                resolved,
                Some(v2::SBOL2_RANGE) | Some(v2::SBOL2_CUT) | Some(v2::SBOL2_GENERIC_LOCATION)
            ) {
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

    /// Unknown SBOL 3 predicates (something added to the spec since we last
    /// updated the table, or a private extension authored in the v3
    /// namespace) cannot be emitted verbatim in an SBOL 2 graph. Archive them
    /// under the backport namespace so the data is preserved for a future
    /// re-upgrade without polluting the SBOL 2 surface.
    pub(super) fn archive_unknown_sbol3_predicate(&mut self, triple: &Triple) {
        let Some(local) = triple.predicate.as_str().strip_prefix(v3::SBOL_NS) else {
            return;
        };
        let preserved = format!("{}{local}", v2::BACKPORT_SBOL3_PREFIX);
        self.output_triples.push(Triple {
            subject: self.rewrite_resource(&triple.subject),
            predicate: Iri::new_unchecked(preserved),
            object: self.rewrite_term(&triple.object),
        });
    }

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

    /// Consumes the next preserved BioPAX variant for
    /// `(subject, sbo_term)` and advances the per-pair cursor. Returns
    /// `None` when the subject has no preserved variants for that SBO
    /// target or when the queue is exhausted; the caller then falls
    /// back to the default `*Region`-style mapping.
    pub(super) fn consume_biopax_variant(
        &mut self,
        subject: Option<&str>,
        sbo_iri: &str,
    ) -> Option<String> {
        let subject = subject?;
        let key = (subject.to_owned(), sbo_iri.to_owned());
        let queue = self.biopax_variant_queue.get(&key)?;
        let cursor = self.biopax_variant_cursor.entry(key).or_insert(0);
        let variant = queue.get(*cursor)?.clone();
        *cursor += 1;
        Some(variant)
    }

    /// Reverses value-level mappings (orientation, encoding, type,
    /// restriction). When `subject` is `Some`, consults the
    /// subject-keyed backport hints for value mappings that are lossy
    /// by themselves (e.g. BioPAX `Dna`/`DnaRegion` collapse).
    ///
    /// Takes `&mut self` because the `sbol3:type` reverse mapping for
    /// BioPAX advances a per-`(subject, sbo_term)` cursor: each input
    /// triple that maps to the same SBO target consumes the next
    /// preserved variant from
    /// [`Engine::biopax_variant_queue`]. Without that statefulness, two
    /// `sbol3:type SBO:0000251` triples both fall back to the first
    /// preserved variant and the second BioPAX type is lost.
    pub(super) fn reverse_value_for_subject(
        &mut self,
        subject: Option<&str>,
        predicate_str: &str,
        object: &Term,
    ) -> Term {
        let iri = match object.as_iri() {
            Some(iri) => iri.as_str(),
            None => return object.clone(),
        };
        let mapped: Option<String> = match predicate_str {
            v3::SBOL_ORIENTATION => values::map_orientation(iri).map(String::from),
            v3::SBOL_ENCODING => values::map_encoding(iri).map(String::from),
            // Multi-valued `sbol3:type` on a Component can mix BioPAX
            // collapses (SBO:DNA, …) with topology / role types (SO:linear).
            // Only the BioPAX side benefits from the backport hint —
            // leave SO and friends to pass through untouched.
            //
            // For each input `sbol3:type` triple that maps to an SBO
            // term the upgrade collapses BioPAX onto, consume the next
            // preserved variant for `(subject, sbo_term)`. This handles
            // the otherwise-lossy case where two distinct BioPAX
            // variants share an SBO target (e.g. `biopax:Dna` and
            // `biopax:DnaRegion` both collapse to `SBO:0000251`) — each
            // input triple gets a distinct variant in restoration order.
            // Fall back to the default `*Region`-style mapping when no
            // hint exists or the queue is exhausted.
            v3::SBOL_TYPE => values::map_biopax_type(iri).map(|default| {
                self.consume_biopax_variant(subject, iri)
                    .unwrap_or_else(|| default.to_owned())
            }),
            v3::SBOL_RESTRICTION => values::map_restriction(iri),
            v3::SBOL_CARDINALITY => values::map_cardinality(iri).map(String::from),
            v3::SBOL_STRATEGY => values::map_strategy(iri).map(String::from),
            v3::SBOL_ROLE_INTEGRATION => values::map_role_integration(iri).map(String::from),
            _ => None,
        };
        match mapped {
            Some(sbol2_iri) => Term::Resource(Resource::Iri(Iri::new_unchecked(sbol2_iri))),
            None => object.clone(),
        }
    }

    pub(super) fn emit_backport_metadata(&mut self) {
        // SBOL 2 requires `persistentIdentity` and `version` on every
        // owned object, not just top-levels. Iterate every subject
        // that received a version suffix during identity restoration
        // (top-levels and their children) and emit both triples.
        //
        // The version for a child is the parent's version, propagated
        // through `iri_rewrites` (see `build_iri_rewrites` phase 2).
        let mut entries: Vec<(String, String)> = self
            .iri_rewrites
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        entries.sort();
        for (sbol3_iri, sbol2_iri) in entries {
            // Skip subjects folded into a structural re-synthesis — the
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

            if !self.resolved_types.contains_key(&sbol3_iri)
                && let Some(backport_type) = self.backport_types.get(&sbol3_iri)
            {
                self.output_triples.push(Triple {
                    subject: subject.clone(),
                    predicate: Iri::from_static(v3::RDF_TYPE),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(
                        backport_type.clone(),
                    ))),
                });
            }

            // sbol3namespace: stash the SBOL 3 object's `hasNamespace` on
            // the SBOL 2 object so sbol-utilities / sbolgraph — and a
            // future sbol-rs re-upgrade — can reconstruct the SBOL 3
            // namespace without re-deriving it from the identity IRI.
            if let Some(namespace) = self.sbol3_namespaces.get(&sbol3_iri) {
                self.output_triples.push(Triple {
                    subject: subject.clone(),
                    predicate: Iri::from_static(v2::BACKPORT_SBOL3_NAMESPACE),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(namespace.clone()))),
                });
            }

            // persistentIdentity: prefer the recorded backport value;
            // otherwise the unversioned SBOL 3 IRI itself is the
            // persistent identity.
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

            // Version: emit when the subject's owning top-level has
            // either a preserved `backport:sbol2version` or
            // synthesis enabled via `default_version`; otherwise skip
            // (SBOL 2 makes `sbol2:version` optional).
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
        if let Some(backport_type) = self.backport_types.get(subject_iri)
            && backport_type_applies_to_sbol3_type(backport_type, object_iri)
        {
            return Some(backport_type.clone());
        }
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
        match self.backport_types.get(component_iri).map(String::as_str) {
            Some(v2::SBOL2_COMPONENT_DEFINITION) => return Some(v2::SBOL2_COMPONENT_DEFINITION),
            Some(v2::SBOL2_MODULE_DEFINITION) => return Some(v2::SBOL2_MODULE_DEFINITION),
            _ => {}
        }
        self.component_splits
            .get(component_iri)
            .map(|split| match split.shape {
                ComponentShape::MdOnly => v2::SBOL2_MODULE_DEFINITION,
                _ => v2::SBOL2_COMPONENT_DEFINITION,
            })
    }
}
