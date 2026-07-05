//! Compliant SBOL 2 identity URL construction.
//!
//! An SBOL 2 compliant identity bakes the version into the URL: a TopLevel is
//! `{namespace}/{displayId}/{version}` and a child is
//! `{parent_persistentIdentity}/{displayId}/{version}`, while the
//! `persistentIdentity` is the same URL without the trailing version. When no
//! version is supplied the SBOL 2 convention `"1"` is used, matching the
//! vendored fixtures.

use crate::error::BuildError;
use crate::identity::{DisplayId, Namespace};
use crate::{Iri, Resource};

pub(crate) const DEFAULT_VERSION: &str = "1";

/// The `(identity, persistentIdentity)` pair for a TopLevel object.
pub(crate) fn build_top_level_identity(
    namespace: &Namespace,
    display_id: &DisplayId,
    version: &str,
) -> (Resource, Resource) {
    let persistent = format!("{}/{}", namespace.as_str(), display_id.as_str());
    let identity = format!("{persistent}/{version}");
    (
        Resource::Iri(Iri::new_unchecked(identity)),
        Resource::Iri(Iri::new_unchecked(persistent)),
    )
}

/// The `(identity, persistentIdentity)` pair for a child object, derived from
/// the parent's `persistentIdentity`.
pub(crate) fn build_child_identity(
    parent_persistent: &Resource,
    display_id: &DisplayId,
    version: &str,
) -> Result<(Resource, Resource), BuildError> {
    let parent = parent_persistent.as_iri().ok_or_else(|| {
        BuildError::InvalidNamespace(format!(
            "parent persistentIdentity must be an IRI, got blank node `{parent_persistent}`"
        ))
    })?;
    let persistent = format!("{}/{}", parent.as_str(), display_id.as_str());
    let identity = format!("{persistent}/{version}");
    Ok((
        Resource::Iri(Iri::new_unchecked(identity)),
        Resource::Iri(Iri::new_unchecked(persistent)),
    ))
}

/// Recomputes an identity URL from a `persistentIdentity` and a version,
/// used by the `version` / `persistent_identity` builder setters.
pub(crate) fn identity_from(persistent: &Resource, version: &str) -> Resource {
    match persistent.as_iri() {
        Some(iri) => Resource::Iri(Iri::new_unchecked(format!("{}/{version}", iri.as_str()))),
        None => persistent.clone(),
    }
}
