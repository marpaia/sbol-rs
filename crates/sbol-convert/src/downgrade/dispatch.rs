//! Per-triple dispatch: routing each SBOL 3 triple through type and
//! predicate handling, including the dual-role and subcomponent-split
//! special cases.

use super::helpers::*;
use super::*;

impl<'a> Engine<'a> {
    pub(super) fn handle_triple(&mut self, triple: &Triple) {
        let predicate = triple.predicate.as_str();

        // Drop every triple whose subject is folded into a structural
        // re-synthesis: ComponentReferences become MapsTo (emit_mapsto),
        // MapsTo-shaped Constraints fold into the same MapsTo (so the
        // Constraint shell disappears), and Interfaces fold into per-FC
        // sbol2:direction triples (emit_fc_directions). Also drop any
        // subject the discovery phase couldn't fold (orphan or partial
        // CRef + Constraint pairs). Without this suppression the
        // original shells would survive into the SBOL 2 output as
        // orphan subjects carrying SBOL 3 predicates.
        if let Some(subject_iri) = triple.subject.as_iri() {
            let s = subject_iri.as_str();
            if self.mapsto_reconstructions.contains_key(s)
                || self.mapsto_constraints.contains(s)
                || self.interface_subjects.contains(s)
                || self.discarded_subjects.contains(s)
            {
                return;
            }
        }

        // Drop the parent-side pointer triples for the same reason:
        // `hasFeature` → CRef, `hasConstraint` → MapsTo-shaped Constraint,
        // `hasInterface` → Interface. The structural re-synthesis emits
        // the SBOL 2 equivalents itself.
        if let Some(object_iri) = triple.object.as_iri() {
            let o = object_iri.as_str();
            if (predicate.starts_with(v3::SBOL_NS) && self.discarded_subjects.contains(o))
                || (predicate == v3::SBOL_HAS_FEATURE
                    && (self.mapsto_reconstructions.contains_key(o)
                        || self.discarded_subjects.contains(o)))
                || (predicate == v3::SBOL_HAS_CONSTRAINT
                    && (self.mapsto_constraints.contains(o) || self.discarded_subjects.contains(o)))
                || (predicate == v3::SBOL_HAS_INTERFACE && self.interface_subjects.contains(o))
            {
                return;
            }
        }

        // Backport namespace: consumed (restoration happens via
        // emit_backport_metadata and the iri_rewrites map).
        if predicate == v2::BACKPORT_SBOL2_VERSION
            || predicate == v2::BACKPORT_SBOL2_PERSISTENT_IDENTITY
            || predicate == v2::BACKPORT_SBOL2_TYPE
            || predicate == v2::BACKPORT_BIOPAX_TYPE
            || predicate == v2::BACKPORT_SEQUENCE_ANNOTATION_DISPLAY_ID
            || predicate == v2::BACKPORT_MAPS_TO_REFINEMENT
            || predicate == v2::BACKPORT_MAPS_TO_DISPLAY_ID
            || predicate.starts_with(v2::BACKPORT_SBOL2_PREFIX)
            || predicate.starts_with(v2::BACKPORT_SEQUENCE_ANNOTATION_PREDICATE_PREFIX)
        {
            // `backport:sbol2_<predicate>` triples represent unrecognized
            // sbol2:* predicates that the upgrade preserved verbatim.
            // Restore them now under their original SBOL 2 IRI.
            if let Some(local) = predicate.strip_prefix(v2::BACKPORT_SBOL2_PREFIX) {
                let original = format!("{}{local}", v2::SBOL2_NS);
                self.output_triples.push(Triple {
                    subject: self.rewrite_resource(&triple.subject),
                    predicate: Iri::new_unchecked(original),
                    object: self.rewrite_term(&triple.object),
                });
            }
            return;
        }

        // dcterms:title / dcterms:description: passthrough (SBOL 2 also
        // uses Dublin Core metadata for these).
        if predicate == v2::DCTERMS_TITLE || predicate == v2::DCTERMS_DESCRIPTION {
            self.output_triples.push(self.rewrite_triple(triple));
            return;
        }

        // rdf:type → run through the type downgrade table.
        if predicate == v3::RDF_TYPE {
            self.handle_type_triple(triple);
            return;
        }

        // SBOL 3 predicates → rewrite to SBOL 2 equivalent.
        if predicate.starts_with(v3::SBOL_NS) {
            self.handle_sbol3_predicate(triple);
            return;
        }

        // Everything else (PROV, custom annotations, etc.) passes
        // through with IRI rewriting only.
        self.output_triples.push(self.rewrite_triple(triple));
    }

