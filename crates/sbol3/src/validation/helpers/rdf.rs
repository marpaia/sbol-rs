//! RDF type inspection: class-hierarchy checks and target/external-resource classification.

use super::*;
use crate::object::ObjectClasses;

pub(crate) fn known_external_resource(iri: &str) -> Option<ExternalResource> {
    let iri = iri.to_ascii_lowercase();
    if iri.starts_with("https://identifiers.org/uniprot:")
        || iri.starts_with("http://identifiers.org/uniprot:")
        || iri.starts_with("http://purl.uniprot.org/uniprot/")
        || iri.starts_with("https://www.uniprot.org/uniprot/")
        || iri.starts_with("https://www.uniprot.org/uniprotkb/")
    {
        return Some(ExternalResource::Uniprot);
    }
    if iri.starts_with("https://identifiers.org/chebi:")
        || iri.starts_with("http://identifiers.org/chebi:")
        || iri.starts_with("http://purl.obolibrary.org/obo/chebi_")
        || iri.starts_with("https://www.ebi.ac.uk/chebi/")
    {
        return Some(ExternalResource::Chebi);
    }
    if iri.starts_with("https://identifiers.org/pubchem.compound:")
        || iri.starts_with("http://identifiers.org/pubchem.compound:")
        || iri.starts_with("https://pubchem.ncbi.nlm.nih.gov/compound/")
    {
        return Some(ExternalResource::Pubchem);
    }
    None
}

pub(crate) fn object_matches_target(object: &Object, target: TargetClass) -> bool {
    match target {
        TargetClass::Sbol(class_iri) => {
            crate::SbolClass::from_iri(&crate::Iri::new_unchecked(class_iri))
                .is_some_and(|class| object.has_class(class))
        }
        TargetClass::ProvActivity => object_has_rdf_type(object, PROV_ACTIVITY),
        TargetClass::ProvAgent => object_has_rdf_type(object, PROV_AGENT_CLASS),
        TargetClass::ProvAssociation => object_has_rdf_type(object, PROV_ASSOCIATION),
        TargetClass::ProvPlan => object_has_rdf_type(object, PROV_PLAN),
        TargetClass::ProvUsage => object_has_rdf_type(object, PROV_USAGE),
        TargetClass::OmMeasure => object_has_rdf_type(object, OM_MEASURE),
        TargetClass::OmUnit => object_type_is_or_inherits(object, OM_UNIT),
        TargetClass::OmPrefix => object_type_is_or_inherits(object, OM_PREFIX),
        _ => false,
    }
}

pub(crate) fn is_external_top_level_reference(target: TargetClass) -> bool {
    matches!(
        target,
        TargetClass::Sbol(class_iri)
            if crate::SbolClass::from_iri(&crate::Iri::new_unchecked(class_iri))
                .is_some_and(|class| class.is_top_level())
    )
}

pub(crate) fn object_has_rdf_type(object: &Object, iri: &str) -> bool {
    object
        .rdf_types()
        .iter()
        .any(|rdf_type| rdf_type.as_str() == iri)
}

pub(crate) fn object_type_is_or_inherits(object: &Object, target: &str) -> bool {
    object
        .rdf_types()
        .iter()
        .any(|rdf_type| class_inherits(rdf_type.as_str(), target))
}

pub(crate) fn class_inherits(class: &str, target: &str) -> bool {
    if class == target {
        return true;
    }
    let Some(spec) = class_spec(class) else {
        return false;
    };
    spec.parents
        .iter()
        .any(|parent| class_inherits(parent, target))
}
