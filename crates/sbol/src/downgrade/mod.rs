//! Convert SBOL 3 documents to SBOL 2 RDF.
//!
//! Most published synbio content (SynBioHub, iGEM Registry, JBEI ICE
//! today) still consumes SBOL 2. This module is the write-direction
//! counterpart to [`crate::upgrade`]: it takes an SBOL 3
//! [`Document`](crate::Document) and produces an [`RdfGraph`] holding
//! the equivalent SBOL 2 RDF, which the caller can serialize in any
//! of the supported RDF formats.
//!
//! ```no_run
//! use sbol::{Document, RdfFormat};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let document = Document::read_path("design.ttl")?;
//! let (sbol2_graph, report) = document.downgrade_to_sbol2()?;
//! for warning in report.warnings() {
//!     eprintln!("{warning:?}");
//! }
//! let sbol2_xml = sbol2_graph.write(RdfFormat::RdfXml)?;
//! std::fs::write("design.xml", sbol2_xml)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Loss model
//!
//! SBOL 3 unifies several SBOL 2 concepts (notably `ComponentDefinition`
//! and `ModuleDefinition`) into a single `Component`. The downgrade is
//! therefore not strictly bijective; the [`DowngradeReport`] surfaces
//! warnings for cases where the SBOL 2 surface forces a representation
//! choice (dual-role Component split, MapsTo reconstruction from
//! `ComponentReference`+`Constraint` pairs, etc.).
//!
//! Documents produced via [`crate::upgrade::sbol2_to_sbol3`] round-trip
//! losslessly because the upgrade preserves SBOL 2 provenance under
//! the `http://sboltools.org/backport#` namespace; the downgrade reads
//! those triples to restore the original SBOL 2 identities and types.
//! Documents authored as native SBOL 3 will lose more on downgrade —
//! the report explains where.
//!
//! # Dual-role Components
//!
//! SBOL 3 lets one `Component` carry both structural data (sequence,
//! sub-parts) and functional data (interactions, an interface). SBOL 2
//! splits these concerns across `ComponentDefinition` and
//! `ModuleDefinition`. When a downgraded Component carries both, this
//! module emits BOTH classes plus a synthesized `FunctionalComponent`
//! linking them, and pushes a [`DowngradeWarning::DualRoleComponent`]
//! into the report. Classification respects `backport:sbol2type` when
//! present (so SBOL 2 → 3 → 2 round-trips stay single-shape); the
//! split only fires for native SBOL 3 designs that genuinely combine
//! the two concerns.
//!
//! For the full conversion model — the backport namespace, structural
//! collapses, dual-role classification rules, known divergences,
//! known limitations — see the [conversion guide][conversion-md].
//!
//! [`RdfGraph`]: crate::RdfGraph
//! [conversion-md]: https://github.com/marpaia/sbol-rs/blob/master/docs/conversion.md

use std::collections::{HashMap, HashSet};

use crate::sbol2_vocab as v2;
use crate::vocab as v3;
use crate::{Document, Iri, Resource, Term, Triple};
use sbol_rdf::Graph;

mod values;

/// Configuration for [`sbol3_to_sbol2`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct DowngradeOptions {
    /// Version string to assign top-level objects whose source did not
    /// carry `backport:sbol2version`. `None` (the default) leaves them
    /// unversioned — SBOL 2 makes `sbol2:version` optional, and the
    /// round-trip stays bit-identical for sources that omitted it.
    /// `Some("1")` matches the libSBOLj / SynBioHub convention of
    /// always carrying a version segment.
    pub default_version: Option<String>,

    /// When a [`Component`](crate::Component) carries both
    /// `hasSequence` and `hasInteraction`, the unified SBOL 3 view
    /// can't be represented as a single SBOL 2 object. With this
    /// option `true` (the default) the downgrade emits BOTH a
    /// `ComponentDefinition` (for the structural side) AND a
    /// `ModuleDefinition` (for the functional side), linked via a
    /// synthesized `FunctionalComponent`. A
    /// [`DowngradeWarning::DualRoleComponent`] is emitted for every
    /// such split so the choice is visible in the report. When this is
    /// `false`, the Component is emitted as one SBOL 2 object; functional
    /// signals win over structural signals, so dual-role Components collapse
    /// to `ModuleDefinition`.
    pub split_dual_role_components: bool,
}

impl Default for DowngradeOptions {
    fn default() -> Self {
        Self {
            default_version: None,
            split_dual_role_components: true,
        }
    }
}

impl DowngradeOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Non-fatal observations from [`sbol3_to_sbol2`]. Each warning
/// records a case where the SBOL 2 representation forced a choice
/// the SBOL 3 source didn't have to make.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DowngradeWarning {
    /// A `Component` carried both `hasSequence` and `hasInteraction`.
    /// With [`DowngradeOptions::split_dual_role_components`] the
    /// downgrade split it into a `ComponentDefinition` plus a
    /// `ModuleDefinition`; the synthesized half receives the same
    /// `displayId` with `_component` or `_module` appended.
    DualRoleComponent {
        component: String,
        component_definition: String,
        module_definition: String,
    },
    /// A `Constraint` couldn't be folded back into a `MapsTo` because
    /// its `subject` or `object` didn't match the expected
    /// `ComponentReference` pattern. The constraint is emitted as a
    /// plain SBOL 2 `SequenceConstraint` in the output.
    UnresolvableConstraintToMapsTo { constraint: String, reason: String },
    /// A `ComponentReference` didn't have a matching `Constraint`
    /// partner. The reference is dropped from the SBOL 2 output —
    /// SBOL 2 has no standalone equivalent.
    OrphanComponentReference { component_reference: String },
    /// A subject of `rdf:type sbol3:T` had no SBOL 2 equivalent for
    /// `T`. The subject and its triples are dropped from the output.
    UnsupportedSbol3Type { subject: String, sbol3_type: String },
    /// A top-level object had no `backport:sbol2version` and
    /// [`DowngradeOptions::default_version`] was set, so the downgrade
    /// synthesized this version on it. Only fires when synthesis is
    /// opted into.
    SynthesizedVersion { subject: String, version: String },
    /// Two or more distinct SBOL 3 subjects rewrite to the same SBOL 2
    /// versioned IRI — for example a Component at `<lab/foo>` carrying
    /// `backport:sbol2version "1"` (rewritten to `<lab/foo/1>`) and a
    /// separate Component at `<lab/foo/1>` carrying no preserved
    /// version (rewritten to `<lab/foo/1>` unchanged). The conversion
    /// proceeds and every input triple is preserved, but they all land
    /// at the same SBOL 2 subject, silently merging the entities into
    /// a single chimeric ComponentDefinition / ModuleDefinition / etc.
    /// The input is technically non-conformant (the implied SBOL 2
    /// versioned identities should be unique), but the merge is
    /// otherwise invisible; this warning surfaces the situation so
    /// callers can audit. Mirrors
    /// [`crate::upgrade::UpgradeWarning::IdentityCollision`].
    IdentityCollision {
        canonical: String,
        sources: Vec<String>,
    },
}

/// Report of every non-fatal issue plus tallies of what was rewritten.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct DowngradeReport {
    warnings: Vec<DowngradeWarning>,
    counts: DowngradeCounts,
}

impl DowngradeReport {
    pub fn warnings(&self) -> &[DowngradeWarning] {
        &self.warnings
    }

    pub fn counts(&self) -> &DowngradeCounts {
        &self.counts
    }

    pub fn is_clean(&self) -> bool {
        self.warnings.is_empty()
    }

    pub(crate) fn push(&mut self, warning: DowngradeWarning) {
        self.warnings.push(warning);
    }
}

/// Tally of how many SBOL 3 constructs were rewritten.
#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct DowngradeCounts {
    pub components_to_component_definition: usize,
    pub components_to_module_definition: usize,
    pub components_split_into_both: usize,
    pub sub_components_emitted: usize,
    pub sequence_features_emitted: usize,
    pub maps_to_reconstructed: usize,
    pub identities_restored_from_backport: usize,
    pub identities_synthesized: usize,
}

