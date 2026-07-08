//! Engine construction and the preflight analysis pass: building IRI
//! rewrites, discovering SequenceAnnotation collapses, and flagging
//! unsupported SBOL 3 subjects.

use super::helpers::*;
use super::*;
use crate::uri;

impl<'a> Engine<'a> {
    pub(super) fn new(input: &'a Document, options: DowngradeOptions) -> Self {
        Self {
            input,
            options,
            versions: HashMap::new(),
            preserved_versions: HashSet::new(),
            persistent_identities: HashMap::new(),
            resolved_types: HashMap::new(),
            resolved_type_sets: HashMap::new(),
            subcomponent_targets: HashMap::new(),
            module_origin_subcomponents: HashSet::new(),
            null_sequence_locations: HashSet::new(),
            top_levels: HashSet::new(),
            sbol3_namespaces: HashMap::new(),
            iri_rewrites: HashMap::new(),
            sa_collapses: HashMap::new(),
            mapsto_reconstructions: HashMap::new(),
            mapsto_constraints: HashSet::new(),
            discarded_subjects: HashSet::new(),
            fc_directions: HashMap::new(),
            interface_subjects: HashSet::new(),
            component_splits: HashMap::new(),
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
                v2::BACKPORT_SBOL2_ORIGINATES_FROM_MODULE => {
                    self.module_origin_subcomponents.insert(subject.clone());
                }
                v2::BACKPORT_SBOL3_TEMP_SEQUENCE_URI => {
                    // The upgrade synthesized an empty Sequence to satisfy
                    // SBOL 3's requirement that every Location carry one.
                    // SBOL 2 has no such requirement, so the Sequence and
                    // every reference to it are dropped on the way down.
                    if let Some(iri) = triple.object.as_iri() {
                        self.discarded_subjects.insert(iri.as_str().to_owned());
                    }
                }
                v2::BACKPORT_SBOL2_LOCATION_SEQUENCE_NULL => {
                    self.null_sequence_locations.insert(subject.clone());
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

        // MapsTo + Interface decompositions: detect ComponentReference /
        // Constraint pairs the upgrade emitted in place of SBOL 2 MapsTo,
        // and Interface subjects emitted in place of FunctionalComponent
        // directions. Both get re-synthesized later from these maps.
        self.discover_mapsto_and_interfaces();

        // Classify each Component into a ComponentDefinition or a
        // ModuleDefinition and compute its SBOL 2 IRI. Must precede
        // `build_iri_rewrites`, which consults the classification.
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
        // when the SBOL 3 IRI carried one. Synthesizing one for documents
        // that had no version originally would pollute round-trips.
        // Top-levels without a version still receive an identity rewrite so
        // `persistentIdentity` is emitted for them.
        let mut subjects: HashSet<String> = HashSet::new();
        for triple in self.input.rdf_graph().triples() {
            if let Some(iri) = triple.subject.as_iri() {
                let iri = iri.as_str();
                if !self.discarded_subjects.contains(iri) {
                    subjects.insert(iri.to_owned());
                }
            }
        }

        // Pass 1: top-levels. Reconstruct the SBOL 2 identity from the
        // reference version-in-IRI SBOL 3 form (`<ns>/<version>/<displayId>`
        // → `<ns>/<displayId>/<version>`), capturing each top-level's
        // persistent identity and version so children nest under it.
        let mut tops: Vec<(String, String, Option<String>)> = Vec::new();
        for iri in &subjects {
            if !self.top_levels.contains(iri) {
                continue;
            }
            let sbol2 = uri::create_sbol2_uri(iri);
            let version = {
                let v = uri::version_sbol3(iri);
                (!v.is_empty()).then(|| v.to_owned())
            };
            let pid = match &version {
                Some(v) => sbol2
                    .strip_suffix(&format!("/{v}"))
                    .unwrap_or(&sbol2)
                    .to_owned(),
                None => sbol2.clone(),
            };
            let effective = match &version {
                Some(v) => {
                    self.versions.insert(iri.clone(), v.clone());
                    self.preserved_versions.insert(iri.clone());
                    self.record_versioned();
                    Some(v.clone())
                }
                None => match self.options.default_version.clone() {
                    Some(dv) => {
                        self.versions.insert(iri.clone(), dv.clone());
                        self.note_synthesized(iri, &dv);
                        Some(dv)
                    }
                    None => None,
                },
            };
            self.persistent_identities.insert(iri.clone(), pid.clone());
            let final_sbol2 = match (&version, &effective) {
                (None, Some(dv)) => format!("{pid}/{dv}"),
                _ => sbol2,
            };
            self.iri_rewrites.insert(iri.clone(), final_sbol2);
            tops.push((iri.clone(), pid, effective));
        }
        tops.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));

        // Pass 2: children nest under their owning top-level. Their SBOL 3
        // IRI is `<topSbol3>/<relative>`; the SBOL 2 form is
        // `<topPersistentId>/<relative>/<version>`.
        for iri in &subjects {
            if self.iri_rewrites.contains_key(iri) {
                continue;
            }
            for (top, pid, version) in &tops {
                let prefix = format!("{top}/");
                if iri.starts_with(&prefix) && iri != top {
                    let relative = &iri[prefix.len()..];
                    let child_pid = format!("{pid}/{relative}");
                    let new = match version {
                        Some(v) => format!("{child_pid}/{v}"),
                        None => child_pid.clone(),
                    };
                    self.persistent_identities.insert(iri.clone(), child_pid);
                    self.iri_rewrites.insert(iri.clone(), new);
                    break;
                }
            }
        }

        // Pass 3: reconstruct each collapsed SequenceAnnotation and its
        // locations under the parent's SBOL 2 persistent identity
        // (`<parentPid>/<saDisplayId>[/<locDisplayId>]`), not under the
        // SubComponent the upgrade relocated them onto. `discover_sa_collapses`
        // seeded the SA IRI from the SBOL 3 (version-in-IRI) parent because
        // the SBOL 2 identities weren't built yet; correct it here so both the
        // SA wrapper and its locations carry proper SBOL 2 identities and a
        // SBOL2→SBOL3→SBOL2→SBOL3 round-trip reaches a fixed point.
        let sa_keys: Vec<String> = self.sa_collapses.keys().cloned().collect();
        for subcomp in sa_keys {
            let (parent_cd, sa_display_id, parent_component, locations) = {
                let info = &self.sa_collapses[&subcomp];
                (
                    info.parent_cd.clone(),
                    info.sa_display_id.clone(),
                    info.parent_component.clone(),
                    info.locations.clone(),
                )
            };
            let parent_pid = self
                .persistent_identities
                .get(&parent_cd)
                .cloned()
                .unwrap_or_else(|| self.rewrite_iri(&parent_cd).to_owned());
            let sa_pid = format!("{parent_pid}/{sa_display_id}");
            self.sa_collapses.get_mut(&subcomp).unwrap().sa_iri_unversioned = sa_pid.clone();

            let version = self.effective_version_for_top_level(&parent_component);
            for loc in locations {
                let display_id = last_segment(&loc).to_owned();
                let loc_pid = format!("{sa_pid}/{display_id}");
                let sbol2 = match &version {
                    Some(v) => format!("{loc_pid}/{v}"),
                    None => loc_pid.clone(),
                };
                self.persistent_identities.insert(loc.clone(), loc_pid);
                self.iri_rewrites.insert(loc, sbol2);
            }
        }
    }

