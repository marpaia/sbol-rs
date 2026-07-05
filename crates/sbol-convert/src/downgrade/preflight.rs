//! Engine construction and the preflight analysis pass: building IRI
//! rewrites, discovering SequenceAnnotation collapses, and flagging
//! unsupported SBOL 3 subjects.

use super::helpers::*;
use super::*;

impl<'a> Engine<'a> {
    pub(super) fn new(input: &'a Document, options: DowngradeOptions) -> Self {
        Self {
            input,
            options,
            versions: HashMap::new(),
            preserved_versions: HashSet::new(),
            persistent_identities: HashMap::new(),
            backport_types: HashMap::new(),
            backport_biopax_types: HashMap::new(),
            biopax_variant_queue: HashMap::new(),
            biopax_variant_cursor: HashMap::new(),
            resolved_types: HashMap::new(),
            resolved_type_sets: HashMap::new(),
            subcomponent_targets: HashMap::new(),
            top_levels: HashSet::new(),
            sbol3_namespaces: HashMap::new(),
            iri_rewrites: HashMap::new(),
            sa_collapses: HashMap::new(),
            mapsto_reconstructions: HashMap::new(),
            mapsto_constraints: HashSet::new(),
            discarded_subjects: HashSet::new(),
            fc_directions: HashMap::new(),
            restored_fc_directions: HashSet::new(),
            interface_subjects: HashSet::new(),
            component_splits: HashMap::new(),
            subcomponent_splits: HashMap::new(),
            participant_remap: HashMap::new(),
            feature_parent: HashMap::new(),
            used_iris: HashSet::new(),
            dcterms_index: HashMap::new(),
            output_triples: Vec::new(),
            report: DowngradeReport::default(),
        }
    }

