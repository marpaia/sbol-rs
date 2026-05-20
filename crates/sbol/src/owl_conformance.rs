//! Schema-conformance machinery for comparing `crates/sbol/src/vocab.rs`
//! against the upstream SBOL 3 OWL document (`sbol-owl3`).
//!
//! The data this module operates on:
//!
//! - The bundled SBOL vocabulary: every `http://sbols.org/v3#X` IRI that
//!   appears as a string constant in `vocab.rs`. Compiled in via
//!   `include_str!`.
//! - A serialized OWL document from
//!   [SynBioDex/sbol-owl3](https://github.com/SynBioDex/sbol-owl3),
//!   parsed at runtime. The
//!   library accepts the RDF/XML payload as a `&str`; callers (binaries
//!   and the integration test) load it from
//!   `tests/fixtures/sbol-owl3/sbol3.rdf`.
//!
//! Two allowlists document every intentional divergence:
//!
//! - [`OWL_ONLY_ALLOWLIST`] — IRIs the OWL declares that `vocab.rs`
//!   deliberately ignores (abstract OWL super-properties, umbrella
//!   enumeration classes).
//! - [`RUST_ONLY_ALLOWLIST`] — IRIs `vocab.rs` declares that the OWL
//!   omits. Every entry cites the SBOL 3.1.0 spec section that
//!   legitimizes it.
//!
//! [`analyze_owl_conformance`] performs the diff against both lists.
//! [`render_owl_conformance_report`] turns the result into a markdown
//! report (committed to `docs/sbol-owl3-conformance.md` and enforced
//! fresh by `tests/sbol_owl3_conformance_report.rs`).

use std::collections::BTreeSet;

use sbol_rdf::{Graph, RdfFormat};

const VOCAB_SOURCE: &str = include_str!("vocab.rs");

const SBOL_PREFIX: &str = "http://sbols.org/v3#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const OWL_CLASS: &str = "http://www.w3.org/2002/07/owl#Class";
const OWL_OBJECT_PROPERTY: &str = "http://www.w3.org/2002/07/owl#ObjectProperty";
const OWL_DATATYPE_PROPERTY: &str = "http://www.w3.org/2002/07/owl#DatatypeProperty";

