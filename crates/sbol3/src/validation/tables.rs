use sbol_ontology::{ComponentTypeFamily, Ontology, OntologyNamespace};

const EDAM_FORMAT: &str = "EDAM:format_1915";
const EDAM_TEXTUAL_FORMAT: &str = "EDAM:format_2330";
const SO_TOPOLOGY_ATTRIBUTE: &str = "SO:0000986";
const SO_STRAND_ATTRIBUTE: &str = "SO:0000983";
const SBO_MODELING_FRAMEWORK: &str = "SBO:0000004";
const SBO_SYSTEMS_DESCRIPTION_PARAMETER: &str = "SBO:0000545";
const SBO_PHYSICAL_ENTITY: &str = "SBO:0000236";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ComponentType {
    iri: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SequenceEncoding {
    iri: String,
}

pub(crate) fn is_table_2_component_type(ontology: &Ontology, iri: &str) -> bool {
    ontology.is_table_2_component_type(iri)
}

pub(crate) fn component_type(ontology: &Ontology, iri: &str) -> Option<ComponentType> {
    ontology
        .is_table_2_component_type(iri)
        .then(|| ComponentType {
            iri: ontology.canonical_iri(iri).unwrap_or(iri).to_owned(),
        })
}

pub(crate) fn is_component_type_term(ontology: &Ontology, iri: &str) -> Option<bool> {
    ontology.is_component_type_term(iri)
}

pub(crate) fn component_type_name(ontology: &Ontology, component_type: &ComponentType) -> String {
    ontology
        .label(&component_type.iri)
        .unwrap_or("unknown Component type")
        .to_owned()
}

pub(crate) fn component_type_has_cross_listed_encoding(
    ontology: &Ontology,
    component_type: &ComponentType,
) -> bool {
    !ontology
        .compatible_sequence_encodings_for_component_type(&component_type.iri)
        .is_empty()
}

pub(crate) fn type_terms_conflict(ontology: &Ontology, left: &str, right: &str) -> Option<bool> {
    ontology.terms_conflict(left, right)
}

pub(crate) fn is_nucleic_acid_component_type(ontology: &Ontology, iri: &str) -> bool {
    ontology.component_type_family(iri) == Some(ComponentTypeFamily::NucleicAcid)
}

pub(crate) fn component_type_family(ontology: &Ontology, iri: &str) -> Option<ComponentTypeFamily> {
    ontology.component_type_family(iri)
}

pub(crate) fn is_topology_type_term(ontology: &Ontology, iri: &str) -> Option<bool> {
    ontology
        .contains_term(iri)
        .then(|| ontology.is_in_branch(iri, SO_TOPOLOGY_ATTRIBUTE))
}

pub(crate) fn is_strand_type_term(ontology: &Ontology, iri: &str) -> Option<bool> {
    ontology
        .contains_term(iri)
        .then(|| ontology.is_in_branch(iri, SO_STRAND_ATTRIBUTE))
}

pub(crate) fn is_feature_role_term(ontology: &Ontology, iri: &str) -> Option<bool> {
    ontology.is_feature_role_term(iri)
}

pub(crate) fn is_component_role_term(ontology: &Ontology, iri: &str) -> Option<bool> {
    ontology.is_component_role_term(iri)
}

pub(crate) fn is_sequence_feature_role_term(ontology: &Ontology, iri: &str) -> Option<bool> {
    ontology.is_sequence_feature_role_term(iri)
}

#[allow(dead_code)] // exposed for future recommendation-rule wiring on Component.type
pub(crate) fn is_cell_type_term(ontology: &Ontology, iri: &str) -> Option<bool> {
    ontology.is_cell_type_term(iri)
}

pub(crate) fn component_role_compatible_with_component_type(
    ontology: &Ontology,
    role: &str,
    component_type: &str,
) -> Option<bool> {
    ontology.component_role_compatible_with_component_type(role, component_type)
}

pub(crate) fn is_interaction_type_term(ontology: &Ontology, iri: &str) -> Option<bool> {
    ontology.is_interaction_type_term(iri)
}

pub(crate) fn is_participation_role_term(ontology: &Ontology, iri: &str) -> Option<bool> {
    ontology.is_participation_role_term(iri)
}

pub(crate) fn participation_role_compatible_with_interaction_type(
    ontology: &Ontology,
    role: &str,
    interaction_type: &str,
) -> Option<bool> {
    ontology.participation_role_compatible_with_interaction_type(role, interaction_type)
}

pub(crate) fn sequence_encoding(ontology: &Ontology, iri: &str) -> Option<SequenceEncoding> {
    ontology
        .is_sequence_encoding_term(iri)
        .is_some_and(|is_encoding| is_encoding)
        .then(|| SequenceEncoding {
            iri: ontology.canonical_iri(iri).unwrap_or(iri).to_owned(),
        })
}

pub(crate) fn is_sequence_encoding_term(ontology: &Ontology, iri: &str) -> Option<bool> {
    ontology.is_sequence_encoding_term(iri)
}

pub(crate) fn sequence_encoding_name(ontology: &Ontology, encoding: &SequenceEncoding) -> String {
    ontology
        .label(&encoding.iri)
        .unwrap_or("unknown Sequence encoding")
        .to_owned()
}

pub(crate) fn canonical_table_1_sequence_encoding_iri(
    ontology: &Ontology,
    iri: &str,
) -> Option<String> {
    ontology
        .is_table_1_sequence_encoding(iri)
        .then(|| ontology.canonical_iri(iri).unwrap_or(iri).to_owned())
}

pub(crate) fn canonical_known_iri(ontology: &Ontology, iri: &str) -> Option<String> {
    ontology.canonical_iri(iri).map(ToOwned::to_owned)
}

pub(crate) fn is_edam_textual_format(ontology: &Ontology, iri: &str) -> Option<bool> {
    ontology
        .contains_term(iri)
        .then(|| ontology.is_in_branch(iri, EDAM_TEXTUAL_FORMAT))
}

pub(crate) fn is_sbo_modeling_framework_term(ontology: &Ontology, iri: &str) -> Option<bool> {
    ontology
        .contains_term(iri)
        .then(|| ontology.is_in_branch(iri, SBO_MODELING_FRAMEWORK))
}

pub(crate) fn is_sbo_systems_description_parameter_term(
    ontology: &Ontology,
    iri: &str,
) -> Option<bool> {
    ontology
        .contains_term(iri)
        .then(|| ontology.is_in_branch(iri, SBO_SYSTEMS_DESCRIPTION_PARAMETER))
}

pub(crate) fn is_sbo_physical_entity_term(ontology: &Ontology, iri: &str) -> Option<bool> {
    if ontology.namespace(iri) != Some(OntologyNamespace::Sbo) {
        return None;
    }
    Some(ontology.is_in_branch(iri, SBO_PHYSICAL_ENTITY))
}

pub(crate) fn is_edam_format_term(ontology: &Ontology, iri: &str) -> Option<bool> {
    ontology.contains_term(iri).then(|| {
        ontology.namespace(iri) == Some(OntologyNamespace::Edam)
            && ontology.is_in_branch(iri, EDAM_FORMAT)
    })
}

/// Table 15: Model language EDAM terms (canonical form).
/// Aliasing accepts the http/https flip; the term tail is matched
/// case-insensitively so `EDAM:format_2585` and `edam:format_2585`
/// both normalize to the canonical lowercase form.
const MODEL_LANGUAGE_TABLE_15: &[&str] = &[
    "https://identifiers.org/edam:format_2585", // SBML
    "https://identifiers.org/edam:format_3240", // CellML
    "https://identifiers.org/edam:format_3156", // BioPAX
];

/// Table 16: Model framework SBO terms (canonical form).
const MODEL_FRAMEWORK_TABLE_16: &[&str] = &[
    "https://identifiers.org/SBO:0000062", // Continuous
    "https://identifiers.org/SBO:0000063", // Discrete
];

fn matches_table_entry(iri: &str, table: &[&'static str]) -> Option<&'static str> {
    table.iter().copied().find(|canonical| {
        iri.eq_ignore_ascii_case(canonical)
            || iri.eq_ignore_ascii_case(&canonical.replace("https://", "http://"))
    })
}

/// Returns the canonical Table 15 (Model language) URI if `iri` is a
/// known alias of one of the three Table 15 entries. Returns `None`
/// for unknown IRIs. The validator treats unknown as undecided rather
/// than emitting a warning, mirroring the Table 1 sequence-encoding
/// pattern.
pub(crate) fn canonical_model_language_iri(iri: &str) -> Option<&'static str> {
    matches_table_entry(iri, MODEL_LANGUAGE_TABLE_15)
}

/// Returns the canonical Table 16 (Model framework) URI if `iri` is a
/// known alias of one of the two Table 16 entries.
pub(crate) fn canonical_model_framework_iri(iri: &str) -> Option<&'static str> {
    matches_table_entry(iri, MODEL_FRAMEWORK_TABLE_16)
}

/// Returns `true` when `iri` is in a known non-EDAM bundled ontology
/// namespace. Used as the "known wrong" half of the
/// `sbol3-12504` check: warn only when we are confident the language
/// IRI is from somewhere other than EDAM (SBO, SO, GO, ChEBI, CL), and
/// stay silent for unknown URIs and EDAM terms not in the bundle.
pub(crate) fn is_known_non_edam_namespace(ontology: &Ontology, iri: &str) -> bool {
    ontology
        .namespace(iri)
        .is_some_and(|ns| ns != OntologyNamespace::Edam)
}

pub(crate) fn sequence_encoding_compatible_with_component_type(
    ontology: &Ontology,
    encoding: &SequenceEncoding,
    component_type: &ComponentType,
) -> bool {
    ontology
        .encoding_compatible_with_component_type(&encoding.iri, &component_type.iri)
        .unwrap_or(false)
}

pub(crate) fn sequence_encodings_conflict(
    ontology: &Ontology,
    left: &SequenceEncoding,
    right: &SequenceEncoding,
) -> bool {
    ontology
        .terms_conflict(&left.iri, &right.iri)
        .unwrap_or(false)
}
