//! The emission and synthesis half of the conversion engine: collapsing
//! SequenceAnnotations, synthesizing Interfaces and MapsTo decompositions,
//! emitting backport metadata, and inferring Location sequences. Pairs with
//! the preflight/handler half in [`super::engine`].

use super::*;

impl<'a> Engine<'a> {
    /// Handles the SA-with-component collapse. When a SequenceAnnotation
    /// references a `sbol2:component`, the SA shell is discarded and its
    /// Locations migrate onto that SubComponent. Returns `true` when the
    /// triple was consumed by this rule; the caller then skips its default
    /// predicate handling.
    pub(super) fn preserve_collapsed_sa_metadata(
        &mut self,
        triple: &Triple,
        target_subcomponent: &str,
    ) {
        if !self.options.preserve_backport {
            return;
        }
        let predicate = triple.predicate.as_str();
        if predicate == v2::SBOL2_DISPLAY_ID
            || predicate == v2::SBOL2_COMPONENT_PROP
            || predicate == v2::SBOL2_LOCATION_PROP
        {
            return;
        }

        let subject = Resource::Iri(Iri::new_unchecked(
            self.identity.rewrite_iri(target_subcomponent).to_owned(),
        ));
        let preserved = format!(
            "{}{}",
            v2::BACKPORT_SEQUENCE_ANNOTATION_PREDICATE_PREFIX,
            hex_encode(predicate.as_bytes())
        );
        self.output_triples.push(Triple {
            subject,
            predicate: Iri::new_unchecked(preserved),
            object: self.identity.rewrite_term(&triple.object),
        });
    }

    pub(super) fn try_collapse_sa_triple(&mut self, triple: &Triple) -> bool {
        let predicate = triple.predicate.as_str();
        let subject_iri = match triple.subject.as_iri() {
            Some(iri) => iri.as_str(),
            None => return false,
        };

        // Drop the parent CD's `sbol2:sequenceAnnotation` triple if it
        // points at a collapsed SA. The SubComponent is already attached as
        // a feature via the CD's own `sbol2:component` triple.
        if predicate == v2::SBOL2_SEQUENCE_ANNOTATION_PROP
            && let Some(target_iri) = triple.object.as_iri()
            && self.sa_to_subcomponent.contains_key(target_iri.as_str())
        {
            return true;
        }

        // Everything that follows acts on triples whose subject IS the
        // collapsed SA shell.
        let target_subcomponent = match self.sa_to_subcomponent.get(subject_iri) {
            Some(target) => target.clone(),
            None => return false,
        };

        // Redirect Location triples onto the SubComponent under the SBOL 3
        // `hasLocation` predicate.
        if predicate == v2::SBOL2_LOCATION_PROP {
            let new_subject = Resource::Iri(Iri::new_unchecked(
                self.identity.rewrite_iri(&target_subcomponent).to_owned(),
            ));
            let object = self.identity.rewrite_term(&triple.object);
            self.output_triples.push(Triple {
                subject: new_subject,
                predicate: Iri::from_static(v3::SBOL_HAS_LOCATION),
                object,
            });
            return true;
        }

        // Preserve the SA's displayId on the SubComponent under the
        // backport namespace so round-trip downgrades can reconstruct the
        // original SA identity.
        if predicate == v2::SBOL2_DISPLAY_ID && self.options.preserve_backport {
            let new_subject = Resource::Iri(Iri::new_unchecked(
                self.identity.rewrite_iri(&target_subcomponent).to_owned(),
            ));
            self.output_triples.push(Triple {
                subject: new_subject,
                predicate: Iri::from_static(v2::BACKPORT_SEQUENCE_ANNOTATION_DISPLAY_ID),
                object: triple.object.clone(),
            });
            return true;
        }

        // `sbol2:component` itself is consumed by the collapse. The
        // resulting SubComponent is reached through the parent CD's own
        // `sbol2:component` triple, so we don't emit any successor here.
        if predicate == v2::SBOL2_COMPONENT_PROP {
            return true;
        }

        // Every other triple on the SA shell is preserved on the
        // replacement SubComponent under an encoded backport predicate.
        // The SA no longer exists in the SBOL 3 graph, and replaying the
        // original predicate directly would leave an orphan subject.
        self.preserve_collapsed_sa_metadata(triple, &target_subcomponent);
        true
    }

