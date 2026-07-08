//! SBOL 2 vocabulary IRIs shared by the upgrade and downgrade modules.
//!
//! These mirror the constants in `sbols.org/v2`. They are intentionally
//! kept `pub(crate)`: SBOL 2 URIs are an implementation detail of the
//! conversion pipeline, never part of the public SBOL 3 surface.
//!
//! The backport namespace preserves SBOL 2-only fields that have no SBOL 3
//! home. Backport annotations are written only during SBOL 2 → SBOL 3;
//! the SBOL 3 → SBOL 2 direction reads them to reconstruct the original
//! SBOL 2 form and emits none of its own.
//!
//! Both directions of the conversion (upgrade and downgrade) reference
//! these symbols, which is why they live in a shared module rather
//! than scoped to one or the other.
#![allow(dead_code)]

pub(crate) const SBOL2_NS: &str = "http://sbols.org/v2#";

/// Backport namespace and recommended prefix for SBOL 2-only fields
/// preserved across an SBOL 2 → SBOL 3 conversion.
pub(crate) const BACKPORT_NS: &str = "https://sbols.org/backport/2_3#";
pub(crate) const BACKPORT_PREFIX: &str = "backport2_3";

/// The original SBOL 2 identity, stamped on every converted SBOL 3 entity
/// (and on every `Metadata` synthesized from a nested SBOL 2 annotation).
pub(crate) const BACKPORT_SBOL2_ORIGINAL_URI: &str =
    "https://sbols.org/backport/2_3#sbol2OriginalURI";

/// The identity of the SBOL 2 `SequenceAnnotation` a `SubComponent` was
/// derived from, so the displayId can be restored on the way down.
pub(crate) const BACKPORT_SBOL2_ORIGINAL_SEQUENCE_ANNOTATION_URI: &str =
    "https://sbols.org/backport/2_3#sbol2OriginalSequenceAnnotationURI";

/// Marks a `Sequence` synthesized to satisfy a `Component` that had none,
/// so the downgrade drops it rather than re-emitting an SBOL 2 Sequence.
pub(crate) const BACKPORT_SBOL3_TEMP_SEQUENCE_URI: &str =
    "https://sbols.org/backport/2_3#sbol3TempSequenceURI";

/// Records that the source SBOL 2 location had no sequence, so the
/// downgrade suppresses the sequence it would otherwise re-emit.
pub(crate) const BACKPORT_SBOL2_LOCATION_SEQUENCE_NULL: &str =
    "https://sbols.org/backport/2_3#sbol2LocationSequenceNull";

/// Marks a `Metadata` child standing in for an SBOL 2 `GenericLocation`
/// (which has no SBOL 3 Location equivalent).
pub(crate) const BACKPORT_SBOL2_GENERIC_LOCATION: &str =
    "https://sbols.org/backport/2_3#sbol2GenericLocation";

/// Set on a generic-location `Metadata` that carried an orientation.
pub(crate) const BACKPORT_SBOL2_ENTITY: &str = "https://sbols.org/backport/2_3#sbol2Entity";

/// Marks a `SubComponent` derived from an SBOL 2 `Module` (rather than a
/// `FunctionalComponent`), so the downgrade restores it as a `Module`.
pub(crate) const BACKPORT_SBOL2_ORIGINATES_FROM_MODULE: &str =
    "https://sbols.org/backport/2_3#sbol2OriginatesFromModule";

/// Marks a `ComponentReference` derived from a `MapsTo` owned by an SBOL 2
/// `FunctionalComponent`, so the downgrade places the restored `MapsTo` on
/// the `FunctionalComponent` rather than the `Module`.
pub(crate) const BACKPORT_SBOL2_MAPSTO_ORIGIN_IN_FC: &str =
    "https://sbols.org/backport/2_3#sbol2MapstoOriginInFC";

pub(crate) const DCTERMS_TITLE: &str = "http://purl.org/dc/terms/title";
pub(crate) const DCTERMS_DESCRIPTION: &str = "http://purl.org/dc/terms/description";

// === Types ===

