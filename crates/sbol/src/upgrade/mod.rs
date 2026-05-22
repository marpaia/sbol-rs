//! Convert SBOL 2 RDF documents to SBOL 3.
//!
//! Most published synbio content (SynBioHub, iGEM Registry, JBEI ICE)
//! predates SBOL 3 and remains in SBOL 2 form. This module reads SBOL 2 RDF
//! and emits SBOL 3 RDF that can be loaded by the rest of the crate and
//! validated like any natively-authored SBOL 3 document.
//!
//! Conversion happens at the RDF triple level, mirroring the canonical engine
//! in `sboltools/sbolgraph`'s `fromSBOL2/toSBOL3.ts`. No external runtime is
//! required: input goes in as bytes (any RDF serialization the [`RdfFormat`]
//! enum supports), output comes back as bytes plus an [`UpgradeReport`] of
//! warnings.
//!
//! ```no_run
//! use sbol::{Document, RdfFormat};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let (document, report) = Document::upgrade_from_sbol2_path("design.xml")?;
//!
//! for warning in report.warnings() {
//!     eprintln!("{warning:?}");
//! }
//!
//! document.check()?;
//! # Ok(())
//! # }
//! ```
//!
//! The [`Document`] is always returned when the input parses as valid SBOL 2
//! RDF — callers gate on validation with the existing [`Document::check`]
//! method if they want a strict pipeline.
//!
//! For the full conversion model — workflows organized by what you have, the
//! `http://sboltools.org/backport#` namespace, structural collapses
//! (SequenceAnnotation, MapsTo, Interface), known divergences, and known
//! limitations — see the [conversion guide][conversion-md].
//!
//! [`Document`]: crate::Document
//! [`Document::check`]: crate::Document::check
//! [`RdfFormat`]: crate::RdfFormat
//! [conversion-md]: https://github.com/marpaia/sbol-rs/blob/master/docs/conversion.md

use std::collections::{HashMap, HashSet};

use crate::sbol2_vocab as v2;
use crate::vocab as v3;
use crate::{Iri, Resource, Term, Triple};
use sbol_rdf::{Graph, ParseError, RdfFormat};

mod identity;
mod values;

use identity::IdentityMap;

/// Configuration for [`sbol2_to_sbol3`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct UpgradeOptions {
    /// Default namespace to attach to top-level objects whose namespace
    /// cannot be derived from the input (no `persistentIdentity`, no
    /// `displayId`, IRI parses as opaque). If `None`, an
    /// [`UpgradeWarning::NamespaceFallback`] is emitted and the object is
    /// left without `hasNamespace`.
    pub default_namespace: Option<Iri>,

    /// Whether to preserve SBOL 2-only fields (`persistentIdentity`,
    /// `version`, and the original `sbol2type`) under the
    /// `http://sboltools.org/backport#` namespace so a later SBOL 3 → SBOL 2
    /// downgrade can reconstruct them losslessly. Defaults to `true`.
    pub preserve_backport: bool,
}

impl Default for UpgradeOptions {
    fn default() -> Self {
        Self {
            default_namespace: None,
            preserve_backport: true,
        }
    }
}

impl UpgradeOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Warnings emitted by [`sbol2_to_sbol3`]. The presence of warnings does not
/// stop conversion — every warning records something the upgrade could not
/// translate cleanly but chose not to fail on.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum UpgradeWarning {
    /// A top-level subject's `hasNamespace` could not be derived from
    /// `persistentIdentity` + `displayId` or from the IRI's URL origin
    /// — the upgrade either used [`UpgradeOptions::default_namespace`]
    /// or left the subject without `hasNamespace`. URL-origin
    /// derivation is the canonical fallback for HTTP-style IRIs and
    /// does not warn.
    NamespaceFallback {
        subject: String,
        source: NamespaceSource,
    },
    /// A `sbol2:MapsTo` was decomposed into a Constraint, but the local or
    /// remote side could not be resolved to a SubComponent. The constraint
    /// was skipped.
    UnresolvedMapsTo { mapsto: String, side: MapsToSide },
    /// A `sbol2:MapsTo` carried a `refinement` value with no SBOL 3
    /// Constraint equivalent (notably `merge`). The Constraint is still
    /// emitted, with its restriction coerced to `sbol3:verifyIdentical`;
    /// the original refinement is preserved on the ComponentReference
    /// as `backport:mapsToRefinement` so a downgrade can restore it.
    UnsupportedRefinement { mapsto: String, refinement: String },
    /// A `sbol2:SequenceAnnotation` referenced a `sbol2:component`. The
    /// upgrade collapsed the SA shell onto the referenced SubComponent.
    SequenceAnnotationWithComponent { annotation: String },
    /// A subject carried an SBOL 2 type the upgrade does not understand.
    /// Its class is archived as `backport:sbol2type` when backport
    /// preservation is enabled and no recognized SBOL 2 class shares the
    /// subject, but the unknown class does not appear in the SBOL 3 typed
    /// model.
    UnknownSbol2Type { subject: String, sbol2_type: String },
    /// A Location belongs to a SequenceFeature on a Component that has no
    /// (or more than one) `sbol2:sequence`, so `sbol3:hasSequence` could not
    /// be unambiguously inferred. The resulting SBOL 3 document will fail
    /// `sbol3-10110` cardinality validation until the source data is
    /// corrected to declare exactly one Sequence per Component.
    LocationWithoutSequence {
        location: String,
        component: String,
        sequence_count: usize,
    },
    /// Two or more distinct SBOL 2 subjects in the input share a
    /// canonical SBOL 3 IRI after version-stripping — for example
    /// `<lab/foo/1>` (whose canonical is `<lab/foo>`) and
    /// `<lab/foo>` itself. The conversion preserves every input triple
    /// but lands all of them at the same SBOL 3 subject, silently
    /// merging the entities into a single chimeric Component / Sequence
    /// / etc. The input was technically non-conformant SBOL 2
    /// (persistentIdentity should be unique), but the merge is
    /// otherwise indistinguishable from intended behavior; this warning
    /// surfaces the situation so callers can audit the source data.
    IdentityCollision {
        canonical: String,
        sources: Vec<String>,
    },
}

/// Indicates which path was taken when a namespace had to be derived without
/// a usable `persistentIdentity` + `displayId` pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NamespaceSource {
    /// Parsed scheme+host of the subject IRI.
    UrlOrigin,
    /// Used [`UpgradeOptions::default_namespace`].
    DefaultOption,
    /// No fallback applied — the object has no `hasNamespace` in the output.
    None,
}

/// Which side of a MapsTo failed to resolve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapsToSide {
    Local,
    Remote,
    /// The MapsTo had no enclosing
    /// [`sbol2:Module`](crate::sbol2_vocab::SBOL2_MODULE) /
    /// [`sbol2:FunctionalComponent`](crate::sbol2_vocab::SBOL2_FUNCTIONAL_COMPONENT)
    /// carrier — i.e. nothing in the source pointed at it via the
    /// `sbol2:mapsTo` containment predicate, so the upgrade has no
    /// place to attach the resulting ComponentReference + Constraint
    /// pair.
    Carrier,
}