/// IRIs declared by the upstream OWL that `crates/sbol/src/vocab.rs`
/// intentionally does not surface as constants. Every entry must carry
/// a one-line reason a future maintainer can re-evaluate without
/// re-deriving it from the spec.
///
/// Categories represented below:
///
/// - Abstract OWL super-properties: aggregating super-properties that
///   sub-properties extend (`comprises`, `directlyComprises`).
/// - Umbrella enumeration classes: the root class of a value enumeration;
///   SBOL documents reference leaf values directly (`Cardinality`,
///   `Orientation`, etc.).
/// - Biological-role and biological-type subclasses: OWL subclasses of
///   `Component`, `Sequence`, `Interaction`, `Participation` that the
///   OWL declares for reasoning. SBOL documents instantiate the parent
///   class and set `sbol:type`/`sbol:role`/`sbol:encoding` instead.
/// - SBOL-namespaced PROV/OM subclasses: SBOL-specific subclasses
///   (`SBOLActivity`, `SBOLMeasure`, ...) that the OWL declares as
///   replacements for the upstream PROV/OM types. SBOL 3.1.0
///   documents use the upstream IRIs (`prov:Activity`, `om:Measure`)
///   directly, matching libSBOLj3 and pySBOL3.
/// - Modeling abstractions that never appear as `rdf:type` in serialized
///   documents (`GenericTopLevel`, `Metadata`, `SBOLValue`).
pub const OWL_ONLY_ALLOWLIST: &[(&str, &str)] = &[
    // Abstract OWL super-properties.
    (
        "http://sbols.org/v3#comprises",
        "abstract OWL super-property; not serialized in SBOL documents",
    ),
    (
        "http://sbols.org/v3#directlyComprises",
        "abstract OWL super-property; not serialized in SBOL documents",
    ),
    // Umbrella enumeration classes (root types for SBOL value enumerations).
    (
        "http://sbols.org/v3#Cardinality",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#CombinatorialDerivationStrategy",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#ComponentRole",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#ComponentType",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#ConstraintRestriction",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#DNARNAComponentType",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#DNARole",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#Encoding",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#IdentityRestriction",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#InteractionType",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#ModelFramework",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#ModelLanguage",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#NucleicAcidTopology",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#Orientation",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#OrientationRestriction",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#ParticipationRole",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#ProteinRole",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#RNARole",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#RoleIntegration",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#SBOLTerm",
        "OWL root for all SBOL value enumerations; not surfaced as a type",
    ),
    (
        "http://sbols.org/v3#SequentialRestriction",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#SmallMoleculeRole",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    (
        "http://sbols.org/v3#TopologyRestriction",
        "umbrella enumeration class; leaf values are referenced directly",
    ),
    // Component biological subclasses (OWL groups Components by type and
    // role; SBOL documents instantiate `sbol:Component` and set
    // `sbol:type`/`sbol:role` instead — verified against the SBOLTestSuite
    // fixture corpus).
    (
        "http://sbols.org/v3#CDSDNAComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type/sbol:role",
    ),
    (
        "http://sbols.org/v3#DNAComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type",
    ),
    (
        "http://sbols.org/v3#EffectorSimpleChemicalComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type/sbol:role",
    ),
    (
        "http://sbols.org/v3#EngineeredRegionDNAComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type/sbol:role",
    ),
    (
        "http://sbols.org/v3#FunctionalEntityComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type",
    ),
    (
        "http://sbols.org/v3#GeneDNAComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type/sbol:role",
    ),
    (
        "http://sbols.org/v3#GenericDNAComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type",
    ),
    (
        "http://sbols.org/v3#NonCovalentComplexComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type",
    ),
    (
        "http://sbols.org/v3#OperatorDNAComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type/sbol:role",
    ),
    (
        "http://sbols.org/v3#PromoterDNAComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type/sbol:role",
    ),
    (
        "http://sbols.org/v3#ProteinComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type",
    ),
    (
        "http://sbols.org/v3#RBSDNAComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type/sbol:role",
    ),
    (
        "http://sbols.org/v3#RNAComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type",
    ),
    (
        "http://sbols.org/v3#SimpleChemicalComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type",
    ),
    (
        "http://sbols.org/v3#TerminatorDNAComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type/sbol:role",
    ),
    (
        "http://sbols.org/v3#TranscriptionFactorProteinComponent",
        "Component biological subclass; SBOL documents use Component + sbol:type/sbol:role",
    ),
    // Interaction subclasses (OWL groups Interactions by SBO term; SBOL
    // documents use `sbol:Interaction` with `sbol:type`).
    (
        "http://sbols.org/v3#BiochemicalReactionInteraction",
        "Interaction biological subclass; SBOL documents use Interaction + sbol:type",
    ),
    (
        "http://sbols.org/v3#ControlInteraction",
        "Interaction biological subclass; SBOL documents use Interaction + sbol:type",
    ),
    (
        "http://sbols.org/v3#DegradationInteraction",
        "Interaction biological subclass; SBOL documents use Interaction + sbol:type",
    ),
    (
        "http://sbols.org/v3#GeneticProductionInteraction",
        "Interaction biological subclass; SBOL documents use Interaction + sbol:type",
    ),
    (
        "http://sbols.org/v3#InhibitionInteraction",
        "Interaction biological subclass; SBOL documents use Interaction + sbol:type",
    ),
    (
        "http://sbols.org/v3#NonCovalentBindingInteraction",
        "Interaction biological subclass; SBOL documents use Interaction + sbol:type",
    ),
    (
        "http://sbols.org/v3#StimulationInteraction",
        "Interaction biological subclass; SBOL documents use Interaction + sbol:type",
    ),
    // Participation role subclasses (OWL groups Participations by SBO role;
    // SBOL documents use `sbol:Participation` with `sbol:role`).
    (
        "http://sbols.org/v3#InhibitedParticipation",
        "Participation role subclass; SBOL documents use Participation + sbol:role",
    ),
    (
        "http://sbols.org/v3#InhibitorParticipation",
        "Participation role subclass; SBOL documents use Participation + sbol:role",
    ),
    (
        "http://sbols.org/v3#ModifiedParticipation",
        "Participation role subclass; SBOL documents use Participation + sbol:role",
    ),
    (
        "http://sbols.org/v3#ModifierParticipation",
        "Participation role subclass; SBOL documents use Participation + sbol:role",
    ),
    (
        "http://sbols.org/v3#ProductParticipation",
        "Participation role subclass; SBOL documents use Participation + sbol:role",
    ),
    (
        "http://sbols.org/v3#PromoterParticipation",
        "Participation role subclass; SBOL documents use Participation + sbol:role",
    ),
    (
        "http://sbols.org/v3#ReactantParticipation",
        "Participation role subclass; SBOL documents use Participation + sbol:role",
    ),
    (
        "http://sbols.org/v3#StimulatedParticipation",
        "Participation role subclass; SBOL documents use Participation + sbol:role",
    ),
    (
        "http://sbols.org/v3#StimulatorParticipation",
        "Participation role subclass; SBOL documents use Participation + sbol:role",
    ),
    (
        "http://sbols.org/v3#TemplateParticipation",
        "Participation role subclass; SBOL documents use Participation + sbol:role",
    ),
    // Sequence encoding subclasses (OWL groups Sequences by encoding; SBOL
    // documents use `sbol:Sequence` with `sbol:encoding`).
    (
        "http://sbols.org/v3#DNASequence",
        "Sequence encoding subclass; SBOL documents use Sequence + sbol:encoding",
    ),
    (
        "http://sbols.org/v3#InChISequence",
        "Sequence encoding subclass; SBOL documents use Sequence + sbol:encoding",
    ),
    (
        "http://sbols.org/v3#ProteinSequence",
        "Sequence encoding subclass; SBOL documents use Sequence + sbol:encoding",
    ),
    (
        "http://sbols.org/v3#RNASequence",
        "Sequence encoding subclass; SBOL documents use Sequence + sbol:encoding",
    ),
    (
        "http://sbols.org/v3#SMILESSequence",
        "Sequence encoding subclass; SBOL documents use Sequence + sbol:encoding",
    ),
    (
        "http://sbols.org/v3#SequenceWithElements",
        "Sequence modeling subclass; SBOL documents use sbol:Sequence directly",
    ),
    // SBOL-namespaced PROV/OM subclasses (the OWL declares replacements for
    // upstream PROV and OM types, but SBOL 3.1.0 documents — and the
    // libSBOLj3 / pySBOL3 reference implementations — use the upstream
    // IRIs directly).
    (
        "http://sbols.org/v3#SBOLActivity",
        "SBOL-namespaced PROV subclass; SBOL documents use prov:Activity",
    ),
    (
        "http://sbols.org/v3#SBOLAgent",
        "SBOL-namespaced PROV subclass; SBOL documents use prov:Agent",
    ),
    (
        "http://sbols.org/v3#SBOLAssociation",
        "SBOL-namespaced PROV subclass; SBOL documents use prov:Association",
    ),
    (
        "http://sbols.org/v3#SBOLPlan",
        "SBOL-namespaced PROV subclass; SBOL documents use prov:Plan",
    ),
    (
        "http://sbols.org/v3#SBOLUsage",
        "SBOL-namespaced PROV subclass; SBOL documents use prov:Usage",
    ),
    (
        "http://sbols.org/v3#SBOLBinaryPrefix",
        "SBOL-namespaced OM subclass; SBOL documents use om:BinaryPrefix",
    ),
    (
        "http://sbols.org/v3#SBOLCompoundUnit",
        "SBOL-namespaced OM subclass; SBOL documents use om:CompoundUnit",
    ),
    (
        "http://sbols.org/v3#SBOLMeasure",
        "SBOL-namespaced OM subclass; SBOL documents use om:Measure",
    ),
    (
        "http://sbols.org/v3#SBOLPrefix",
        "SBOL-namespaced OM subclass; SBOL documents use om:Prefix",
    ),
    (
        "http://sbols.org/v3#SBOLPrefixedUnit",
        "SBOL-namespaced OM subclass; SBOL documents use om:PrefixedUnit",
    ),
    (
        "http://sbols.org/v3#SBOLSIPrefix",
        "SBOL-namespaced OM subclass; SBOL documents use om:SIPrefix",
    ),
    (
        "http://sbols.org/v3#SBOLSingularUnit",
        "SBOL-namespaced OM subclass; SBOL documents use om:SingularUnit",
    ),
    (
        "http://sbols.org/v3#SBOLUnit",
        "SBOL-namespaced OM subclass; SBOL documents use om:Unit",
    ),
    (
        "http://sbols.org/v3#SBOLUnitDivision",
        "SBOL-namespaced OM subclass; SBOL documents use om:UnitDivision",
    ),
    (
        "http://sbols.org/v3#SBOLUnitExponentiation",
        "SBOL-namespaced OM subclass; SBOL documents use om:UnitExponentiation",
    ),
    (
        "http://sbols.org/v3#SBOLUnitMultiplication",
        "SBOL-namespaced OM subclass; SBOL documents use om:UnitMultiplication",
    ),
    // Modeling abstractions that never appear as rdf:type on the wire.
    (
        "http://sbols.org/v3#GenericTopLevel",
        "OWL modeling abstraction; not instantiated as rdf:type in SBOL documents",
    ),
    (
        "http://sbols.org/v3#Metadata",
        "OWL modeling abstraction; not instantiated as rdf:type in SBOL documents",
    ),
    (
        "http://sbols.org/v3#SBOLValue",
        "OWL modeling abstraction; not instantiated as rdf:type in SBOL documents",
    ),
];