pub(crate) const SBOL2_COMPONENT_DEFINITION: &str = "http://sbols.org/v2#ComponentDefinition";
pub(crate) const SBOL2_MODULE_DEFINITION: &str = "http://sbols.org/v2#ModuleDefinition";
pub(crate) const SBOL2_COMPONENT: &str = "http://sbols.org/v2#Component";
pub(crate) const SBOL2_MODULE: &str = "http://sbols.org/v2#Module";
pub(crate) const SBOL2_FUNCTIONAL_COMPONENT: &str = "http://sbols.org/v2#FunctionalComponent";
pub(crate) const SBOL2_SEQUENCE_ANNOTATION: &str = "http://sbols.org/v2#SequenceAnnotation";
pub(crate) const SBOL2_SEQUENCE_CONSTRAINT: &str = "http://sbols.org/v2#SequenceConstraint";
pub(crate) const SBOL2_SEQUENCE: &str = "http://sbols.org/v2#Sequence";
pub(crate) const SBOL2_MODEL: &str = "http://sbols.org/v2#Model";
pub(crate) const SBOL2_INTERACTION: &str = "http://sbols.org/v2#Interaction";
pub(crate) const SBOL2_PARTICIPATION: &str = "http://sbols.org/v2#Participation";
pub(crate) const SBOL2_COLLECTION: &str = "http://sbols.org/v2#Collection";
pub(crate) const SBOL2_IMPLEMENTATION: &str = "http://sbols.org/v2#Implementation";
pub(crate) const SBOL2_ATTACHMENT: &str = "http://sbols.org/v2#Attachment";
pub(crate) const SBOL2_EXPERIMENT: &str = "http://sbols.org/v2#Experiment";
pub(crate) const SBOL2_EXPERIMENTAL_DATA: &str = "http://sbols.org/v2#ExperimentalData";
pub(crate) const SBOL2_COMBINATORIAL_DERIVATION: &str =
    "http://sbols.org/v2#CombinatorialDerivation";
pub(crate) const SBOL2_VARIABLE_COMPONENT: &str = "http://sbols.org/v2#VariableComponent";
pub(crate) const SBOL2_RANGE: &str = "http://sbols.org/v2#Range";
pub(crate) const SBOL2_CUT: &str = "http://sbols.org/v2#Cut";
pub(crate) const SBOL2_GENERIC_LOCATION: &str = "http://sbols.org/v2#GenericLocation";
pub(crate) const SBOL2_MAPS_TO: &str = "http://sbols.org/v2#MapsTo";

// === Predicates ===

