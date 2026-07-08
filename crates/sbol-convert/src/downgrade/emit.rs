//! The main conversion pass and the synthesized-triple emitters
//! (MapsTo decompositions, FunctionalComponent directions,
//! SequenceAnnotation wrappers, instance defaults).

use super::helpers::*;
use super::*;

impl<'a> Engine<'a> {
    /// Main pass: walk every triple, applying IRI rewrites, type and
    /// predicate downgrades, and value-level reverse mappings.
    pub(super) fn convert(&mut self) {
        // Iterate by index to avoid cloning the entire triple slice
        // up front; the loop bound is captured before mutation so the
        // structural emissions below don't feed back into the walk.
        let n = self.input.rdf_graph().triples().len();
        for i in 0..n {
            let triple = self.input.rdf_graph().triples()[i].clone();
            self.handle_triple(&triple);
        }
        self.emit_sa_wrappers();
        self.emit_mapsto_decompositions();
        self.emit_fc_directions();
        self.emit_backport_metadata();
        self.emit_component_instance_defaults();
    }

    /// Re-emits the SBOL 2 MapsTo for every ComponentReference + Constraint
    /// pair the upgrade decomposed.
    pub(super) fn emit_mapsto_decompositions(&mut self) {
        let mut entries: Vec<(String, &MapsToReconstruction)> = self
            .mapsto_reconstructions
            .iter()
            .map(|(k, v)| (k.clone(), v))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        for (_cref_iri, info) in entries {
            // MapsTo lives under the carrier in SBOL 2. The carrier's
            // top-level (the parent Component / ModuleDefinition)
            // dictates the version segment; without an owning top-level
            // fall through to the global default_version (or unversioned
            // if synthesis is disabled).
            let owning_top_level = self.owning_top_level_of(&info.carrier_v3);
            let version = match owning_top_level.as_deref() {
                Some(tl) => self.effective_version_for_top_level(tl),
                None => self.options.default_version.clone(),
            };

            // The MapsTo nests under the carrier's SBOL 2 identity, so the
            // parent IRI must be the *downgraded* carrier — not the SBOL 3
            // `inChildOf`, whose version-in-IRI shape would otherwise leak
            // into the SBOL 2 MapsTo identity. The MapsTo hangs off the
            // carrier's version-free persistent identity and carries its own
            // version segment.
            let carrier_v2 = self.rewrite_iri(&info.carrier_v3).to_owned();
            let carrier_base = match crate::uri::version_sbol2(&carrier_v2) {
                Some(v) if !v.is_empty() => carrier_v2
                    .strip_suffix(&format!("/{v}"))
                    .unwrap_or(&carrier_v2)
                    .to_owned(),
                _ => carrier_v2.clone(),
            };
            // Route the synthesized MapsTo IRI through the shared used-IRI
            // pool. Canonical form is `{carrier}/{display_id}`; if a
            // pre-existing subject already lives at that IRI (a Location
            // sharing the MapsTo's displayId under the same SubComponent,
            // for example), the allocator disambiguates with a `_N`
            // suffix and the emitted displayId picks up the new last
            // segment to stay SBOL 2 sbol-12302 compliant.
            let (mapsto_display_id, mapsto_unversioned) =
                next_available_child_iri(&carrier_base, &info.display_id, &mut self.used_iris);
            let mapsto_v2_iri = match &version {
                Some(v) => append_segment(&mapsto_unversioned, v),
                None => mapsto_unversioned.clone(),
            };
            let mapsto_resource = Resource::Iri(Iri::new_unchecked(mapsto_v2_iri.clone()));

            // Attach the MapsTo to its carrier.
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(carrier_v2)),
                predicate: Iri::from_static(v2::SBOL2_MAPS_TO_PROP),
                object: Term::Resource(mapsto_resource.clone()),
            });

            self.output_triples.push(Triple {
                subject: mapsto_resource.clone(),
                predicate: Iri::from_static(v3::RDF_TYPE),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(v2::SBOL2_MAPS_TO))),
            });
            self.output_triples.push(Triple {
                subject: mapsto_resource.clone(),
                predicate: Iri::from_static(v2::SBOL2_DISPLAY_ID),
                object: Term::Literal(sbol_rdf::Literal::simple(mapsto_display_id)),
            });
            let local_v2 = self.rewrite_iri(&info.local_v3).to_owned();
            self.output_triples.push(Triple {
                subject: mapsto_resource.clone(),
                predicate: Iri::from_static(v2::SBOL2_LOCAL),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(local_v2))),
            });
            let remote_v2 = self.rewrite_iri(&info.remote_v3).to_owned();
            self.output_triples.push(Triple {
                subject: mapsto_resource.clone(),
                predicate: Iri::from_static(v2::SBOL2_REMOTE),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(remote_v2))),
            });
            let refinement = info
                .refinement
                .clone()
                .unwrap_or_else(|| v2::SBOL2_REFINEMENT_USE_LOCAL.to_owned());
            self.output_triples.push(Triple {
                subject: mapsto_resource.clone(),
                predicate: Iri::from_static(v2::SBOL2_REFINEMENT),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(refinement))),
            });

            self.output_triples.push(Triple {
                subject: mapsto_resource.clone(),
                predicate: Iri::from_static(v2::SBOL2_PERSISTENT_IDENTITY),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(mapsto_unversioned))),
            });
            if let Some(v) = version {
                self.output_triples.push(Triple {
                    subject: mapsto_resource,
                    predicate: Iri::from_static(v2::SBOL2_VERSION),
                    object: Term::Literal(sbol_rdf::Literal::simple(v)),
                });
            }
            self.report.counts.maps_to_reconstructed += 1;
        }
    }

    /// Emits SBOL 2 interface metadata for Features listed under an SBOL 3
    /// Interface. FunctionalComponents get `sbol2:direction`; structural
    /// Components get `sbol2:access public`, matching the SBOL 2 → 3 mapping.
    pub(super) fn emit_fc_directions(&mut self) {
        let mut entries: Vec<(String, FcDirection)> = self
            .fc_directions
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        // Build the (subject, predicate) index once instead of rescanning
        // `output_triples` per check. Without this the per-feature
        // existence check is O(N) and the loop overall is O(M·N), slow on
        // library-scale designs.
        let mut existing = self.build_subject_predicate_index();

        for (fc_v3, direction) in entries {
            let Some((subject, emits_as)) = self.interface_feature_emission(&fc_v3) else {
                continue;
            };
            match emits_as {
                InterfaceFeatureKind::Component => {
                    self.emit_default_if_missing(
                        &mut existing,
                        &subject,
                        v2::SBOL2_ACCESS,
                        v2::SBOL2_ACCESS_PUBLIC,
                    );
                }
                InterfaceFeatureKind::Module => {}
                InterfaceFeatureKind::FunctionalComponent => {
                    self.emit_default_if_missing(
                        &mut existing,
                        &subject,
                        v2::SBOL2_ACCESS,
                        v2::SBOL2_ACCESS_PUBLIC,
                    );
                    self.emit_default_if_missing(
                        &mut existing,
                        &subject,
                        v2::SBOL2_DIRECTION,
                        direction.sbol2_iri(),
                    );
                }
            }
        }
    }

    pub(super) fn interface_feature_emission(
        &self,
        feature_v3: &str,
    ) -> Option<(String, InterfaceFeatureKind)> {
        let feature_v2 = self.rewrite_iri(feature_v3).to_owned();
        let parent_type = self
            .feature_parent
            .get(feature_v3)
            .and_then(|parent| self.resolved_types.get(parent))
            .map(String::as_str);
        let target_type = self
            .subcomponent_targets
            .get(feature_v3)
            .and_then(|target| self.resolved_types.get(target))
            .map(String::as_str);
        let emits_as = match parent_type {
            Some(v2::SBOL2_COMPONENT_DEFINITION) => InterfaceFeatureKind::Component,
            Some(v2::SBOL2_MODULE_DEFINITION) => match target_type {
                Some(v2::SBOL2_MODULE_DEFINITION) => InterfaceFeatureKind::Module,
                _ => InterfaceFeatureKind::FunctionalComponent,
            },
            _ => self
                .resolved_types
                .get(feature_v3)
                .map(String::as_str)
                .map(|ty| match ty {
                    v2::SBOL2_COMPONENT => InterfaceFeatureKind::Component,
                    v2::SBOL2_MODULE => InterfaceFeatureKind::Module,
                    _ => InterfaceFeatureKind::FunctionalComponent,
                })
                .unwrap_or(InterfaceFeatureKind::FunctionalComponent),
        };
        Some((feature_v2, emits_as))
    }

    pub(super) fn emit_component_instance_defaults(&mut self) {
        let mut components = HashSet::new();
        let mut functional_components = HashSet::new();
        for triple in &self.output_triples {
            if triple.predicate.as_str() != v3::RDF_TYPE {
                continue;
            }
            let (Some(subject), Some(object)) = (triple.subject.as_iri(), triple.object.as_iri())
            else {
                continue;
            };
            match object.as_str() {
                v2::SBOL2_COMPONENT => {
                    components.insert(subject.as_str().to_owned());
                }
                v2::SBOL2_FUNCTIONAL_COMPONENT => {
                    functional_components.insert(subject.as_str().to_owned());
                }
                _ => {}
            }
        }

        // One pass over `output_triples` to build the (subject, predicate)
        // index. Each default-emission then checks for the presence of an
        // existing triple in O(1), turning the per-feature scan from
        // O(N) into O(1). The previous shape was O(M·N) on the
        // FunctionalComponent loop (two checks × number of FCs × every
        // existing triple), visible at library scale.
        let mut existing = self.build_subject_predicate_index();

        let mut components: Vec<String> = components.into_iter().collect();
        components.sort();
        for subject in components {
            self.emit_default_if_missing(
                &mut existing,
                &subject,
                v2::SBOL2_ACCESS,
                v2::SBOL2_ACCESS_PRIVATE,
            );
        }

        let mut functional_components: Vec<String> = functional_components.into_iter().collect();
        functional_components.sort();
        for subject in functional_components {
            self.emit_default_if_missing(
                &mut existing,
                &subject,
                v2::SBOL2_ACCESS,
                v2::SBOL2_ACCESS_PRIVATE,
            );
            self.emit_default_if_missing(
                &mut existing,
                &subject,
                v2::SBOL2_DIRECTION,
                v2::SBOL2_DIRECTION_NONE,
            );
        }
    }

    /// Builds a `(subject_iri, predicate_iri)` set from every triple
    /// currently in `output_triples`. Callers that emit many default
    /// triples conditional on existence use this to avoid repeated linear
    /// scans of the output graph.
    pub(super) fn build_subject_predicate_index(&self) -> HashSet<(String, String)> {
        self.output_triples
            .iter()
            .filter_map(|triple| {
                triple.subject.as_iri().map(|iri| {
                    (
                        iri.as_str().to_owned(),
                        triple.predicate.as_str().to_owned(),
                    )
                })
            })
            .collect()
    }

    /// Emits a (subject, predicate, object) triple iff no triple already
    /// exists in `output_triples` for `(subject, predicate)`. Updates
    /// `existing` so a later check sees the new triple in O(1) without
    /// rebuilding the index.
    pub(super) fn emit_default_if_missing(
        &mut self,
        existing: &mut HashSet<(String, String)>,
        subject: &str,
        predicate: &'static str,
        object: &'static str,
    ) {
        let key = (subject.to_owned(), predicate.to_owned());
        if existing.contains(&key) {
            return;
        }
        self.output_triples.push(Triple {
            subject: Resource::Iri(Iri::new_unchecked(subject.to_owned())),
            predicate: Iri::from_static(predicate),
            object: Term::Resource(Resource::Iri(Iri::new_unchecked(object))),
        });
        existing.insert(key);
    }

    /// Walks back from an arbitrary IRI to the top-level whose IRI is
    /// its prefix, or returns `None` if no top-level claims it.
    pub(super) fn owning_top_level_of(&self, iri: &str) -> Option<String> {
        if self.top_levels.contains(iri) {
            return Some(iri.to_owned());
        }
        self.top_levels
            .iter()
            .filter(|top| {
                let prefix = format!("{top}/");
                iri.starts_with(&prefix)
            })
            .max_by(|a, b| a.len().cmp(&b.len()).then_with(|| b.cmp(a)))
            .cloned()
    }

    /// For every SubComponent that was the target of an SA-with-component
    /// collapse, re-emit the SBOL 2 SequenceAnnotation wrapper that the
    /// upgrade discarded. Each wrapper points at the SubComponent via
    /// `sbol2:component` and at every Location attached to that
    /// SubComponent via `sbol2:location`. The parent ComponentDefinition
    /// gains a `sbol2:sequenceAnnotation` pointer to the new SA.
    pub(super) fn emit_sa_wrappers(&mut self) {
        // Stable iteration: sort by SubComponent IRI for deterministic
        // output (round-trip diffs depend on it).
        let mut entries: Vec<(String, &SaCollapseInfo)> = self
            .sa_collapses
            .iter()
            .map(|(k, v)| (k.clone(), v))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        for (subcomp_v3, info) in entries {
            let version = self.effective_version_for_top_level(&info.parent_component);
            let sa_v2_iri = match &version {
                Some(v) => append_segment(&info.sa_iri_unversioned, v),
                None => info.sa_iri_unversioned.clone(),
            };
            let sa_resource = Resource::Iri(Iri::new_unchecked(sa_v2_iri.clone()));

            // Point the parent CD at the new SA.
            let parent_cd_v2 = self.rewrite_iri(&info.parent_cd).to_owned();
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(parent_cd_v2)),
                predicate: Iri::from_static(v2::SBOL2_SEQUENCE_ANNOTATION_PROP),
                object: Term::Resource(sa_resource.clone()),
            });

            // SA properties: rdf:type, displayId, component, location*.
            self.output_triples.push(Triple {
                subject: sa_resource.clone(),
                predicate: Iri::from_static(v3::RDF_TYPE),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(
                    v2::SBOL2_SEQUENCE_ANNOTATION,
                ))),
            });
            self.output_triples.push(Triple {
                subject: sa_resource.clone(),
                predicate: Iri::from_static(v2::SBOL2_DISPLAY_ID),
                object: Term::Literal(sbol_rdf::Literal::simple(info.sa_display_id.clone())),
            });
            let subcomp_v2 = self.rewrite_iri(&subcomp_v3).to_owned();
            self.output_triples.push(Triple {
                subject: sa_resource.clone(),
                predicate: Iri::from_static(v2::SBOL2_COMPONENT_PROP),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(subcomp_v2))),
            });
            for loc_v3 in &info.locations {
                let loc_v2 = self.rewrite_iri(loc_v3).to_owned();
                self.output_triples.push(Triple {
                    subject: sa_resource.clone(),
                    predicate: Iri::from_static(v2::SBOL2_LOCATION_PROP),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(loc_v2))),
                });
            }

            // SA identity metadata.
            self.output_triples.push(Triple {
                subject: sa_resource.clone(),
                predicate: Iri::from_static(v2::SBOL2_PERSISTENT_IDENTITY),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(
                    info.sa_iri_unversioned.clone(),
                ))),
            });
            if let Some(v) = version {
                self.output_triples.push(Triple {
                    subject: sa_resource.clone(),
                    predicate: Iri::from_static(v2::SBOL2_VERSION),
                    object: Term::Literal(sbol_rdf::Literal::simple(v)),
                });
            }
        }
    }
}
