//! Free helper functions shared across the downgrade engine: IRI synthesis
//! and the SBOL 3 -> SBOL 2 type/predicate lookup tables.

use super::*;

/// Appends `/segment` to `iri`, collapsing a doubled `/` when `iri`
/// already ends with one.
pub(super) fn append_segment(iri: &str, segment: &str) -> String {
    if iri.ends_with('/') {
        format!("{iri}{segment}")
    } else {
        format!("{iri}/{segment}")
    }
}

/// Returns a child `(display_id, iri)` under `parent` whose IRI is not already
/// in `used`, inserting the chosen IRI. Disambiguates by appending `_2`,
/// `_3`, … and keeps displayId aligned with the child IRI.
pub(super) fn next_available_child_iri(
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

pub(super) fn type_set_contains(
    types_by_subject: &HashMap<String, HashSet<String>>,
    subject: &str,
    ty: &str,
) -> bool {
    types_by_subject
        .get(subject)
        .is_some_and(|types| types.contains(ty))
}

/// Default SBOL 3 type → SBOL 2 type mapping. Component maps to
/// ComponentDefinition here; the classification pass refines a Component to a
/// ModuleDefinition when its shape calls for it.
pub(super) fn map_sbol3_type_to_sbol2(iri: &str) -> Option<&'static str> {
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
        // Component subtypes that don't exist in SBOL 2: skip them
        // (caller surfaces an UnsupportedSbol3Type warning).
        _ => return None,
    })
}

/// SBOL 3 predicate → SBOL 2 predicate. Single-target rewrites only;
/// predicates that need context to resolve (`hasFeature` could be
/// `component`, `functionalComponent`, `module`, or `sequenceAnnotation`
/// depending on what kind of feature it points at) are handled by the
/// subject-aware predicate pass instead.
pub(super) fn map_sbol3_predicate_to_sbol2(iri: &str) -> Option<&'static str> {
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
        // `hasNamespace` is dropped earlier (no SBOL 2 equivalent;
        // the namespace lives implicitly in the restored
        // persistentIdentity).
        _ => return None,
    })
}