    pub(super) fn handle_type_triple(&mut self, triple: &Triple) {
        // Blank-node-typed subjects don't participate in any SBOL 3-specific
        // structural rewrite (no backport hints, no split classification),
        // so pass them through unchanged and skip the IRI-keyed lookups
        // below.
        let subject_iri = match triple.subject.as_iri() {
            Some(iri) => iri.as_str().to_owned(),
            None => {
                self.output_triples.push(self.rewrite_triple(triple));
                return;
            }
        };
        let object_iri = match triple.object.as_iri() {
            Some(iri) => iri.as_str(),
            None => {
                self.output_triples.push(self.rewrite_triple(triple));
                return;
            }
        };

        // Non-SBOL type assertions (PROV, extension classes, etc.) are
        // annotations on the subject, not part of the SBOL class conversion.
        // Preserve them even when the subject also carries a backport SBOL 2
        // type hint.
        if !object_iri.starts_with(v3::SBOL_NS) {
            self.output_triples.push(self.rewrite_triple(triple));
            return;
        }

        // SubComponents under a dual-role parent triple into three
        // SBOL 2 objects (Component, FunctionalComponent, Module). Emit
        // each variant's rdf:type here; the parent's `hasFeature`
        // emission handles the containment triples. Only consume the actual
        // SBOL 3 SubComponent class assertion; future SBOL namespace classes
        // should still flow to the unsupported/archive path below.
        if object_iri == v3::SBOL_SUB_COMPONENT_CLASS
            && let Some(split) = self.subcomponent_splits.get(&subject_iri).cloned()
        {
            self.emit_subcomponent_split_types(&split);
            self.report.counts.sub_components_emitted += 1;
            return;
        }

        // Dual-role Components emit BOTH a ComponentDefinition (on the
        // CD half) and a ModuleDefinition (on the MD half). Restrict this
        // to the real SBOL 3 Component class so extra rdf:type assertions do
        // not duplicate the split or disappear.
        if object_iri == v3::SBOL_COMPONENT_CLASS
            && let Some(split) = self.component_splits.get(&subject_iri).cloned()
            && split.shape == ComponentShape::DualRole
        {
            self.emit_component_split_types(&split, &subject_iri);
            return;
        }

        // Use the backport-recorded SBOL 2 type when available; it's
        // the authoritative signal for documents that came through
        // sbol-rs upgrade.
        let target = self.sbol2_type_for_subject_type(&subject_iri, object_iri);

        if let Some(sbol2_type) = target {
            self.output_triples.push(Triple {
                subject: self.rewrite_resource(&triple.subject),
                predicate: Iri::from_static(v3::RDF_TYPE),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(sbol2_type.clone()))),
            });
            match sbol2_type.as_str() {
                v2::SBOL2_COMPONENT_DEFINITION => {
                    self.report.counts.components_to_component_definition += 1;
                }
                v2::SBOL2_MODULE_DEFINITION => {
                    self.report.counts.components_to_module_definition += 1;
                }
                v2::SBOL2_COMPONENT | v2::SBOL2_FUNCTIONAL_COMPONENT | v2::SBOL2_MODULE => {
                    self.report.counts.sub_components_emitted += 1;
                }
                v2::SBOL2_SEQUENCE_ANNOTATION => {
                    self.report.counts.sequence_features_emitted += 1;
                }
                _ => {}
            }
            return;
        }

        // Unknown SBOL 3 type: surface as a warning and drop.
        self.report.push(DowngradeWarning::UnsupportedSbol3Type {
            subject: subject_iri,
            sbol3_type: object_iri.to_owned(),
        });
    }

    /// Emits the rdf:type triples for the CD and MD halves of a
    /// dual-role Component split, plus the synthesized linking
    /// FunctionalComponent (whose containment by the MD is emitted
    /// separately in [`emit_dual_role_components`]).
    pub(super) fn emit_component_split_types(&mut self, split: &ComponentSplit, sbol3_iri: &str) {
        let cd_v2 = self.rewrite_iri(&split.cd_iri).to_owned();
        let md_v2 = self.rewrite_iri(&split.md_iri).to_owned();
        self.output_triples.push(Triple {
            subject: Resource::Iri(Iri::new_unchecked(cd_v2)),
            predicate: Iri::from_static(v3::RDF_TYPE),
            object: Term::Resource(Resource::Iri(Iri::new_unchecked(
                v2::SBOL2_COMPONENT_DEFINITION,
            ))),
        });
        self.output_triples.push(Triple {
            subject: Resource::Iri(Iri::new_unchecked(md_v2)),
            predicate: Iri::from_static(v3::RDF_TYPE),
            object: Term::Resource(Resource::Iri(Iri::new_unchecked(
                v2::SBOL2_MODULE_DEFINITION,
            ))),
        });
        self.report.counts.components_split_into_both += 1;
        self.report.push(DowngradeWarning::DualRoleComponent {
            component: sbol3_iri.to_owned(),
            component_definition: split.cd_iri.clone(),
            module_definition: split.md_iri.clone(),
        });
    }

    /// Emits the rdf:type triples for each variant of a SubComponent
    /// triple-split. Existence of `module_iri` is gated by the
    /// SubComponent's target shape: only MD-shaped targets receive a
    /// `sbol2:Module` variant (a Module's `definition` must be an MD).
    pub(super) fn emit_subcomponent_split_types(&mut self, split: &SubComponentSplit) {
        let component_v2 = self.rewrite_iri(&split.component_iri).to_owned();
        self.output_triples.push(Triple {
            subject: Resource::Iri(Iri::new_unchecked(component_v2)),
            predicate: Iri::from_static(v3::RDF_TYPE),
            object: Term::Resource(Resource::Iri(Iri::new_unchecked(v2::SBOL2_COMPONENT))),
        });
        let fc_v2 = self.rewrite_iri(&split.functional_component_iri).to_owned();
        self.output_triples.push(Triple {
            subject: Resource::Iri(Iri::new_unchecked(fc_v2)),
            predicate: Iri::from_static(v3::RDF_TYPE),
            object: Term::Resource(Resource::Iri(Iri::new_unchecked(
                v2::SBOL2_FUNCTIONAL_COMPONENT,
            ))),
        });
        if let Some(module_iri) = &split.module_iri {
            let module_v2 = self.rewrite_iri(module_iri).to_owned();
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(module_v2)),
                predicate: Iri::from_static(v3::RDF_TYPE),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(v2::SBOL2_MODULE))),
            });
        }
    }

    /// Routes a single predicate triple whose subject is a SubComponent
    /// under a dual-role parent through its three SBOL 2 variants
    /// (`sbol2:Component`, `sbol2:FunctionalComponent`, `sbol2:Module`).
    /// `sbol3:instanceOf` becomes three `sbol2:definition` triples
    /// (each pointing at the target's CD or MD as appropriate);
    /// Identified properties land on every variant so each is
    /// individually conformant; predicates with no SBOL 2 analogue
    /// route to the FunctionalComponent variant by default.
    pub(super) fn handle_subcomponent_split_predicate(
        &mut self,
        triple: &Triple,
        split: &SubComponentSplit,
    ) {
        let predicate = triple.predicate.as_str();
        let component_v2 = self.rewrite_iri(&split.component_iri).to_owned();
        let fc_v2 = self.rewrite_iri(&split.functional_component_iri).to_owned();
        let module_v2 = split
            .module_iri
            .as_ref()
            .map(|m| self.rewrite_iri(m).to_owned());

        if predicate == v3::SBOL_INSTANCE_OF {
            let target_iri = match triple.object.as_iri() {
                Some(iri) => iri.as_str().to_owned(),
                None => return,
            };
            let (target_cd, target_md) = match self.component_splits.get(&target_iri) {
                Some(target_split) => (
                    self.rewrite_iri(&target_split.cd_iri).to_owned(),
                    self.rewrite_iri(&target_split.md_iri).to_owned(),
                ),
                // Target isn't a tracked Component split (rare: would
                // be a SubComponent whose target is somehow not the
                // SBOL 3 graph). Fall back to the rewritten target IRI
                // for all three definition triples.
                None => {
                    let target_v2 = self.rewrite_iri(&target_iri).to_owned();
                    (target_v2.clone(), target_v2)
                }
            };
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(component_v2)),
                predicate: Iri::from_static(v2::SBOL2_DEFINITION),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(target_cd.clone()))),
            });
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(fc_v2)),
                predicate: Iri::from_static(v2::SBOL2_DEFINITION),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(target_cd))),
            });
            if let Some(module_v2) = module_v2 {
                self.output_triples.push(Triple {
                    subject: Resource::Iri(Iri::new_unchecked(module_v2)),
                    predicate: Iri::from_static(v2::SBOL2_DEFINITION),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(target_md))),
                });
            }
            return;
        }

        // Identified properties go on every variant so each is
        // individually valid SBOL 2. The CD-side Component, MD-side
        // FunctionalComponent, and MD-side Module all need their own
        // displayId / name / description for SBOL 2 validators.
        // SBOL 2 IRI compliance (sbol-12302) requires `displayId` to
        // equal the last path segment of `persistentIdentity`. The
        // triple-split puts `_fc` / `_m` suffixes on the variant IRIs,
        // so each variant needs a displayId matching its own suffix.
        // The component variant keeps the bare displayId; the FC and
        // Module variants derive theirs from their IRI's last segment.
        if predicate == v3::SBOL_DISPLAY_ID {
            let component_did = last_segment(&split.component_iri).to_owned();
            let fc_did = last_segment(&split.functional_component_iri).to_owned();
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(component_v2)),
                predicate: Iri::from_static(v2::SBOL2_DISPLAY_ID),
                object: Term::Literal(sbol_rdf::Literal::simple(component_did)),
            });
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(fc_v2)),
                predicate: Iri::from_static(v2::SBOL2_DISPLAY_ID),
                object: Term::Literal(sbol_rdf::Literal::simple(fc_did)),
            });
            if let (Some(module_v2), Some(module_iri)) = (module_v2, split.module_iri.as_ref()) {
                let module_did = last_segment(module_iri).to_owned();
                self.output_triples.push(Triple {
                    subject: Resource::Iri(Iri::new_unchecked(module_v2)),
                    predicate: Iri::from_static(v2::SBOL2_DISPLAY_ID),
                    object: Term::Literal(sbol_rdf::Literal::simple(module_did)),
                });
            }
            return;
        }

        // `sbol3:name` / `sbol3:description` are human-readable labels;
        // they're not subject to the IRI-compliance rule and stay
        // identical across variants so each is independently
        // self-describing.
        if predicate == v3::SBOL_NAME || predicate == v3::SBOL_DESCRIPTION {
            let sbol2_pred = match predicate {
                v3::SBOL_NAME => v2::DCTERMS_TITLE,
                _ => v2::DCTERMS_DESCRIPTION,
            };
            let object = self.rewrite_term(&triple.object);
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(component_v2)),
                predicate: Iri::from_static(sbol2_pred),
                object: object.clone(),
            });
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(fc_v2)),
                predicate: Iri::from_static(sbol2_pred),
                object: object.clone(),
            });
            if let Some(module_v2) = module_v2 {
                self.output_triples.push(Triple {
                    subject: Resource::Iri(Iri::new_unchecked(module_v2)),
                    predicate: Iri::from_static(sbol2_pred),
                    object,
                });
            }
            return;
        }

        // `hasLocation` on a SubComponent is represented by a synthesized
        // SequenceAnnotation wrapper in SBOL 2, not a location directly on
        // the Component / FunctionalComponent / Module variants.
        if predicate == v3::SBOL_HAS_LOCATION
            && let Some(subject_iri) = triple.subject.as_iri()
            && self.sa_collapses.contains_key(subject_iri.as_str())
        {
            return;
        }

        // Everything else routes to the FunctionalComponent variant.
        // That's where SBOL 2 plumbing (measure, sourceLocation,
        // roleIntegration, …) most naturally lives on subcomponents.
        let object = self.rewrite_term(&triple.object);
        if let Some(renamed) = map_sbol3_predicate_to_sbol2(predicate) {
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(fc_v2)),
                predicate: Iri::from_static(renamed),
                object,
            });
        } else if predicate.starts_with(v3::SBOL_NS) && predicate != v3::SBOL_HAS_NAMESPACE {
            self.archive_unknown_sbol3_predicate(triple);
        }
    }

    /// Routes a single predicate triple whose subject is a dual-role
    /// Component to the CD half, the MD half, or both. Structural
    /// predicates (`sbol3:type` for biopax/SO, `sbol3:role`,
    /// `sbol3:hasSequence`, `sbol3:hasConstraint`) land on CD; functional
    /// ones (`sbol3:hasInteraction`, `sbol3:hasModel`) land on MD;
    /// Identified properties (`sbol3:displayId`, `sbol3:name`,
    /// `sbol3:description`) land on both with the appropriate
    /// `_component` / `_module` suffix appended to the displayId of the
    /// synthesized half. `sbol3:hasNamespace` and `sbol3:hasInterface`
    /// are dropped (the latter is recovered via Interface synthesis).
    pub(super) fn handle_dual_role_predicate(&mut self, triple: &Triple, split: &ComponentSplit) {
        let predicate = triple.predicate.as_str();
        let cd_v2 = self.rewrite_iri(&split.cd_iri).to_owned();
        let md_v2 = self.rewrite_iri(&split.md_iri).to_owned();
        let cd_subject = Resource::Iri(Iri::new_unchecked(cd_v2));
        let md_subject = Resource::Iri(Iri::new_unchecked(md_v2));
        let object = self.rewrite_term(&triple.object);

        match predicate {
            v3::SBOL_DISPLAY_ID => {
                // Derive each half's displayId from its IRI's last
                // segment rather than from `source + suffix`. This
                // matters when `next_available_iri` had to disambiguate
                // the half's IRI (e.g. `_component_2`); using the
                // source displayId + raw suffix would leave displayId
                // out of sync with the IRI's last segment and violate
                // SBOL 2 compliance (sbol-12302).
                let cd_did = last_segment(&split.cd_iri).to_owned();
                let md_did = last_segment(&split.md_iri).to_owned();
                self.output_triples.push(Triple {
                    subject: cd_subject,
                    predicate: Iri::from_static(v2::SBOL2_DISPLAY_ID),
                    object: Term::Literal(sbol_rdf::Literal::simple(cd_did)),
                });
                self.output_triples.push(Triple {
                    subject: md_subject,
                    predicate: Iri::from_static(v2::SBOL2_DISPLAY_ID),
                    object: Term::Literal(sbol_rdf::Literal::simple(md_did)),
                });
            }
            v3::SBOL_NAME => {
                self.output_triples.push(Triple {
                    subject: cd_subject,
                    predicate: Iri::from_static(v2::DCTERMS_TITLE),
                    object: object.clone(),
                });
                self.output_triples.push(Triple {
                    subject: md_subject,
                    predicate: Iri::from_static(v2::DCTERMS_TITLE),
                    object,
                });
            }
            v3::SBOL_DESCRIPTION => {
                self.output_triples.push(Triple {
                    subject: cd_subject,
                    predicate: Iri::from_static(v2::DCTERMS_DESCRIPTION),
                    object: object.clone(),
                });
                self.output_triples.push(Triple {
                    subject: md_subject,
                    predicate: Iri::from_static(v2::DCTERMS_DESCRIPTION),
                    object,
                });
            }
            v3::SBOL_TYPE => {
                // The SBO:0000241 (FunctionalEntity) marker is what the
                // upgrade synthesizes for MD-derived Components so the
                // SBOL 3 type cardinality holds; SBOL 2 MDs don't
                // express it, so drop it on the downgrade.
                if triple.object.as_iri().map(|i| i.as_str())
                    == Some("https://identifiers.org/SBO:0000241")
                {
                    return;
                }
                let subject_iri = triple.subject.as_iri().map(|i| i.as_str());
                let rewritten = self.reverse_value_for_subject(subject_iri, predicate, &object);
                self.output_triples.push(Triple {
                    subject: cd_subject,
                    predicate: Iri::from_static(v2::SBOL2_TYPE),
                    object: rewritten,
                });
            }
            v3::SBOL_ROLE => {
                self.output_triples.push(Triple {
                    subject: cd_subject,
                    predicate: Iri::from_static(v2::SBOL2_ROLE),
                    object,
                });
            }
            v3::SBOL_HAS_SEQUENCE => {
                self.output_triples.push(Triple {
                    subject: cd_subject,
                    predicate: Iri::from_static(v2::SBOL2_SEQUENCE_PROP),
                    object,
                });
            }
            v3::SBOL_HAS_CONSTRAINT => {
                self.output_triples.push(Triple {
                    subject: cd_subject,
                    predicate: Iri::from_static(v2::SBOL2_SEQUENCE_CONSTRAINT_PROP),
                    object,
                });
            }
            v3::SBOL_HAS_INTERACTION => {
                self.output_triples.push(Triple {
                    subject: md_subject,
                    predicate: Iri::from_static(v2::SBOL2_INTERACTION_PROP),
                    object,
                });
            }
            v3::SBOL_HAS_MODEL => {
                self.output_triples.push(Triple {
                    subject: md_subject,
                    predicate: Iri::from_static(v2::SBOL2_MODEL_PROP),
                    object,
                });
            }
            v3::SBOL_HAS_FEATURE => {
                self.handle_dual_role_has_feature(triple, split);
            }
            v3::SBOL_HAS_NAMESPACE | v3::SBOL_HAS_INTERFACE => {
                // namespace: dropped (implicit in restored persistentIdentity).
                // hasInterface: dropped (FCs receive direction triples
                // via `emit_fc_directions`).
            }
            _ => {
                // Anything else: rewrite the predicate via the default
                // table and emit on the half whose suffix is empty.
                // The half that kept the bare IRI matches the original
                // SBOL 2 source (CD or MD) when there is one, so
                // attaching the unclassified predicate there preserves
                // the most natural attribution for documents that came
                // through the upgrade pipeline. For natively-authored
                // SBOL 3, the bare half is whichever classify_components
                // picked as canonical, which gives a stable but
                // arbitrary default until each predicate is explicitly
                // routed.
                if let Some(renamed) = map_sbol3_predicate_to_sbol2(predicate) {
                    let primary_subject = if split.md_display_suffix.is_empty() {
                        Resource::Iri(Iri::new_unchecked(
                            self.rewrite_iri(&split.md_iri).to_owned(),
                        ))
                    } else {
                        Resource::Iri(Iri::new_unchecked(
                            self.rewrite_iri(&split.cd_iri).to_owned(),
                        ))
                    };
                    self.output_triples.push(Triple {
                        subject: primary_subject,
                        predicate: Iri::from_static(renamed),
                        object,
                    });
                } else if predicate.starts_with(v3::SBOL_NS) {
                    self.archive_unknown_sbol3_predicate(triple);
                }
            }
        }
    }

    /// `hasFeature` under a dual-role Component routes by feature type:
    /// SequenceFeatures become `sbol2:sequenceAnnotation` on the CD;
    /// SubComponents emit triple-variants (a `sbol2:component` on CD,
    /// `sbol2:functionalComponent` on MD, and `sbol2:module` on MD when
    /// the target is itself an MD).
    pub(super) fn handle_dual_role_has_feature(&mut self, triple: &Triple, split: &ComponentSplit) {
        let feature_iri = match triple.object.as_iri() {
            Some(iri) => iri.as_str().to_owned(),
            None => return,
        };
        let feature_type = self.resolved_types.get(&feature_iri).cloned();
        let cd_v2 = self.rewrite_iri(&split.cd_iri).to_owned();
        let md_v2 = self.rewrite_iri(&split.md_iri).to_owned();

        if feature_type.as_deref() == Some(v2::SBOL2_SEQUENCE_ANNOTATION) {
            let feature_v2 = self.rewrite_iri(&feature_iri).to_owned();
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(cd_v2)),
                predicate: Iri::from_static(v2::SBOL2_SEQUENCE_ANNOTATION_PROP),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(feature_v2))),
            });
            return;
        }

        if let Some(sc_split) = self.subcomponent_splits.get(&feature_iri).cloned() {
            let component_v2 = self.rewrite_iri(&sc_split.component_iri).to_owned();
            let fc_v2 = self
                .rewrite_iri(&sc_split.functional_component_iri)
                .to_owned();
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(cd_v2.clone())),
                predicate: Iri::from_static(v2::SBOL2_COMPONENT_PROP),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(component_v2))),
            });
            self.output_triples.push(Triple {
                subject: Resource::Iri(Iri::new_unchecked(md_v2.clone())),
                predicate: Iri::from_static(v2::SBOL2_FUNCTIONAL_COMPONENT_PROP),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(fc_v2.clone()))),
            });
            if let Some(module_iri) = &sc_split.module_iri {
                let module_v2 = self.rewrite_iri(module_iri).to_owned();
                self.output_triples.push(Triple {
                    subject: Resource::Iri(Iri::new_unchecked(md_v2)),
                    predicate: Iri::from_static(v2::SBOL2_MODULE_PROP),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(module_v2))),
                });
            }
            // Update participant_remap so any Participation referencing
            // this SubComponent rewrites to the FC variant; that's
            // where SBOL 2 expects `sbol2:participant` to point.
            self.participant_remap.insert(feature_iri, fc_v2);
            return;
        }

        // No subcomponent split (no MdOnly/DualRole target known): fall
        // back to emitting a single `sbol2:component` on the CD half
        // using the existing rewrite.
        let feature_v2 = self.rewrite_iri(&feature_iri).to_owned();
        self.output_triples.push(Triple {
            subject: Resource::Iri(Iri::new_unchecked(cd_v2)),
            predicate: Iri::from_static(v2::SBOL2_COMPONENT_PROP),
            object: Term::Resource(Resource::Iri(Iri::new_unchecked(feature_v2))),
        });
    }
}