    /// First pass: read every triple to populate the IRI rewrite map,
    /// the version map, and the backport-type map. No output is
    /// produced yet.
    pub(super) fn preflight(&mut self) {
        // Seed the used-IRI pool with every input subject IRI. Every
        // synthesis site downstream (classify_components, the SA wrapper
        // discovery, the MapsTo reconstruction emission) routes its
        // candidate IRIs through `next_available_*` against this pool,
        // so no synthesized SBOL 2 IRI can silently land on an existing
        // subject, the invariant the pool exists to enforce.
        for triple in self.input.rdf_graph().triples() {
            if let Some(iri) = triple.subject.as_iri() {
                self.used_iris.insert(iri.as_str().to_owned());
            }
        }
        for triple in self.input.rdf_graph().triples() {
            let subject = match triple.subject.as_iri() {
                Some(iri) => iri.as_str().to_owned(),
                None => continue,
            };
            match triple.predicate.as_str() {
                v2::BACKPORT_SBOL2_VERSION => {
                    if let Some(lit) = triple.object.as_literal() {
                        self.versions
                            .insert(subject.clone(), lit.value().to_owned());
                        self.preserved_versions.insert(subject.clone());
                    }
                }
                v2::BACKPORT_SBOL2_PERSISTENT_IDENTITY => {
                    if let Some(iri) = triple.object.as_iri() {
                        self.persistent_identities
                            .insert(subject.clone(), iri.as_str().to_owned());
                    }
                }
                v2::BACKPORT_SBOL2_TYPE => {
                    if let Some(iri) = triple.object.as_iri() {
                        self.backport_types
                            .insert(subject.clone(), iri.as_str().to_owned());
                    }
                }
                v2::BACKPORT_BIOPAX_TYPE => {
                    if let Some(iri) = triple.object.as_iri() {
                        self.backport_biopax_types
                            .entry(subject.clone())
                            .or_default()
                            .insert(iri.as_str().to_owned());
                    }
                }
                v2::BACKPORT_SBOL2_DIRECTION => {
                    self.restored_fc_directions.insert(subject.clone());
                }
                v3::SBOL_HAS_NAMESPACE => {
                    if let Some(iri) = triple.object.as_iri() {
                        self.sbol3_namespaces
                            .insert(subject.clone(), iri.as_str().to_owned());
                    }
                }
                v2::DCTERMS_TITLE => {
                    self.dcterms_index
                        .entry((subject.clone(), v2::DCTERMS_TITLE))
                        .or_default()
                        .insert(triple.object.clone());
                }
                v2::DCTERMS_DESCRIPTION => {
                    self.dcterms_index
                        .entry((subject.clone(), v2::DCTERMS_DESCRIPTION))
                        .or_default()
                        .insert(triple.object.clone());
                }
                _ => {}
            }
        }

        // Identify top-level subjects from typed accessors so we know
        // which IRIs need a version suffix.
        for component in self.input.components() {
            if let Some(iri) = component.identity.as_iri() {
                self.top_levels.insert(iri.as_str().to_owned());
            }
        }
        for sequence in self.input.sequences() {
            if let Some(iri) = sequence.identity.as_iri() {
                self.top_levels.insert(iri.as_str().to_owned());
            }
        }
        for collection in self.input.collections() {
            if let Some(iri) = collection.identity.as_iri() {
                self.top_levels.insert(iri.as_str().to_owned());
            }
        }
        for model in self.input.models() {
            if let Some(iri) = model.identity.as_iri() {
                self.top_levels.insert(iri.as_str().to_owned());
            }
        }
        for implementation in self.input.implementations() {
            if let Some(iri) = implementation.identity.as_iri() {
                self.top_levels.insert(iri.as_str().to_owned());
            }
        }
        for attachment in self.input.attachments() {
            if let Some(iri) = attachment.identity.as_iri() {
                self.top_levels.insert(iri.as_str().to_owned());
            }
        }
        for experiment in self.input.experiments() {
            if let Some(iri) = experiment.identity.as_iri() {
                self.top_levels.insert(iri.as_str().to_owned());
            }
        }
        for experimental_data in self.input.experimental_data() {
            if let Some(iri) = experimental_data.identity.as_iri() {
                self.top_levels.insert(iri.as_str().to_owned());
            }
        }
        for cd in self.input.combinatorial_derivations() {
            if let Some(iri) = cd.identity.as_iri() {
                self.top_levels.insert(iri.as_str().to_owned());
            }
        }
        for activity in self.input.activities() {
            if let Some(iri) = activity.identity.as_iri() {
                self.top_levels.insert(iri.as_str().to_owned());
            }
        }
        for agent in self.input.agents() {
            if let Some(iri) = agent.identity.as_iri() {
                self.top_levels.insert(iri.as_str().to_owned());
            }
        }
        for plan in self.input.plans() {
            if let Some(iri) = plan.identity.as_iri() {
                self.top_levels.insert(iri.as_str().to_owned());
            }
        }

        // `sbol3:hasNamespace` is carried by (and only by) SBOL 3 top-level
        // objects. Custom-typed GenericTopLevels have no typed accessor above,
        // so pick them up here: any subject with `hasNamespace` is a
        // top-level and needs its SBOL 2 identity (persistent identity /
        // version) restored.
        for subject in self.sbol3_namespaces.keys().cloned().collect::<Vec<_>>() {
            self.top_levels.insert(subject);
        }

        // Unknown future SBOL 2 classes are archived by the upgrade as a
        // backport type but have no SBOL 3 class triple. Treat those
        // backport-only root subjects as top-levels so identity restoration
        // and metadata emission can still round-trip them.
        let mut sbol3_typed_subjects: HashSet<String> = HashSet::new();
        for triple in self.input.rdf_graph().triples() {
            if triple.predicate.as_str() != v3::RDF_TYPE {
                continue;
            }
            let (Some(subject), Some(object)) = (triple.subject.as_iri(), triple.object.as_iri())
            else {
                continue;
            };
            if object.as_str().starts_with(v3::SBOL_NS) {
                sbol3_typed_subjects.insert(subject.as_str().to_owned());
            }
        }
        let mut backport_only_subjects: Vec<String> = self
            .backport_types
            .keys()
            .filter(|subject| !sbol3_typed_subjects.contains(*subject))
            .cloned()
            .collect();
        backport_only_subjects.sort();
        for subject in backport_only_subjects {
            let is_known_child = self.top_levels.iter().any(|top| {
                let prefix = format!("{top}/");
                subject.starts_with(&prefix)
            });
            if !is_known_child {
                self.top_levels.insert(subject);
            }
        }

        // Precompute the per-(subject, sbo_term) ordering of preserved
        // BioPAX variants. Lets the downgrade resolve each `sbol3:type`
        // triple to a distinct variant when multiple were preserved for
        // the same SBO target (e.g. a Component carrying both
        // `biopax:Dna` and `biopax:DnaRegion`, both of which collapsed
        // to `SBO:0000251` on upgrade).
        for (subject, variants) in &self.backport_biopax_types {
            let mut by_sbo: HashMap<String, Vec<String>> = HashMap::new();
            for variant in variants {
                if let Some(sbo) = values::sbo_for_biopax(variant) {
                    by_sbo
                        .entry(sbo.to_owned())
                        .or_default()
                        .push(variant.clone());
                }
            }
            for (sbo, mut group) in by_sbo {
                // Sort for deterministic round-trips; HashSet's iteration
                // order is unstable.
                group.sort();
                self.biopax_variant_queue
                    .insert((subject.clone(), sbo), group);
            }
        }

        // MapsTo + Interface decompositions: detect ComponentReference /
        // Constraint pairs the upgrade emitted in place of SBOL 2 MapsTo,
        // and Interface subjects emitted in place of FunctionalComponent
        // directions. Both get re-synthesized later from these maps.
        self.discover_mapsto_and_interfaces();

        // Classify each Component into CD-only / MD-only / dual-role and
        // compute the split IRIs. Must precede `build_iri_rewrites`
        // because dual-role Components need version suffixes appended to
        // BOTH the CD and MD halves' IRIs, not just the original.
        self.classify_components();

        // SA-with-component collapses and native located SubComponents:
        // gather everything the emit_sa_wrappers pass needs to rebuild or
        // synthesize the SBOL 2 SA shell. Must precede `build_iri_rewrites`
        // because Pass 3 of that routine overrides each collapsed Location's
        // SBOL 2 IRI from the persistentIdentity it carries.
        self.discover_sa_collapses();

        // Drop SBOL 3-only concrete classes that have no SBOL 2 equivalent
        // before we build identity rewrites. Otherwise their child-like IRIs
        // can be versioned and later receive SBOL 2 identity metadata.
        self.discover_unsupported_sbol3_subjects();

        // Build the IRI rewrite map: every IRI that needs a version
        // suffix gets one. Subjects come from `top_levels`; non
        // top-level objects (SubComponent, Range, SequenceFeature,
        // etc.) inherit their parent's version, computed below.
        self.build_iri_rewrites();

        // Surface identity collisions: two or more distinct SBOL 3
        // input subjects whose `iri_rewrites` rewrite to the same
        // SBOL 2 versioned IRI. The conversion still runs and the
        // merged subject keeps every input triple, but the result is a
        // chimeric SBOL 2 entity that almost certainly isn't intended;
        // the warning lets callers audit. Mirrors the
        // `UpgradeWarning::IdentityCollision` detection on the inverse
        // direction. Note: identity rewrites between an unversioned
        // SBOL 3 input subject and its own versioned form (the bare-CD
        // round-trip case) don't collide because each input IRI is a
        // distinct map key and only those keys whose values overlap
        // are flagged.
        let mut canonical_sources: HashMap<String, Vec<String>> = HashMap::new();
        for (sbol3_iri, sbol2_iri) in &self.iri_rewrites {
            canonical_sources
                .entry(sbol2_iri.clone())
                .or_default()
                .push(sbol3_iri.clone());
        }
        let mut collisions: Vec<(String, Vec<String>)> = canonical_sources
            .into_iter()
            .filter(|(_, sources)| sources.len() > 1)
            .collect();
        collisions.sort_by(|a, b| a.0.cmp(&b.0));
        for (canonical, mut sources) in collisions {
            sources.sort();
            self.report
                .push(DowngradeWarning::IdentityCollision { canonical, sources });
        }

        // Resolve each typed subject's SBOL 2 type, combining the
        // backport-recorded type (authoritative) with the default
        // SBOL 3 → SBOL 2 table. Also capture each SubComponent's
        // `instanceOf` target so the hasFeature dispatch can
        // distinguish Module (target is MD) from FunctionalComponent
        // (target is CD).
        for triple in self.input.rdf_graph().triples() {
            if triple.predicate.as_str() != v3::SBOL_INSTANCE_OF {
                continue;
            }
            let (Some(subject), Some(object)) = (triple.subject.as_iri(), triple.object.as_iri())
            else {
                continue;
            };
            self.subcomponent_targets
                .insert(subject.as_str().to_owned(), object.as_str().to_owned());
        }
        for triple in self.input.rdf_graph().triples() {
            if triple.predicate.as_str() != v3::RDF_TYPE {
                continue;
            }
            let (Some(subject), Some(object)) = (triple.subject.as_iri(), triple.object.as_iri())
            else {
                continue;
            };
            let subject_iri = subject.as_str().to_owned();
            if let Some(resolved) = self.sbol2_type_for_subject_type(&subject_iri, object.as_str())
            {
                self.resolved_type_sets
                    .entry(subject_iri.clone())
                    .or_default()
                    .insert(resolved.clone());
                self.resolved_types.entry(subject_iri).or_insert(resolved);
            }
        }
    }

