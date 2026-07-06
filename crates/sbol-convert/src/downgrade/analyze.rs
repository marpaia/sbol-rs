//! Structural analysis: recovering MapsTo / Interface information and
//! classifying every SBOL 3 Component into its SBOL 2 shape (CD, MD, or a
//! dual-role split).

use super::helpers::*;
use super::*;

impl<'a> Engine<'a> {
    /// Pairs every SBOL 3 ComponentReference with the Constraint that
    /// names it via `sbol3:object`, recovering the original SBOL 2
    /// MapsTo. Also indexes every Interface's
    /// `input` / `output` / `nondirectional` triples so the downgrade
    /// can re-emit per-FC `sbol2:direction`.
    pub(super) fn discover_mapsto_and_interfaces(&mut self) {
        #[derive(Default)]
        struct CrefAttrs {
            in_child_of: Option<String>,
            refers_to: Option<String>,
            display_id: Option<String>,
            /// Original `sbol2:refinement` IRI the upgrade preserved on
            /// the ComponentReference under `backport:mapsToRefinement`.
            /// When present this is both an authoritative signal that
            /// the paired Constraint is a MapsTo back-half AND the
            /// lossless source for the refinement value.
            backport_refinement: Option<String>,
            /// Original SBOL 2 MapsTo displayId, preserved only when the
            /// upgrade had to rename the ComponentReference to avoid an IRI
            /// collision under the enclosing Component.
            backport_display_id: Option<String>,
        }
        #[derive(Default)]
        struct ConstraintAttrs {
            subject: Option<String>,
            object: Option<String>,
            restriction: Option<String>,
        }

        let mut cref_attrs: HashMap<String, CrefAttrs> = HashMap::new();
        let mut constraint_attrs: HashMap<String, ConstraintAttrs> = HashMap::new();
        let mut subject_types: HashMap<String, HashSet<String>> = HashMap::new();
        let mut interfaces: HashMap<String, Vec<(String, FcDirection)>> = HashMap::new();

        for triple in self.input.rdf_graph().triples() {
            let subject = match triple.subject.as_iri() {
                Some(iri) => iri.as_str().to_owned(),
                None => continue,
            };
            let predicate = triple.predicate.as_str();
            let object_iri = triple.object.as_iri().map(|i| i.as_str().to_owned());
            let object_literal = triple.object.as_literal().map(|l| l.value().to_owned());

            if predicate == v3::RDF_TYPE {
                if let Some(obj) = &object_iri {
                    subject_types
                        .entry(subject.clone())
                        .or_default()
                        .insert(obj.clone());
                    if obj == v3::SBOL_INTERFACE_CLASS {
                        self.interface_subjects.insert(subject.clone());
                    }
                }
                continue;
            }
            if predicate == v3::SBOL_IN_CHILD_OF {
                if let Some(obj) = object_iri {
                    cref_attrs.entry(subject).or_default().in_child_of = Some(obj);
                }
                continue;
            }
            if predicate == v3::SBOL_REFERS_TO {
                if let Some(obj) = object_iri {
                    cref_attrs.entry(subject).or_default().refers_to = Some(obj);
                }
                continue;
            }
            if predicate == v3::SBOL_DISPLAY_ID {
                if let Some(lit) = object_literal {
                    cref_attrs.entry(subject).or_default().display_id = Some(lit);
                }
                continue;
            }
            if predicate == v2::BACKPORT_MAPS_TO_REFINEMENT {
                if let Some(obj) = object_iri {
                    cref_attrs.entry(subject).or_default().backport_refinement = Some(obj);
                }
                continue;
            }
            if predicate == v2::BACKPORT_MAPS_TO_DISPLAY_ID {
                if let Some(lit) = object_literal {
                    cref_attrs.entry(subject).or_default().backport_display_id = Some(lit);
                }
                continue;
            }
            if predicate == v3::SBOL_SUBJECT {
                if let Some(obj) = object_iri {
                    constraint_attrs.entry(subject).or_default().subject = Some(obj);
                }
                continue;
            }
            if predicate == v3::SBOL_OBJECT {
                if let Some(obj) = object_iri {
                    constraint_attrs.entry(subject).or_default().object = Some(obj);
                }
                continue;
            }
            if predicate == v3::SBOL_RESTRICTION {
                if let Some(obj) = object_iri {
                    constraint_attrs.entry(subject).or_default().restriction = Some(obj);
                }
                continue;
            }
            if predicate == v3::SBOL_INPUT {
                if let Some(fc) = object_iri {
                    interfaces
                        .entry(subject)
                        .or_default()
                        .push((fc, FcDirection::In));
                }
                continue;
            }
            if predicate == v3::SBOL_OUTPUT {
                if let Some(fc) = object_iri {
                    interfaces
                        .entry(subject)
                        .or_default()
                        .push((fc, FcDirection::Out));
                }
                continue;
            }
            if predicate == v3::SBOL_NONDIRECTIONAL {
                if let Some(fc) = object_iri {
                    interfaces
                        .entry(subject)
                        .or_default()
                        .push((fc, FcDirection::NoneDirection));
                }
                continue;
            }
        }

        // Pair each MapsTo-shaped Constraint with its ComponentReference.
        // Per SBOL 3.1.0 §10.2 the CRef can live in either `sbol3:subject`
        // (useRemote / verifyIdentical) or `sbol3:object` (useLocal) of
        // the Constraint, and the *other* position holds the local
        // SubComponent. The pairing is only considered a MapsTo back-half
        // when the Constraint's restriction is one of the two values the
        // forward map ever emits for this shape (`verifyIdentical` or
        // `replaces`). Without that filter a native SBOL 3 Constraint
        // that happened to point at a CRef with `precedes` (or any
        // structural restriction) would be silently folded into a fake
        // MapsTo. A `backport:mapsToRefinement` triple on the CRef is
        // the strongest possible signal and short-circuits the
        // restriction check.
        //
        // The CRef position determines which side of the Constraint
        // supplies the local SubComponent IRI; it's captured alongside
        // the constraint IRI so the downstream reconstruction can route
        // accordingly.
        let mut cref_to_constraint: HashMap<String, (String, values::CRefPosition)> =
            HashMap::new();
        for (constraint_iri, attrs) in &constraint_attrs {
            if !type_set_contains(&subject_types, constraint_iri, v3::SBOL_CONSTRAINT_CLASS) {
                continue;
            }
            let (cref, position) = match (attrs.subject.as_deref(), attrs.object.as_deref()) {
                (Some(s), _)
                    if type_set_contains(&subject_types, s, v3::SBOL_COMPONENT_REFERENCE_CLASS) =>
                {
                    (s.to_owned(), values::CRefPosition::Subject)
                }
                (_, Some(o))
                    if type_set_contains(&subject_types, o, v3::SBOL_COMPONENT_REFERENCE_CLASS) =>
                {
                    (o.to_owned(), values::CRefPosition::Object)
                }
                _ => continue,
            };
            let has_backport_refinement = cref_attrs
                .get(&cref)
                .and_then(|c| c.backport_refinement.as_deref())
                .is_some();
            let restriction_matches = attrs
                .restriction
                .as_deref()
                .is_some_and(|r| r == v3::SBOL_VERIFY_IDENTICAL || r == v3::SBOL_REPLACES);
            if !has_backport_refinement && !restriction_matches {
                continue;
            }
            cref_to_constraint.insert(cref, (constraint_iri.clone(), position));
        }

        for (cref_iri, attrs) in cref_attrs {
            if !type_set_contains(
                &subject_types,
                &cref_iri,
                v3::SBOL_COMPONENT_REFERENCE_CLASS,
            ) {
                continue;
            }
            let Some((constraint_iri, cref_position)) = cref_to_constraint.get(&cref_iri).cloned()
            else {
                // No paired Constraint; the CRef can't fold into a
                // MapsTo. Discard its triples so they don't survive as
                // an orphan SBOL 3 subject in the SBOL 2 output.
                self.report
                    .push(DowngradeWarning::OrphanComponentReference {
                        component_reference: cref_iri.clone(),
                    });
                self.discarded_subjects.insert(cref_iri);
                continue;
            };
            let constraint = constraint_attrs.get(&constraint_iri);
            // The CRef represents the `remote` side; the SubComponent
            // on the *other* position of the Constraint is the `local`.
            let local = match cref_position {
                values::CRefPosition::Subject => constraint.and_then(|c| c.object.clone()),
                values::CRefPosition::Object => constraint.and_then(|c| c.subject.clone()),
            };
            let restriction = constraint.and_then(|c| c.restriction.clone());
            // Prefer the explicit backport hint (lossless for the
            // useLocal/useRemote/merge family); fall back to
            // position-aware inference from the restriction.
            let refinement = attrs.backport_refinement.clone().or_else(|| {
                restriction
                    .as_deref()
                    .and_then(|r| values::map_restriction_to_refinement(r, cref_position))
                    .map(str::to_owned)
            });

            let display_id = attrs.backport_display_id.or(attrs.display_id);
            let (Some(carrier_v3), Some(remote_v3), Some(display_id), Some(local_v3)) =
                (attrs.in_child_of, attrs.refers_to, display_id, local)
            else {
                self.report
                    .push(DowngradeWarning::UnresolvableConstraintToMapsTo {
                        constraint: constraint_iri.clone(),
                        reason: "ComponentReference+Constraint pair was missing one of \
                                 in_child_of / refers_to / displayId / subject/object"
                            .to_string(),
                    });
                // Discard both shells so their triples don't survive as
                // orphan subjects in the SBOL 2 output.
                self.discarded_subjects.insert(cref_iri);
                self.discarded_subjects.insert(constraint_iri);
                continue;
            };

            self.mapsto_reconstructions.insert(
                cref_iri,
                MapsToReconstruction {
                    carrier_v3,
                    display_id,
                    local_v3,
                    remote_v3,
                    refinement,
                },
            );
            self.mapsto_constraints.insert(constraint_iri);
        }

        // Flatten the Interface index to per-FC direction.
        for (interface_iri, fcs) in interfaces {
            if !self.interface_subjects.contains(&interface_iri) {
                continue;
            }
            for (fc_iri, direction) in fcs {
                let merged = match (self.fc_directions.get(&fc_iri).copied(), direction) {
                    (Some(FcDirection::Inout), _) | (_, FcDirection::Inout) => FcDirection::Inout,
                    (Some(FcDirection::In), FcDirection::Out)
                    | (Some(FcDirection::Out), FcDirection::In) => FcDirection::Inout,
                    (Some(FcDirection::NoneDirection), d) => d,
                    (Some(existing), FcDirection::NoneDirection) => existing,
                    (None, d) => d,
                    (Some(existing), _) => existing,
                };
                self.fc_directions.insert(fc_iri, merged);
            }
        }
    }