/// IRIs `crates/sbol/src/vocab.rs` exposes that the pinned OWL does not.
/// Every entry must cite the SBOL 3.1.0 spec section that legitimizes
/// it; if you cannot point at the spec, the constant is almost
/// certainly a bug (see the `#zero` / `#none` removals in `CHANGELOG.md`
/// for the reference cases).
pub const RUST_ONLY_ALLOWLIST: &[(&str, &str)] = &[
    (
        "http://sbols.org/v3#design",
        "SBOL 3.1.0 Appendix A.1 Activity type convention (Table 19)",
    ),
    (
        "http://sbols.org/v3#build",
        "SBOL 3.1.0 Appendix A.1 Activity type convention (Table 19)",
    ),
    (
        "http://sbols.org/v3#test",
        "SBOL 3.1.0 Appendix A.1 Activity type convention (Table 19)",
    ),
    (
        "http://sbols.org/v3#learn",
        "SBOL 3.1.0 Appendix A.1 Activity type convention (Table 19)",
    ),
];

/// SBOL-namespaced IRIs the pinned OWL declares, grouped by OWL type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwlIdentifiers {
    pub classes: BTreeSet<String>,
    pub object_properties: BTreeSet<String>,
    pub datatype_properties: BTreeSet<String>,
}

impl OwlIdentifiers {
    /// Union of all three buckets.
    pub fn all(&self) -> BTreeSet<String> {
        let mut union = BTreeSet::new();
        union.extend(self.classes.iter().cloned());
        union.extend(self.object_properties.iter().cloned());
        union.extend(self.datatype_properties.iter().cloned());
        union
    }
}