    pub(super) fn build_iri_rewrites(&mut self) {
        // For every IRI that appears as a subject in the document,
        // compute its SBOL 2 form. The version segment is only appended
        // when the source carried a `backport:sbol2version` triple.
        // Synthesizing one for documents that had no version originally
        // would pollute round-trips. Top-levels without a preserved
        // version still receive an identity rewrite so
        // `emit_backport_metadata` knows to emit `persistentIdentity`.
        let mut subjects: HashSet<String> = HashSet::new();
        for triple in self.input.rdf_graph().triples() {
            if let Some(iri) = triple.subject.as_iri() {
                let iri = iri.as_str();
                if !self.discarded_subjects.contains(iri) {
                    subjects.insert(iri.to_owned());
                }
            }
        }

        // Pass 1: top-level subjects.
        for iri in &subjects {
            if !self.top_levels.contains(iri) {
                continue;
            }
            let new = match self.preserved_version_for_top_level(iri) {
                Some(v) => {
                    self.record_restored();
                    append_segment(iri, &v)
                }
                None => match self.options.default_version.clone() {
                    Some(v) => {
                        self.note_synthesized(iri, &v);
                        append_segment(iri, &v)
                    }
                    None => iri.clone(),
                },
            };
            self.iri_rewrites.insert(iri.clone(), new);
        }

        // Pass 2: child subjects (anything whose IRI begins with a
        // top-level IRI's prefix + `/`). Inherit the parent's
        // effective version (preserved or synthesized).
        let top_level_iris: Vec<String> = self.top_levels.iter().cloned().collect();
        let mut top_levels: Vec<(String, Option<String>)> = top_level_iris
            .into_iter()
            .map(|tl| {
                let v = self.effective_version_for_top_level(&tl);
                (tl, v)
            })
            .collect();
        top_levels.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
        for iri in &subjects {
            if self.iri_rewrites.contains_key(iri) {
                continue;
            }
            for (top_iri, version) in &top_levels {
                let prefix = format!("{top_iri}/");
                if iri.starts_with(&prefix) && iri != top_iri {
                    let new = match version {
                        Some(v) => append_segment(iri, v),
                        None => iri.clone(),
                    };
                    self.iri_rewrites.insert(iri.clone(), new);
                    break;
                }
            }
        }

        // Pass 3: SA-collapsed Locations. The upgrade moved each Location
        // to live under the SubComponent at `{SubComp}/{lastSegmentOfOrig}`,
        // but the original SBOL 2 IRI was `{SA}/{LocDisplayId}`. Reconstruct
        // the SBOL 2 IRI from `backport:sbol2persistentIdentity` so the
        // re-upgrade (which sees a real SA wrapper this time) collapses
        // each Location back to the SAME SubComponent-relative IRI the
        // original SBOL 3 had. Without this override the round-trip would
        // emit Locations at a doubly-versioned IRI like `.../component1/1/1`.
        let location_overrides: Vec<(String, String)> = self
            .sa_collapses
            .values()
            .flat_map(|info| info.locations.clone())
            .filter_map(|loc| {
                let pid = self.persistent_identities.get(&loc).cloned()?;
                Some((loc, pid))
            })
            .collect();
        for (loc, pid) in location_overrides {
            let new = match self.effective_version_for_iri(&loc) {
                Some(version) => append_segment(&pid, &version),
                None => pid,
            };
            self.iri_rewrites.insert(loc, new);
        }

        // Dual-role split halves and SubComponent triple-variants:
        // each non-bare half needs an iri_rewrites entry mapping its
        // unversioned IRI to the versioned form so emission can use a
        // single `self.rewrite_iri(&split.cd_iri)` lookup. The bare
        // half already has its rewrite from Pass 1 / Pass 2 via the
        // original SBOL 3 IRI.
        let split_iris: Vec<(String, Option<String>)> = self
            .component_splits
            .iter()
            .filter(|(_, split)| split.shape == ComponentShape::DualRole)
            .flat_map(|(parent_iri, split)| {
                let parent_version = self.effective_version_for_top_level(parent_iri);
                [
                    (split.cd_iri.clone(), parent_version.clone()),
                    (split.md_iri.clone(), parent_version.clone()),
                ]
            })
            .chain(self.subcomponent_splits.iter().flat_map(|(sc_iri, split)| {
                let version = self.effective_version_for_iri(sc_iri);
                let mut out = vec![
                    (split.component_iri.clone(), version.clone()),
                    (split.functional_component_iri.clone(), version.clone()),
                ];
                if let Some(module_iri) = split.module_iri.clone() {
                    out.push((module_iri, version.clone()));
                }
                out
            }))
            .collect();
        for (iri, version) in split_iris {
            if self.iri_rewrites.contains_key(&iri) {
                continue;
            }
            let new = match version {
                Some(v) => append_segment(&iri, &v),
                None => iri.clone(),
            };
            self.iri_rewrites.insert(iri, new);
        }

        // Linking FunctionalComponent IRIs for each DualRole split.
        let linking_fcs: Vec<(String, Option<String>)> = self
            .component_splits
            .iter()
            .filter_map(|(parent_iri, split)| {
                let fc_iri = split.linking_fc_iri.clone()?;
                let version = self.effective_version_for_top_level(parent_iri);
                Some((fc_iri, version))
            })
            .collect();
        for (iri, version) in linking_fcs {
            if self.iri_rewrites.contains_key(&iri) {
                continue;
            }
            let new = match version {
                Some(v) => append_segment(&iri, &v),
                None => iri.clone(),
            };
            self.iri_rewrites.insert(iri, new);
        }
    }

