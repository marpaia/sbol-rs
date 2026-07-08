//! Structural analysis: recovering MapsTo / Interface information and
//! classifying every SBOL 3 Component into its single SBOL 2 shape
//! (ComponentDefinition or ModuleDefinition).

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
                    // A non-directional interface feature restores to an
                    // SBOL 2 `inout` FunctionalComponent, matching the
                    // reference converter.
                    interfaces
                        .entry(subject)
                        .or_default()
                        .push((fc, FcDirection::Inout));
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
        // MapsTo.
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
            let restriction_matches = attrs
                .restriction
                .as_deref()
                .is_some_and(|r| r == v3::SBOL_VERIFY_IDENTICAL || r == v3::SBOL_REPLACES);
            if !restriction_matches {
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
            // The refinement is inferred from the Constraint's restriction and
            // the CRef's position (`sbol3:subject` vs `sbol3:object`).
            let refinement = restriction
                .as_deref()
                .and_then(|r| values::map_restriction_to_refinement(r, cref_position))
                .map(str::to_owned);

            let (Some(carrier_v3), Some(remote_v3), Some(display_id), Some(local_v3)) =
                (attrs.in_child_of, attrs.refers_to, attrs.display_id, local)
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
    pub(super) fn record_versioned(&mut self) {
        self.report.counts.identities_versioned += 1;
    }

    /// Decides each Component's [`ComponentShape`] (ComponentDefinition or
    /// ModuleDefinition) from its outgoing triples, and computes its SBOL 2
    /// IRI. Also indexes each SubComponent's enclosing parent so later passes
    /// can dispatch on the parent's shape.
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
                v3::SBOL_HAS_INTERACTION => {
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

        // A Component is a ModuleDefinition iff it has interactions or the
        // FunctionalEntity type (the base `functional` set), OR it contains a
        // SubComponent whose `instanceOf` is (recursively) a
        // ModuleDefinition. Expand the base set to a fixpoint over each
        // Component's subcomponent-instanceOf targets.
        let mut instance_of: HashMap<String, String> = HashMap::new();
        for triple in self.input.rdf_graph().triples() {
            if triple.predicate.as_str() != v3::SBOL_INSTANCE_OF {
                continue;
            }
            if let (Some(s), Some(o)) = (triple.subject.as_iri(), triple.object.as_iri()) {
                instance_of.insert(s.as_str().to_owned(), o.as_str().to_owned());
            }
        }
        let mut child_targets: HashMap<String, Vec<String>> = HashMap::new();
        for (child, parent) in &self.feature_parent {
            if type_set_contains(&sbol3_types, child, v3::SBOL_SUB_COMPONENT_CLASS)
                && let Some(target) = instance_of.get(child)
            {
                child_targets
                    .entry(parent.clone())
                    .or_default()
                    .push(target.clone());
            }
        }
        let mut module_definitions: HashSet<String> = functional.clone();
        loop {
            let mut changed = false;
            for comp in &component_iris {
                if module_definitions.contains(comp) {
                    continue;
                }
                if let Some(targets) = child_targets.get(comp)
                    && targets.iter().any(|t| module_definitions.contains(t))
                {
                    module_definitions.insert(comp.clone());
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        for component_iri in &component_iris {
            // Each SBOL 3 Component maps to exactly one SBOL 2 class: a
            // ModuleDefinition when it carries functional signals, otherwise
            // a ComponentDefinition. Its SBOL 2 IRI is the Component's own.
            let shape = if module_definitions.contains(component_iri) {
                ComponentShape::MdOnly
            } else {
                ComponentShape::CdOnly
            };
            self.component_splits.insert(
                component_iri.clone(),
                ComponentSplit {
                    shape,
                    cd_iri: component_iri.clone(),
                },
            );
        }
        let _ = (&structural, &display_ids);
    }
}
