//! Identity URL construction helpers used by the typed constructors.
//!
//! For TopLevel classes the compliant identity is `{namespace}/{display_id}`
//! (SBOL rule sbol3-10102). For child classes the compliant identity is
//! `{parent_identity}/{display_id}` (rule sbol3-10104). These helpers are the
//! single place those URL shapes are assembled.

use crate::error::BuildError;
use crate::identity::{DisplayId, Namespace};
use crate::{Iri, Resource};

pub(crate) fn build_top_level_identity(namespace: &Namespace, display_id: &DisplayId) -> Resource {
    Resource::Iri(Iri::new_unchecked(format!(
        "{}/{}",
        namespace.as_str(),
        display_id.as_str()
    )))
}

pub(crate) fn build_child_identity(
    parent: &Resource,
    display_id: &DisplayId,
) -> Result<Resource, BuildError> {
    let parent_iri = parent.as_iri().ok_or_else(|| {
        BuildError::InvalidNamespace(format!(
            "parent identity must be an IRI, got blank node `{parent}`"
        ))
    })?;
    Ok(Resource::Iri(Iri::new_unchecked(format!(
        "{}/{}",
        parent_iri.as_str(),
        display_id.as_str()
    ))))
}