    /// Returns `Some(version)` when this top-level subject was given a
    /// `backport:sbol2version` in the source, i.e. the SBOL 2 source
    /// carried an explicit version. Otherwise `None`, signalling that
    /// the version was not preserved and the caller must decide whether
    /// to synthesize one from [`DowngradeOptions::default_version`].
    pub(super) fn preserved_version_for_top_level(&self, iri: &str) -> Option<String> {
        if self.preserved_versions.contains(iri) {
            return self.versions.get(iri).cloned();
        }
        None
    }

    /// Returns the version that should be applied to this top-level:
    /// the preserved one if present, the synthesized fallback if
    /// [`DowngradeOptions::default_version`] is set, otherwise `None`
    /// (leave the subject unversioned). SBOL 2 makes `sbol2:version`
    /// optional, so `None` produces a valid document.
    pub(super) fn effective_version_for_top_level(&self, iri: &str) -> Option<String> {
        self.preserved_version_for_top_level(iri)
            .or_else(|| self.options.default_version.clone())
    }

    /// Returns the version for any subject (top-level or child)
    /// resolved via the owning top-level's classification.
    pub(super) fn effective_version_for_iri(&self, iri: &str) -> Option<String> {
        if self.top_levels.contains(iri) {
            return self.effective_version_for_top_level(iri);
        }
        self.owning_top_level_of(iri)
            .and_then(|top| self.effective_version_for_top_level(&top))
            .or_else(|| self.options.default_version.clone())
    }