/// Parse the OWL document and extract every `http://sbols.org/v3#X` IRI
/// declared as an `owl:Class`, `owl:ObjectProperty`, or
/// `owl:DatatypeProperty`. Errors only on malformed RDF/XML.
pub fn extract_owl_identifiers(rdf_xml: &str) -> Result<OwlIdentifiers, String> {
    let graph = Graph::parse(rdf_xml, RdfFormat::RdfXml)
        .map_err(|e| format!("parse OWL document as RDF/XML: {e}"))?;
    let mut classes = BTreeSet::new();
    let mut object_properties = BTreeSet::new();
    let mut datatype_properties = BTreeSet::new();
    for triple in graph.triples() {
        if triple.predicate.as_str() != RDF_TYPE {
            continue;
        }
        let Some(subject) = triple.subject.as_iri() else {
            continue;
        };
        if !subject.as_str().starts_with(SBOL_PREFIX) {
            continue;
        }
        let Some(object) = triple.object.as_iri() else {
            continue;
        };
        let iri = subject.as_str().to_string();
        match object.as_str() {
            OWL_CLASS => {
                classes.insert(iri);
            }
            OWL_OBJECT_PROPERTY => {
                object_properties.insert(iri);
            }
            OWL_DATATYPE_PROPERTY => {
                datatype_properties.insert(iri);
            }
            _ => {}
        }
    }
    Ok(OwlIdentifiers {
        classes,
        object_properties,
        datatype_properties,
    })
}