    /// Returns `Some(version)` when this top-level subject's SBOL 3 IRI
    /// carried a version segment. Otherwise `None`, signalling that the
    /// caller must decide whether to synthesize one from
    /// [`DowngradeOptions::default_version`].
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

        for triple in self.input.rdf_graph().triples() {
            let Some(subject) = triple.subject.as_iri() else {
                continue;
            };
            let subject = subject.as_str().to_owned();

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
                v2::BACKPORT_SBOL2_ORIGINAL_SEQUENCE_ANNOTATION_URI => {
                    // The upgrade stamps the original SBOL 2 SequenceAnnotation
                    // identity; its displayId names the reconstructed SA.
                    if let Some(sa_uri) = triple.object.as_iri()
                        && let Some(display_id) = uri::display_id_sbol2(sa_uri.as_str())
                    {
                        backport_sa_display_ids
                            .entry(subject)
                            .or_insert(display_id.to_owned());
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
            // A SubComponent whose parent Component becomes a ModuleDefinition
            // downgrades to a FunctionalComponent, which cannot own a
            // SequenceAnnotation; its locations are dropped entirely, matching
            // the reference.
            if self
                .component_splits
                .get(&parent_component)
                .is_some_and(|split| split.shape == ComponentShape::MdOnly)
            {
                if let Some(locations) = locations_of.get(&subcomp) {
                    for location in locations {
                        self.discarded_subjects.insert(location.clone());
                    }
                }
                continue;
            }
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
            let mut locations = locations_of.get(&subcomp).cloned().unwrap_or_default();
            locations.sort();
            // Native SBOL 3 SubComponents (no backport hint) name their
            // synthesized SequenceAnnotation `{subComponent}_{firstLocation}`,
            // matching the reference converter. The stored hint wins when
            // present so an SBOL 2-origin SA restores its original displayId.
            let base = match backport_sa_display_ids.get(&subcomp) {
                Some(display_id) => display_id.clone(),
                None => {
                    let sub_display_id = display_ids
                        .get(&subcomp)
                        .cloned()
                        .unwrap_or_else(|| last_segment(&subcomp).to_owned());
                    match locations.first().and_then(|loc| display_ids.get(loc)) {
                        Some(loc_display_id) => format!("{sub_display_id}_{loc_display_id}"),
                        None => format!("{sub_display_id}_annotation"),
                    }
                }
            };
            let (sa_display_id, sa_iri_unversioned) =
                next_available_child_iri(&parent_cd, &base, &mut self.used_iris);
            self.sa_collapses.insert(
                subcomp,
                SaCollapseInfo {
                    sa_display_id,
                    sa_iri_unversioned,
                    parent_component,
                    parent_cd,
                    locations,
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