/// Tally of how many SBOL 2 constructs the upgrade rewrote into SBOL 3
/// constructs. Useful for end-of-run summaries and CI signals (e.g. an
/// expected document type accidentally dropping to zero across a refactor).
#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct UpgradeCounts {
    /// `sbol2:ComponentDefinition` subjects rewritten to `sbol3:Component`.
    pub component_definitions: usize,
    /// `sbol2:ModuleDefinition` subjects rewritten to `sbol3:Component`
    /// (with synthesized `SBO:functionalEntity` type).
    pub module_definitions: usize,
    /// `sbol2:Component` / `sbol2:Module` / `sbol2:FunctionalComponent`
    /// subjects rewritten to `sbol3:SubComponent`.
    pub sub_components: usize,
    /// Standalone `sbol2:SequenceAnnotation` subjects rewritten to
    /// `sbol3:SequenceFeature` (i.e. SAs that did NOT carry a
    /// `sbol2:component` reference).
    pub sequence_features: usize,
    /// SequenceAnnotations that referenced a `sbol2:component` and were
    /// collapsed onto the referenced SubComponent.
    pub sequence_annotations_collapsed: usize,
    /// `sbol2:MapsTo` subjects decomposed into ComponentReference +
    /// Constraint pairs.
    pub mapstos_decomposed: usize,
    /// `sbol3:Interface` objects synthesized from FunctionalComponent
    /// direction information.
    pub interfaces_synthesized: usize,
    /// Locations that received a `sbol3:hasSequence` triple inferred from
    /// their enclosing Component's single sequence.
    pub locations_with_inferred_sequence: usize,
}

/// Report of every non-fatal issue encountered during an upgrade run, plus
/// counts of what the converter rewrote.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct UpgradeReport {
    warnings: Vec<UpgradeWarning>,
    counts: UpgradeCounts,
}

impl UpgradeReport {
    /// Iterates over every warning recorded during the run.
    pub fn warnings(&self) -> &[UpgradeWarning] {
        &self.warnings
    }

    /// Returns the per-construct counts.
    pub fn counts(&self) -> &UpgradeCounts {
        &self.counts
    }

    /// Returns `true` if no warnings were recorded.
    pub fn is_clean(&self) -> bool {
        self.warnings.is_empty()
    }

    pub(crate) fn push(&mut self, warning: UpgradeWarning) {
        self.warnings.push(warning);
    }
}

/// Errors returned by [`sbol2_to_sbol3`].
#[derive(Debug)]
#[non_exhaustive]
pub enum UpgradeError {
    /// The input bytes did not parse as RDF.
    Parse(ParseError),
    /// The input parsed, but no SBOL 2 typed subjects were detected. Pass an
    /// SBOL 2 document; SBOL 3 input does not need an upgrade.
    NotSbol2,
}

impl std::fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "failed to parse input as RDF: {err}"),
            Self::NotSbol2 => write!(
                f,
                "input contains no SBOL 2 typed subjects \
                 (http://sbols.org/v2# rdf:type) — nothing to upgrade",
            ),
        }
    }
}

impl std::error::Error for UpgradeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(err) => Some(err),
            Self::NotSbol2 => None,
        }
    }
}

impl From<ParseError> for UpgradeError {
    fn from(err: ParseError) -> Self {
        Self::Parse(err)
    }
}

/// Converts an SBOL 2 RDF graph to SBOL 3 RDF.
///
/// The input graph must contain at least one subject typed in the SBOL 2
/// namespace (`http://sbols.org/v2#…`); otherwise [`UpgradeError::NotSbol2`]
/// is returned. Returns the new graph plus an [`UpgradeReport`] of any
/// non-fatal issues.
pub fn sbol2_to_sbol3(
    graph: &Graph,
    options: UpgradeOptions,
) -> Result<(Graph, UpgradeReport), UpgradeError> {
    let mut engine = Engine::new(graph, options);
    engine.preflight()?;
    engine.convert();
    Ok((Graph::new(engine.output_triples), engine.report))
}

/// Convenience wrapper: parses input bytes, calls [`sbol2_to_sbol3`], then
/// returns both the SBOL 3 [`Graph`] and the [`UpgradeReport`].
pub fn parse_and_upgrade(
    input: &str,
    format: RdfFormat,
    options: UpgradeOptions,
) -> Result<(Graph, UpgradeReport), UpgradeError> {
    let graph = Graph::parse(input, format)?;
    sbol2_to_sbol3(&graph, options)
}

#[derive(Clone, Copy)]
enum FcDirection {
    Input,
    Output,
    Inout,
}

/// Per-MapsTo selector for the constraint shape emitted by
/// [`Engine::synthesize_mapsto_decomposition`]. Derived from the source
/// MapsTo's `sbol2:refinement` value (or its absence).
#[derive(Clone, Copy)]
enum RefinementShape {
    UseLocal,
    UseRemote,
    VerifyIdentical,
}

#[derive(Default, Clone)]
struct MapsToInfo {
    local: Option<String>,
    remote: Option<String>,
    refinement: Option<String>,
    display_id: Option<String>,
}

struct Engine<'a> {
    input: &'a Graph,
    options: UpgradeOptions,
    identity: IdentityMap,
    /// Subject IRI → its rdf:type IRI string, for every subject the
    /// upgrade may need to namespace (SBOL 2 typed subjects, plus the
    /// PROV top-level classes that SBOL 2 documents commonly carry).
    /// Both populate `hasNamespace` on emission.
    typed_subjects: HashMap<String, String>,
    /// Original CD IRI → list of Sequence IRIs it references via
    /// `sbol2:sequence`. Used to infer `hasSequence` on Locations after
    /// conversion.
    cd_sequences: HashMap<String, Vec<String>>,
    /// Original Location IRI (Range/Cut/GenericLocation) → original SA IRI
    /// that owns it. Used together with [`sa_to_cd`] to walk back to the
    /// owning Component and infer the Location's `hasSequence`.
    location_to_sa: HashMap<String, String>,
    /// Original SA IRI → original CD IRI that owns it.
    sa_to_cd: HashMap<String, String>,
    /// Original SA IRI → original SubComponent (Component) IRI it references
    /// via `sbol2:component`. When this mapping is non-empty for an SA, the
    /// SA shell is discarded and its Locations move onto the referenced
    /// SubComponent (SEP-010 / sbolgraph SequenceAnnotation collapse rule).
    sa_to_subcomponent: HashMap<String, String>,
    /// Subject IRI → its SBOL 2 displayId. Used when structural rewrites move
    /// child subjects under a different parent and must preserve SBOL 3 child
    /// identity rules.
    display_ids: HashMap<String, String>,
    /// Owning-relation index: child subject IRI → parent subject IRI. Built
    /// from every SBOL 2 containment predicate (module, functionalComponent,
    /// component, mapsTo, interaction, participation, location, etc.). Used
    /// by [`Engine::owning_top_level`] to walk back to the enclosing
    /// top-level subject when synthesizing new triples (e.g. attaching
    /// MapsTo-derived ComponentReferences to the enclosing Component).
    owned_by: HashMap<String, String>,
    /// Original MapsTo IRI → its parsed local/remote/refinement values plus
    /// the MapsTo's displayId.
    mapsto_info: HashMap<String, MapsToInfo>,
    /// Original FunctionalComponent IRI → the SBOL 3 Interface slot it
    /// should occupy. SBOL 2 `inout` and public+`none` components both map
    /// to SBOL 3 `nondirectional`.
    fc_directions: HashMap<String, FcDirection>,
    /// FunctionalComponents that explicitly declared `sbol2:direction none`.
    /// The SBOL 2→3 mapping still lists these under Interface.nondirectional
    /// when their access is public.
    fc_direction_none: HashSet<String>,
    /// FunctionalComponents that explicitly declared `sbol2:access public`.
    fc_public_access: HashSet<String>,
    output_triples: Vec<Triple>,
    /// Subjects already given a `sbol3:hasNamespace` triple. Used to avoid
    /// duplicate emissions when a subject has multiple `sbol2:displayId`
    /// triples or appears more than once in pre-scan.
    namespaced_subjects: HashSet<String>,
    /// Every IRI the upgrade has observed in the input graph (rewritten
    /// to its canonical SBOL 3 form) or allocated as a synthesized
    /// SBOL 3 subject so far. Every IRI-synthesis site (SA-collapse
    /// Location rewrite, MapsTo decomposition CRef/Constraint,
    /// synthesized Interface) routes its candidate IRI through
    /// `next_available_*` against this pool. The invariant the pool
    /// enforces — mirrored from the downgrade — is: **no two distinct
    /// SBOL 3 entities ever land at the same IRI**, regardless of how
    /// creatively the input names things.
    used_iris: HashSet<String>,
    /// Original Location IRI → displayId literal value to emit on the
    /// rewritten subject. Populated only when the SA-collapse Location
    /// rewrite had to disambiguate the Location's new IRI with a `_N`
    /// suffix, so the emitted `sbol3:displayId` still matches the
    /// IRI's last segment (sbol3-10204 compliance).
    location_display_id_overrides: HashMap<String, String>,
    report: UpgradeReport,
}