    /// Records that a top-level whose version was preserved was used to
    /// restore an SBOL 2 identity. Bumps the counter for the
    /// `DowngradeCounts` summary.
    pub(super) fn record_restored(&mut self) {
        self.report.counts.identities_restored_from_backport += 1;
    }

    /// Decides each Component's [`ComponentShape`] from its outgoing
    /// triples plus any `backport:sbol2type` hint, then computes the
    /// IRIs and display-id suffixes both halves of a dual-role split
    /// will use. Also indexes each SubComponent's enclosing parent so
    /// later passes can dispatch on the parent's shape.
    pub(super) fn classify_components(&mut self) {
        // Index rdf:type of every SBOL 3 typed subject so we can tell
        // SubComponent / SequenceFeature / Component apart.
        let mut sbol3_types: HashMap<String, HashSet<String>> = HashMap::new();
        for triple in self.input.rdf_graph().triples() {
            if triple.predicate.as_str() != v3::RDF_TYPE {
                continue;
            }
            let (Some(subject), Some(object)) = (triple.subject.as_iri(), triple.object.as_iri())
            else {
                continue;
            };
            sbol3_types
                .entry(subject.as_str().to_owned())
                .or_default()
                .insert(object.as_str().to_owned());
        }
        let mut located_features: HashSet<String> = HashSet::new();
        for triple in self.input.rdf_graph().triples() {
            if triple.predicate.as_str() != v3::SBOL_HAS_LOCATION {
                continue;
            }
            if let Some(subject) = triple.subject.as_iri() {
                located_features.insert(subject.as_str().to_owned());
            }
        }

        // Index hasFeature parent-of-child so we can later route
        // SubComponent triples through their parent's split shape.
        for triple in self.input.rdf_graph().triples() {
            if triple.predicate.as_str() != v3::SBOL_HAS_FEATURE {
                continue;
            }
            let (Some(parent), Some(child)) = (triple.subject.as_iri(), triple.object.as_iri())
            else {
                continue;
            };
            self.feature_parent
                .insert(child.as_str().to_owned(), parent.as_str().to_owned());
        }

        // Scan each Component's outgoing triples for structural vs.
        // functional signals.
        let mut structural: HashSet<String> = HashSet::new();
        let mut functional: HashSet<String> = HashSet::new();
        let mut display_ids: HashMap<String, String> = HashMap::new();
        let component_iris: HashSet<String> = sbol3_types
            .iter()
            .filter(|(_, types)| types.contains(v3::SBOL_COMPONENT_CLASS))
            .map(|(iri, _)| iri.clone())
            .collect();

        for triple in self.input.rdf_graph().triples() {
            let subject = match triple.subject.as_iri() {
                Some(iri) => iri.as_str(),
                None => continue,
            };
            if !component_iris.contains(subject) {
                continue;
            }
            let predicate = triple.predicate.as_str();
            match predicate {
                v3::SBOL_DISPLAY_ID => {
                    if let Some(lit) = triple.object.as_literal() {
                        display_ids
                            .entry(subject.to_owned())
                            .or_insert(lit.value().to_owned());
                    }
                }
                v3::SBOL_ROLE | v3::SBOL_HAS_SEQUENCE => {
                    structural.insert(subject.to_owned());
                }
                v3::SBOL_HAS_CONSTRAINT => {
                    // A Constraint that's actually the back-half of a
                    // decomposed SBOL 2 MapsTo doesn't count as
                    // structural; it's pure functional plumbing the
                    // upgrade left in place. Discarded constraints
                    // (failed CRef + Constraint pair) also don't count:
                    // they won't survive into the SBOL 2 output, so
                    // attributing structural intent to their parent
                    // would mis-classify the Component.
                    let drop = triple
                        .object
                        .as_iri()
                        .map(|i| {
                            let s = i.as_str();
                            self.mapsto_constraints.contains(s)
                                || self.discarded_subjects.contains(s)
                        })
                        .unwrap_or(false);
                    if !drop {
                        structural.insert(subject.to_owned());
                    }
                }
                v3::SBOL_TYPE => {
                    // The upgrade synthesizes SBO:0000241 (FunctionalEntity)
                    // for every MD-derived Component so the SBOL 3 type
                    // cardinality holds. Treat that specific value as a
                    // functional signal; every other type IRI counts as
                    // structural.
                    match triple.object.as_iri().map(|i| i.as_str()) {
                        Some("https://identifiers.org/SBO:0000241") => {
                            functional.insert(subject.to_owned());
                        }
                        Some(_) => {
                            structural.insert(subject.to_owned());
                        }
                        None => {}
                    }
                }
                v3::SBOL_HAS_INTERACTION | v3::SBOL_HAS_INTERFACE | v3::SBOL_HAS_MODEL => {
                    functional.insert(subject.to_owned());
                }
                v3::SBOL_HAS_FEATURE => {
                    // SequenceFeature children are a structural signal.
                    // Located SubComponents are also structural because SBOL 2
                    // represents their locations through a ComponentDefinition
                    // SequenceAnnotation wrapper. Other SubComponent /
                    // LocalSubComponent / ExternallyDefined children are not
                    // (SBOL 2 ModuleDefinitions also carry SubComponents via
                    // `functionalComponent`).
                    // ComponentReferences that are the front-half of a
                    // MapsTo decomposition are functional plumbing the
                    // upgrade emitted, not real features.
                    if let Some(child) = triple.object.as_iri() {
                        let child_str = child.as_str();
                        if self.mapsto_reconstructions.contains_key(child_str) {
                            continue;
                        }
                        let is_sequence_feature = type_set_contains(
                            &sbol3_types,
                            child_str,
                            v3::SBOL_SEQUENCE_FEATURE_CLASS,
                        );
                        let is_located_subcomponent = located_features.contains(child_str)
                            && type_set_contains(
                                &sbol3_types,
                                child_str,
                                v3::SBOL_SUB_COMPONENT_CLASS,
                            );
                        if is_sequence_feature || is_located_subcomponent {
                            structural.insert(subject.to_owned());
                        }
                    }
                }
                _ => {}
            }
        }

        // Decide each Component's shape, then derive split IRIs.
        // A `backport:sbol2type` hint is authoritative: SBOL 2 sources
        // unambiguously chose one class or the other, so we honor that
        // choice even when the SBOL 3 surface carries triples that
        // could be read as the other shape (e.g. an SBOL 2
        // ModuleDefinition with a `sbol:role` triple, legal in SBOL 2,
        // but `role` is also a structural signal for native SBOL 3).
        // DualRole only fires when there's no SBOL 2 ancestor to
        // disambiguate.
        for component_iri in &component_iris {
            let backport = self.backport_types.get(component_iri).map(String::as_str);
            let has_structural = structural.contains(component_iri);
            let has_functional = functional.contains(component_iri);

            let shape = match backport {
                Some(v2::SBOL2_COMPONENT_DEFINITION) => ComponentShape::CdOnly,
                Some(v2::SBOL2_MODULE_DEFINITION) => ComponentShape::MdOnly,
                _ => {
                    if has_structural && has_functional && self.options.split_dual_role_components {
                        ComponentShape::DualRole
                    } else if has_functional {
                        ComponentShape::MdOnly
                    } else {
                        // Components with no signals default to CD.
                        // SBOL 2 ComponentDefinition is the more
                        // permissive class and matches the natural
                        // shape of structural-but-empty designs.
                        ComponentShape::CdOnly
                    }
                }
            };

            let (cd_suffix, md_suffix) = match shape {
                ComponentShape::CdOnly => ("", "_module"),
                ComponentShape::MdOnly => ("_component", ""),
                ComponentShape::DualRole => match backport {
                    Some(v2::SBOL2_MODULE_DEFINITION) => ("_component", ""),
                    Some(v2::SBOL2_COMPONENT_DEFINITION) => ("", "_module"),
                    _ => {
                        // No hint. sbolgraph heuristic: anything with
                        // interactions keeps the bare IRI on the MD;
                        // otherwise on the CD.
                        if has_functional {
                            ("_component", "")
                        } else {
                            ("", "_module")
                        }
                    }
                },
            };

            // The bare half (whichever has an empty suffix) keeps the
            // Component's original IRI. That IRI is already in
            // `used_iris` from the input-subject seed and represents
            // the Component's identity. The non-bare half is synthesized
            // by appending `_component` / `_module` directly; we route
            // it through the suffix allocator so any collision with an
            // existing subject (e.g. a separately-named Component at
            // `{X}_component`) picks up a `_2` / `_3` … disambiguation
            // tail instead of merging two distinct entities at one IRI.
            let cd_iri = if cd_suffix.is_empty() {
                component_iri.clone()
            } else {
                next_available_iri(&format!("{component_iri}{cd_suffix}"), &mut self.used_iris)
            };
            let md_iri = if md_suffix.is_empty() {
                component_iri.clone()
            } else {
                next_available_iri(&format!("{component_iri}{md_suffix}"), &mut self.used_iris)
            };

            let original_display_id = display_ids
                .get(component_iri)
                .cloned()
                .unwrap_or_else(|| last_segment(component_iri).to_owned());

            let (linking_fc_iri, linking_fc_display_id) = if shape == ComponentShape::DualRole {
                // The canonical linking-FC IRI is `{md_iri}/{displayId}`.
                // If anything already occupies that IRI (a SubComponent
                // that shares its parent's displayId is the canonical
                // case), pick the next available `{displayId}_N` so the
                // synthesized FC doesn't merge with existing triples.
                // That would put two contradictory rdf:types on the
                // same IRI.
                let (display_id, iri) =
                    next_available_child_iri(&md_iri, &original_display_id, &mut self.used_iris);
                (Some(iri), Some(display_id))
            } else {
                (None, None)
            };

            self.component_splits.insert(
                component_iri.clone(),
                ComponentSplit {
                    shape,
                    cd_iri,
                    md_iri,
                    linking_fc_iri,
                    linking_fc_display_id,
                    cd_display_suffix: cd_suffix,
                    md_display_suffix: md_suffix,
                    original_display_id,
                },
            );
        }

        // Pre-scan `sbol3:instanceOf` so SubComponent triple-emission
        // can decide whether a Module variant is needed (only when the
        // target is itself a Module-shaped Component).
        let mut instance_of: HashMap<String, String> = HashMap::new();
        for triple in self.input.rdf_graph().triples() {
            if triple.predicate.as_str() != v3::SBOL_INSTANCE_OF {
                continue;
            }
            let (Some(subject), Some(object)) = (triple.subject.as_iri(), triple.object.as_iri())
            else {
                continue;
            };
            instance_of.insert(subject.as_str().to_owned(), object.as_str().to_owned());
        }

        // Deterministic order so the disambiguation index lands
        // consistently across runs; HashMap iteration is unstable.
        let mut sc_parents: Vec<(String, String)> =
            self.feature_parent.clone().into_iter().collect();
        sc_parents.sort();

        // For each SubComponent under a DualRole parent, compute the
        // triple-variant IRIs. Non-bare variants (the ones carrying an
        // `_c` / `_fc` / `_m` suffix) go through
        // [`next_available_child_iri`] against the shared `used_iris`
        // set. Without this, a synthesized variant can land on top of
        // a sibling SubComponent whose displayId happens to match the
        // variant's suffix shape (e.g. siblings named `foo` and
        // `foo_fc` produce two SBOL 2 objects at the same IRI).
        for (sc_iri, parent_iri) in sc_parents {
            if !type_set_contains(&sbol3_types, &sc_iri, v3::SBOL_SUB_COMPONENT_CLASS) {
                continue;
            }
            let Some(parent_split) = self.component_splits.get(&parent_iri) else {
                continue;
            };
            if parent_split.shape != ComponentShape::DualRole {
                continue;
            }
            let backport = self.backport_types.get(&sc_iri).map(String::as_str);
            let (component_suffix, fc_suffix, module_suffix) = match backport {
                Some(v2::SBOL2_MODULE) => ("_c", "_fc", ""),
                Some(v2::SBOL2_FUNCTIONAL_COMPONENT) => ("_c", "", "_m"),
                // Default and `sbol2:Component`: the C variant keeps
                // the bare IRI; the MD-side FC and Module get suffixes.
                _ => ("", "_fc", "_m"),
            };

            let sc_did = last_segment(&sc_iri);
            // Allocates the IRI for a single variant of the split.
            // Empty-suffix variants reuse the SubComponent's input IRI
            // unchanged (it's the SubComponent's identity, already in
            // `used_iris`). Non-empty suffixes go through
            // [`next_available_child_iri`] under the SubComponent's
            // parent so any collision picks up a `_N` numeric tail
            // instead of merging onto an existing subject.
            let allocate_variant = |suffix: &str, used: &mut HashSet<String>| -> String {
                if suffix.is_empty() {
                    sc_iri.clone()
                } else {
                    let base = format!("{sc_did}{suffix}");
                    let (_did, iri) = next_available_child_iri(&parent_iri, &base, used);
                    iri
                }
            };

            let component_iri = allocate_variant(component_suffix, &mut self.used_iris);
            let functional_component_iri = allocate_variant(fc_suffix, &mut self.used_iris);
            let module_iri = instance_of.get(&sc_iri).and_then(|target| {
                let target_shape =
                    self.component_splits
                        .get(target)
                        .map(|s| s.shape)
                        .or_else(|| {
                            if self.backport_types.get(target).map(String::as_str)
                                == Some(v2::SBOL2_MODULE_DEFINITION)
                            {
                                Some(ComponentShape::MdOnly)
                            } else {
                                None
                            }
                        });
                match target_shape {
                    Some(ComponentShape::MdOnly) | Some(ComponentShape::DualRole) => {
                        Some(allocate_variant(module_suffix, &mut self.used_iris))
                    }
                    _ => None,
                }
            });

            self.subcomponent_splits.insert(
                sc_iri,
                SubComponentSplit {
                    component_iri,
                    functional_component_iri,
                    module_iri,
                },
            );
        }
    }
}