/// Returns every `http://sbols.org/v3#X` IRI (X non-empty) declared as a
/// string literal in `crates/sbol/src/vocab.rs`. Compiled in at build
/// time.
pub fn extract_vocab_iris() -> BTreeSet<String> {
    extract_vocab_iris_from(VOCAB_SOURCE)
}

fn extract_vocab_iris_from(source: &str) -> BTreeSet<String> {
    let mut iris = BTreeSet::new();
    let bytes = source.as_bytes();
    let needle = b"\"http://sbols.org/v3#";
    let mut i = 0usize;
    while i + needle.len() < bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end] != b'"' {
                end += 1;
            }
            let iri = std::str::from_utf8(&bytes[start..end]).unwrap();
            if iri.len() > SBOL_PREFIX.len() {
                iris.insert(iri.to_string());
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
    iris
}

/// Result of comparing the OWL identifier set against `vocab.rs`.
///
/// `overlap` holds IRIs declared on both sides. `missing_from_vocab`
/// holds OWL-declared IRIs that `vocab.rs` does not surface and that
/// are *not* on the [`OWL_ONLY_ALLOWLIST`]; that set is the actionable
/// drift in the OWL → Rust direction (and should be empty in a healthy
/// repository). `missing_from_owl` is the equivalent in the other
/// direction. `stale_owl_only` and `stale_rust_only` capture allowlist
/// entries that no longer apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwlConformanceReport {
    pub owl: OwlIdentifiers,
    pub vocab_iris: BTreeSet<String>,
    pub overlap: BTreeSet<String>,
    pub missing_from_vocab: BTreeSet<String>,
    pub missing_from_owl: BTreeSet<String>,
    pub owl_only_in_use: BTreeSet<String>,
    pub rust_only_in_use: BTreeSet<String>,
    pub stale_owl_only: BTreeSet<String>,
    pub stale_rust_only: BTreeSet<String>,
}

impl OwlConformanceReport {
    /// True when the IRI surfaces agree modulo the two allowlists and
    /// neither allowlist has stale entries.
    pub fn is_clean(&self) -> bool {
        self.missing_from_vocab.is_empty()
            && self.missing_from_owl.is_empty()
            && self.stale_owl_only.is_empty()
            && self.stale_rust_only.is_empty()
    }
}