pub(crate) const SBOL2_DISPLAY_ID: &str = "http://sbols.org/v2#displayId";
pub(crate) const SBOL2_PERSISTENT_IDENTITY: &str = "http://sbols.org/v2#persistentIdentity";
pub(crate) const SBOL2_VERSION: &str = "http://sbols.org/v2#version";
pub(crate) const SBOL2_ORIENTATION: &str = "http://sbols.org/v2#orientation";
pub(crate) const SBOL2_BUILT: &str = "http://sbols.org/v2#built";
pub(crate) const SBOL2_SEQUENCE_PROP: &str = "http://sbols.org/v2#sequence";
pub(crate) const SBOL2_SEQUENCE_ANNOTATION_PROP: &str = "http://sbols.org/v2#sequenceAnnotation";
pub(crate) const SBOL2_SEQUENCE_CONSTRAINT_PROP: &str = "http://sbols.org/v2#sequenceConstraint";
pub(crate) const SBOL2_COMPONENT_PROP: &str = "http://sbols.org/v2#component";
pub(crate) const SBOL2_FUNCTIONAL_COMPONENT_PROP: &str = "http://sbols.org/v2#functionalComponent";
pub(crate) const SBOL2_MODULE_PROP: &str = "http://sbols.org/v2#module";
pub(crate) const SBOL2_INTERACTION_PROP: &str = "http://sbols.org/v2#interaction";
pub(crate) const SBOL2_PARTICIPATION_PROP: &str = "http://sbols.org/v2#participation";
pub(crate) const SBOL2_LOCATION_PROP: &str = "http://sbols.org/v2#location";
pub(crate) const SBOL2_DEFINITION: &str = "http://sbols.org/v2#definition";
pub(crate) const SBOL2_VARIABLE_COMPONENT_PROP: &str = "http://sbols.org/v2#variableComponent";
pub(crate) const SBOL2_OPERATOR: &str = "http://sbols.org/v2#operator";
pub(crate) const SBOL2_VARIABLE: &str = "http://sbols.org/v2#variable";
pub(crate) const SBOL2_VARIANT: &str = "http://sbols.org/v2#variant";
pub(crate) const SBOL2_VARIANT_COLLECTION: &str = "http://sbols.org/v2#variantCollection";
pub(crate) const SBOL2_VARIANT_DERIVATION: &str = "http://sbols.org/v2#variantDerivation";
pub(crate) const SBOL2_MODEL_PROP: &str = "http://sbols.org/v2#model";
pub(crate) const SBOL2_ATTACHMENT_PROP: &str = "http://sbols.org/v2#attachment";
pub(crate) const SBOL2_RESTRICTION: &str = "http://sbols.org/v2#restriction";
pub(crate) const SBOL2_SUBJECT: &str = "http://sbols.org/v2#subject";
pub(crate) const SBOL2_OBJECT: &str = "http://sbols.org/v2#object";
pub(crate) const SBOL2_PARTICIPANT: &str = "http://sbols.org/v2#participant";
pub(crate) const SBOL2_REFINEMENT: &str = "http://sbols.org/v2#refinement";
pub(crate) const SBOL2_LOCAL: &str = "http://sbols.org/v2#local";
pub(crate) const SBOL2_REMOTE: &str = "http://sbols.org/v2#remote";
pub(crate) const SBOL2_MAPS_TO_PROP: &str = "http://sbols.org/v2#mapsTo";
pub(crate) const SBOL2_DIRECTION: &str = "http://sbols.org/v2#direction";
pub(crate) const SBOL2_ACCESS: &str = "http://sbols.org/v2#access";
pub(crate) const SBOL2_STRATEGY: &str = "http://sbols.org/v2#strategy";
pub(crate) const SBOL2_TEMPLATE: &str = "http://sbols.org/v2#template";
pub(crate) const SBOL2_MEMBER: &str = "http://sbols.org/v2#member";
pub(crate) const SBOL2_EXPERIMENTAL_DATA_PROP: &str = "http://sbols.org/v2#experimentalData";
pub(crate) const SBOL2_ROLE: &str = "http://sbols.org/v2#role";
pub(crate) const SBOL2_ROLE_INTEGRATION: &str = "http://sbols.org/v2#roleIntegration";
pub(crate) const SBOL2_TYPE: &str = "http://sbols.org/v2#type";
pub(crate) const SBOL2_ELEMENTS: &str = "http://sbols.org/v2#elements";
pub(crate) const SBOL2_ENCODING: &str = "http://sbols.org/v2#encoding";
pub(crate) const SBOL2_SOURCE: &str = "http://sbols.org/v2#source";
pub(crate) const SBOL2_FORMAT: &str = "http://sbols.org/v2#format";
pub(crate) const SBOL2_SIZE: &str = "http://sbols.org/v2#size";
pub(crate) const SBOL2_HASH: &str = "http://sbols.org/v2#hash";
pub(crate) const SBOL2_HASH_ALGORITHM: &str = "http://sbols.org/v2#hashAlgorithm";
pub(crate) const SBOL2_LANGUAGE: &str = "http://sbols.org/v2#language";
pub(crate) const SBOL2_FRAMEWORK: &str = "http://sbols.org/v2#framework";
pub(crate) const SBOL2_START: &str = "http://sbols.org/v2#start";
pub(crate) const SBOL2_END: &str = "http://sbols.org/v2#end";
pub(crate) const SBOL2_AT: &str = "http://sbols.org/v2#at";

// === Enumerated values ===

pub(crate) const SBOL2_ORIENTATION_INLINE: &str = "http://sbols.org/v2#inline";
pub(crate) const SBOL2_ORIENTATION_REVERSE_COMPLEMENT: &str =
    "http://sbols.org/v2#reverseComplement";

// SBOL 2 sequence encoding URIs (legacy IUPAC pages), mapped to/from SBOL 3 EDAM URIs.
pub(crate) const SBOL2_ENCODING_IUPAC_DNA: &str =
    "http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html";
