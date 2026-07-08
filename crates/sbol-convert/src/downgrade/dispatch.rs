//! Per-triple dispatch: routing each SBOL 3 triple through type and
//! predicate handling, including the Component → ComponentDefinition /
//! ModuleDefinition classification and the SubComponent routing.

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

        // Backport annotations are conversion hints the preflight reads to
        // reconstruct the SBOL 2 form; they never appear in the SBOL 2 output.
        if predicate.starts_with(v2::BACKPORT_NS) {
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

    /// Emits `sbol2:access` on a Component / FunctionalComponent subject.
    fn emit_access(&mut self, subject: &Resource, access: &'static str) {
        self.output_triples.push(Triple {
            subject: subject.clone(),
            predicate: Iri::from_static(v2::SBOL2_ACCESS),
            object: Term::Resource(Resource::Iri(Iri::from_static(access))),
        });
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

        // Use the backport-recorded SBOL 2 type when available; it's
        // the authoritative signal for documents that came through
        // sbol-rs upgrade.
        let target = self.sbol2_type_for_subject_type(&subject_iri, object_iri);

        if let Some(sbol2_type) = target {
            let subject = self.rewrite_resource(&triple.subject);
            self.output_triples.push(Triple {
                subject: subject.clone(),
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
                v2::SBOL2_COMPONENT => {
                    self.report.counts.sub_components_emitted += 1;
                    // Every SBOL 2 Component carries an access; the reference
                    // assigns PUBLIC to CD subcomponents.
                    self.emit_access(&subject, v2::SBOL2_ACCESS_PUBLIC);
                }
                v2::SBOL2_FUNCTIONAL_COMPONENT => {
                    self.report.counts.sub_components_emitted += 1;
                    // A FunctionalComponent's access is PUBLIC when it belongs
                    // to its Component's Interface, otherwise PRIVATE.
                    let access = if triple
                        .subject
                        .as_iri()
                        .is_some_and(|i| self.fc_directions.contains_key(i.as_str()))
                    {
                        v2::SBOL2_ACCESS_PUBLIC
                    } else {
                        v2::SBOL2_ACCESS_PRIVATE
                    };
                    self.emit_access(&subject, access);
                }
                v2::SBOL2_MODULE => {
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
}