/// Errors returned by [`sbol3_to_sbol2`]. These are fatal — non-fatal
/// observations live on [`DowngradeReport`] instead.
#[derive(Debug)]
#[non_exhaustive]
pub enum DowngradeError {
    /// [`DowngradeOptions::default_version`] was set to `Some("")`.
    /// Use `None` to disable version synthesis or `Some(v)` with a
    /// non-empty value.
    InvalidDefaultVersion(String),
}

impl std::fmt::Display for DowngradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDefaultVersion(msg) => {
                write!(f, "invalid default version: {msg}")
            }
        }
    }
}

impl std::error::Error for DowngradeError {}

/// Converts an SBOL 3 [`Document`] to an [`RdfGraph`] of SBOL 2 RDF.
///
/// The caller chooses how to serialize the resulting graph
/// (`graph.write(RdfFormat::RdfXml)` / `Turtle` / …).
///
/// [`RdfGraph`]: crate::RdfGraph
pub fn sbol3_to_sbol2(
    document: &Document,
    options: DowngradeOptions,
) -> Result<(Graph, DowngradeReport), DowngradeError> {
    if matches!(options.default_version.as_deref(), Some("")) {
        return Err(DowngradeError::InvalidDefaultVersion(
            "default version must be non-empty when set; use None to disable synthesis".to_string(),
        ));
    }
    let mut engine = Engine::new(document, options);
    engine.preflight();
    engine.convert();
    Ok((Graph::new(engine.output_triples), engine.report))
}

impl Document {
    /// Downgrades this SBOL 3 document to SBOL 2 RDF, returning the
    /// resulting graph alongside a [`DowngradeReport`]. See
    /// [`sbol::downgrade`](crate::downgrade) for the loss model.
    pub fn downgrade_to_sbol2(&self) -> Result<(Graph, DowngradeReport), DowngradeError> {
        sbol3_to_sbol2(self, DowngradeOptions::default())
    }

    /// Like [`Document::downgrade_to_sbol2`], with explicit
    /// [`DowngradeOptions`].
    pub fn downgrade_to_sbol2_with(
        &self,
        options: DowngradeOptions,
    ) -> Result<(Graph, DowngradeReport), DowngradeError> {
        sbol3_to_sbol2(self, options)
    }
}

struct Engine<'a> {
    input: &'a Document,
    options: DowngradeOptions,
    /// Subject IRI → version string (from `backport:sbol2version`, or
    /// the synthesized default). Determines the `/version` suffix
    /// added back to top-level identities.
    versions: HashMap<String, String>,
    /// Subjects whose version came from an explicit `backport:sbol2version`
    /// triple in the source document, rather than being synthesized by
    /// [`Engine::version_for`]. Only these get a `sbol2:version` property
    /// triple emitted in the SBOL 2 output — synthesizing the IRI segment
    /// is necessary for SBOL 2 structure, but emitting a fake version
    /// triple would pollute round-trips of documents that originally
    /// carried no `sbol2:version` (the SBOL 2 spec makes it optional).
    preserved_versions: HashSet<String>,
    /// Subject IRI → original SBOL 2 persistent identity (from
    /// `backport:sbol2persistentIdentity`).
    persistent_identities: HashMap<String, String>,
    /// Subject IRI → original SBOL 2 rdf:type IRI (from
    /// `backport:sbol2type`). When present, the downgrade uses this
    /// instead of inferring the type from SBOL 3 shape.
    backport_types: HashMap<String, String>,
    /// Subject IRI → set of original BioPAX type IRIs (from
    /// `backport:biopaxType`). Preserves which BioPAX variant
    /// (`Dna` vs `DnaRegion`, etc.) the SBOL 2 source carried so the
    /// downgrade restores it instead of always picking the `*Region`
    /// form for SBO terms that collapse on the way up.
    ///
    /// The actual triple-by-triple resolution lives in
    /// [`Engine::biopax_variant_queue`] / [`Engine::biopax_variant_cursor`];
    /// this set is the raw input to that precomputation.
    backport_biopax_types: HashMap<String, HashSet<String>>,
    /// `(subject, sbo_term)` → ordered list of preserved BioPAX variant
    /// IRIs that map to that SBO term. Derived from
    /// [`Engine::backport_biopax_types`] in preflight.
    ///
    /// Allows multiple distinct BioPAX variants under the *same* SBO
    /// target (e.g. a Component carrying both `biopax:Dna` and
    /// `biopax:DnaRegion`, both of which collapse to `SBO:0000251`) to
    /// round-trip distinctly: each input `sbol3:type` triple consumes
    /// one variant from the head of its list via
    /// [`Engine::biopax_variant_cursor`].
    biopax_variant_queue: HashMap<(String, String), Vec<String>>,
    /// `(subject, sbo_term)` → index of the next BioPAX variant to
    /// consume from [`Engine::biopax_variant_queue`]. Each
    /// reverse-mapped `sbol3:type` triple advances the cursor by 1;
    /// once it exceeds the queue length the resolver falls back to the
    /// default `*Region`-style mapping.
    biopax_variant_cursor: HashMap<(String, String), usize>,
    /// Subject IRI → primary resolved SBOL 2 type. Combines `backport_types`
    /// with the default SBOL 3→SBOL 2 type table. Used by phase 3 to
    /// disambiguate context-dependent predicates (e.g. `hasFeature`
    /// becomes `component` for CDs, `functionalComponent` for MDs).
    resolved_types: HashMap<String, String>,
    /// Subject IRI → every resolved SBOL 2 type asserted for the subject.
    /// Used when a context-sensitive predicate cares about an additional
    /// type rather than the primary one selected for structural routing.
    resolved_type_sets: HashMap<String, HashSet<String>>,
    /// SubComponent IRI → IRI of the Component it `instanceOf`s.
    /// Lets us decide whether a SubComponent under an MD should be
    /// emitted as `Module` (target is MD) or `FunctionalComponent`
    /// (target is CD).
    subcomponent_targets: HashMap<String, String>,
    /// Top-level subjects whose IRIs need a version suffix appended.
    top_levels: HashSet<String>,
    /// IRI rewrite map (SBOL 3 IRI → SBOL 2 IRI). Built during
    /// preflight; applied to every subject and object during the main
    /// pass.
    iri_rewrites: HashMap<String, String>,
    /// SubComponent IRI → info needed to reconstruct or synthesize an SBOL 2
    /// SequenceAnnotation wrapper. Populated for SubComponents that carry the
    /// upgrade's `backport:sequenceAnnotationDisplayId` hint, and for native
    /// SBOL 3 SubComponents with `hasLocation`.
    sa_collapses: HashMap<String, SaCollapseInfo>,
    /// ComponentReference IRI → reconstructed MapsTo. The upgrade
    /// decomposed each SBOL 2 MapsTo into a paired ComponentReference +
    /// Constraint; the downgrade walks them back into a single MapsTo
    /// attached to the inChildOf carrier.
    mapsto_reconstructions: HashMap<String, MapsToReconstruction>,
    /// Constraint IRI → set of triples to drop because the constraint
    /// is the back-half of a MapsTo decomposition (not a real SBOL 3
    /// Constraint).
    mapsto_constraints: HashSet<String>,
    /// Subjects whose triples must be dropped from the SBOL 2 output
    /// because they are SBOL 3-only structural plumbing the downgrade
    /// recognized but could not fold. Without suppression these would
    /// survive as orphan subjects carrying SBOL 3 predicates — e.g.
    /// ComponentReferences whose paired Constraint was missing fields,
    /// which couldn't be reconstructed into a MapsTo.
    discarded_subjects: HashSet<String>,
    /// FunctionalComponent (SBOL 3 SubComponent) IRI → direction
    /// recovered from the enclosing Interface's
    /// input/output/nondirectional triples.
    fc_directions: HashMap<String, FcDirection>,
    /// FunctionalComponent subjects whose original SBOL 2 direction was
    /// restored from `backport:sbol2_direction`. Interface-derived direction
    /// emission must skip these subjects to avoid writing a contradictory
    /// second `sbol2:direction` triple.
    restored_fc_directions: HashSet<String>,
    /// Interface IRIs to drop from the output (their data is folded
    /// into per-FC `sbol2:direction` triples).
    interface_subjects: HashSet<String>,
    /// Per-Component split decision. SBOL 2 separates the structural
    /// (ComponentDefinition) and functional (ModuleDefinition) concerns
    /// that SBOL 3 unifies into one [`Component`]. The downgrade
    /// classifies each Component into a shape and, for dual-role
    /// Components, emits both halves linked by a synthesized
    /// FunctionalComponent.
    component_splits: HashMap<String, ComponentSplit>,
    /// SBOL 3 SubComponent IRI → its triple-emitted SBOL 2 variants when
    /// it lives under a dual-role parent. For SubComponents under a
    /// single-shape parent the map is empty and the existing
    /// `handle_has_feature` path emits the single appropriate variant.
    subcomponent_splits: HashMap<String, SubComponentSplit>,
    /// SBOL 3 SubComponent IRI → the SBOL 2 IRI a Participation should
    /// point at when its participant is this SubComponent. For
    /// single-shape parents this is the SubComponent's own rewritten
    /// IRI; for dual-role parents it is the MD-side FunctionalComponent
    /// variant. Populated by `handle_dual_role_has_feature` during the
    /// main convert walk; consumed by `rewrite_participants` at the end
    /// of `convert` so all Participation triples already exist in
    /// `output_triples` by the time the rewrite runs.
    participant_remap: HashMap<String, String>,
    /// SBOL 3 SubComponent IRI → SBOL 3 Component IRI of its enclosing
    /// parent (built from the input `sbol3:hasFeature` index in
    /// preflight). Lets us look up the parent's split shape.
    feature_parent: HashMap<String, String>,
    /// Every IRI the downgrade has either observed in the input graph
    /// or allocated as a synthesized SBOL 2 subject so far. Every
    /// IRI-synthesis site (linking FC, SubComponent triple-split
    /// variants, dual-role CD/MD halves, SA wrapper, reconstructed
    /// MapsTo) routes its candidate IRI through one of the
    /// `next_available_*` helpers against this set. The invariant the
    /// pool enforces is: **no two distinct SBOL 2 entities ever land at
    /// the same IRI**, regardless of how creatively the input names
    /// things. Without it, every synthesis site is a potential
    /// silent-merge bug.
    used_iris: HashSet<String>,
    /// (subject IRI, dcterms predicate) → set of objects already present
    /// in the input graph. Used to suppress duplicate emission of
    /// `dcterms:title` / `dcterms:description` when both Dublin Core
    /// and SBOL 3 forms (`sbol3:name` / `sbol3:description`) exist for
    /// the same value — without this the downgrade would emit each
    /// dcterms triple twice. O(1) lookup; the equivalent scan in
    /// `subject_already_has` was O(N) per call.
    dcterms_index: HashMap<(String, &'static str), HashSet<Term>>,
    output_triples: Vec<Triple>,
    report: DowngradeReport,
}