pub(crate) const SBOL2_ENCODING_IUPAC_PROTEIN: &str = "http://www.chem.qmul.ac.uk/iupac/AminoAcid/";
pub(crate) const SBOL2_ENCODING_SMILES: &str = "http://www.opensmiles.org/opensmiles.html";
pub(crate) const SBOL2_ENCODING_INCHI: &str =
    "http://www.chem.qmul.ac.uk/iupac/bibliog/whitebook.html";

// BioPAX type URIs used by SBOL 2 ComponentDefinitions. Both the bare type
// names (`Dna`, `Rna`) and the `*Region` variants appear in the wild
// (SynBioHub, the SBOLTestSuite). All map to the same SBO equivalents.
pub(crate) const BIOPAX_DNA: &str = "http://www.biopax.org/release/biopax-level3.owl#Dna";
pub(crate) const BIOPAX_DNA_REGION: &str =
    "http://www.biopax.org/release/biopax-level3.owl#DnaRegion";
pub(crate) const BIOPAX_RNA: &str = "http://www.biopax.org/release/biopax-level3.owl#Rna";
pub(crate) const BIOPAX_RNA_REGION: &str =
    "http://www.biopax.org/release/biopax-level3.owl#RnaRegion";
pub(crate) const BIOPAX_PROTEIN: &str = "http://www.biopax.org/release/biopax-level3.owl#Protein";
pub(crate) const BIOPAX_SMALL_MOLECULE: &str =
    "http://www.biopax.org/release/biopax-level3.owl#SmallMolecule";
pub(crate) const BIOPAX_COMPLEX: &str = "http://www.biopax.org/release/biopax-level3.owl#Complex";

// SBOL 2 MapsTo refinement values, mapped to/from SBOL 3 Constraint restriction values.
pub(crate) const SBOL2_REFINEMENT_VERIFY_IDENTICAL: &str = "http://sbols.org/v2#verifyIdentical";
pub(crate) const SBOL2_REFINEMENT_USE_LOCAL: &str = "http://sbols.org/v2#useLocal";
pub(crate) const SBOL2_REFINEMENT_USE_REMOTE: &str = "http://sbols.org/v2#useRemote";
pub(crate) const SBOL2_REFINEMENT_MERGE: &str = "http://sbols.org/v2#merge";

// SBOL 2 FunctionalComponent.direction values.
pub(crate) const SBOL2_DIRECTION_IN: &str = "http://sbols.org/v2#in";
pub(crate) const SBOL2_DIRECTION_OUT: &str = "http://sbols.org/v2#out";
pub(crate) const SBOL2_DIRECTION_INOUT: &str = "http://sbols.org/v2#inout";
pub(crate) const SBOL2_DIRECTION_NONE: &str = "http://sbols.org/v2#none";

pub(crate) const SBOL2_ACCESS_PUBLIC: &str = "http://sbols.org/v2#public";
pub(crate) const SBOL2_ACCESS_PRIVATE: &str = "http://sbols.org/v2#private";

// SBOL 2 VariableComponent.operator values.
pub(crate) const SBOL2_ONE: &str = "http://sbols.org/v2#one";
pub(crate) const SBOL2_ZERO_OR_ONE: &str = "http://sbols.org/v2#zeroOrOne";
pub(crate) const SBOL2_ONE_OR_MORE: &str = "http://sbols.org/v2#oneOrMore";
pub(crate) const SBOL2_ZERO_OR_MORE: &str = "http://sbols.org/v2#zeroOrMore";

// SBOL 2 CombinatorialDerivation.strategy values.
pub(crate) const SBOL2_ENUMERATE: &str = "http://sbols.org/v2#enumerate";
pub(crate) const SBOL2_SAMPLE: &str = "http://sbols.org/v2#sample";

// SBOL 2 Component / FunctionalComponent roleIntegration values.
pub(crate) const SBOL2_MERGE_ROLES: &str = "http://sbols.org/v2#mergeRoles";
pub(crate) const SBOL2_OVERRIDE_ROLES: &str = "http://sbols.org/v2#overrideRoles";

