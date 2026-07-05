//! RDF vocabulary IRIs for the SBOL 2.3.0 data model plus the PROV-O, OM,
//! Dublin Core Terms, and RDF Schema terms the specification adopts.

pub const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
pub const SBOL2_NS: &str = "http://sbols.org/v2#";
pub const PROV_NS: &str = "http://www.w3.org/ns/prov#";
pub const OM_NS: &str = "http://www.ontology-of-units-of-measure.org/resource/om-2/";
pub const DCTERMS_NS: &str = "http://purl.org/dc/terms/";
pub const RDFS_NS: &str = "http://www.w3.org/2000/01/rdf-schema#";

// === SBOL 2 class IRIs (including the abstract mixins) ===

pub const SBOL2_IDENTIFIED_CLASS: &str = "http://sbols.org/v2#Identified";
pub const SBOL2_TOP_LEVEL_CLASS: &str = "http://sbols.org/v2#TopLevel";
pub const SBOL2_MEASURED_CLASS: &str = "http://sbols.org/v2#Measured";
pub const SBOL2_COMPONENT_INSTANCE_CLASS: &str = "http://sbols.org/v2#ComponentInstance";
pub const SBOL2_LOCATION_CLASS: &str = "http://sbols.org/v2#Location";
pub const SBOL2_SEQUENCE_CLASS: &str = "http://sbols.org/v2#Sequence";
pub const SBOL2_COMPONENT_DEFINITION_CLASS: &str = "http://sbols.org/v2#ComponentDefinition";
pub const SBOL2_MODULE_DEFINITION_CLASS: &str = "http://sbols.org/v2#ModuleDefinition";
pub const SBOL2_MODEL_CLASS: &str = "http://sbols.org/v2#Model";
pub const SBOL2_COLLECTION_CLASS: &str = "http://sbols.org/v2#Collection";
pub const SBOL2_COMBINATORIAL_DERIVATION_CLASS: &str =
    "http://sbols.org/v2#CombinatorialDerivation";
pub const SBOL2_IMPLEMENTATION_CLASS: &str = "http://sbols.org/v2#Implementation";
pub const SBOL2_ATTACHMENT_CLASS: &str = "http://sbols.org/v2#Attachment";
pub const SBOL2_EXPERIMENTAL_DATA_CLASS: &str = "http://sbols.org/v2#ExperimentalData";
pub const SBOL2_EXPERIMENT_CLASS: &str = "http://sbols.org/v2#Experiment";
pub const SBOL2_GENERIC_TOP_LEVEL_CLASS: &str = "http://sbols.org/v2#GenericTopLevel";
pub const SBOL2_COMPONENT_CLASS: &str = "http://sbols.org/v2#Component";
pub const SBOL2_FUNCTIONAL_COMPONENT_CLASS: &str = "http://sbols.org/v2#FunctionalComponent";
pub const SBOL2_MODULE_CLASS: &str = "http://sbols.org/v2#Module";
pub const SBOL2_MAPS_TO_CLASS: &str = "http://sbols.org/v2#MapsTo";
pub const SBOL2_SEQUENCE_ANNOTATION_CLASS: &str = "http://sbols.org/v2#SequenceAnnotation";
pub const SBOL2_SEQUENCE_CONSTRAINT_CLASS: &str = "http://sbols.org/v2#SequenceConstraint";
pub const SBOL2_VARIABLE_COMPONENT_CLASS: &str = "http://sbols.org/v2#VariableComponent";
pub const SBOL2_INTERACTION_CLASS: &str = "http://sbols.org/v2#Interaction";
pub const SBOL2_PARTICIPATION_CLASS: &str = "http://sbols.org/v2#Participation";
pub const SBOL2_RANGE_CLASS: &str = "http://sbols.org/v2#Range";
pub const SBOL2_CUT_CLASS: &str = "http://sbols.org/v2#Cut";
pub const SBOL2_GENERIC_LOCATION_CLASS: &str = "http://sbols.org/v2#GenericLocation";

// === PROV-O and OM class IRIs (namespace-identical to SBOL 3's adoption) ===

pub const PROV_ACTIVITY: &str = "http://www.w3.org/ns/prov#Activity";
pub const PROV_AGENT_CLASS: &str = "http://www.w3.org/ns/prov#Agent";
pub const PROV_PLAN: &str = "http://www.w3.org/ns/prov#Plan";
pub const PROV_ASSOCIATION: &str = "http://www.w3.org/ns/prov#Association";
pub const PROV_USAGE: &str = "http://www.w3.org/ns/prov#Usage";