/// Whether a Component downgrades to a single SBOL 2 class or splits
/// into a `ComponentDefinition` + `ModuleDefinition` pair joined by a
/// synthesized `FunctionalComponent`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComponentShape {
    /// Maps to a single `sbol2:ComponentDefinition`.
    CdOnly,
    /// Maps to a single `sbol2:ModuleDefinition`.
    MdOnly,
    /// Splits into both a CD and an MD. The Component's structural data
    /// (`sbol3:type`, `sbol3:role`, `sbol3:hasSequence`, SequenceFeature
    /// children, Constraints) goes onto the CD; functional data
    /// (`sbol3:hasInteraction`, `sbol3:hasModel`, interfaces) goes onto
    /// the MD; a `sbol2:FunctionalComponent` is synthesized to link the
    /// MD's functional view to the CD's structural one.
    DualRole,
}

#[derive(Clone)]
/// Per-Component IRI/displayId assignments built during preflight.
///
/// For [`ComponentShape::CdOnly`] and [`ComponentShape::MdOnly`] the CD
/// and MD IRIs both equal the Component's own SBOL 2 IRI — the
/// non-applicable half is never emitted. For [`ComponentShape::DualRole`]
/// the two IRIs differ: whichever half the SBOL 2 source originally was
/// (per `backport:sbol2type`) keeps the bare IRI; the synthesized half
/// gets a `_component` or `_module` suffix.
struct ComponentSplit {
    shape: ComponentShape,
    /// SBOL 2 IRI of the CD half (versioned via `iri_rewrites`).
    cd_iri: String,
    /// SBOL 2 IRI of the MD half.
    md_iri: String,
    /// SBOL 2 IRI of the synthesized linking FunctionalComponent (only
    /// populated for [`ComponentShape::DualRole`]).
    linking_fc_iri: Option<String>,
    /// `displayId` for the synthesized linking FunctionalComponent. The
    /// canonical case is the parent's displayId; when that IRI would
    /// collide with an existing child (e.g. a SubComponent named after
    /// its parent) the IRI gets a `_2` / `_3` / … suffix and this
    /// displayId carries the same suffix so SBOL 2 IRI compliance
    /// (`displayId` == last segment of `persistentIdentity`) holds.
    linking_fc_display_id: Option<String>,
    /// Suffix applied to the CD's `displayId` (`""` for the half that
    /// kept the original identity, `"_component"` for the synthesized
    /// half).
    cd_display_suffix: &'static str,
    /// Suffix applied to the MD's `displayId`.
    md_display_suffix: &'static str,
    /// Original SBOL 3 displayId (used to construct the synthesized FC's
    /// displayId).
    original_display_id: String,
}

#[derive(Clone)]
/// A SubComponent under a [`ComponentShape::DualRole`] parent triples
/// into three SBOL 2 objects:
///
/// - a `sbol2:Component` under the CD half
/// - a `sbol2:FunctionalComponent` under the MD half
/// - a `sbol2:Module` under the MD half (only when the SubComponent's
///   target is itself an MD)
///
/// Whichever variant matches the SubComponent's original SBOL 2 class
/// (per `backport:sbol2type`) keeps the bare IRI; the others get `_c` /
/// `_fc` / `_m` suffixes. Suffixed variants go through
/// [`next_available_child_iri`] against the pass-wide `used_iris`
/// set, so a sibling SubComponent already at e.g. `parent/foo_fc`
/// pushes `foo`'s FC variant to `foo_fc_2` rather than overwriting
/// the sibling's triples.
struct SubComponentSplit {
    /// SBOL 2 IRI for the CD-side `sbol2:Component` variant.
    component_iri: String,
    /// SBOL 2 IRI for the MD-side `sbol2:FunctionalComponent` variant.
    functional_component_iri: String,
    /// SBOL 2 IRI for the MD-side `sbol2:Module` variant. Only set when
    /// the SubComponent's target is itself an MD-shaped Component.
    module_iri: Option<String>,
}

/// Direction recovered from an SBOL 3 Interface for an enclosed
/// FunctionalComponent SubComponent.
#[derive(Clone, Copy, Debug)]
enum FcDirection {
    In,
    Out,
    Inout,
    NoneDirection,
}

#[derive(Clone, Copy)]
enum InterfaceFeatureKind {
    Component,
    FunctionalComponent,
    Module,
}

impl FcDirection {
    fn sbol2_iri(self) -> &'static str {
        match self {
            FcDirection::In => v2::SBOL2_DIRECTION_IN,
            FcDirection::Out => v2::SBOL2_DIRECTION_OUT,
            FcDirection::Inout => v2::SBOL2_DIRECTION_INOUT,
            FcDirection::NoneDirection => v2::SBOL2_DIRECTION_NONE,
        }
    }
}

