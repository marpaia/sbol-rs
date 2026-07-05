//! Common ontology IRIs for building SBOL 2 documents.
//!
//! These are canonical IRIs from BioPAX, the Systems Biology Ontology (SBO),
//! the Sequence Ontology (SO), and the SBOL 2 vocabulary itself. Each constant
//! is a [`pub const Iri`](crate::Iri), so they cost nothing as builder
//! arguments.

use crate::Iri;
use crate::vocab;

// === BioPAX: ComponentDefinition.type values ===

pub const BIOPAX_DNA: Iri =
    Iri::from_static("http://www.biopax.org/release/biopax-level3.owl#DnaRegion");
pub const BIOPAX_RNA: Iri =
    Iri::from_static("http://www.biopax.org/release/biopax-level3.owl#RnaRegion");
pub const BIOPAX_PROTEIN: Iri =
    Iri::from_static("http://www.biopax.org/release/biopax-level3.owl#Protein");
pub const BIOPAX_SMALL_MOLECULE: Iri =
    Iri::from_static("http://www.biopax.org/release/biopax-level3.owl#SmallMolecule");
pub const BIOPAX_COMPLEX: Iri =
    Iri::from_static("http://www.biopax.org/release/biopax-level3.owl#Complex");

// === Sequence Ontology (SO): ComponentDefinition.role values ===

pub const SO_PROMOTER: Iri = Iri::from_static("http://identifiers.org/so/SO:0000167");
pub const SO_CDS: Iri = Iri::from_static("http://identifiers.org/so/SO:0000316");
pub const SO_RBS: Iri = Iri::from_static("http://identifiers.org/so/SO:0000139");
pub const SO_TERMINATOR: Iri = Iri::from_static("http://identifiers.org/so/SO:0000141");
pub const SO_ENGINEERED_REGION: Iri = Iri::from_static("http://identifiers.org/so/SO:0000804");

// === Sequence encodings ===

pub const IUPAC_DNA_ENCODING: Iri =
    Iri::from_static("http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html");
pub const IUPAC_PROTEIN_ENCODING: Iri =
    Iri::from_static("http://www.chem.qmul.ac.uk/iupac/AminoAcid/");

// === ComponentInstance.access values ===

pub const ACCESS_PUBLIC: Iri = Iri::from_static(vocab::SBOL2_PUBLIC);
pub const ACCESS_PRIVATE: Iri = Iri::from_static(vocab::SBOL2_PRIVATE);

// === Location.orientation values ===

pub const ORIENTATION_INLINE: Iri = Iri::from_static(vocab::SBOL2_INLINE);
pub const ORIENTATION_REVERSE_COMPLEMENT: Iri =
    Iri::from_static(vocab::SBOL2_REVERSE_COMPLEMENT);

// === FunctionalComponent.direction values ===

pub const DIRECTION_IN: Iri = Iri::from_static("http://sbols.org/v2#in");
pub const DIRECTION_OUT: Iri = Iri::from_static("http://sbols.org/v2#out");
pub const DIRECTION_INOUT: Iri = Iri::from_static("http://sbols.org/v2#inout");
pub const DIRECTION_NONE: Iri = Iri::from_static("http://sbols.org/v2#none");

// === SequenceConstraint.restriction values ===

pub const RESTRICTION_PRECEDES: Iri = Iri::from_static("http://sbols.org/v2#precedes");
pub const RESTRICTION_SAME_ORIENTATION_AS: Iri =
    Iri::from_static("http://sbols.org/v2#sameOrientationAs");
pub const RESTRICTION_OPPOSITE_ORIENTATION_AS: Iri =
    Iri::from_static("http://sbols.org/v2#oppositeOrientationAs");
pub const RESTRICTION_DIFFERENT_FROM: Iri =
    Iri::from_static("http://sbols.org/v2#differentFrom");

// === MapsTo.refinement values ===

pub const REFINEMENT_USE_REMOTE: Iri = Iri::from_static("http://sbols.org/v2#useRemote");
pub const REFINEMENT_USE_LOCAL: Iri = Iri::from_static("http://sbols.org/v2#useLocal");
pub const REFINEMENT_VERIFY_IDENTICAL: Iri =
    Iri::from_static("http://sbols.org/v2#verifyIdentical");
pub const REFINEMENT_MERGE: Iri = Iri::from_static("http://sbols.org/v2#merge");

// === CombinatorialDerivation.strategy values ===

pub const STRATEGY_ENUMERATE: Iri = Iri::from_static("http://sbols.org/v2#enumerate");
pub const STRATEGY_SAMPLE: Iri = Iri::from_static("http://sbols.org/v2#sample");

// === VariableComponent.operator values ===

pub const OPERATOR_ZERO_OR_ONE: Iri = Iri::from_static("http://sbols.org/v2#zeroOrOne");
pub const OPERATOR_ONE: Iri = Iri::from_static("http://sbols.org/v2#one");
pub const OPERATOR_ZERO_OR_MORE: Iri = Iri::from_static("http://sbols.org/v2#zeroOrMore");
pub const OPERATOR_ONE_OR_MORE: Iri = Iri::from_static("http://sbols.org/v2#oneOrMore");
