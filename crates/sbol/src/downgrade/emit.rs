//! The main conversion pass and the synthesized-triple emitters
//! (dual-role splits, MapsTo decompositions, FunctionalComponent
//! directions, SequenceAnnotation wrappers, instance defaults).

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
        self.emit_dual_role_components();
        self.duplicate_collection_memberships();
        self.emit_backport_metadata();
        self.emit_component_instance_defaults();
        self.rewrite_participants();
    }

    /// For each `sbol2:member` triple whose object is the bare-IRI
    /// half of a dual-role split, emit a companion member pointing at
    /// the other half. Without this an SBOL 2 Collection in the output
    /// would only reference one half — losing the structural OR
    /// functional view of the split Component.
    pub(super) fn duplicate_collection_memberships(&mut self) {
        if self.component_splits.is_empty() {
            return;
        }
        let mut additions = Vec::new();
        // Build a lookup from each split's bare-IRI to its other-half
        // versioned IRI. The bare IRI is whichever side has an empty
        // display_suffix.
        let mut other_half: HashMap<String, String> = HashMap::new();
        for split in self.component_splits.values() {
            if split.shape != ComponentShape::DualRole {
                continue;
            }
            let cd_v2 = self.rewrite_iri(&split.cd_iri).to_owned();
            let md_v2 = self.rewrite_iri(&split.md_iri).to_owned();
            if split.cd_display_suffix.is_empty() {
                other_half.insert(cd_v2, md_v2);
            } else if split.md_display_suffix.is_empty() {
                other_half.insert(md_v2, cd_v2);
            }
        }
        for triple in self.output_triples.iter() {
            if triple.predicate.as_str() != v2::SBOL2_MEMBER {
                continue;
            }
            let Some(object_iri) = triple.object.as_iri() else {
                continue;
            };
            if let Some(other) = other_half.get(object_iri.as_str()) {
                additions.push(Triple {
                    subject: triple.subject.clone(),
                    predicate: triple.predicate.clone(),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(other.clone()))),
                });
            }
        }
        self.output_triples.extend(additions);
    }

    /// Synthesizes the linking FunctionalComponent for each dual-role
    /// Component split and stamps `backport:sbol3identity` on both
    /// halves so the inverse direction can re-merge.
    pub(super) fn emit_dual_role_components(&mut self) {
        let mut entries: Vec<(String, ComponentSplit)> = self
            .component_splits
            .iter()
            .filter(|(_, split)| split.shape == ComponentShape::DualRole)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        for (sbol3_iri, split) in entries {
            let cd_v2 = self.rewrite_iri(&split.cd_iri).to_owned();
            let md_v2 = self.rewrite_iri(&split.md_iri).to_owned();

            // Stamp backport:sbol3identity on both halves so downstream
            // tools (and a future re-upgrade pass) can see they share
            // an SBOL 3 origin.
            let sbol3_identity_object =
                Term::Resource(Resource::Iri(Iri::new_unchecked(sbol3_iri)));
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(cd_v2.clone())),
                predicate: Iri::from_static(v2::BACKPORT_SBOL3_IDENTITY),
                object: sbol3_identity_object.clone(),
            });
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(md_v2.clone())),
                predicate: Iri::from_static(v2::BACKPORT_SBOL3_IDENTITY),
                object: sbol3_identity_object,
            });

            // Synthesize the linking FunctionalComponent on the MD
            // pointing at the CD via `sbol2:definition`. Without this
            // the MD half is dangling: SBOL 2 ModuleDefinitions are
            // only useful when their FCs reference real CDs.
            let Some(fc_iri) = split.linking_fc_iri.as_ref() else {
                continue;
            };
            let fc_v2 = self.rewrite_iri(fc_iri).to_owned();
            let fc_resource = Resource::Iri(Iri::new_unchecked(fc_v2.clone()));
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(md_v2)),
                predicate: Iri::from_static(v2::SBOL2_FUNCTIONAL_COMPONENT_PROP),
                object: Term::Resource(fc_resource.clone()),
            });
            self.output_triples.push(Triple {
                subject: fc_resource.clone(),
                predicate: Iri::from_static(v3::RDF_TYPE),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(
                    v2::SBOL2_FUNCTIONAL_COMPONENT,
                ))),
            });
            // Prefer the disambiguated displayId stored alongside
            // `linking_fc_iri`. Falling back to `original_display_id`
            // would re-introduce the SBOL 2 compliance mismatch the
            // collision-avoidance allocator was designed to prevent.
            let fc_display_id = split
                .linking_fc_display_id
                .clone()
                .unwrap_or_else(|| split.original_display_id.clone());
            self.output_triples.push(Triple {
                subject: fc_resource.clone(),
                predicate: Iri::from_static(v2::SBOL2_DISPLAY_ID),
                object: Term::Literal(sbol_rdf::Literal::simple(fc_display_id)),
            });
            self.output_triples.push(Triple {
                subject: fc_resource.clone(),
                predicate: Iri::from_static(v2::SBOL2_DEFINITION),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(cd_v2))),
            });
            self.output_triples.push(Triple {
                subject: fc_resource.clone(),
                predicate: Iri::from_static(v2::SBOL2_ACCESS),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(v2::SBOL2_ACCESS_PUBLIC))),
            });
            self.output_triples.push(Triple {
                subject: fc_resource.clone(),
                predicate: Iri::from_static(v2::SBOL2_DIRECTION),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(v2::SBOL2_DIRECTION_NONE))),
            });
            self.output_triples.push(Triple {
                subject: fc_resource,
                predicate: Iri::from_static(v2::BACKPORT_TYPE),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(
                    v2::BACKPORT_SPLIT_COMPONENT_COMPOSITION,
                ))),
            });
        }
    }

    /// Rewrites every `sbol2:participant` object that points at an
    /// SBOL 3 SubComponent under a dual-role parent to land on its
    /// FunctionalComponent variant instead. SBOL 2 Participations
    /// reference FCs, not bare Components.
    pub(super) fn rewrite_participants(&mut self) {
        if self.participant_remap.is_empty() {
            return;
        }
        let remap = self.participant_remap.clone();
        for triple in &mut self.output_triples {
            if triple.predicate.as_str() != v2::SBOL2_PARTICIPANT {
                continue;
            }
            let Some(target_iri) = triple.object.as_iri().map(|i| i.as_str().to_owned()) else {
                continue;
            };
            if let Some(fc_iri) = remap.get(&target_iri) {
                triple.object = Term::Resource(Resource::Iri(Iri::new_unchecked(fc_iri.clone())));
            }
        }
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

            // Route the synthesized MapsTo IRI through the shared used-IRI
            // pool. Canonical form is `{carrier}/{display_id}`; if a
            // pre-existing subject already lives at that IRI (a Location
            // sharing the MapsTo's displayId under the same SubComponent,
            // for example), the allocator disambiguates with a `_N`
            // suffix and the emitted displayId picks up the new last
            // segment to stay SBOL 2 sbol-12302 compliant.
            let (mapsto_display_id, mapsto_unversioned) =
                next_available_child_iri(&info.carrier_v3, &info.display_id, &mut self.used_iris);
            let mapsto_v2_iri = match &version {
                Some(v) => append_segment(&mapsto_unversioned, v),
                None => mapsto_unversioned.clone(),
            };
            let mapsto_resource = Resource::Iri(Iri::new_unchecked(mapsto_v2_iri.clone()));

            // Attach the MapsTo to its carrier.
            let carrier_v2 = self.rewrite_iri(&info.carrier_v3).to_owned();
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
        // existence check is O(N) and the loop overall is O(M·N) — slow on
        // library-scale designs.
        let mut existing = self.build_subject_predicate_index();

        for (fc_v3, direction) in entries {
            let Some((subject, emits_as)) = self.interface_feature_emission(&fc_v3) else {
                continue;
            };
            let native_feature = !self.backport_types.contains_key(&fc_v3);
            match emits_as {
                InterfaceFeatureKind::Component => {
                    if native_feature {
                        self.emit_default_if_missing(
                            &mut existing,
                            &subject,
                            v2::SBOL2_ACCESS,
                            v2::SBOL2_ACCESS_PUBLIC,
                        );
                    }
                }
                InterfaceFeatureKind::Module => {}
                InterfaceFeatureKind::FunctionalComponent => {
                    if native_feature {
                        self.emit_default_if_missing(
                            &mut existing,
                            &subject,
                            v2::SBOL2_ACCESS,
                            v2::SBOL2_ACCESS_PUBLIC,
                        );
                    }
                    if !self.restored_fc_directions.contains(&fc_v3) {
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
    }

    pub(super) fn interface_feature_emission(
        &self,
        feature_v3: &str,
    ) -> Option<(String, InterfaceFeatureKind)> {
        if let Some(split) = self.subcomponent_splits.get(feature_v3) {
            return Some((
                self.rewrite_iri(&split.functional_component_iri).to_owned(),
                InterfaceFeatureKind::FunctionalComponent,
            ));
        }

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
        let backported_subjects: HashSet<String> = self
            .backport_types
            .keys()
            .map(|subject| self.rewrite_iri(subject).to_owned())
            .collect();
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
        // existing triple) — visible at library scale.
        let mut existing = self.build_subject_predicate_index();

        let mut components: Vec<String> = components.into_iter().collect();
        components.sort();
        for subject in components {
            if backported_subjects.contains(&subject) {
                continue;
            }
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
            if backported_subjects.contains(&subject) {
                continue;
            }
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

            let has_preserved_persistent_identity = info
                .preserved_metadata
                .iter()
                .any(|metadata| metadata.predicate == v2::SBOL2_PERSISTENT_IDENTITY);
            let has_preserved_version = info
                .preserved_metadata
                .iter()
                .any(|metadata| metadata.predicate == v2::SBOL2_VERSION);

            for metadata in &info.preserved_metadata {
                self.output_triples.push(Triple {
                    subject: sa_resource.clone(),
                    predicate: Iri::new_unchecked(metadata.predicate.clone()),
                    object: self.rewrite_term(&metadata.object),
                });
            }

            // SA identity metadata: emit defaults only when the original SA
            // did not preserve explicit identity/version metadata.
            if !has_preserved_persistent_identity {
                self.output_triples.push(Triple {
                    subject: sa_resource.clone(),
                    predicate: Iri::from_static(v2::SBOL2_PERSISTENT_IDENTITY),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(
                        info.sa_iri_unversioned.clone(),
                    ))),
                });
            }
            if !has_preserved_version && let Some(v) = version {
                self.output_triples.push(Triple {
                    subject: sa_resource.clone(),
                    predicate: Iri::from_static(v2::SBOL2_VERSION),
                    object: Term::Literal(sbol_rdf::Literal::simple(v)),
                });
            }
        }
    }
}