    pub(super) fn note_synthesized(&mut self, iri: &str, version: &str) {
        self.report.counts.identities_synthesized += 1;
        self.report.push(DowngradeWarning::SynthesizedVersion {
            subject: iri.to_owned(),
            version: version.to_owned(),
        });
    }

    /// Detects every SubComponent that needs an SBOL 2 SequenceAnnotation
    /// wrapper and records what the downgrade needs to emit it.
    ///
    /// Round-tripped SBOL 2 inputs carry the
    /// `backport:sequenceAnnotationDisplayId` triple that the upgrade writes
    /// when collapsing an SA that referenced a `sbol2:component`. Native SBOL
    /// 3 inputs do not have that hint, but any `SubComponent` with
    /// `hasLocation` still needs the same SBOL 2 wrapper because `sbol2:Component`
    /// itself cannot carry locations.
    pub(super) fn discover_sa_collapses(&mut self) {
        let parent_of = self.feature_parent.clone();
        let mut locations_of: HashMap<String, Vec<String>> = HashMap::new();
        let mut display_ids: HashMap<String, String> = HashMap::new();
        let mut backport_sa_display_ids: HashMap<String, String> = HashMap::new();
        let mut subcomponents: HashSet<String> = HashSet::new();
        let mut preserved_metadata: HashMap<String, Vec<PreservedSaTriple>> = HashMap::new();

        for triple in self.input.rdf_graph().triples() {
            let Some(subject) = triple.subject.as_iri() else {
                continue;
            };
            let subject = subject.as_str().to_owned();

            if let Some(encoded) = triple
                .predicate
                .as_str()
                .strip_prefix(v2::BACKPORT_SEQUENCE_ANNOTATION_PREDICATE_PREFIX)
            {
                if let Some(predicate) = hex_decode_to_string(encoded) {
                    preserved_metadata.entry(subject.clone()).or_default().push(
                        PreservedSaTriple {
                            predicate,
                            object: triple.object.clone(),
                        },
                    );
                }
                continue;
            }

            match triple.predicate.as_str() {
                v3::RDF_TYPE => {
                    if triple.object.as_iri().map(|i| i.as_str())
                        == Some(v3::SBOL_SUB_COMPONENT_CLASS)
                    {
                        subcomponents.insert(subject);
                    }
                }
                v3::SBOL_DISPLAY_ID => {
                    if let Some(lit) = triple.object.as_literal() {
                        display_ids.entry(subject).or_insert(lit.value().to_owned());
                    }
                }
                v3::SBOL_HAS_LOCATION => {
                    if let Some(target) = triple.object.as_iri() {
                        locations_of
                            .entry(subject)
                            .or_default()
                            .push(target.as_str().to_owned());
                    }
                }
                v2::BACKPORT_SEQUENCE_ANNOTATION_DISPLAY_ID => {
                    if let Some(lit) = triple.object.as_literal() {
                        backport_sa_display_ids
                            .entry(subject)
                            .or_insert(lit.value().to_owned());
                    }
                }
                _ => {}
            }
        }

        // Engine-level `used_iris` already contains every input subject
        // (seeded at the top of preflight) and every split-half /
        // linking-FC / SubComponent variant IRI (inserted by
        // `classify_components`, which runs before this pass). No
        // duplicate bookkeeping needed.

        let mut candidates: HashSet<String> = backport_sa_display_ids.keys().cloned().collect();
        candidates.extend(preserved_metadata.keys().cloned());
        candidates.extend(
            locations_of
                .keys()
                .filter(|subject| subcomponents.contains(*subject))
                .cloned(),
        );
        let mut candidates: Vec<String> = candidates.into_iter().collect();
        candidates.sort();

        for subcomp in candidates {
            let Some(parent_component) = parent_of.get(&subcomp).cloned() else {
                continue;
            };
            let parent_cd = self
                .component_splits
                .get(&parent_component)
                .map(|split| split.cd_iri.clone())
                .unwrap_or_else(|| parent_component.clone());
            // Both the backport-preserved and the native-synthesis
            // paths route through `next_available_child_iri` against
            // the shared used-IRI pool. The backport-preserved case
            // *almost* always lands on its canonical
            // `{parent_cd}/{displayId}` IRI on the first try (the
            // upgrade wrote that IRI and the downgrade has now seen
            // it as an input subject). But if the original SA shared
            // a displayId with some other child of the parent CD, the
            // allocator will disambiguate with a `_N` suffix and we
            // honor that here rather than silently overwriting.
            let base = match backport_sa_display_ids.get(&subcomp) {
                Some(display_id) => display_id.clone(),
                None => display_ids
                    .get(&subcomp)
                    .map(|display_id| format!("{display_id}_annotation"))
                    .unwrap_or_else(|| format!("{}_annotation", last_segment(&subcomp))),
            };
            let (sa_display_id, sa_iri_unversioned) =
                next_available_child_iri(&parent_cd, &base, &mut self.used_iris);
            let mut locations = locations_of.get(&subcomp).cloned().unwrap_or_default();
            locations.sort();
            let mut metadata = preserved_metadata.remove(&subcomp).unwrap_or_default();
            metadata.sort_by(|a, b| {
                a.predicate
                    .cmp(&b.predicate)
                    .then_with(|| canonical_term_key(&a.object).cmp(&canonical_term_key(&b.object)))
            });
            self.sa_collapses.insert(
                subcomp,
                SaCollapseInfo {
                    sa_display_id,
                    sa_iri_unversioned,
                    parent_component,
                    parent_cd,
                    locations,
                    preserved_metadata: metadata,
                },
            );
        }
    }