/// All the data the downgrade needs to re-emit an SBOL 2 MapsTo from
/// the ComponentReference + Constraint pair the upgrade produced.
struct MapsToReconstruction {
    /// SBOL 2 IRI of the carrier SubComponent (Module / FunctionalComponent)
    /// the MapsTo hangs off of in SBOL 2.
    carrier_v3: String,
    /// MapsTo's `displayId` (preserved on the ComponentReference).
    display_id: String,
    /// MapsTo `local` value — the SBOL 3 IRI of the FC at the same
    /// level as the carrier, recovered from the Constraint's `subject`.
    local_v3: String,
    /// MapsTo `remote` value — the SBOL 3 IRI of the FC inside the
    /// carrier's instanceOf target, recovered from the
    /// ComponentReference's `refersTo`.
    remote_v3: String,
    /// MapsTo `refinement` value. Resolved in priority order:
    ///
    /// 1. Explicit `backport:mapsToRefinement` hint on the
    ///    ComponentReference. Lossless for `merge` and for any
    ///    refinement IRI position alone can't encode.
    /// 2. Otherwise, position-aware inference from the Constraint's
    ///    restriction plus the CRef's position in the
    ///    `sbol3:subject` / `sbol3:object` pair per SBOL 3.1.0 §10.2:
    ///    - `replaces` + CRef in subject → `useRemote`
    ///    - `replaces` + CRef in object  → `useLocal`
    ///    - `verifyIdentical` + CRef in either → `verifyIdentical`
    /// 3. `None` when even position can't resolve it; the emitter
    ///    then synthesizes `useLocal` so no data is lost.
    refinement: Option<String>,
}

#[derive(Clone)]
struct PreservedSaTriple {
    predicate: String,
    object: Term,
}

/// Records what the downgrade needs to know about a SubComponent whose
/// Location-bearing SBOL 3 shape needs an SBOL 2 SequenceAnnotation wrapper.
/// This covers both SubComponents that originated from an SBOL 2
/// SequenceAnnotation collapse and native SBOL 3 SubComponents with
/// `hasLocation`.
struct SaCollapseInfo {
    /// `displayId` the SBOL 2 SequenceAnnotation should carry. For
    /// round-tripped SBOL 2 this is preserved by the upgrade as
    /// `backport:sequenceAnnotationDisplayId`; for native SBOL 3 it is
    /// synthesized from the SubComponent displayId.
    sa_display_id: String,
    /// SBOL 2 IRI of the reconstructed SequenceAnnotation, minus the
    /// version suffix. Derived as `{parent_cd_unversioned}/{sa_display_id}` or
    /// collision-disambiguated for native SBOL 3 inputs.
    sa_iri_unversioned: String,
    /// Original SBOL 3 IRI of the parent Component. Used to inherit the
    /// preserved/synthesized top-level version even when `parent_cd` is a
    /// synthetic dual-role split half.
    parent_component: String,
    /// SBOL 3 IRI of the CD half that owns the SequenceAnnotation in the SBOL
    /// 2 output. Usually equals `parent_component`; for native dual-role
    /// Components it may be the synthesized `_component` half.
    parent_cd: String,
    /// SBOL 3 IRIs of every Location attached to the SubComponent via
    /// `hasLocation`. The reconstructed SA emits `sbol2:location` to each.
    locations: Vec<String>,
    /// Non-structural triples archived from the collapsed SBOL 2 SA shell.
    preserved_metadata: Vec<PreservedSaTriple>,
}

