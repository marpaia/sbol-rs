//! SBOL 2 vocabulary IRIs shared by the upgrade and downgrade modules.
//!
//! These mirror the constants in `sbols.org/v2` and the
//! `sboltools/sbolgraph` TypeScript port. They are intentionally kept
//! `pub(crate)`: SBOL 2 URIs are an implementation detail of the
//! conversion pipeline, never part of the public SBOL 3 surface.
//!
//! Backport namespace constants follow the convention used by
//! `sbolgraph` for round-trip preservation of SBOL 2-only fields that
//! no longer have an SBOL 3 home (`persistentIdentity`, `version`, the
//! original `sbol2type`, and structural-collapse hints).
//!
//! Both directions of the conversion (upgrade and downgrade) reference
//! these symbols, which is why they live in a shared module rather
//! than scoped to one or the other.
#![allow(dead_code)]

pub(crate) const SBOL2_NS: &str = "http://sbols.org/v2#";
pub(crate) const BACKPORT_NS: &str = "http://sboltools.org/backport#";

/// Prefix for SBOL 2 predicates that the upgrade could not route through
/// its rename table. The local name follows `sbol2_`.
pub(crate) const BACKPORT_SBOL2_PREFIX: &str = "http://sboltools.org/backport#sbol2_";

/// Prefix for SBOL 3 predicates that the downgrade could not route
/// through its rename table. The local name follows `sbol3_`.
pub(crate) const BACKPORT_SBOL3_PREFIX: &str = "http://sboltools.org/backport#sbol3_";
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

// === Backport namespace (SBOL 2 ↔ SBOL 3 round-trip preservation) ===

pub(crate) const BACKPORT_SBOL2_TYPE: &str = "http://sboltools.org/backport#sbol2type";
pub(crate) const BACKPORT_SBOL2_PERSISTENT_IDENTITY: &str =
    "http://sboltools.org/backport#sbol2persistentIdentity";
pub(crate) const BACKPORT_SBOL2_VERSION: &str = "http://sboltools.org/backport#sbol2version";
pub(crate) const BACKPORT_SBOL2_DIRECTION: &str = "http://sboltools.org/backport#sbol2_direction";

/// FunctionalComponent `access`, preserved through the round trip. Byte
/// interoperable with sbol-utilities' `BACKPORT_NAMESPACE + 'sbol2_access'`.
/// The generic `sbol2_` prefix already produces this exact IRI for the
/// unmapped `sbol2:access` predicate; the named constant documents the
/// interop contract and lets both directions match on it explicitly.
pub(crate) const BACKPORT_SBOL2_ACCESS: &str = "http://sboltools.org/backport#sbol2_access";

/// The SBOL 3 object's `hasNamespace`, stashed on the corresponding SBOL 2
/// object by the downgrade so sbol-utilities / sbolgraph can reconstruct
/// the SBOL 3 namespace, and read by the upgrade when present. Byte
/// interoperable with sbol-utilities' `BACKPORT3_NAMESPACE`.
pub(crate) const BACKPORT_SBOL3_NAMESPACE: &str = "http://sboltools.org/backport#sbol3namespace";
pub(crate) const BACKPORT_SEQUENCE_ANNOTATION_DISPLAY_ID: &str =
    "http://sboltools.org/backport#sequenceAnnotationDisplayId";
pub(crate) const BACKPORT_SEQUENCE_ANNOTATION_PREDICATE_PREFIX: &str =
    "http://sboltools.org/backport#sequenceAnnotationPredicate_";

/// Records the original `sbol2:refinement` of a MapsTo on the SBOL 3
/// ComponentReference that replaces it. The forward map from
/// `{useLocal, useRemote}` collapses to a single `sbol3:replaces`
/// restriction, so the downgrade needs this hint to restore the
/// original refinement without guessing.
pub(crate) const BACKPORT_MAPS_TO_REFINEMENT: &str =
    "http://sboltools.org/backport#mapsToRefinement";

/// Records the original SBOL 2 MapsTo displayId when the upgrade had to
/// rename the synthesized SBOL 3 ComponentReference to avoid an IRI
/// collision under the enclosing Component.
pub(crate) const BACKPORT_MAPS_TO_DISPLAY_ID: &str =
    "http://sboltools.org/backport#mapsToDisplayId";

/// Records the original BioPAX type URI from a `sbol2:type` triple on a
/// ComponentDefinition. The forward map collapses `BIOPAX_DNA` and
/// `BIOPAX_DNA_REGION` to the same SBO term (and likewise for Rna), so
/// the downgrade needs this hint to restore the original BioPAX
/// variant without guessing.
pub(crate) const BACKPORT_BIOPAX_TYPE: &str = "http://sboltools.org/backport#biopaxType";

/// Stamped on the CD and MD halves of a dual-role Component split so the
/// inverse direction (and external tools) can see they share an SBOL 3
/// origin. The object IRI is the original SBOL 3 Component identity.
pub(crate) const BACKPORT_SBOL3_IDENTITY: &str = "http://sboltools.org/backport#sbol3identity";

/// Marks the synthesized FunctionalComponent that links the MD half of a
/// dual-role split to its CD half. Object value: this marker IRI.
pub(crate) const BACKPORT_TYPE: &str = "http://sboltools.org/backport#type";
pub(crate) const BACKPORT_SPLIT_COMPONENT_COMPOSITION: &str =
    "http://sboltools.org/backport#SplitComponentComposition";