    /// Rewrites object values when the predicate carries a known enumerated
    /// SBOL 2 vocabulary value.
    pub(super) fn rewrite_value(&self, predicate_str: &str, object: &Term) -> Term {
        let iri = match object.as_iri() {
            Some(iri) => iri.as_str(),
            None => return object.clone(),
        };

        let mapped = match predicate_str {
            v2::SBOL2_ORIENTATION => values::map_orientation(iri),
            v2::SBOL2_ENCODING => values::map_encoding(iri),
            v2::SBOL2_TYPE => values::map_biopax_type(iri),
            v2::SBOL2_RESTRICTION => values::map_restriction(iri),
            v2::SBOL2_OPERATOR => values::map_operator(iri),
            v2::SBOL2_STRATEGY => values::map_strategy(iri),
            v2::SBOL2_ROLE_INTEGRATION => values::map_role_integration(iri),
            _ => None,
        };

        match mapped {
            Some(v3_iri) => Term::Resource(Resource::Iri(Iri::new_unchecked(v3_iri))),
            None => object.clone(),
        }
    }

    pub(super) fn emit_synthesized_triples(&mut self) {
        for (sbol2_iri, sbol2_type) in self.typed_subjects.clone().iter() {
            if !is_top_level_sbol2(sbol2_type) {
                continue;
            }

            let canonical = self.identity.rewrite_iri(sbol2_iri).to_owned();
            if self.namespaced_subjects.contains(&canonical) {
                continue;
            }
            self.namespaced_subjects.insert(canonical.clone());

            let namespace = match self.identity.namespace_for(&canonical) {
                Some(ns) => Some(ns.to_owned()),
                None => self.fallback_namespace(sbol2_iri),
            };

            if let Some(namespace) = namespace {
                self.output_triples.push(Triple {
                    subject: Resource::Iri(Iri::new_unchecked(canonical)),
                    predicate: Iri::from_static(v3::SBOL_HAS_NAMESPACE),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(namespace))),
                });
            }
        }

        // Custom-typed (GenericTopLevel) subjects are SBOL 3 top-levels and
        // need `hasNamespace` too. Their canonical IRI is already computed;
        // derive the namespace the same way as for typed top-levels.
        for canonical in self.generic_top_levels.clone() {
            if self.namespaced_subjects.contains(&canonical) {
                continue;
            }
            self.namespaced_subjects.insert(canonical.clone());
            let namespace = match self.identity.namespace_for(&canonical) {
                Some(ns) => Some(ns.to_owned()),
                None => self.fallback_namespace(&canonical),
            };
            if let Some(namespace) = namespace {
                self.output_triples.push(Triple {
                    subject: Resource::Iri(Iri::new_unchecked(canonical)),
                    predicate: Iri::from_static(v3::SBOL_HAS_NAMESPACE),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(namespace))),
                });
            }
        }

        // Promote `dcterms:title` → `sbol3:name` and
        // `dcterms:description` → `sbol3:description` for any subject
        // that has them. We don't drop the originals; sbolgraph
        // overwrites them, but keeping them preserves Dublin Core
        // metadata that SPARQL clients might query. Scan
        // `output_triples` (not the input) so dcterms attached to
        // structurally-consumed subjects (e.g. collapsed
        // SequenceAnnotation shells) aren't reintroduced as orphan
        // properties on subjects that no longer exist.
        let promoted: Vec<Triple> = self
            .output_triples
            .iter()
            .filter_map(|t| match t.predicate.as_str() {
                v2::DCTERMS_TITLE => Some(Triple {
                    subject: t.subject.clone(),
                    predicate: Iri::from_static(v3::SBOL_NAME),
                    object: t.object.clone(),
                }),
                v2::DCTERMS_DESCRIPTION => Some(Triple {
                    subject: t.subject.clone(),
                    predicate: Iri::from_static(v3::SBOL_DESCRIPTION),
                    object: t.object.clone(),
                }),
                _ => None,
            })
            .collect();
        self.output_triples.extend(promoted);

        self.infer_location_sequences();
        self.synthesize_mapsto_decomposition();
        self.synthesize_interfaces();
    }

    /// Synthesizes an `sbol3:Interface` on each Component that has at least
    /// one directional FunctionalComponent, or at least one public
    /// direction-none FunctionalComponent. The Interface lists input / output
    /// / nondirectional SubComponents per sbolgraph's mapping:
    /// `in` → input, `out` → output, `inout` and public+`none` →
    /// nondirectional.
    pub(super) fn synthesize_interfaces(&mut self) {
        let mut by_top_level: HashMap<String, Vec<(String, FcDirection)>> = HashMap::new();
        for (fc_iri, direction) in self.fc_directions.clone() {
            let top_v2 = self.owning_top_level(&fc_iri);
            by_top_level
                .entry(top_v2)
                .or_default()
                .push((fc_iri, direction));
        }

        for (top_v2, fcs) in by_top_level {
            let top_v3 = self.identity.rewrite_iri(&top_v2).to_owned();
            let (interface_display_id, interface_iri) =
                next_available_child_iri(&top_v3, "Interface", &mut self.used_iris);
            let interface_resource = Resource::Iri(Iri::new_unchecked(interface_iri.clone()));

            self.output_triples.push(Triple {
                subject: interface_resource.clone(),
                predicate: Iri::from_static(v3::RDF_TYPE),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(v3::SBOL_INTERFACE_CLASS))),
            });
            self.output_triples.push(Triple {
                subject: interface_resource.clone(),
                predicate: Iri::from_static(v3::SBOL_DISPLAY_ID),
                object: Term::Literal(sbol_rdf::Literal::simple(interface_display_id)),
            });

            for (fc_iri, direction) in fcs {
                let fc_v3 = self.identity.rewrite_iri(&fc_iri).to_owned();
                let fc_term = Term::Resource(Resource::Iri(Iri::new_unchecked(fc_v3.clone())));
                let predicates: &[&'static str] = match direction {
                    FcDirection::Input => &[v3::SBOL_INPUT],
                    FcDirection::Output => &[v3::SBOL_OUTPUT],
                    FcDirection::Inout => &[v3::SBOL_NONDIRECTIONAL],
                };
                for predicate in predicates {
                    self.output_triples.push(Triple {
                        subject: interface_resource.clone(),
                        predicate: Iri::from_static(predicate),
                        object: fc_term.clone(),
                    });
                }
            }

            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(top_v3)),
                predicate: Iri::from_static(v3::SBOL_HAS_INTERFACE),
                object: Term::Resource(interface_resource),
            });
            self.report.counts.interfaces_synthesized += 1;
        }
    }

    /// Decomposes each SBOL 2 MapsTo into an SBOL 3 ComponentReference and a
    /// Constraint, both attached to the enclosing top-level Component.
    ///
    /// Each MapsTo gets a fresh `{top_level}/{display_id}` ComponentReference
    /// IRI and a `{top_level}/{display_id}_constraint` Constraint IRI.
    /// SBOL 2 MapsTos under different carriers (Modules / FunctionalComponents)
    /// that share an enclosing top-level can collide on `displayId`; the
    /// engine numbers any subsequent collisions (`{display_id}_2`, `_3`, …)
    /// so each pair keeps a distinct identity under the same top-level.
    pub(super) fn synthesize_mapsto_decomposition(&mut self) {
        let mapstos: Vec<(String, MapsToInfo)> = self
            .mapsto_info
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        // Sorted iteration keeps numbering deterministic across runs.
        let mut mapstos = mapstos;
        mapstos.sort_by(|a, b| a.0.cmp(&b.0));
        for (mapsto_iri, info) in mapstos {
            let carrier = match self.owned_by.get(&mapsto_iri).cloned() {
                Some(c) => c,
                None => {
                    self.report.push(UpgradeWarning::UnresolvedMapsTo {
                        mapsto: mapsto_iri.clone(),
                        side: MapsToSide::Carrier,
                    });
                    continue;
                }
            };
            let top_level_v2 = self.owning_top_level(&carrier);
            let top_level = self.identity.rewrite_iri(&top_level_v2).to_owned();
            let carrier_v3 = self.identity.rewrite_iri(&carrier).to_owned();

            let local = match info.local.as_ref() {
                Some(l) => self.identity.rewrite_iri(l).to_owned(),
                None => {
                    self.report.push(UpgradeWarning::UnresolvedMapsTo {
                        mapsto: mapsto_iri.clone(),
                        side: MapsToSide::Local,
                    });
                    continue;
                }
            };
            let remote = match info.remote.as_ref() {
                Some(r) => self.identity.rewrite_iri(r).to_owned(),
                None => {
                    self.report.push(UpgradeWarning::UnresolvedMapsTo {
                        mapsto: mapsto_iri.clone(),
                        side: MapsToSide::Remote,
                    });
                    continue;
                }
            };

            // ComponentReference must be a direct child of the enclosing
            // Component per SBOL 3 IRI compliance (sbol3-10104). The MapsTo's
            // original IRI lives under the carrier (Module/FC), so we
            // synthesize a fresh IRI under the top-level Component using the
            // MapsTo's displayId (or its IRI's last path segment as a
            // fallback).
            let base_display_id = info
                .display_id
                .clone()
                .unwrap_or_else(|| last_path_segment(&mapsto_iri).to_owned());
            let (cref_display_id, cref_iri, constraint_display_id, constraint_iri) =
                next_available_mapsto_iris(&top_level, &base_display_id, &mut self.used_iris);

            self.emit_component_reference(
                &top_level,
                &cref_iri,
                &cref_display_id,
                &carrier_v3,
                &remote,
            );

            // Preserve the original refinement on the ComponentReference
            // under the backport namespace. Position in the emitted
            // Constraint already encodes `useLocal` / `useRemote` /
            // `verifyIdentical` losslessly per SBOL 3.1.0 §10.2, so the
            // hint is only strictly required for `sbol2:merge` (which
            // the spec collapses to `useRemote`) and for unknown
            // refinement IRIs. We emit it for every refinement anyway.
            // It's defense-in-depth against downstream tools that swap
            // the Constraint's subject/object, and it lets the downgrade
            // restore the exact source IRI even when position alone
            // would suffice.
            if self.options.preserve_backport
                && let Some(refinement) = info.refinement.as_ref()
            {
                self.output_triples.push(Triple {
                    subject: Resource::Iri(Iri::new_unchecked(cref_iri.clone())),
                    predicate: Iri::from_static(v2::BACKPORT_MAPS_TO_REFINEMENT),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(refinement.clone()))),
                });
            }
            if self.options.preserve_backport && cref_display_id != base_display_id {
                self.output_triples.push(Triple {
                    subject: Resource::Iri(Iri::new_unchecked(cref_iri.clone())),
                    predicate: Iri::from_static(v2::BACKPORT_MAPS_TO_DISPLAY_ID),
                    object: Term::Literal(sbol_rdf::Literal::simple(base_display_id.clone())),
                });
            }

            // Per SBOL 3.1.0 §10.2 the (subject, object, restriction) shape
            // of the Constraint is keyed on the MapsTo refinement:
            //   useLocal        → subject=SubComponent, object=CRef,        restriction=replaces
            //   useRemote       → subject=CRef,         object=SubComponent, restriction=replaces
            //   verifyIdentical → subject=CRef,         object=SubComponent, restriction=verifyIdentical
            //   merge           → handled as useRemote (merge was never well-defined and was removed)
            // Unknown refinements fall back to verifyIdentical with the CRef
            // in subject position, and warn so callers can see the coercion.
            let refinement_kind = match info.refinement.as_deref() {
                Some(v2::SBOL2_REFINEMENT_USE_LOCAL) => RefinementShape::UseLocal,
                Some(v2::SBOL2_REFINEMENT_USE_REMOTE) | Some(v2::SBOL2_REFINEMENT_MERGE) => {
                    RefinementShape::UseRemote
                }
                Some(v2::SBOL2_REFINEMENT_VERIFY_IDENTICAL) | None => {
                    RefinementShape::VerifyIdentical
                }
                Some(other) => {
                    self.report.push(UpgradeWarning::UnsupportedRefinement {
                        mapsto: mapsto_iri.clone(),
                        refinement: other.to_owned(),
                    });
                    RefinementShape::VerifyIdentical
                }
            };
            let (subject_iri, object_iri, restriction) = match refinement_kind {
                RefinementShape::UseLocal => (&local, &cref_iri, v3::SBOL_REPLACES),
                RefinementShape::UseRemote => (&cref_iri, &local, v3::SBOL_REPLACES),
                RefinementShape::VerifyIdentical => (&cref_iri, &local, v3::SBOL_VERIFY_IDENTICAL),
            };

            self.emit_constraint(
                &top_level,
                &constraint_iri,
                &constraint_display_id,
                subject_iri,
                object_iri,
                restriction,
            );
            self.report.counts.mapstos_decomposed += 1;
        }
    }

    pub(super) fn emit_component_reference(
        &mut self,
        top_level: &str,
        cref_iri: &str,
        display_id: &str,
        in_child_of: &str,
        refers_to: &str,
    ) {
        let cref_resource = Resource::Iri(Iri::new_unchecked(cref_iri.to_owned()));
        let top_resource = Resource::Iri(Iri::new_unchecked(top_level.to_owned()));

        self.output_triples.push(Triple {
            subject: cref_resource.clone(),
            predicate: Iri::from_static(v3::RDF_TYPE),
            object: Term::Resource(Resource::Iri(Iri::new_unchecked(
                v3::SBOL_COMPONENT_REFERENCE_CLASS,
            ))),
        });
        self.output_triples.push(Triple {
            subject: cref_resource.clone(),
            predicate: Iri::from_static(v3::SBOL_DISPLAY_ID),
            object: Term::Literal(sbol_rdf::Literal::simple(display_id)),
        });
        self.output_triples.push(Triple {
            subject: cref_resource.clone(),
            predicate: Iri::from_static(v3::SBOL_IN_CHILD_OF),
            object: Term::Resource(Resource::Iri(Iri::new_unchecked(in_child_of.to_owned()))),
        });
        self.output_triples.push(Triple {
            subject: cref_resource.clone(),
            predicate: Iri::from_static(v3::SBOL_REFERS_TO),
            object: Term::Resource(Resource::Iri(Iri::new_unchecked(refers_to.to_owned()))),
        });
        self.output_triples.push(Triple {
            subject: top_resource,
            predicate: Iri::from_static(v3::SBOL_HAS_FEATURE),
            object: Term::Resource(cref_resource),
        });
    }

    pub(super) fn emit_constraint(
        &mut self,
        top_level: &str,
        constraint_iri: &str,
        display_id: &str,
        subject_iri: &str,
        object_iri: &str,
        restriction: &'static str,
    ) {
        let constraint_resource = Resource::Iri(Iri::new_unchecked(constraint_iri.to_owned()));
        let top_resource = Resource::Iri(Iri::new_unchecked(top_level.to_owned()));

        self.output_triples.push(Triple {
            subject: constraint_resource.clone(),
            predicate: Iri::from_static(v3::RDF_TYPE),
            object: Term::Resource(Resource::Iri(Iri::new_unchecked(v3::SBOL_CONSTRAINT_CLASS))),
        });
        self.output_triples.push(Triple {
            subject: constraint_resource.clone(),
            predicate: Iri::from_static(v3::SBOL_DISPLAY_ID),
            object: Term::Literal(sbol_rdf::Literal::simple(display_id)),
        });
        self.output_triples.push(Triple {
            subject: constraint_resource.clone(),
            predicate: Iri::from_static(v3::SBOL_SUBJECT),
            object: Term::Resource(Resource::Iri(Iri::new_unchecked(subject_iri.to_owned()))),
        });
        self.output_triples.push(Triple {
            subject: constraint_resource.clone(),
            predicate: Iri::from_static(v3::SBOL_OBJECT),
            object: Term::Resource(Resource::Iri(Iri::new_unchecked(object_iri.to_owned()))),
        });
        self.output_triples.push(Triple {
            subject: constraint_resource.clone(),
            predicate: Iri::from_static(v3::SBOL_RESTRICTION),
            object: Term::Resource(Resource::Iri(Iri::new_unchecked(restriction.to_owned()))),
        });
        self.output_triples.push(Triple {
            subject: top_resource,
            predicate: Iri::from_static(v3::SBOL_HAS_CONSTRAINT),
            object: Term::Resource(constraint_resource),
        });
    }

    /// SBOL 3 requires every Range/Cut/EntireSequence to carry a
    /// `hasSequence` pointing at the Sequence it ranges over. SBOL 2
    /// expresses this implicitly via the parent
    /// (Range → SA → ComponentDefinition → Sequence). When the enclosing CD
    /// has exactly one sequence, we attach it; ambiguous cases (multiple
    /// sequences) are left untouched so validation can flag them.
    pub(super) fn infer_location_sequences(&mut self) {
        // Snapshot (location, sa) pairs up front so the mutations
        // below don't fight the iterator borrow.
        let pairs: Vec<(String, String)> = self
            .location_to_sa
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (sbol2_location, sa) in pairs {
            let Some(cd) = self.sa_to_cd.get(&sa).cloned() else {
                continue;
            };
            let sequences = self.cd_sequences.get(&cd).cloned().unwrap_or_default();
            if sequences.len() != 1 {
                self.report.push(UpgradeWarning::LocationWithoutSequence {
                    location: sbol2_location,
                    component: cd,
                    sequence_count: sequences.len(),
                });
                continue;
            }

            let location_canonical = self.identity.rewrite_iri(&sbol2_location).to_owned();
            let sequence_canonical = self.identity.rewrite_iri(&sequences[0]).to_owned();
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(location_canonical)),
                predicate: Iri::from_static(v3::SBOL_HAS_SEQUENCE),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(sequence_canonical))),
            });
            self.report.counts.locations_with_inferred_sequence += 1;
        }
    }

    pub(super) fn fallback_namespace(&mut self, sbol2_iri: &str) -> Option<String> {
        // URL-origin derivation is the canonical path for HTTP-style
        // IRIs; SynBioHub data takes it for every top-level. Emitting a
        // per-subject warning there drowns out the cases that genuinely
        // warrant attention (default-namespace coercion or no namespace
        // at all), so stay silent.
        if let Some(origin) = url_origin(sbol2_iri) {
            return Some(origin);
        }
        if let Some(default) = self.options.default_namespace.as_ref() {
            self.report.push(UpgradeWarning::NamespaceFallback {
                subject: sbol2_iri.to_owned(),
                source: NamespaceSource::DefaultOption,
            });
            return Some(default.as_str().to_owned());
        }
        self.report.push(UpgradeWarning::NamespaceFallback {
            subject: sbol2_iri.to_owned(),
            source: NamespaceSource::None,
        });
        None
    }
}