/// Compare the OWL identifier set against the `vocab.rs` IRI set,
/// applying both allowlists.
pub fn analyze_owl_conformance(rdf_xml: &str) -> Result<OwlConformanceReport, String> {
    let owl = extract_owl_identifiers(rdf_xml)?;
    let vocab_iris = extract_vocab_iris();
    let owl_iris = owl.all();

    let owl_only_allowed: BTreeSet<String> = OWL_ONLY_ALLOWLIST
        .iter()
        .map(|(iri, _)| (*iri).to_string())
        .collect();
    let rust_only_allowed: BTreeSet<String> = RUST_ONLY_ALLOWLIST
        .iter()
        .map(|(iri, _)| (*iri).to_string())
        .collect();

    let overlap: BTreeSet<String> = owl_iris.intersection(&vocab_iris).cloned().collect();
    let missing_from_vocab: BTreeSet<String> = owl_iris
        .difference(&vocab_iris)
        .filter(|iri| !owl_only_allowed.contains(*iri))
        .cloned()
        .collect();
    let missing_from_owl: BTreeSet<String> = vocab_iris
        .difference(&owl_iris)
        .filter(|iri| !rust_only_allowed.contains(*iri))
        .cloned()
        .collect();
    let owl_only_in_use: BTreeSet<String> = owl_only_allowed
        .iter()
        .filter(|iri| owl_iris.contains(*iri) && !vocab_iris.contains(*iri))
        .cloned()
        .collect();
    let rust_only_in_use: BTreeSet<String> = rust_only_allowed
        .iter()
        .filter(|iri| vocab_iris.contains(*iri) && !owl_iris.contains(*iri))
        .cloned()
        .collect();
    let stale_owl_only: BTreeSet<String> =
        owl_only_allowed.difference(&owl_iris).cloned().collect();
    let stale_rust_only: BTreeSet<String> = rust_only_allowed
        .iter()
        .filter(|iri| owl_iris.contains(*iri))
        .cloned()
        .collect();

    Ok(OwlConformanceReport {
        owl,
        vocab_iris,
        overlap,
        missing_from_vocab,
        missing_from_owl,
        owl_only_in_use,
        rust_only_in_use,
        stale_owl_only,
        stale_rust_only,
    })
}

/// Metadata about the pinned OWL fixture. Surfaced verbatim in the
/// rendered report so the report is self-contained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwlPinInfo<'a> {
    pub upstream_repo: &'a str,
    pub source_url: &'a str,
    pub commit: &'a str,
    pub committer_date: &'a str,
    pub sha256: &'a str,
    pub fetched_at: &'a str,
}