pub const OM_MEASURE: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/Measure";
pub const OM_UNIT: &str = "http://www.ontology-of-units-of-measure.org/resource/om-2/Unit";
pub const OM_SINGULAR_UNIT: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/SingularUnit";
pub const OM_COMPOUND_UNIT: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/CompoundUnit";
pub const OM_UNIT_MULTIPLICATION: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/UnitMultiplication";
pub const OM_UNIT_DIVISION: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/UnitDivision";
pub const OM_UNIT_EXPONENTIATION: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/UnitExponentiation";
pub const OM_PREFIXED_UNIT: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/PrefixedUnit";
pub const OM_PREFIX: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/Prefix";
pub const OM_SI_PREFIX: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/SIPrefix";
pub const OM_BINARY_PREFIX: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/BinaryPrefix";

// === Identified / TopLevel predicates ===

pub const SBOL2_PERSISTENT_IDENTITY: &str = "http://sbols.org/v2#persistentIdentity";
pub const SBOL2_DISPLAY_ID: &str = "http://sbols.org/v2#displayId";
pub const SBOL2_VERSION: &str = "http://sbols.org/v2#version";
pub const DCTERMS_TITLE: &str = "http://purl.org/dc/terms/title";
pub const DCTERMS_DESCRIPTION: &str = "http://purl.org/dc/terms/description";
pub const PROV_WAS_DERIVED_FROM: &str = "http://www.w3.org/ns/prov#wasDerivedFrom";
pub const PROV_WAS_GENERATED_BY: &str = "http://www.w3.org/ns/prov#wasGeneratedBy";
pub const SBOL2_ATTACHMENT: &str = "http://sbols.org/v2#attachment";
pub const SBOL2_MEASURE: &str = "http://sbols.org/v2#measure";

// === ComponentInstance / Location predicates ===

pub const SBOL2_DEFINITION: &str = "http://sbols.org/v2#definition";
pub const SBOL2_ACCESS: &str = "http://sbols.org/v2#access";
pub const SBOL2_MAPS_TO: &str = "http://sbols.org/v2#mapsTo";
pub const SBOL2_ORIENTATION: &str = "http://sbols.org/v2#orientation";
pub const SBOL2_SEQUENCE: &str = "http://sbols.org/v2#sequence";

// === Sequence / ComponentDefinition / ModuleDefinition predicates ===

pub const SBOL2_ELEMENTS: &str = "http://sbols.org/v2#elements";
pub const SBOL2_ENCODING: &str = "http://sbols.org/v2#encoding";
pub const SBOL2_TYPE: &str = "http://sbols.org/v2#type";
pub const SBOL2_ROLE: &str = "http://sbols.org/v2#role";
pub const SBOL2_COMPONENT: &str = "http://sbols.org/v2#component";
pub const SBOL2_SEQUENCE_ANNOTATION: &str = "http://sbols.org/v2#sequenceAnnotation";
pub const SBOL2_SEQUENCE_CONSTRAINT: &str = "http://sbols.org/v2#sequenceConstraint";
pub const SBOL2_MODULE: &str = "http://sbols.org/v2#module";
pub const SBOL2_FUNCTIONAL_COMPONENT: &str = "http://sbols.org/v2#functionalComponent";
pub const SBOL2_INTERACTION: &str = "http://sbols.org/v2#interaction";
pub const SBOL2_MODEL: &str = "http://sbols.org/v2#model";

// === Model / Attachment predicates ===

pub const SBOL2_SOURCE: &str = "http://sbols.org/v2#source";
pub const SBOL2_LANGUAGE: &str = "http://sbols.org/v2#language";
pub const SBOL2_FRAMEWORK: &str = "http://sbols.org/v2#framework";
pub const SBOL2_FORMAT: &str = "http://sbols.org/v2#format";
pub const SBOL2_SIZE: &str = "http://sbols.org/v2#size";
pub const SBOL2_HASH: &str = "http://sbols.org/v2#hash";

// === Collection / CombinatorialDerivation / Implementation / Experiment ===

pub const SBOL2_MEMBER: &str = "http://sbols.org/v2#member";
pub const SBOL2_TEMPLATE: &str = "http://sbols.org/v2#template";
pub const SBOL2_VARIABLE_COMPONENT: &str = "http://sbols.org/v2#variableComponent";
pub const SBOL2_STRATEGY: &str = "http://sbols.org/v2#strategy";
pub const SBOL2_BUILT: &str = "http://sbols.org/v2#built";
pub const SBOL2_EXPERIMENTAL_DATA: &str = "http://sbols.org/v2#experimentalData";
pub const SBOL2_RDF_TYPE: &str = "http://sbols.org/v2#rdfType";