    pub(super) fn discover_unsupported_sbol3_subjects(&mut self) {
        let mut unsupported: HashMap<String, Vec<String>> = HashMap::new();
        let mut supported: HashSet<String> = HashSet::new();

        for triple in self.input.rdf_graph().triples() {
            if triple.predicate.as_str() != v3::RDF_TYPE {
                continue;
            }
            let (Some(subject), Some(object)) = (triple.subject.as_iri(), triple.object.as_iri())
            else {
                continue;
            };
            let subject = subject.as_str();
            let object = object.as_str();
            if !object.starts_with(v3::SBOL_NS) {
                continue;
            }
            if self.sbol2_type_for_subject_type(subject, object).is_some() {
                supported.insert(subject.to_owned());
            } else {
                unsupported
                    .entry(subject.to_owned())
                    .or_default()
                    .push(object.to_owned());
            }
        }

        let mut entries: Vec<(String, Vec<String>)> = unsupported
            .into_iter()
            .filter(|(subject, _)| {
                !supported.contains(subject)
                    && !self.mapsto_reconstructions.contains_key(subject)
                    && !self.mapsto_constraints.contains(subject)
                    && !self.interface_subjects.contains(subject)
                    && !self.discarded_subjects.contains(subject)
            })
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        for (subject, mut types) in entries {
            types.sort();
            if let Some(sbol3_type) = types.into_iter().next() {
                self.discarded_subjects.insert(subject.clone());
                self.report.push(DowngradeWarning::UnsupportedSbol3Type {
                    subject,
                    sbol3_type,
                });
            }
        }
    }
}