/// Render the conformance report as a markdown document.
pub fn render_owl_conformance_report(
    report: &OwlConformanceReport,
    pin: &OwlPinInfo<'_>,
) -> String {
    let mut out = String::new();

    out.push_str("# SBOL 3 schema conformance against sbol-owl3\n\n");
    out.push_str(
        "This file is generated by \
         `cargo run -p sbol --bin generate-sbol-owl3-conformance-report`.\n\
         It is committed and CI runs `git diff --exit-code \
         docs/sbol-owl3-conformance.md` to enforce freshness after every \
         change that affects the IRI surface — either the pinned OWL or \
         `crates/sbol/src/vocab.rs`.\n\n",
    );
    out.push_str(
        "Authoritative model source: SBOL 3.1.0 PDF (`spec/SBOL3.1.0.md`).\n\
         Cross-check source: SBOL OWL published by \
         [SynBioDex/sbol-owl3](https://github.com/SynBioDex/sbol-owl3).\n\
         Compared against: `crates/sbol/src/vocab.rs`.\n\n",
    );
    out.push_str("See [`docs/ontology-conformance.md`](ontology-conformance.md) ");
    out.push_str("for the design and triage flow.\n\n");

    out.push_str("## Pinned OWL\n\n");
    out.push_str("| Field | Value |\n");
    out.push_str("| --- | --- |\n");
    out.push_str(&format!("| Upstream repo | <{}> |\n", pin.upstream_repo));
    out.push_str(&format!("| Source URL | <{}> |\n", pin.source_url));
    out.push_str(&format!("| Commit | `{}` |\n", pin.commit));
    out.push_str(&format!("| Committer date | `{}` |\n", pin.committer_date));
    out.push_str(&format!("| Fixture sha256 | `{}` |\n", pin.sha256));
    out.push_str(&format!("| Fetched | `{}` |\n", pin.fetched_at));
    out.push('\n');

    let total_owl = report.owl.all().len();
    let total_vocab = report.vocab_iris.len();
    let overlap = report.overlap.len();
    out.push_str("## Coverage summary\n\n");
    out.push_str("| Surface | Count |\n");
    out.push_str("| --- | --- |\n");
    out.push_str(&format!(
        "| OWL classes (`owl:Class`) | {} |\n",
        report.owl.classes.len()
    ));
    out.push_str(&format!(
        "| OWL object properties (`owl:ObjectProperty`) | {} |\n",
        report.owl.object_properties.len()
    ));
    out.push_str(&format!(
        "| OWL datatype properties (`owl:DatatypeProperty`) | {} |\n",
        report.owl.datatype_properties.len()
    ));
    out.push_str(&format!(
        "| **Total OWL `sbols.org/v3#` IRIs** | **{total_owl}** |\n"
    ));
    out.push_str(&format!("| `vocab.rs` SBOL IRIs | {total_vocab} |\n"));
    out.push_str(&format!("| IRIs declared by both | {overlap} |\n"));
    out.push_str(&format!(
        "| OWL-only allowlist entries (`OWL_ONLY_ALLOWLIST`) | {} |\n",
        OWL_ONLY_ALLOWLIST.len()
    ));
    out.push_str(&format!(
        "| Rust-only allowlist entries (`RUST_ONLY_ALLOWLIST`) | {} |\n",
        RUST_ONLY_ALLOWLIST.len()
    ));
    out.push_str(&format!(
        "| **Unallowlisted drift (OWL → vocab.rs)** | **{}** |\n",
        report.missing_from_vocab.len()
    ));
    out.push_str(&format!(
        "| **Unallowlisted drift (vocab.rs → OWL)** | **{}** |\n",
        report.missing_from_owl.len()
    ));
    out.push_str(&format!(
        "| Stale `OWL_ONLY_ALLOWLIST` entries | {} |\n",
        report.stale_owl_only.len()
    ));
    out.push_str(&format!(
        "| Stale `RUST_ONLY_ALLOWLIST` entries | {} |\n",
        report.stale_rust_only.len()
    ));
    out.push('\n');

    out.push_str("## Status\n\n");
    if report.is_clean() {
        out.push_str(
            "**No unallowlisted divergence.** Every SBOL IRI declared in \
             the pinned OWL is either surfaced by `vocab.rs` or recorded \
             as an intentional omission, and every SBOL IRI in `vocab.rs` \
             is either declared in the OWL or recorded as a likely \
             upstream defect with a spec citation. Both allowlists are \
             current.\n\n\
             Note the asymmetry: the OWL-only allowlist captures \
             deliberate modeling differences where both sides agree, \
             while the Rust-only allowlist captures values the SBOL \
             3.1.0 PDF mandates that the OWL has not yet transcribed. \
             The latter entries are bugs to file upstream against \
             `SynBioDex/sbol-owl3`, not symmetric agreements.\n\n",
        );
    } else {
        out.push_str(
            "**Drift detected.** One or more IRI sets diverge outside the \
             allowlists. Each divergence is a bug to fix: either add a \
             constant in `vocab.rs`, remove a stale constant, file \
             upstream, or update the allowlist with a rationale or spec \
             citation.\n\n",
        );
        if !report.missing_from_vocab.is_empty() {
            push_drift_list(
                &mut out,
                "### OWL declares; `vocab.rs` does not surface (unallowlisted)",
                &report.missing_from_vocab,
            );
        }
        if !report.missing_from_owl.is_empty() {
            push_drift_list(
                &mut out,
                "### `vocab.rs` declares; OWL does not (unallowlisted)",
                &report.missing_from_owl,
            );
        }
        if !report.stale_owl_only.is_empty() {
            push_drift_list(
                &mut out,
                "### `OWL_ONLY_ALLOWLIST` entries no longer in the pinned OWL",
                &report.stale_owl_only,
            );
        }
        if !report.stale_rust_only.is_empty() {
            push_drift_list(
                &mut out,
                "### `RUST_ONLY_ALLOWLIST` entries now in the pinned OWL (remove from allowlist)",
                &report.stale_rust_only,
            );
        }
    }

    out.push_str("## Intentional omissions from `vocab.rs`\n\n");
    out.push_str(
        "Entries from [`OWL_ONLY_ALLOWLIST`](../crates/sbol/src/owl_conformance.rs). \
         These are IRIs the upstream \
         OWL declares for modeling purposes that `vocab.rs` deliberately \
         does not surface — abstract OWL super-properties whose subclasses \
         carry the wire-level semantics, and umbrella enumeration classes \
         whose leaf values are referenced directly. **Both sides agree**: \
         the omissions are intentional and aligned with how SBOL documents \
         are serialized.\n\n\
         If an upstream change removes one of these, the Status section \
         above will flag it as stale and the entry should be revisited.\n\n",
    );
    out.push_str("| IRI | Rationale | Active in this pin |\n");
    out.push_str("| --- | --- | --- |\n");
    for (iri, rationale) in OWL_ONLY_ALLOWLIST {
        let active = if report.owl_only_in_use.contains(*iri) {
            "yes"
        } else {
            "**stale**"
        };
        out.push_str(&format!("| `{iri}` | {rationale} | {active} |\n"));
    }
    out.push('\n');

    out.push_str("## Spec-mandated values missing from the pinned OWL\n\n");
    out.push_str(
        "Entries from [`RUST_ONLY_ALLOWLIST`](../crates/sbol/src/owl_conformance.rs). \
         Each IRI is enumerated in a specific table of the SBOL 3.1.0 PDF \
         — which is the authoritative source — but the pinned OWL does not \
         declare it. These are likely upstream defects rather than \
         symmetric intentional differences: we track them here so the \
         regression test stays green while the OWL catches up, and the \
         rows below are an actionable punch list for issues to file \
         against `SynBioDex/sbol-owl3`.\n\n\
         If an entry below ever shows up in the OWL, the Status section \
         will flag it as ready to remove from the allowlist. Conversely, \
         a new constant that cannot be backed by a spec table is almost \
         certainly a bug in `vocab.rs` — see the `#zero` and `#none` \
         removals in `CHANGELOG.md` for the reference cases.\n\n",
    );
    out.push_str("| IRI | Spec citation | Still absent from OWL |\n");
    out.push_str("| --- | --- | --- |\n");
    for (iri, citation) in RUST_ONLY_ALLOWLIST {
        let absent = if report.rust_only_in_use.contains(*iri) {
            "yes"
        } else {
            "**now in OWL — remove from allowlist**"
        };
        out.push_str(&format!("| `{iri}` | {citation} | {absent} |\n"));
    }
    out.push('\n');

    out.push_str("## How to refresh\n\n");
    out.push_str(
        "1. `cargo run -p sbol-ontology --bin update-sbol-owl3-fixture` — \
         re-pin the OWL against the current `main` of \
         `SynBioDex/sbol-owl3`.\n\
         2. `cargo run -p sbol --bin generate-sbol-owl3-conformance-report` \
         — regenerate this file.\n\
         3. `cargo test -p sbol --test sbol_owl3_conformance --test \
         sbol_owl3_conformance_report` — confirm the assertions and the \
         freshness gate pass.\n\
         4. Commit the pin, manifest, this report, and any allowlist \
         changes in one commit so the trail is auditable.\n",
    );

    out
}