impl<'a> Engine<'a> {
    fn new(input: &'a Graph, options: UpgradeOptions) -> Self {
        let identity = IdentityMap::build(input);
        Self {
            input,
            options,
            identity,
            typed_subjects: HashMap::new(),
            cd_sequences: HashMap::new(),
            location_to_sa: HashMap::new(),
            sa_to_cd: HashMap::new(),
            sa_to_subcomponent: HashMap::new(),
            display_ids: HashMap::new(),
            owned_by: HashMap::new(),
            mapsto_info: HashMap::new(),
            fc_directions: HashMap::new(),
            fc_direction_none: HashSet::new(),
            fc_public_access: HashSet::new(),
            output_triples: Vec::new(),
            namespaced_subjects: HashSet::new(),
            used_iris: HashSet::new(),
            location_display_id_overrides: HashMap::new(),
            report: UpgradeReport::default(),
        }
    }

    fn preflight(&mut self) -> Result<(), UpgradeError> {
        // Seed the used-IRI pool with every input subject IRI in its
        // canonical (post-`identity.rewrite_iri`) form. Every synthesis
        // site downstream — SA-collapse Location rewrite,
        // ComponentReference/Constraint allocation for MapsTo
        // decomposition, synthesized Interface IRI — routes its
        // candidate IRI through `next_available_*` against this pool,
        // so no synthesized SBOL 3 IRI silently lands on an existing
        // subject. (Mirrors the same invariant the downgrade engine
        // enforces.)
        //
        // The same pass groups input subjects by canonical IRI so we
        // can surface identity collisions — two distinct input
        // subjects whose version-stripped (or persistentIdentity-keyed)
        // canonical forms coincide. The conversion still runs and the
        // merged subject keeps every input triple, but the result is a
        // chimeric SBOL 3 entity that almost certainly isn't what the
        // author intended; the warning lets callers audit.
        let mut input_subjects: HashSet<String> = HashSet::new();
        for triple in self.input.triples() {
            if let Some(iri) = triple.subject.as_iri() {
                input_subjects.insert(iri.as_str().to_owned());
            }
        }
        let mut canonical_sources: HashMap<String, Vec<String>> = HashMap::new();
        for subject in input_subjects {
            let canonical = self.identity.rewrite_iri(&subject).to_owned();
            self.used_iris.insert(canonical.clone());
            canonical_sources
                .entry(canonical)
                .or_default()
                .push(subject);
        }
        let mut collisions: Vec<(String, Vec<String>)> = canonical_sources
            .into_iter()
            .filter(|(_, sources)| sources.len() > 1)
            .collect();
        collisions.sort_by(|a, b| a.0.cmp(&b.0));
        for (canonical, mut sources) in collisions {
            sources.sort();
            self.report
                .push(UpgradeWarning::IdentityCollision { canonical, sources });
        }
        for triple in self.input.triples() {
            let predicate = triple.predicate.as_str();

            if predicate == v3::RDF_TYPE {
                if let Some(iri) = triple.object.as_iri()
                    && let Some(subject_iri) = triple.subject.as_iri()
                {
                    let object = iri.as_str();
                    // Capture SBOL 2 types directly, plus the PROV top-level
                    // classes (Activity / Agent / Plan) that SBOL 2 documents
                    // commonly carry alongside SBOL 2 typed objects (e.g.
                    // SynBioHub's igem2sbol conversion provenance). PROV
                    // top-levels are normative top-levels in SBOL 3 too and
                    // need `hasNamespace` synthesized.
                    if object.starts_with(v2::SBOL2_NS)
                        || object == v3::PROV_ACTIVITY
                        || object == v3::PROV_AGENT_CLASS
                        || object == v3::PROV_PLAN
                    {
                        let subject = subject_iri.as_str().to_owned();
                        let should_replace =
                            self.typed_subjects.get(&subject).is_none_or(|existing| {
                                type_precedence(object) > type_precedence(existing)
                            });
                        if should_replace {
                            self.typed_subjects.insert(subject, object.to_owned());
                        }
                    }
                }
                continue;
            }

            let subject = match triple.subject.as_iri() {
                Some(iri) => iri.as_str().to_owned(),
                None => continue,
            };

            if predicate == v2::SBOL2_DISPLAY_ID
                && let Some(lit) = triple.object.as_literal()
            {
                self.display_ids
                    .entry(subject.clone())
                    .or_insert_with(|| lit.value().to_owned());
            }

            let object = match triple.object.as_iri() {
                Some(iri) => iri.as_str().to_owned(),
                None => continue,
            };

            match predicate {
                v2::SBOL2_SEQUENCE_PROP => {
                    self.cd_sequences
                        .entry(subject.clone())
                        .or_default()
                        .push(object.clone());
                }
                v2::SBOL2_SEQUENCE_ANNOTATION_PROP => {
                    self.sa_to_cd.insert(object.clone(), subject.clone());
                    self.owned_by.insert(object, subject);
                }
                v2::SBOL2_LOCATION_PROP => {
                    self.location_to_sa.insert(object.clone(), subject.clone());
                    self.owned_by.insert(object, subject);
                }
                v2::SBOL2_COMPONENT_PROP
                | v2::SBOL2_FUNCTIONAL_COMPONENT_PROP
                | v2::SBOL2_MODULE_PROP
                | v2::SBOL2_INTERACTION_PROP
                | v2::SBOL2_PARTICIPATION_PROP
                | v2::SBOL2_MAPS_TO_PROP
                | v2::SBOL2_VARIABLE_COMPONENT_PROP => {
                    self.owned_by.insert(object, subject);
                }
                _ => {}
            }
        }

        if !self
            .typed_subjects
            .values()
            .any(|t| t.starts_with(v2::SBOL2_NS))
        {
            return Err(UpgradeError::NotSbol2);
        }

        // Second pass to resolve SA-with-component: `sbol2:component` is
        // overloaded across CDs and SAs, so we need the rdf:type map from
        // the first pass to know which subjects are SAs.
        for triple in self.input.triples() {
            if triple.predicate.as_str() != v2::SBOL2_COMPONENT_PROP {
                continue;
            }
            let subject = match triple.subject.as_iri() {
                Some(iri) => iri.as_str().to_owned(),
                None => continue,
            };
            if self.typed_subjects.get(&subject).map(String::as_str)
                != Some(v2::SBOL2_SEQUENCE_ANNOTATION)
            {
                continue;
            }
            let target = match triple.object.as_iri() {
                Some(iri) => iri.as_str().to_owned(),
                None => continue,
            };
            self.sa_to_subcomponent.insert(subject, target);
        }

        // Locations under collapsed SAs must migrate to be children of their
        // new SubComponent parent so SBOL 3 IRI compliance (sbol3-10104)
        // holds. The new IRI is `{SubComponent}/{displayId or last segment}`.
        //
        // Allocation routes through `next_available_child_iri` against
        // `self.used_iris` so two SAs both pointing at the same
        // SubComponent and carrying Locations of the same displayId
        // produce DISTINCT new Location IRIs (one disambiguated with a
        // `_N` suffix) rather than silently merging both Locations
        // onto a single subject. When disambiguation fires the
        // emitted `sbol3:displayId` value must also pick up the
        // suffix so sbol3-10204 still holds — that override is
        // recorded in `location_display_id_overrides` and consulted by
        // `handle_sbol2_predicate` when it emits the rewritten
        // `sbol3:displayId` triple.
        //
        // Sorted to keep the disambiguation index deterministic across
        // runs (HashMap iteration order is unstable).
        let mut location_owners: Vec<(String, String)> = self
            .location_to_sa
            .iter()
            .filter_map(|(loc, sa)| {
                self.sa_to_subcomponent
                    .get(sa)
                    .map(|sc| (loc.clone(), sc.clone()))
            })
            .collect();
        location_owners.sort();
        for (loc_iri, subcomponent_iri) in location_owners {
            let subcomponent_canonical = self.identity.rewrite_iri(&subcomponent_iri).to_owned();
            let base_display_id = self.display_ids.get(&loc_iri).cloned().unwrap_or_else(|| {
                last_path_segment(self.identity.rewrite_iri(&loc_iri)).to_owned()
            });
            // Release the Location's existing canonical IRI from the
            // pool before allocating — we're about to re-map this
            // identity onto its new SubComponent-relative slot, so its
            // old slot no longer represents a distinct subject. Without
            // this, an upgrade-downgrade-upgrade round-trip where the
            // downgrade emitted the Location at its post-collapse IRI
            // (because that's what `backport:sbol2persistentIdentity`
            // preserved) would see the Location's own IRI in
            // `used_iris` and disambiguate the re-collapse target to
            // `_2`, drifting the round-trip.
            let loc_canonical = self.identity.rewrite_iri(&loc_iri).to_owned();
            self.used_iris.remove(&loc_canonical);
            let (chosen_display_id, new_iri) = next_available_child_iri(
                &subcomponent_canonical,
                &base_display_id,
                &mut self.used_iris,
            );
            if chosen_display_id != base_display_id {
                self.location_display_id_overrides
                    .insert(loc_iri.clone(), chosen_display_id);
            }
            self.identity.add_rewrite(loc_iri, new_iri);
        }

        // Capture FunctionalComponent directions so we can synthesize an
        // SBOL 3 Interface per Component. Per SBOL 3.1.0 section 10.2,
        // public SBOL 2 FunctionalComponents with direction none also belong
        // in the SBOL 3 Interface as nondirectional features; we collect the
        // two required signals in separate scans and merge them below.
        for triple in self.input.triples() {
            if triple.predicate.as_str() != v2::SBOL2_DIRECTION {
                continue;
            }
            let subject = match triple.subject.as_iri() {
                Some(iri) => iri.as_str(),
                None => continue,
            };
            if self.typed_subjects.get(subject).map(String::as_str)
                != Some(v2::SBOL2_FUNCTIONAL_COMPONENT)
            {
                continue;
            }
            let dir_iri = match triple.object.as_iri() {
                Some(iri) => iri.as_str(),
                None => continue,
            };
            let direction = match dir_iri {
                v2::SBOL2_DIRECTION_IN => FcDirection::Input,
                v2::SBOL2_DIRECTION_OUT => FcDirection::Output,
                v2::SBOL2_DIRECTION_INOUT => FcDirection::Inout,
                v2::SBOL2_DIRECTION_NONE => {
                    self.fc_direction_none.insert(subject.to_owned());
                    continue;
                }
                _ => continue,
            };
            self.fc_directions.insert(subject.to_owned(), direction);
        }
        for triple in self.input.triples() {
            if triple.predicate.as_str() != v2::SBOL2_ACCESS {
                continue;
            }
            let subject = match triple.subject.as_iri() {
                Some(iri) => iri.as_str(),
                None => continue,
            };
            if triple.object.as_iri().map(|i| i.as_str()) != Some(v2::SBOL2_ACCESS_PUBLIC) {
                continue;
            }
            match self.typed_subjects.get(subject).map(String::as_str) {
                Some(v2::SBOL2_FUNCTIONAL_COMPONENT) => {
                    self.fc_public_access.insert(subject.to_owned());
                }
                Some(v2::SBOL2_COMPONENT) => {
                    self.fc_directions
                        .entry(subject.to_owned())
                        .or_insert(FcDirection::Inout);
                }
                _ => {}
            }
        }
        for fc_iri in self
            .fc_direction_none
            .intersection(&self.fc_public_access)
            .cloned()
            .collect::<Vec<_>>()
        {
            self.fc_directions
                .entry(fc_iri)
                .or_insert(FcDirection::Inout);
        }

        // Third pass to populate per-MapsTo details (local / remote /
        // refinement / displayId), keyed by the original MapsTo IRI.
        for triple in self.input.triples() {
            let subject = match triple.subject.as_iri() {
                Some(iri) => iri.as_str(),
                None => continue,
            };
            if self.typed_subjects.get(subject).map(String::as_str) != Some(v2::SBOL2_MAPS_TO) {
                continue;
            }
            let info = self.mapsto_info.entry(subject.to_owned()).or_default();
            match triple.predicate.as_str() {
                v2::SBOL2_LOCAL => {
                    if let Some(iri) = triple.object.as_iri() {
                        info.local = Some(iri.as_str().to_owned());
                    }
                }
                v2::SBOL2_REMOTE => {
                    if let Some(iri) = triple.object.as_iri() {
                        info.remote = Some(iri.as_str().to_owned());
                    }
                }
                v2::SBOL2_REFINEMENT => {
                    if let Some(iri) = triple.object.as_iri() {
                        info.refinement = Some(iri.as_str().to_owned());
                    }
                }
                v2::SBOL2_DISPLAY_ID => {
                    if let Some(lit) = triple.object.as_literal() {
                        info.display_id = Some(lit.value().to_owned());
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Walks the `owned_by` chain up from `subject` until reaching a
    /// top-level subject (one with no owner). Returns the top-level subject's
    /// IRI string, or `subject` itself if it is already top-level.
    fn owning_top_level(&self, subject: &str) -> String {
        let mut current = subject.to_owned();
        while let Some(parent) = self.owned_by.get(&current) {
            current = parent.clone();
        }
        current
    }

    fn convert(&mut self) {
        // Iterate by index to avoid cloning the entire triple slice.
        // `handle_triple` only reads its argument; the &mut self is
        // needed for `self.output_triples` and the report. The input
        // slice is bounded by the loop's stored length, so growing
        // `self.output_triples` mid-loop is safe.
        let n = self.input.triples().len();
        for i in 0..n {
            let triple = self.input.triples()[i].clone();
            self.handle_triple(&triple);
        }
        self.emit_synthesized_triples();
    }

    fn handle_triple(&mut self, triple: &Triple) {
        let predicate = triple.predicate.as_str();

        // Metadata on a collapsed SA cannot pass through on the SA
        // subject because the SA shell no longer exists in SBOL 3. Archive
        // it on the replacement SubComponent under encoded backport
        // predicates so the downgrade can rebuild the original SA wrapper.
        if !predicate.starts_with(v2::SBOL2_NS)
            && let Some(subject_iri) = triple.subject.as_iri()
            && let Some(target) = self.sa_to_subcomponent.get(subject_iri.as_str()).cloned()
            && (predicate != v3::RDF_TYPE
                || triple.object.as_iri().map(|i| i.as_str())
                    != Some(v2::SBOL2_SEQUENCE_ANNOTATION))
        {
            self.preserve_collapsed_sa_metadata(triple, &target);
            return;
        }

        if predicate == v3::RDF_TYPE {
            self.handle_type_triple(triple);
            return;
        }

        // Predicates in the SBOL 2 namespace get rewritten or dropped.
        if predicate.starts_with(v2::SBOL2_NS) {
            self.handle_sbol2_predicate(triple);
            return;
        }

        // Restore a `backport:sbol3_*` predicate to its `sbol3:*` form.
        // The downgrade archives unmapped SBOL 3 predicates under that
        // namespace so they don't pollute the SBOL 2 surface; the
        // upgrade reverses that here.
        if let Some(local) = predicate.strip_prefix(v2::BACKPORT_SBOL3_PREFIX) {
            let restored = format!("{}{local}", v3::SBOL_NS);
            let rewritten = self.identity.rewrite_triple(triple);
            self.output_triples.push(Triple {
                subject: rewritten.subject,
                predicate: Iri::new_unchecked(restored),
                object: rewritten.object,
            });
            return;
        }

        // Everything else (PROV, dcterms, custom annotations) passes through
        // with identity rewriting on subject and object.
        self.output_triples
            .push(self.identity.rewrite_triple(triple));
    }

    fn handle_type_triple(&mut self, triple: &Triple) {
        let object_iri = match triple.object.as_iri() {
            Some(iri) => iri.as_str(),
            None => {
                self.output_triples
                    .push(self.identity.rewrite_triple(triple));
                return;
            }
        };

        // SAs that reference a SubComponent collapse into that SubComponent
        // (their locations migrate via the location-rewrite path). Suppress
        // the SequenceFeature type triple entirely.
        if object_iri == v2::SBOL2_SEQUENCE_ANNOTATION
            && let Some(subject_iri) = triple.subject.as_iri()
            && self.sa_to_subcomponent.contains_key(subject_iri.as_str())
        {
            self.report
                .push(UpgradeWarning::SequenceAnnotationWithComponent {
                    annotation: subject_iri.as_str().to_owned(),
                });
            self.report.counts.sequence_annotations_collapsed += 1;
            return;
        }

        // MapsTo is decomposed into ComponentReference + Constraint at the
        // emit-synthesized-triples stage. Suppress the original type triple.
        if object_iri == v2::SBOL2_MAPS_TO {
            return;
        }

        let subject = self.identity.rewrite_resource(&triple.subject);

        let v3_type: Option<&'static str> = match object_iri {
            v2::SBOL2_COMPONENT_DEFINITION | v2::SBOL2_MODULE_DEFINITION => {
                Some(v3::SBOL_COMPONENT_CLASS)
            }
            v2::SBOL2_COMPONENT | v2::SBOL2_MODULE | v2::SBOL2_FUNCTIONAL_COMPONENT => {
                Some(v3::SBOL_SUB_COMPONENT_CLASS)
            }
            v2::SBOL2_SEQUENCE_ANNOTATION => Some(v3::SBOL_SEQUENCE_FEATURE_CLASS),
            v2::SBOL2_SEQUENCE_CONSTRAINT => Some(v3::SBOL_CONSTRAINT_CLASS),
            v2::SBOL2_SEQUENCE => Some(v3::SBOL_SEQUENCE_CLASS),
            v2::SBOL2_MODEL => Some(v3::SBOL_MODEL_CLASS),
            v2::SBOL2_INTERACTION => Some(v3::SBOL_INTERACTION_CLASS),
            v2::SBOL2_PARTICIPATION => Some(v3::SBOL_PARTICIPATION_CLASS),
            v2::SBOL2_COLLECTION => Some(v3::SBOL_COLLECTION_CLASS),
            v2::SBOL2_IMPLEMENTATION => Some(v3::SBOL_IMPLEMENTATION_CLASS),
            v2::SBOL2_ATTACHMENT => Some(v3::SBOL_ATTACHMENT_CLASS),
            v2::SBOL2_EXPERIMENT => Some(v3::SBOL_EXPERIMENT_CLASS),
            v2::SBOL2_EXPERIMENTAL_DATA => Some(v3::SBOL_EXPERIMENTAL_DATA_CLASS),
            v2::SBOL2_COMBINATORIAL_DERIVATION => Some(v3::SBOL_COMBINATORIAL_DERIVATION_CLASS),
            v2::SBOL2_VARIABLE_COMPONENT => Some(v3::SBOL_VARIABLE_FEATURE_CLASS),
            v2::SBOL2_RANGE => Some(v3::SBOL_RANGE_CLASS),
            v2::SBOL2_CUT => Some(v3::SBOL_CUT_CLASS),
            v2::SBOL2_GENERIC_LOCATION => Some(v3::SBOL_LOCATION_CLASS),
            other if other.starts_with(v2::SBOL2_NS) => {
                if let Some(iri) = triple.subject.as_iri() {
                    self.report.push(UpgradeWarning::UnknownSbol2Type {
                        subject: iri.as_str().to_owned(),
                        sbol2_type: other.to_owned(),
                    });
                }
                let subject_has_known_type = triple
                    .subject
                    .as_iri()
                    .and_then(|iri| self.typed_subjects.get(iri.as_str()))
                    .is_some_and(|ty| is_known_sbol2_type(ty));
                if self.options.preserve_backport && !subject_has_known_type {
                    self.output_triples.push(Triple {
                        subject,
                        predicate: Iri::from_static(v2::BACKPORT_SBOL2_TYPE),
                        object: Term::Resource(Resource::Iri(Iri::new_unchecked(other))),
                    });
                }
                return;
            }
            _ => {
                // Non-SBOL2 type (e.g. BioPAX or custom) — keep, possibly
                // rewriting BioPAX → SBO terms.
                let mapped = values::map_biopax_type(object_iri).unwrap_or(object_iri);
                self.output_triples.push(Triple {
                    subject,
                    predicate: triple.predicate.clone(),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(mapped))),
                });
                return;
            }
        };

        if let Some(target) = v3_type {
            self.output_triples.push(Triple {
                subject: subject.clone(),
                predicate: triple.predicate.clone(),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(target))),
            });

            match object_iri {
                v2::SBOL2_COMPONENT_DEFINITION => {
                    self.report.counts.component_definitions += 1;
                }
                v2::SBOL2_MODULE_DEFINITION => {
                    self.report.counts.module_definitions += 1;
                    // SBOL 2 ModuleDefinitions don't carry a `type` — but
                    // the SBOL 3 Component they become requires at least
                    // one. Inject `SBO:functionalEntity` so the converted
                    // document satisfies sbol3-10110 cardinality without
                    // overriding any types the user explicitly authored.
                    self.output_triples.push(Triple {
                        subject: subject.clone(),
                        predicate: Iri::from_static(v3::SBOL_TYPE),
                        object: Term::Resource(Resource::Iri(Iri::new_unchecked(
                            "https://identifiers.org/SBO:0000241",
                        ))),
                    });
                }
                v2::SBOL2_COMPONENT | v2::SBOL2_MODULE | v2::SBOL2_FUNCTIONAL_COMPONENT => {
                    self.report.counts.sub_components += 1;
                }
                v2::SBOL2_SEQUENCE_ANNOTATION => {
                    // SAs that get *collapsed* never reach this branch (we
                    // short-circuited at the top of handle_type_triple), so
                    // any SA here is becoming a standalone SequenceFeature.
                    self.report.counts.sequence_features += 1;
                }
                _ => {}
            }

            if self.options.preserve_backport {
                self.output_triples.push(Triple {
                    subject,
                    predicate: Iri::from_static(v2::BACKPORT_SBOL2_TYPE),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(object_iri))),
                });
            }
        }
    }

    fn handle_sbol2_predicate(&mut self, triple: &Triple) {
        let predicate = triple.predicate.as_str();

        if self.try_collapse_sa_triple(triple) {
            return;
        }

        // Drop the parent's `sbol2:mapsTo` triple — the MapsTo shell is
        // replaced by ComponentReference + Constraint in
        // emit_synthesized_triples().
        if predicate == v2::SBOL2_MAPS_TO_PROP
            && let Some(target_iri) = triple.object.as_iri()
            && self.mapsto_info.contains_key(target_iri.as_str())
        {
            return;
        }

        // Drop every triple whose subject is a MapsTo — the decomposition
        // emits the equivalent SBOL 3 triples explicitly.
        if let Some(subject_iri) = triple.subject.as_iri()
            && self.mapsto_info.contains_key(subject_iri.as_str())
        {
            return;
        }

        let subject = self.identity.rewrite_resource(&triple.subject);
        let object = self.identity.rewrite_term(&triple.object);

        // Single-predicate rename cases.
        let renamed: Option<&'static str> = match predicate {
            v2::SBOL2_DISPLAY_ID => Some(v3::SBOL_DISPLAY_ID),
            v2::SBOL2_BUILT => Some(v3::SBOL_BUILT),
            v2::SBOL2_TYPE => Some(v3::SBOL_TYPE),
            v2::SBOL2_ROLE => Some(v3::SBOL_ROLE),
            v2::SBOL2_ROLE_INTEGRATION => Some(v3::SBOL_ROLE_INTEGRATION),
            v2::SBOL2_ELEMENTS => Some(v3::SBOL_ELEMENTS),
            v2::SBOL2_SOURCE => Some(v3::SBOL_SOURCE),
            v2::SBOL2_FORMAT => Some(v3::SBOL_FORMAT),
            v2::SBOL2_SIZE => Some(v3::SBOL_SIZE),
            v2::SBOL2_HASH => Some(v3::SBOL_HASH),
            v2::SBOL2_HASH_ALGORITHM => Some(v3::SBOL_HASH_ALGORITHM),
            v2::SBOL2_LANGUAGE => Some(v3::SBOL_LANGUAGE),
            v2::SBOL2_FRAMEWORK => Some(v3::SBOL_FRAMEWORK),
            v2::SBOL2_START => Some(v3::SBOL_START),
            v2::SBOL2_END => Some(v3::SBOL_END),
            v2::SBOL2_AT => Some(v3::SBOL_AT),
            v2::SBOL2_SEQUENCE_PROP => Some(v3::SBOL_HAS_SEQUENCE),
            v2::SBOL2_SEQUENCE_ANNOTATION_PROP => Some(v3::SBOL_HAS_FEATURE),
            v2::SBOL2_SEQUENCE_CONSTRAINT_PROP => Some(v3::SBOL_HAS_CONSTRAINT),
            v2::SBOL2_COMPONENT_PROP => Some(v3::SBOL_HAS_FEATURE),
            v2::SBOL2_FUNCTIONAL_COMPONENT_PROP => Some(v3::SBOL_HAS_FEATURE),
            v2::SBOL2_MODULE_PROP => Some(v3::SBOL_HAS_FEATURE),
            v2::SBOL2_INTERACTION_PROP => Some(v3::SBOL_HAS_INTERACTION),
            v2::SBOL2_PARTICIPATION_PROP => Some(v3::SBOL_HAS_PARTICIPATION),
            v2::SBOL2_LOCATION_PROP => Some(v3::SBOL_HAS_LOCATION),
            v2::SBOL2_DEFINITION => Some(v3::SBOL_INSTANCE_OF),
            v2::SBOL2_VARIABLE_COMPONENT_PROP => Some(v3::SBOL_HAS_VARIABLE_FEATURE),
            v2::SBOL2_OPERATOR => Some(v3::SBOL_CARDINALITY),
            v2::SBOL2_VARIABLE => Some(v3::SBOL_VARIABLE),
            v2::SBOL2_VARIANT => Some(v3::SBOL_VARIANT),
            v2::SBOL2_VARIANT_COLLECTION => Some(v3::SBOL_VARIANT_COLLECTION),
            v2::SBOL2_VARIANT_DERIVATION => Some(v3::SBOL_VARIANT_DERIVATION),
            v2::SBOL2_MODEL_PROP => Some(v3::SBOL_HAS_MODEL),
            v2::SBOL2_ATTACHMENT_PROP => Some(v3::SBOL_HAS_ATTACHMENT),
            v2::SBOL2_RESTRICTION => Some(v3::SBOL_RESTRICTION),
            v2::SBOL2_SUBJECT => Some(v3::SBOL_SUBJECT),
            v2::SBOL2_OBJECT => Some(v3::SBOL_OBJECT),
            v2::SBOL2_PARTICIPANT => Some(v3::SBOL_PARTICIPANT),
            v2::SBOL2_STRATEGY => Some(v3::SBOL_STRATEGY),
            v2::SBOL2_TEMPLATE => Some(v3::SBOL_TEMPLATE),
            v2::SBOL2_MEMBER => Some(v3::SBOL_MEMBER),
            v2::SBOL2_EXPERIMENTAL_DATA_PROP => Some(v3::SBOL_MEMBER),
            _ => None,
        };

        if let Some(target) = renamed {
            let mut object_with_value_rewrites =
                self.rewrite_value(triple.predicate.as_str(), &object);

            // When a Location's IRI got disambiguated during the
            // SA-collapse rewrite, override the displayId literal so it
            // matches the new IRI's last segment. Otherwise the
            // rewritten Location would emit `<...range1_2> sbol3:displayId
            // "range1"` and fail sbol3-10204.
            if predicate == v2::SBOL2_DISPLAY_ID
                && let Some(subject_iri) = triple.subject.as_iri()
                && let Some(override_did) =
                    self.location_display_id_overrides.get(subject_iri.as_str())
            {
                object_with_value_rewrites =
                    Term::Literal(sbol_rdf::Literal::simple(override_did.clone()));
            }

            // Preserve the original BioPAX URI when the value got
            // collapsed (BIOPAX_DNA and BIOPAX_DNA_REGION share an SBO
            // term, etc.). Without this hint a downgrade has to pick
            // one variant by convention and round-trip drifts.
            let biopax_original = (predicate == v2::SBOL2_TYPE)
                .then(|| object.as_iri().map(|i| i.as_str().to_owned()))
                .flatten()
                .filter(|iri| values::map_biopax_type(iri).is_some());

            self.output_triples.push(Triple {
                subject: subject.clone(),
                predicate: Iri::from_static(target),
                object: object_with_value_rewrites,
            });
            if self.options.preserve_backport
                && let Some(original) = biopax_original
            {
                self.output_triples.push(Triple {
                    subject,
                    predicate: Iri::from_static(v2::BACKPORT_BIOPAX_TYPE),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(original))),
                });
            }
            return;
        }

        // Encoding gets a value rewrite as part of the predicate rewrite.
        if predicate == v2::SBOL2_ENCODING {
            let rewritten = self.rewrite_value(predicate, &object);
            self.output_triples.push(Triple {
                subject,
                predicate: Iri::from_static(v3::SBOL_ENCODING),
                object: rewritten,
            });
            return;
        }

        // Orientation is renamed and value-rewritten.
        if predicate == v2::SBOL2_ORIENTATION {
            let rewritten = self.rewrite_value(predicate, &object);
            self.output_triples.push(Triple {
                subject,
                predicate: Iri::from_static(v3::SBOL_ORIENTATION),
                object: rewritten,
            });
            return;
        }

        // Persistent identity and version are archived under the backport
        // namespace if requested.
        if self.options.preserve_backport {
            let backport: Option<&'static str> = match predicate {
                v2::SBOL2_PERSISTENT_IDENTITY => Some(v2::BACKPORT_SBOL2_PERSISTENT_IDENTITY),
                v2::SBOL2_VERSION => Some(v2::BACKPORT_SBOL2_VERSION),
                _ => None,
            };
            if let Some(target) = backport {
                self.output_triples.push(Triple {
                    subject,
                    predicate: Iri::from_static(target),
                    object,
                });
                return;
            }
        }

        // Any other `sbol2:*` triple that reaches this point is one we don't
        // have a structured SBOL 3 rewrite for (e.g. `sbol2:access`,
        // `sbol2:direction`, or any future / custom extension predicate in
        // the SBOL 2 namespace that we haven't recognized). Preserve it
        // under the backport namespace so callers don't lose data, and so
        // an eventual SBOL 3 → SBOL 2 downconverter can reconstruct it.
        // Anything fully absorbed by a structural transform (MapsTo's
        // `local`/`remote`/`refinement`, collapsed-SA `component`,
        // `mapsTo` itself) has already been short-circuited above.
        if self.options.preserve_backport && predicate.starts_with(v2::SBOL2_NS) {
            let local = &predicate[v2::SBOL2_NS.len()..];
            let preserved = format!("{}{local}", v2::BACKPORT_SBOL2_PREFIX);
            self.output_triples.push(Triple {
                subject,
                predicate: Iri::new_unchecked(preserved),
                object,
            });
        }
    }

    /// Handles the SA-with-component collapse. When a SequenceAnnotation
    /// references a `sbol2:component`, the SA shell is discarded and its
    /// Locations migrate onto that SubComponent. Returns `true` when the
    /// triple was consumed by this rule; the caller then skips its default
    /// predicate handling.
    fn preserve_collapsed_sa_metadata(&mut self, triple: &Triple, target_subcomponent: &str) {
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

    fn try_collapse_sa_triple(&mut self, triple: &Triple) -> bool {
        let predicate = triple.predicate.as_str();
        let subject_iri = match triple.subject.as_iri() {
            Some(iri) => iri.as_str(),
            None => return false,
        };

        // Drop the parent CD's `sbol2:sequenceAnnotation` triple if it
        // points at a collapsed SA — the SubComponent is already attached as
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

        // `sbol2:component` itself is consumed by the collapse — the
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
    fn rewrite_value(&self, predicate_str: &str, object: &Term) -> Term {
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

    fn emit_synthesized_triples(&mut self) {
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

        // Promote `dcterms:title` → `sbol3:name` and
        // `dcterms:description` → `sbol3:description` for any subject
        // that has them. We don't drop the originals — sbolgraph
        // overwrites them, but keeping them preserves Dublin Core
        // metadata that SPARQL clients might query. Scan
        // `output_triples` (not the input) so dcterms attached to
        // structurally-consumed subjects — e.g. collapsed
        // SequenceAnnotation shells — aren't reintroduced as orphan
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
    fn synthesize_interfaces(&mut self) {
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
    fn synthesize_mapsto_decomposition(&mut self) {
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
            // refinement IRIs. We emit it for every refinement anyway —
            // it's defense-in-depth against downstream tools that swap
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

    fn emit_component_reference(
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

    fn emit_constraint(
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
    fn infer_location_sequences(&mut self) {
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

    fn fallback_namespace(&mut self, sbol2_iri: &str) -> Option<String> {
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

fn is_top_level_sbol2(sbol2_type: &str) -> bool {
    matches!(
        sbol2_type,
        v2::SBOL2_COMPONENT_DEFINITION
            | v2::SBOL2_MODULE_DEFINITION
            | v2::SBOL2_SEQUENCE
            | v2::SBOL2_MODEL
            | v2::SBOL2_COLLECTION
            | v2::SBOL2_IMPLEMENTATION
            | v2::SBOL2_ATTACHMENT
            | v2::SBOL2_EXPERIMENT
            | v2::SBOL2_EXPERIMENTAL_DATA
            | v2::SBOL2_COMBINATORIAL_DERIVATION
    ) || is_prov_top_level(sbol2_type)
}

/// PROV classes that act as SBOL 3 top-level objects (they appear at the
/// document root and need `sbol3:hasNamespace`). PROV `Association` and
/// `Usage` are intentionally absent — those are children of `Activity`.
fn is_prov_top_level(type_iri: &str) -> bool {
    matches!(
        type_iri,
        v3::PROV_ACTIVITY | v3::PROV_AGENT_CLASS | v3::PROV_PLAN
    )
}

fn type_precedence(type_iri: &str) -> u8 {
    if is_known_sbol2_type(type_iri) || is_prov_top_level(type_iri) {
        2
    } else if type_iri.starts_with(v2::SBOL2_NS) {
        1
    } else {
        0
    }
}

fn is_known_sbol2_type(type_iri: &str) -> bool {
    matches!(
        type_iri,
        v2::SBOL2_COMPONENT_DEFINITION
            | v2::SBOL2_MODULE_DEFINITION
            | v2::SBOL2_COMPONENT
            | v2::SBOL2_MODULE
            | v2::SBOL2_FUNCTIONAL_COMPONENT
            | v2::SBOL2_SEQUENCE_ANNOTATION
            | v2::SBOL2_SEQUENCE_CONSTRAINT
            | v2::SBOL2_SEQUENCE
            | v2::SBOL2_MODEL
            | v2::SBOL2_INTERACTION
            | v2::SBOL2_PARTICIPATION
            | v2::SBOL2_COLLECTION
            | v2::SBOL2_IMPLEMENTATION
            | v2::SBOL2_ATTACHMENT
            | v2::SBOL2_EXPERIMENT
            | v2::SBOL2_EXPERIMENTAL_DATA
            | v2::SBOL2_COMBINATORIAL_DERIVATION
            | v2::SBOL2_VARIABLE_COMPONENT
            | v2::SBOL2_RANGE
            | v2::SBOL2_CUT
            | v2::SBOL2_GENERIC_LOCATION
            | v2::SBOL2_MAPS_TO
    )
}

use crate::iri_util::last_iri_segment as last_path_segment;

/// Returns a child `(display_id, iri)` under `parent` whose IRI is not
/// already in `used`, inserting the chosen IRI. Disambiguates by appending
/// `_2`, `_3`, … and keeps displayId aligned with the child IRI.
fn next_available_child_iri(
    parent: &str,
    base_display_id: &str,
    used: &mut HashSet<String>,
) -> (String, String) {
    let mut counter: usize = 1;
    loop {
        let display_id = if counter == 1 {
            base_display_id.to_owned()
        } else {
            format!("{base_display_id}_{counter}")
        };
        let iri = format!("{parent}/{display_id}");
        if used.insert(iri.clone()) {
            return (display_id, iri);
        }
        counter += 1;
    }
}

/// Returns a `(display_id, cref_iri, constraint_display_id, constraint_iri)`
/// quadruple whose IRIs aren't already in `used` and inserts them. Disambiguates
/// repeated displayIds by appending `_2`, `_3`, … starting from the second
/// hit. The displayId carries the same numbering so SBOL 3 IRI compliance
/// (`{namespace}/{displayId}`) holds.
fn next_available_mapsto_iris(
    top_level: &str,
    base_display_id: &str,
    used: &mut HashSet<String>,
) -> (String, String, String, String) {
    let mut counter: usize = 1;
    loop {
        let display_id = if counter == 1 {
            base_display_id.to_owned()
        } else {
            format!("{base_display_id}_{counter}")
        };
        let cref_iri = format!("{top_level}/{display_id}");
        let constraint_display_id = format!("{display_id}_constraint");
        let constraint_iri = format!("{top_level}/{constraint_display_id}");
        if !used.contains(&cref_iri) && !used.contains(&constraint_iri) {
            used.insert(cref_iri.clone());
            used.insert(constraint_iri.clone());
            return (display_id, cref_iri, constraint_display_id, constraint_iri);
        }
        counter += 1;
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Canonical N-Triples line for a single triple, used by the snapshot
/// regenerator and the conformance test so both sides agree on formatting.
/// `xsd:string`-typed literals are emitted in bare form (matching
/// sbol-utilities and the N-Triples 1.1 convention) so plain-string
/// divergences don't masquerade as data drift. Embedded control
/// characters in literals are escaped per the N-Triples grammar
/// (`\n`, `\r`, `\t`, `\\`, `\"`) so each triple stays on a single line
/// for diff-friendly snapshot files.
pub fn canonical_nt_line(triple: &Triple) -> String {
    const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
    // `Resource` and `Term` are `#[non_exhaustive]`, so the match
    // requires a fall-through arm. A future variant should fail the
    // snapshot loudly rather than silently produce invalid N-Triples;
    // bumping the underlying RDF crate must come with explicit
    // handling here.
    let subject = match &triple.subject {
        Resource::Iri(iri) => format!("<{}>", iri.as_str()),
        Resource::BlankNode(b) => format!("_:{}", b.as_str()),
        other => unreachable!("unhandled Resource variant: {other:?}"),
    };
    let predicate = format!("<{}>", triple.predicate.as_str());
    let object = match &triple.object {
        Term::Resource(Resource::Iri(iri)) => format!("<{}>", iri.as_str()),
        Term::Resource(Resource::BlankNode(b)) => format!("_:{}", b.as_str()),
        Term::Literal(literal) => {
            let escaped = escape_nt_string(literal.value());
            if let Some(lang) = literal.language() {
                format!("\"{escaped}\"@{lang}")
            } else if literal.datatype().as_str() == XSD_STRING {
                format!("\"{escaped}\"")
            } else {
                format!("\"{escaped}\"^^<{}>", literal.datatype().as_str())
            }
        }
        other => unreachable!("unhandled Term variant: {other:?}"),
    };
    format!("{subject} {predicate} {object} .")
}

fn escape_nt_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // N-Triples 1.1 forbids unescaped control characters and
            // DEL in literals. Emit `\uXXXX` / `\UXXXXXXXX` for anything
            // in the C0/C1 control ranges and U+007F.
            c if (c as u32) < 0x20 || c as u32 == 0x7F => {
                out.push_str(&format!("\\u{:04X}", c as u32));
            }
            _ => out.push(c),
        }
    }
    out
}

fn url_origin(iri: &str) -> Option<String> {
    // URL form: `scheme://host/path` → `scheme://host`.
    if let Some((scheme, rest)) = iri.split_once("://") {
        let authority_end = rest.find('/').unwrap_or(rest.len());
        let host = &rest[..authority_end];
        if !scheme.is_empty() && !host.is_empty() {
            return Some(format!("{scheme}://{host}"));
        }
    }

    // URN form (RFC 8141): `urn:nid:nss` → `urn:nid`. Treat the namespace
    // identifier as the origin so URN-style IRIs get a stable
    // `hasNamespace` even when persistentIdentity+displayId derivation
    // fails. Tolerate either `:` or `/` between nid and nss.
    if let Some(after_urn) = iri.strip_prefix("urn:")
        && !after_urn.is_empty()
    {
        let nid_end = after_urn.find([':', '/']).unwrap_or(after_urn.len());
        let nid = &after_urn[..nid_end];
        if !nid.is_empty() {
            return Some(format!("urn:{nid}"));
        }
    }
    None
}