// === Component / FunctionalComponent / Module predicates ===

pub const SBOL2_ROLE_INTEGRATION: &str = "http://sbols.org/v2#roleIntegration";
pub const SBOL2_SOURCE_LOCATION: &str = "http://sbols.org/v2#sourceLocation";
pub const SBOL2_DIRECTION: &str = "http://sbols.org/v2#direction";

// === MapsTo / SequenceAnnotation / SequenceConstraint predicates ===

pub const SBOL2_LOCAL: &str = "http://sbols.org/v2#local";
pub const SBOL2_REMOTE: &str = "http://sbols.org/v2#remote";
pub const SBOL2_REFINEMENT: &str = "http://sbols.org/v2#refinement";
pub const SBOL2_LOCATION: &str = "http://sbols.org/v2#location";
pub const SBOL2_SUBJECT: &str = "http://sbols.org/v2#subject";
pub const SBOL2_OBJECT: &str = "http://sbols.org/v2#object";
pub const SBOL2_RESTRICTION: &str = "http://sbols.org/v2#restriction";

// === Range / Cut predicates ===

pub const SBOL2_START: &str = "http://sbols.org/v2#start";
pub const SBOL2_END: &str = "http://sbols.org/v2#end";
pub const SBOL2_AT: &str = "http://sbols.org/v2#at";

// === Interaction / Participation predicates ===

pub const SBOL2_PARTICIPATION: &str = "http://sbols.org/v2#participation";
pub const SBOL2_PARTICIPANT: &str = "http://sbols.org/v2#participant";

// === VariableComponent predicates ===

pub const SBOL2_VARIABLE: &str = "http://sbols.org/v2#variable";
pub const SBOL2_VARIANT: &str = "http://sbols.org/v2#variant";
pub const SBOL2_VARIANT_COLLECTION: &str = "http://sbols.org/v2#variantCollection";
pub const SBOL2_VARIANT_DERIVATION: &str = "http://sbols.org/v2#variantDerivation";
pub const SBOL2_OPERATOR: &str = "http://sbols.org/v2#operator";

// === PROV-O predicates ===

pub const PROV_STARTED_AT_TIME: &str = "http://www.w3.org/ns/prov#startedAtTime";
pub const PROV_ENDED_AT_TIME: &str = "http://www.w3.org/ns/prov#endedAtTime";
pub const PROV_QUALIFIED_ASSOCIATION: &str =
    "http://www.w3.org/ns/prov#qualifiedAssociation";
pub const PROV_QUALIFIED_USAGE: &str = "http://www.w3.org/ns/prov#qualifiedUsage";
pub const PROV_WAS_INFORMED_BY: &str = "http://www.w3.org/ns/prov#wasInformedBy";
pub const PROV_AGENT: &str = "http://www.w3.org/ns/prov#agent";
pub const PROV_HAD_ROLE: &str = "http://www.w3.org/ns/prov#hadRole";
pub const PROV_HAD_PLAN: &str = "http://www.w3.org/ns/prov#hadPlan";
pub const PROV_ENTITY: &str = "http://www.w3.org/ns/prov#entity";

// === OM predicates ===

pub const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
pub const RDFS_COMMENT: &str = "http://www.w3.org/2000/01/rdf-schema#comment";
pub const OM_SYMBOL: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/symbol";
pub const OM_ALTERNATIVE_SYMBOL: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/alternativeSymbol";
pub const OM_ALTERNATIVE_LABEL: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/alternativeLabel";
pub const OM_LONG_COMMENT: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/longcomment";
pub const OM_HAS_NUMERICAL_VALUE: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/hasNumericalValue";
pub const OM_HAS_UNIT: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/hasUnit";
pub const OM_HAS_FACTOR: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/hasFactor";
pub const OM_HAS_TERM1: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/hasTerm1";
pub const OM_HAS_TERM2: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/hasTerm2";
pub const OM_HAS_NUMERATOR: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/hasNumerator";
pub const OM_HAS_DENOMINATOR: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/hasDenominator";
pub const OM_HAS_BASE: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/hasBase";
pub const OM_HAS_EXPONENT: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/hasExponent";
pub const OM_HAS_PREFIX: &str =
    "http://www.ontology-of-units-of-measure.org/resource/om-2/hasPrefix";

// === Controlled values ===

pub const SBOL2_PUBLIC: &str = "http://sbols.org/v2#public";
pub const SBOL2_PRIVATE: &str = "http://sbols.org/v2#private";
pub const SBOL2_INLINE: &str = "http://sbols.org/v2#inline";
pub const SBOL2_REVERSE_COMPLEMENT: &str = "http://sbols.org/v2#reverseComplement";