fn push_drift_list(out: &mut String, heading: &str, items: &BTreeSet<String>) {
    out.push_str(heading);
    out.push_str("\n\n");
    for iri in items {
        out.push_str(&format!("- `{iri}`\n"));
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_vocab_iris_skips_bare_namespace_prefix() {
        let synthetic = r#"
            const NS: &str = "http://sbols.org/v3#";
            const A: &str = "http://sbols.org/v3#alpha";
            const B: &str = "http://sbols.org/v3#beta";
        "#;
        let iris = extract_vocab_iris_from(synthetic);
        assert!(!iris.contains("http://sbols.org/v3#"));
        assert!(iris.contains("http://sbols.org/v3#alpha"));
        assert!(iris.contains("http://sbols.org/v3#beta"));
        assert_eq!(iris.len(), 2);
    }

    #[test]
    fn bundled_vocab_extraction_covers_known_classes_and_properties() {
        // Smoke-test that the compile-time include picks up the
        // expected shape; the full assertions live in the dedicated
        // conformance test.
        let iris = extract_vocab_iris();
        assert!(iris.contains("http://sbols.org/v3#Component"));
        assert!(iris.contains("http://sbols.org/v3#hasFeature"));
        assert!(iris.contains("http://sbols.org/v3#displayId"));
        assert!(!iris.contains("http://sbols.org/v3#zero"));
    }
}
