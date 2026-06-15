//! [`OntologyRegistry`]: a read-only view layering extension snapshots over
//! the bundled [`Ontology`].

use std::borrow::Cow;

use crate::Ontology;

/// A read-only view that layers zero-or-more extension snapshots on top of
/// the bundled [`Ontology`].
///
/// The bundled snapshot is always present; extensions add new terms, aliases,
/// branch memberships, and compatibility rows without overriding bundled
/// facts. Construct one through [`OntologyRegistry::bundled_only`] or
/// [`OntologyRegistry::bundled_with`] and pass it (or its inner [`Ontology`])
/// to the validator.
#[derive(Clone, Debug)]
pub struct OntologyRegistry {
    inner: Cow<'static, Ontology>,
}

impl OntologyRegistry {
    /// Registry containing only the bundled snapshot. Zero allocation.
    pub fn bundled_only() -> Self {
        Self {
            inner: Cow::Borrowed(Ontology::bundled()),
        }
    }

    /// Registry containing the bundled snapshot plus the provided extension
    /// snapshots applied in order. Bundled facts win on conflicts.
    pub fn bundled_with<I>(extensions: I) -> Self
    where
        I: IntoIterator<Item = Ontology>,
    {
        let mut iter = extensions.into_iter();
        let Some(first) = iter.next() else {
            return Self::bundled_only();
        };
        let mut merged = Ontology::bundled().clone();
        merged.extend_with(first);
        for ext in iter {
            merged.extend_with(ext);
        }
        Self {
            inner: Cow::Owned(merged),
        }
    }

    /// Adds another extension snapshot on top of this registry.
    pub fn with_extension(mut self, extension: Ontology) -> Self {
        let merged = self.inner.to_mut();
        merged.extend_with(extension);
        self
    }

    /// Returns the merged snapshot as an [`Ontology`].
    pub fn ontology(&self) -> &Ontology {
        self.inner.as_ref()
    }
}

impl Default for OntologyRegistry {
    fn default() -> Self {
        Self::bundled_only()
    }
}

impl AsRef<Ontology> for OntologyRegistry {
    fn as_ref(&self) -> &Ontology {
        self.ontology()
    }
}