impl<'a> Engine<'a> {
    fn new(input: &'a Document, options: DowngradeOptions) -> Self {
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
    fn preflight(&mut self) {
        // Seed the used-IRI pool with every input subject IRI. Every
        // synthesis site downstream (classify_components, the SA wrapper
        // discovery, the MapsTo reconstruction emission) routes its
        // candidate IRIs through `next_available_*` against this pool,
        // so no synthesized SBOL 2 IRI can silently land on an existing
        // subject — the invariant the pool exists to enforce.
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
                // Sort for deterministic round-trips — HashSet's iteration
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

    fn build_iri_rewrites(&mut self) {
        // For every IRI that appears as a subject in the document,
        // compute its SBOL 2 form. The version segment is only appended
        // when the source carried a `backport:sbol2version` triple —
        // synthesizing one for documents that had no version originally
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
        // re-upgrade — which sees a real SA wrapper this time — collapses
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
    /// `backport:sbol2version` in the source — i.e. the SBOL 2 source
    /// carried an explicit version. Otherwise `None`, signalling that
    /// the version was not preserved and the caller must decide whether
    /// to synthesize one from [`DowngradeOptions::default_version`].
    fn preserved_version_for_top_level(&self, iri: &str) -> Option<String> {
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
    fn effective_version_for_top_level(&self, iri: &str) -> Option<String> {
        self.preserved_version_for_top_level(iri)
            .or_else(|| self.options.default_version.clone())
    }

    /// Returns the version for any subject — top-level or child —
    /// resolved via the owning top-level's classification.
    fn effective_version_for_iri(&self, iri: &str) -> Option<String> {
        if self.top_levels.contains(iri) {
            return self.effective_version_for_top_level(iri);
        }
        self.owning_top_level_of(iri)
            .and_then(|top| self.effective_version_for_top_level(&top))
            .or_else(|| self.options.default_version.clone())
    }

    fn note_synthesized(&mut self, iri: &str, version: &str) {
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
    fn discover_sa_collapses(&mut self) {
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
            // it as an input subject) — but if the original SA shared
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

    fn discover_unsupported_sbol3_subjects(&mut self) {
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

    /// Pairs every SBOL 3 ComponentReference with the Constraint that
    /// names it via `sbol3:object`, recovering the original SBOL 2
    /// MapsTo. Also indexes every Interface's
    /// `input` / `output` / `nondirectional` triples so the downgrade
    /// can re-emit per-FC `sbol2:direction`.
    fn discover_mapsto_and_interfaces(&mut self) {
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
        // `replaces`) — without that filter a native SBOL 3 Constraint
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
                // No paired Constraint — the CRef can't fold into a
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
    fn record_restored(&mut self) {
        self.report.counts.identities_restored_from_backport += 1;
    }

    /// Decides each Component's [`ComponentShape`] from its outgoing
    /// triples plus any `backport:sbol2type` hint, then computes the
    /// IRIs and display-id suffixes both halves of a dual-role split
    /// will use. Also indexes each SubComponent's enclosing parent so
    /// later passes can dispatch on the parent's shape.
    fn classify_components(&mut self) {
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
                    // structural — it's pure functional plumbing the
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
        // A `backport:sbol2type` hint is authoritative — SBOL 2 sources
        // unambiguously chose one class or the other, so we honor that
        // choice even when the SBOL 3 surface carries triples that
        // could be read as the other shape (e.g. an SBOL 2
        // ModuleDefinition with a `sbol:role` triple — legal in SBOL 2,
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
                        // Components with no signals default to CD —
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
                        // No hint — sbolgraph heuristic: anything with
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
            // Component's original IRI — that IRI is already in
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
                // synthesized FC doesn't merge with existing triples —
                // that would put two contradictory rdf:types on the
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
        // consistently across runs — HashMap iteration is unstable.
        let mut sc_parents: Vec<(String, String)> =
            self.feature_parent.clone().into_iter().collect();
        sc_parents.sort();

        // For each SubComponent under a DualRole parent, compute the
        // triple-variant IRIs. Non-bare variants (the ones carrying an
        // `_c` / `_fc` / `_m` suffix) go through
        // [`next_available_child_iri`] against the shared `used_iris`
        // set — without this, a synthesized variant can land on top of
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

    /// Main pass: walk every triple, applying IRI rewrites, type and
    /// predicate downgrades, and value-level reverse mappings.
    fn convert(&mut self) {
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
    fn duplicate_collection_memberships(&mut self) {
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
    fn emit_dual_role_components(&mut self) {
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
    fn rewrite_participants(&mut self) {
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
    fn emit_mapsto_decompositions(&mut self) {
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
    fn emit_fc_directions(&mut self) {
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

    fn interface_feature_emission(
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

    fn emit_component_instance_defaults(&mut self) {
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
    fn build_subject_predicate_index(&self) -> HashSet<(String, String)> {
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
    fn emit_default_if_missing(
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
    fn owning_top_level_of(&self, iri: &str) -> Option<String> {
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
    fn emit_sa_wrappers(&mut self) {
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

    fn handle_triple(&mut self, triple: &Triple) {
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

        // dcterms:title / dcterms:description — passthrough (SBOL 2 also
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

    fn handle_type_triple(&mut self, triple: &Triple) {
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

        // Use the backport-recorded SBOL 2 type when available — it's
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

        // Unknown SBOL 3 type — surface as a warning and drop.
        self.report.push(DowngradeWarning::UnsupportedSbol3Type {
            subject: subject_iri,
            sbol3_type: object_iri.to_owned(),
        });
    }

    /// Emits the rdf:type triples for the CD and MD halves of a
    /// dual-role Component split, plus the synthesized linking
    /// FunctionalComponent (whose containment by the MD is emitted
    /// separately in [`emit_dual_role_components`]).
    fn emit_component_split_types(&mut self, split: &ComponentSplit, sbol3_iri: &str) {
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
    /// SubComponent's target shape — only MD-shaped targets receive a
    /// `sbol2:Module` variant (a Module's `definition` must be an MD).
    fn emit_subcomponent_split_types(&mut self, split: &SubComponentSplit) {
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
    fn handle_subcomponent_split_predicate(&mut self, triple: &Triple, split: &SubComponentSplit) {
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
                // Target isn't a tracked Component split (rare — would
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

        // Everything else routes to the FunctionalComponent variant —
        // that's where SBOL 2 plumbing (measure, sourceLocation,
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
    fn handle_dual_role_predicate(&mut self, triple: &Triple, split: &ComponentSplit) {
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
                // the half's IRI (e.g. `_component_2`) — using the
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
                // table and emit on the half whose suffix is empty —
                // the half that kept the bare IRI matches the original
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
    fn handle_dual_role_has_feature(&mut self, triple: &Triple, split: &ComponentSplit) {
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
            // this SubComponent rewrites to the FC variant — that's
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

    fn handle_sbol3_predicate(&mut self, triple: &Triple) {
        let predicate = triple.predicate.as_str();

        // SubComponent under a dual-role parent: route to the three
        // SBOL 2 variants emitted in `emit_subcomponent_split_types`.
        if let Some(subject_iri) = triple.subject.as_iri() {
            let s = subject_iri.as_str().to_owned();
            if let Some(sc_split) = self.subcomponent_splits.get(&s).cloned() {
                self.handle_subcomponent_split_predicate(triple, &sc_split);
                return;
            }
        }

        // Dual-role Component: every predicate routes to the structural
        // half (CD), the functional half (MD), or both, based on whether
        // it describes structure (sequence, role, type) or function
        // (interaction, model). Identified properties (displayId, name,
        // description) land on both halves.
        if let Some(subject_iri) = triple.subject.as_iri() {
            let s = subject_iri.as_str().to_owned();
            if let Some(split) = self.component_splits.get(&s).cloned()
                && split.shape == ComponentShape::DualRole
            {
                self.handle_dual_role_predicate(triple, &split);
                return;
            }
        }

        // Drop predicates that have no SBOL 2 equivalent. `hasNamespace`
        // is the most important one — the namespace is implicit in
        // the restored persistentIdentity / versioned IRI in SBOL 2.
        if predicate == v3::SBOL_HAS_NAMESPACE {
            return;
        }

        // `hasFeature` is context-dependent in SBOL 2 — it becomes
        // `component`, `functionalComponent`, `module`, or
        // `sequenceAnnotation` depending on what the feature is and
        // what its parent is. Resolve here using the type maps built
        // in preflight.
        if predicate == v3::SBOL_HAS_FEATURE {
            self.handle_has_feature(triple);
            return;
        }

        // `hasLocation` on a SubComponent that was a collapsed SA's
        // component — drop here; the reconstructed SA emits
        // `sbol2:location` itself in `emit_sa_wrappers`.
        if predicate == v3::SBOL_HAS_LOCATION
            && let Some(subject_iri) = triple.subject.as_iri()
            && self.sa_collapses.contains_key(subject_iri.as_str())
        {
            return;
        }

        // `sbol3:type` on an MD-derived Component drops the
        // synthesized `SBO:functionalEntity` term — SBOL 2 MDs don't
        // carry it. The original SBOL 2 type triples (if any) are
        // still emitted because they pass through the same predicate.
        if predicate == v3::SBOL_TYPE
            && let Some(subject_iri) = triple.subject.as_iri()
            && self
                .backport_types
                .get(subject_iri.as_str())
                .map(String::as_str)
                == Some(v2::SBOL2_MODULE_DEFINITION)
            && triple.object.as_iri().map(|i| i.as_str())
                == Some("https://identifiers.org/SBO:0000241")
        {
            return;
        }

        // SBOL 3 requires `hasSequence` on every Range / Cut /
        // Location; SBOL 2 represents that linkage implicitly through
        // the location's parent SequenceAnnotation → ComponentDefinition
        // → sequence chain. Forwarding `hasSequence` here would emit a
        // bare `sbol2:sequence` triple on the Range, which the upgrade
        // round-trip then duplicates against the inferred location
        // sequence. Drop on Location-typed subjects.
        if predicate == v3::SBOL_HAS_SEQUENCE
            && let Some(subject_iri) = triple.subject.as_iri()
        {
            let resolved = self
                .resolved_types
                .get(subject_iri.as_str())
                .map(String::as_str);
            if matches!(
                resolved,
                Some(v2::SBOL2_RANGE) | Some(v2::SBOL2_CUT) | Some(v2::SBOL2_GENERIC_LOCATION)
            ) {
                return;
            }
        }
        // Skip `sbol3:name` / `sbol3:description` when the same
        // subject already carries the matching `dcterms:` form. The
        // upgrade preserves dcterms metadata alongside the
        // synthesized sbol3:name; emitting both back here would
        // duplicate the triple.
        if predicate == v3::SBOL_NAME || predicate == v3::SBOL_DESCRIPTION {
            let dcterms_predicate = if predicate == v3::SBOL_NAME {
                v2::DCTERMS_TITLE
            } else {
                v2::DCTERMS_DESCRIPTION
            };
            if self.subject_already_has(
                triple.subject.as_iri().map(|i| i.as_str()),
                dcterms_predicate,
                &triple.object,
            ) {
                return;
            }
        }

        let subject_iri = triple.subject.as_iri().map(|i| i.as_str().to_owned());
        let subject = self.rewrite_resource(&triple.subject);
        let object = self.rewrite_term(&triple.object);

        if let Some(renamed) =
            self.map_sbol3_predicate_to_sbol2_for_subject(subject_iri.as_deref(), predicate)
        {
            // Value-level reverse mappings apply to a few predicates
            // (orientation, encoding, type, restriction).
            let rewritten_object =
                self.reverse_value_for_subject(subject_iri.as_deref(), predicate, &object);
            self.output_triples.push(Triple {
                subject,
                predicate: Iri::from_static(renamed),
                object: rewritten_object,
            });
            return;
        }

        self.archive_unknown_sbol3_predicate(triple);
    }

    fn map_sbol3_predicate_to_sbol2_for_subject(
        &self,
        subject_iri: Option<&str>,
        predicate: &str,
    ) -> Option<&'static str> {
        if predicate == v3::SBOL_MEMBER
            && subject_iri
                .and_then(|iri| self.resolved_type_sets.get(iri))
                .is_some_and(|types| types.contains(v2::SBOL2_EXPERIMENT))
        {
            return Some(v2::SBOL2_EXPERIMENTAL_DATA_PROP);
        }
        map_sbol3_predicate_to_sbol2(predicate)
    }

    /// Unknown SBOL 3 predicates (something added to the spec since we last
    /// updated the table, or a private extension authored in the v3
    /// namespace) cannot be emitted verbatim in an SBOL 2 graph. Archive them
    /// under the backport namespace so the data is preserved for a future
    /// re-upgrade without polluting the SBOL 2 surface.
    fn archive_unknown_sbol3_predicate(&mut self, triple: &Triple) {
        let Some(local) = triple.predicate.as_str().strip_prefix(v3::SBOL_NS) else {
            return;
        };
        let preserved = format!("{}{local}", v2::BACKPORT_SBOL3_PREFIX);
        self.output_triples.push(Triple {
            subject: self.rewrite_resource(&triple.subject),
            predicate: Iri::new_unchecked(preserved),
            object: self.rewrite_term(&triple.object),
        });
    }

    /// Returns true if the input document already carries a
    /// `(subject, predicate, object)` triple for one of the indexed
    /// dcterms predicates. Used to avoid duplicating `dcterms:title` /
    /// `dcterms:description` when the upgrade preserved both Dublin
    /// Core and SBOL 3 forms. O(1) via [`Engine::dcterms_index`].
    fn subject_already_has(
        &self,
        subject_iri: Option<&str>,
        predicate: &'static str,
        object: &Term,
    ) -> bool {
        let Some(subject_iri) = subject_iri else {
            return false;
        };
        self.dcterms_index
            .get(&(subject_iri.to_owned(), predicate))
            .map(|objects| objects.contains(object))
            .unwrap_or(false)
    }

    /// Routes `sbol3:hasFeature` to the right SBOL 2 predicate based
    /// on the resolved types of the parent (CD vs MD) and the feature
    /// itself (SubComponent, SequenceFeature, ComponentReference).
    ///
    /// Decision table:
    /// - parent = CD, feature = SubComponent → `sbol2:component`
    /// - parent = CD, feature = SequenceFeature → `sbol2:sequenceAnnotation`
    /// - parent = MD, feature = SubComponent → `sbol2:functionalComponent`,
    ///   or `sbol2:module` when the SubComponent's `instanceOf` is an MD
    /// - parent unknown / other shape → `sbol2:component` (safe default)
    fn handle_has_feature(&mut self, triple: &Triple) {
        let parent_iri = triple
            .subject
            .as_iri()
            .map(|i| i.as_str().to_owned())
            .unwrap_or_default();
        let feature_iri = triple
            .object
            .as_iri()
            .map(|i| i.as_str().to_owned())
            .unwrap_or_default();

        let parent_type = self.resolved_types.get(&parent_iri).cloned();
        let feature_type = self.resolved_types.get(&feature_iri).cloned();

        let predicate: &'static str = match (parent_type.as_deref(), feature_type.as_deref()) {
            (Some(v2::SBOL2_MODULE_DEFINITION), Some(v2::SBOL2_COMPONENT))
            | (Some(v2::SBOL2_MODULE_DEFINITION), Some(v2::SBOL2_FUNCTIONAL_COMPONENT))
            | (Some(v2::SBOL2_MODULE_DEFINITION), Some(v2::SBOL2_MODULE)) => {
                // SubComponent inside an MD-derived Component. The
                // module-vs-FunctionalComponent distinction depends on
                // what the SubComponent points at via instanceOf.
                let target = self.subcomponent_targets.get(&feature_iri).cloned();
                let target_type = target
                    .as_deref()
                    .and_then(|t| self.resolved_types.get(t).cloned());
                match target_type.as_deref() {
                    Some(v2::SBOL2_MODULE_DEFINITION) => v2::SBOL2_MODULE_PROP,
                    _ => v2::SBOL2_FUNCTIONAL_COMPONENT_PROP,
                }
            }
            (_, Some(v2::SBOL2_SEQUENCE_ANNOTATION)) => v2::SBOL2_SEQUENCE_ANNOTATION_PROP,
            _ => v2::SBOL2_COMPONENT_PROP,
        };

        self.output_triples.push(Triple {
            subject: self.rewrite_resource(&triple.subject),
            predicate: Iri::from_static(predicate),
            object: self.rewrite_term(&triple.object),
        });
    }

    /// Consumes the next preserved BioPAX variant for
    /// `(subject, sbo_term)` and advances the per-pair cursor. Returns
    /// `None` when the subject has no preserved variants for that SBO
    /// target or when the queue is exhausted; the caller then falls
    /// back to the default `*Region`-style mapping.
    fn consume_biopax_variant(&mut self, subject: Option<&str>, sbo_iri: &str) -> Option<String> {
        let subject = subject?;
        let key = (subject.to_owned(), sbo_iri.to_owned());
        let queue = self.biopax_variant_queue.get(&key)?;
        let cursor = self.biopax_variant_cursor.entry(key).or_insert(0);
        let variant = queue.get(*cursor)?.clone();
        *cursor += 1;
        Some(variant)
    }

    /// Reverses value-level mappings (orientation, encoding, type,
    /// restriction). When `subject` is `Some`, consults the
    /// subject-keyed backport hints for value mappings that are lossy
    /// by themselves (e.g. BioPAX `Dna`/`DnaRegion` collapse).
    ///
    /// Takes `&mut self` because the `sbol3:type` reverse mapping for
    /// BioPAX advances a per-`(subject, sbo_term)` cursor: each input
    /// triple that maps to the same SBO target consumes the next
    /// preserved variant from
    /// [`Engine::biopax_variant_queue`]. Without that statefulness, two
    /// `sbol3:type SBO:0000251` triples both fall back to the first
    /// preserved variant and the second BioPAX type is lost.
    fn reverse_value_for_subject(
        &mut self,
        subject: Option<&str>,
        predicate_str: &str,
        object: &Term,
    ) -> Term {
        let iri = match object.as_iri() {
            Some(iri) => iri.as_str(),
            None => return object.clone(),
        };
        let mapped: Option<String> = match predicate_str {
            v3::SBOL_ORIENTATION => values::map_orientation(iri).map(String::from),
            v3::SBOL_ENCODING => values::map_encoding(iri).map(String::from),
            // Multi-valued `sbol3:type` on a Component can mix BioPAX
            // collapses (SBO:DNA, …) with topology / role types (SO:linear).
            // Only the BioPAX side benefits from the backport hint —
            // leave SO and friends to pass through untouched.
            //
            // For each input `sbol3:type` triple that maps to an SBO
            // term the upgrade collapses BioPAX onto, consume the next
            // preserved variant for `(subject, sbo_term)`. This handles
            // the otherwise-lossy case where two distinct BioPAX
            // variants share an SBO target (e.g. `biopax:Dna` and
            // `biopax:DnaRegion` both collapse to `SBO:0000251`) — each
            // input triple gets a distinct variant in restoration order.
            // Fall back to the default `*Region`-style mapping when no
            // hint exists or the queue is exhausted.
            v3::SBOL_TYPE => values::map_biopax_type(iri).map(|default| {
                self.consume_biopax_variant(subject, iri)
                    .unwrap_or_else(|| default.to_owned())
            }),
            v3::SBOL_RESTRICTION => values::map_restriction(iri),
            v3::SBOL_CARDINALITY => values::map_cardinality(iri).map(String::from),
            v3::SBOL_STRATEGY => values::map_strategy(iri).map(String::from),
            v3::SBOL_ROLE_INTEGRATION => values::map_role_integration(iri).map(String::from),
            _ => None,
        };
        match mapped {
            Some(sbol2_iri) => Term::Resource(Resource::Iri(Iri::new_unchecked(sbol2_iri))),
            None => object.clone(),
        }
    }

    fn emit_backport_metadata(&mut self) {
        // SBOL 2 requires `persistentIdentity` and `version` on every
        // owned object, not just top-levels. Iterate every subject
        // that received a version suffix during identity restoration
        // (top-levels and their children) and emit both triples.
        //
        // The version for a child is the parent's version, propagated
        // through `iri_rewrites` (see `build_iri_rewrites` phase 2).
        let mut entries: Vec<(String, String)> = self
            .iri_rewrites
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        entries.sort();
        for (sbol3_iri, sbol2_iri) in entries {
            // Skip subjects folded into a structural re-synthesis — the
            // synthesizer emits persistentIdentity / version itself
            // (typically at a different IRI). Without this skip the
            // metadata would land on a leftover IRI as an orphan
            // subject in the SBOL 2 output and round-trip back as
            // unwanted backport triples on a CRef/Constraint/Interface.
            if self.mapsto_reconstructions.contains_key(&sbol3_iri)
                || self.mapsto_constraints.contains(&sbol3_iri)
                || self.interface_subjects.contains(&sbol3_iri)
                || self.discarded_subjects.contains(&sbol3_iri)
            {
                continue;
            }
            let subject = Resource::Iri(Iri::new_unchecked(sbol2_iri.clone()));

            if !self.resolved_types.contains_key(&sbol3_iri)
                && let Some(backport_type) = self.backport_types.get(&sbol3_iri)
            {
                self.output_triples.push(Triple {
                    subject: subject.clone(),
                    predicate: Iri::from_static(v3::RDF_TYPE),
                    object: Term::Resource(Resource::Iri(Iri::new_unchecked(
                        backport_type.clone(),
                    ))),
                });
            }

            // persistentIdentity: prefer the recorded backport value;
            // otherwise the unversioned SBOL 3 IRI itself is the
            // persistent identity.
            let persistent = self
                .persistent_identities
                .get(&sbol3_iri)
                .cloned()
                .unwrap_or_else(|| sbol3_iri.clone());
            self.output_triples.push(Triple {
                subject: subject.clone(),
                predicate: Iri::from_static(v2::SBOL2_PERSISTENT_IDENTITY),
                object: Term::Resource(Resource::Iri(Iri::new_unchecked(persistent))),
            });

            // Version: emit when the subject's owning top-level has
            // either a preserved `backport:sbol2version` or
            // synthesis enabled via `default_version`; otherwise skip
            // (SBOL 2 makes `sbol2:version` optional).
            if let Some(version) = self.effective_version_for_iri(&sbol3_iri) {
                self.output_triples.push(Triple {
                    subject,
                    predicate: Iri::from_static(v2::SBOL2_VERSION),
                    object: Term::Literal(sbol_rdf::Literal::simple(version)),
                });
            }
        }
    }

    fn rewrite_iri<'b>(&'b self, iri: &'b str) -> &'b str {
        self.iri_rewrites
            .get(iri)
            .map(String::as_str)
            .unwrap_or(iri)
    }

    fn rewrite_resource(&self, resource: &Resource) -> Resource {
        match resource {
            Resource::Iri(iri) => {
                let new = self.rewrite_iri(iri.as_str());
                if new == iri.as_str() {
                    resource.clone()
                } else {
                    Resource::Iri(Iri::new_unchecked(new))
                }
            }
            _ => resource.clone(),
        }
    }

    fn rewrite_term(&self, term: &Term) -> Term {
        match term {
            Term::Resource(resource) => Term::Resource(self.rewrite_resource(resource)),
            _ => term.clone(),
        }
    }

    fn rewrite_triple(&self, triple: &Triple) -> Triple {
        Triple {
            subject: self.rewrite_resource(&triple.subject),
            predicate: triple.predicate.clone(),
            object: self.rewrite_term(&triple.object),
        }
    }

    fn sbol2_type_for_subject_type(&self, subject_iri: &str, object_iri: &str) -> Option<String> {
        if let Some(backport_type) = self.backport_types.get(subject_iri)
            && backport_type_applies_to_sbol3_type(backport_type, object_iri)
        {
            return Some(backport_type.clone());
        }
        if object_iri == v3::SBOL_SUB_COMPONENT_CLASS {
            return Some(self.default_subcomponent_type(subject_iri).to_owned());
        }
        if object_iri == v3::SBOL_COMPONENT_CLASS {
            return Some(
                match self
                    .component_splits
                    .get(subject_iri)
                    .map(|split| split.shape)
                {
                    Some(ComponentShape::MdOnly) => v2::SBOL2_MODULE_DEFINITION,
                    _ => v2::SBOL2_COMPONENT_DEFINITION,
                }
                .to_owned(),
            );
        }
        map_sbol3_type_to_sbol2(object_iri).map(str::to_owned)
    }

    fn default_subcomponent_type(&self, subject_iri: &str) -> &'static str {
        let parent_type = self
            .feature_parent
            .get(subject_iri)
            .and_then(|parent| self.component_sbol2_type(parent));
        match parent_type {
            Some(v2::SBOL2_MODULE_DEFINITION) => {
                let target_type = self
                    .subcomponent_targets
                    .get(subject_iri)
                    .and_then(|target| self.component_sbol2_type(target));
                match target_type {
                    Some(v2::SBOL2_MODULE_DEFINITION) => v2::SBOL2_MODULE,
                    _ => v2::SBOL2_FUNCTIONAL_COMPONENT,
                }
            }
            _ => v2::SBOL2_COMPONENT,
        }
    }

    fn component_sbol2_type(&self, component_iri: &str) -> Option<&'static str> {
        match self.backport_types.get(component_iri).map(String::as_str) {
            Some(v2::SBOL2_COMPONENT_DEFINITION) => return Some(v2::SBOL2_COMPONENT_DEFINITION),
            Some(v2::SBOL2_MODULE_DEFINITION) => return Some(v2::SBOL2_MODULE_DEFINITION),
            _ => {}
        }
        self.component_splits
            .get(component_iri)
            .map(|split| match split.shape {
                ComponentShape::MdOnly => v2::SBOL2_MODULE_DEFINITION,
                _ => v2::SBOL2_COMPONENT_DEFINITION,
            })
    }
}

fn hex_decode_to_string(encoded: &str) -> Option<String> {
    if !encoded.len().is_multiple_of(2) {
        return None;
    }
    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    for chunk in encoded.as_bytes().chunks(2) {
        let high = hex_value(chunk[0])?;
        let low = hex_value(chunk[1])?;
        bytes.push((high << 4) | low);
    }
    String::from_utf8(bytes).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn canonical_term_key(term: &Term) -> String {
    match term {
        Term::Resource(Resource::Iri(iri)) => format!("iri:{}", iri.as_str()),
        Term::Resource(Resource::BlankNode(blank)) => format!("blank:{}", blank.as_str()),
        Term::Literal(literal) => format!(
            "literal:{}|{}|{}",
            literal.value(),
            literal.language().unwrap_or(""),
            literal.datatype().as_str()
        ),
        other => format!("{other:?}"),
    }
}

/// Appends `/segment` to `iri`, collapsing a doubled `/` when `iri`
/// already ends with one.
fn append_segment(iri: &str, segment: &str) -> String {
    if iri.ends_with('/') {
        format!("{iri}{segment}")
    } else {
        format!("{iri}/{segment}")
    }
}

/// Returns a child `(display_id, iri)` under `parent` whose IRI is not already
/// in `used`, inserting the chosen IRI. Disambiguates by appending `_2`,
/// `_3`, … and keeps displayId aligned with the child IRI.
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
        let iri = append_segment(parent, &display_id);
        if used.insert(iri.clone()) {
            return (display_id, iri);
        }
        counter += 1;
    }
}

/// Returns an IRI starting from `base` that is not already in `used`,
/// inserting the chosen IRI. Tries `base`, then `base_2`, `base_3`, …
/// until it finds one that is available.
///
/// Unlike [`next_available_child_iri`], no separator is inserted between
/// `base` and the disambiguation counter — `base` is taken as the
/// complete candidate IRI. Used for sibling-style synthesis where the
/// candidate IRI is built by appending a suffix (e.g. `_component`,
/// `_module`) directly to an existing IRI rather than by adding a new
/// path segment.
fn next_available_iri(base: &str, used: &mut HashSet<String>) -> String {
    if used.insert(base.to_owned()) {
        return base.to_owned();
    }
    let mut counter: usize = 2;
    loop {
        let candidate = format!("{base}_{counter}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        counter += 1;
    }
}

use crate::iri_util::last_iri_segment as last_segment;

fn type_set_contains(
    types_by_subject: &HashMap<String, HashSet<String>>,
    subject: &str,
    ty: &str,
) -> bool {
    types_by_subject
        .get(subject)
        .is_some_and(|types| types.contains(ty))
}

fn backport_type_applies_to_sbol3_type(backport_type: &str, object_iri: &str) -> bool {
    match object_iri {
        v3::SBOL_COMPONENT_CLASS => matches!(
            backport_type,
            v2::SBOL2_COMPONENT_DEFINITION | v2::SBOL2_MODULE_DEFINITION
        ),
        v3::SBOL_SUB_COMPONENT_CLASS => matches!(
            backport_type,
            v2::SBOL2_COMPONENT | v2::SBOL2_FUNCTIONAL_COMPONENT | v2::SBOL2_MODULE
        ),
        v3::SBOL_SEQUENCE_FEATURE_CLASS => backport_type == v2::SBOL2_SEQUENCE_ANNOTATION,
        v3::SBOL_CONSTRAINT_CLASS => backport_type == v2::SBOL2_SEQUENCE_CONSTRAINT,
        v3::SBOL_SEQUENCE_CLASS => backport_type == v2::SBOL2_SEQUENCE,
        v3::SBOL_MODEL_CLASS => backport_type == v2::SBOL2_MODEL,
        v3::SBOL_INTERACTION_CLASS => backport_type == v2::SBOL2_INTERACTION,
        v3::SBOL_PARTICIPATION_CLASS => backport_type == v2::SBOL2_PARTICIPATION,
        v3::SBOL_COLLECTION_CLASS => backport_type == v2::SBOL2_COLLECTION,
        v3::SBOL_IMPLEMENTATION_CLASS => backport_type == v2::SBOL2_IMPLEMENTATION,
        v3::SBOL_ATTACHMENT_CLASS => backport_type == v2::SBOL2_ATTACHMENT,
        v3::SBOL_EXPERIMENT_CLASS => backport_type == v2::SBOL2_EXPERIMENT,
        v3::SBOL_EXPERIMENTAL_DATA_CLASS => backport_type == v2::SBOL2_EXPERIMENTAL_DATA,
        v3::SBOL_COMBINATORIAL_DERIVATION_CLASS => {
            backport_type == v2::SBOL2_COMBINATORIAL_DERIVATION
        }
        v3::SBOL_VARIABLE_FEATURE_CLASS => backport_type == v2::SBOL2_VARIABLE_COMPONENT,
        v3::SBOL_RANGE_CLASS => backport_type == v2::SBOL2_RANGE,
        v3::SBOL_CUT_CLASS => backport_type == v2::SBOL2_CUT,
        v3::SBOL_LOCATION_CLASS => backport_type == v2::SBOL2_GENERIC_LOCATION,
        _ => false,
    }
}

/// Default SBOL 3 type → SBOL 2 type mapping. Used when the subject
/// has no `backport:sbol2type` triple to consult. Component is mapped
/// to ComponentDefinition here; phase 3 refines this to split
/// dual-role Components into CD + MD.
fn map_sbol3_type_to_sbol2(iri: &str) -> Option<&'static str> {
    Some(match iri {
        v3::SBOL_COMPONENT_CLASS => v2::SBOL2_COMPONENT_DEFINITION,
        v3::SBOL_SUB_COMPONENT_CLASS => v2::SBOL2_COMPONENT,
        v3::SBOL_SEQUENCE_FEATURE_CLASS => v2::SBOL2_SEQUENCE_ANNOTATION,
        v3::SBOL_CONSTRAINT_CLASS => v2::SBOL2_SEQUENCE_CONSTRAINT,
        v3::SBOL_SEQUENCE_CLASS => v2::SBOL2_SEQUENCE,
        v3::SBOL_MODEL_CLASS => v2::SBOL2_MODEL,
        v3::SBOL_INTERACTION_CLASS => v2::SBOL2_INTERACTION,
        v3::SBOL_PARTICIPATION_CLASS => v2::SBOL2_PARTICIPATION,
        v3::SBOL_COLLECTION_CLASS => v2::SBOL2_COLLECTION,
        v3::SBOL_IMPLEMENTATION_CLASS => v2::SBOL2_IMPLEMENTATION,
        v3::SBOL_ATTACHMENT_CLASS => v2::SBOL2_ATTACHMENT,
        v3::SBOL_EXPERIMENT_CLASS => v2::SBOL2_EXPERIMENT,
        v3::SBOL_EXPERIMENTAL_DATA_CLASS => v2::SBOL2_EXPERIMENTAL_DATA,
        v3::SBOL_COMBINATORIAL_DERIVATION_CLASS => v2::SBOL2_COMBINATORIAL_DERIVATION,
        v3::SBOL_VARIABLE_FEATURE_CLASS => v2::SBOL2_VARIABLE_COMPONENT,
        v3::SBOL_RANGE_CLASS => v2::SBOL2_RANGE,
        v3::SBOL_CUT_CLASS => v2::SBOL2_CUT,
        v3::SBOL_LOCATION_CLASS => v2::SBOL2_GENERIC_LOCATION,
        // Component subtypes that don't exist in SBOL 2 — skip them
        // (caller surfaces an UnsupportedSbol3Type warning).
        _ => return None,
    })
}

/// SBOL 3 predicate → SBOL 2 predicate. Single-target rewrites only;
/// predicates that need context to resolve (`hasFeature` could be
/// `component`, `functionalComponent`, `module`, or `sequenceAnnotation`
/// depending on what kind of feature it points at) get refined in
/// phase 3.
fn map_sbol3_predicate_to_sbol2(iri: &str) -> Option<&'static str> {
    Some(match iri {
        v3::SBOL_DISPLAY_ID => v2::SBOL2_DISPLAY_ID,
        v3::SBOL_NAME => v2::DCTERMS_TITLE,
        v3::SBOL_DESCRIPTION => v2::DCTERMS_DESCRIPTION,
        v3::SBOL_TYPE => v2::SBOL2_TYPE,
        v3::SBOL_ROLE => v2::SBOL2_ROLE,
        v3::SBOL_ROLE_INTEGRATION => v2::SBOL2_ROLE_INTEGRATION,
        v3::SBOL_ELEMENTS => v2::SBOL2_ELEMENTS,
        v3::SBOL_ENCODING => v2::SBOL2_ENCODING,
        v3::SBOL_SOURCE => v2::SBOL2_SOURCE,
        v3::SBOL_FORMAT => v2::SBOL2_FORMAT,
        v3::SBOL_SIZE => v2::SBOL2_SIZE,
        v3::SBOL_HASH => v2::SBOL2_HASH,
        v3::SBOL_HASH_ALGORITHM => v2::SBOL2_HASH_ALGORITHM,
        v3::SBOL_LANGUAGE => v2::SBOL2_LANGUAGE,
        v3::SBOL_FRAMEWORK => v2::SBOL2_FRAMEWORK,
        v3::SBOL_START => v2::SBOL2_START,
        v3::SBOL_END => v2::SBOL2_END,
        v3::SBOL_AT => v2::SBOL2_AT,
        v3::SBOL_BUILT => v2::SBOL2_BUILT,
        v3::SBOL_ORIENTATION => v2::SBOL2_ORIENTATION,
        v3::SBOL_HAS_SEQUENCE => v2::SBOL2_SEQUENCE_PROP,
        v3::SBOL_HAS_CONSTRAINT => v2::SBOL2_SEQUENCE_CONSTRAINT_PROP,
        v3::SBOL_HAS_INTERACTION => v2::SBOL2_INTERACTION_PROP,
        v3::SBOL_HAS_PARTICIPATION => v2::SBOL2_PARTICIPATION_PROP,
        v3::SBOL_HAS_LOCATION => v2::SBOL2_LOCATION_PROP,
        v3::SBOL_HAS_MODEL => v2::SBOL2_MODEL_PROP,
        v3::SBOL_HAS_ATTACHMENT => v2::SBOL2_ATTACHMENT_PROP,
        v3::SBOL_INSTANCE_OF => v2::SBOL2_DEFINITION,
        v3::SBOL_HAS_VARIABLE_FEATURE => v2::SBOL2_VARIABLE_COMPONENT_PROP,
        v3::SBOL_CARDINALITY => v2::SBOL2_OPERATOR,
        v3::SBOL_VARIABLE => v2::SBOL2_VARIABLE,
        v3::SBOL_VARIANT => v2::SBOL2_VARIANT,
        v3::SBOL_VARIANT_COLLECTION => v2::SBOL2_VARIANT_COLLECTION,
        v3::SBOL_VARIANT_DERIVATION => v2::SBOL2_VARIANT_DERIVATION,
        v3::SBOL_RESTRICTION => v2::SBOL2_RESTRICTION,
        v3::SBOL_SUBJECT => v2::SBOL2_SUBJECT,
        v3::SBOL_OBJECT => v2::SBOL2_OBJECT,
        v3::SBOL_PARTICIPANT => v2::SBOL2_PARTICIPANT,
        v3::SBOL_STRATEGY => v2::SBOL2_STRATEGY,
        v3::SBOL_TEMPLATE => v2::SBOL2_TEMPLATE,
        v3::SBOL_MEMBER => v2::SBOL2_MEMBER,
        // `hasFeature` is context-dependent and handled by
        // `Engine::handle_has_feature` ahead of this table.
        // `hasNamespace` is dropped earlier (no SBOL 2 equivalent —
        // the namespace lives implicitly in the restored
        // persistentIdentity).
        _ => return None,
    })
}
