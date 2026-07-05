//! Validated identity helpers used by the typed builder surface.
//!
//! These newtypes catch invalid SBOL identifiers and namespaces at construction
//! time rather than at `Document::validate*` time, so a `Component::new(...)`
//! call fails fast on inputs the validator would otherwise reject.

use sbol_rdf::{Iri, Resource};

use crate::error::BuildError;
use crate::syntax::is_valid_display_id;

/// A validated SBOL `displayId`.
///
/// Constructed values are guaranteed to match the SBOL 3.1.0 lexical form for
/// `sbol:displayId` (rule sbol3-10201): the first character is an ASCII letter
/// or underscore, and every remaining character is an ASCII alphanumeric or
/// underscore.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DisplayId(String);

impl DisplayId {
    /// Build a `DisplayId` from a string, returning [`BuildError::InvalidDisplayId`]
    /// if the value fails the SBOL lexical form check.
    pub fn new(value: impl Into<String>) -> Result<Self, BuildError> {
        let value = value.into();
        if !is_valid_display_id(&value) {
            return Err(BuildError::InvalidDisplayId(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for DisplayId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DisplayId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<&str> for DisplayId {
    type Error = BuildError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for DisplayId {
    type Error = BuildError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&String> for DisplayId {
    type Error = BuildError;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::new(value.clone())
    }
}

/// A validated SBOL namespace URL.
///
/// Constructed values use an `http` or `https` scheme, contain no trailing
/// slash, and have at least a host component. The validator enforces the same
/// constraints under rule sbol3-10301.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Namespace(Iri);

impl Namespace {
    pub fn new(value: impl Into<String>) -> Result<Self, BuildError> {
        let value = value.into();
        if !is_valid_namespace(&value) {
            return Err(BuildError::InvalidNamespace(value));
        }
        Ok(Self(Iri::new_unchecked(value)))
    }

    pub fn as_iri(&self) -> &Iri {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_iri(self) -> Iri {
        self.0
    }
}

impl AsRef<str> for Namespace {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl TryFrom<&str> for Namespace {
    type Error = BuildError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for Namespace {
    type Error = BuildError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&String> for Namespace {
    type Error = BuildError;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::new(value.clone())
    }
}

fn is_valid_namespace(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let lower = value.to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return false;
    }
    if value.ends_with('/') {
        return false;
    }
    let scheme_end = lower.find("://").expect("scheme presence checked above") + 3;
    let after_scheme = &value[scheme_end..];
    if after_scheme.is_empty() {
        return false;
    }
    let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
    let host = &after_scheme[..host_end];
    !host.is_empty()
}

/// A namespace + displayId pair that resolves to a compliant SBOL identity URL.
///
/// Use this when you want to keep the two pieces around for traversal or
/// reporting; the typed `new`/`builder` constructors accept the pieces
/// separately so this type is optional.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SbolIdentity {
    namespace: Namespace,
    display_id: DisplayId,
}

impl SbolIdentity {
    pub fn new(namespace: Namespace, display_id: DisplayId) -> Self {
        Self {
            namespace,
            display_id,
        }
    }

    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    pub fn display_id(&self) -> &DisplayId {
        &self.display_id
    }

    /// Compliant identity URL: `{namespace}/{display_id}`.
    pub fn to_iri(&self) -> Iri {
        Iri::new_unchecked(format!(
            "{}/{}",
            self.namespace.as_str(),
            self.display_id.as_str()
        ))
    }

    /// Compliant identity as a [`Resource`].
    pub fn to_resource(&self) -> Resource {
        Resource::Iri(self.to_iri())
    }
}

impl std::fmt::Display for SbolIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.namespace, self.display_id)
    }
}

/// A validated hash algorithm token for `Attachment.hashAlgorithm`.
///
/// The set of accepted tokens mirrors the SBOL 3.1.0 specification. The
/// validator rejects values outside this set under rule sbol3-10802.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HashAlgorithm(&'static str);

impl HashAlgorithm {
    pub const SHA1: Self = Self("SHA1");
    pub const SHA224: Self = Self("SHA224");
    pub const SHA256: Self = Self("SHA256");
    pub const SHA384: Self = Self("SHA384");
    pub const SHA512: Self = Self("SHA512");
    pub const SHA3_224: Self = Self("SHA3-224");
    pub const SHA3_256: Self = Self("SHA3-256");
    pub const SHA3_384: Self = Self("SHA3-384");
    pub const SHA3_512: Self = Self("SHA3-512");

    const ALL: &'static [&'static str] = &[
        "SHA1", "SHA224", "SHA256", "SHA384", "SHA512", "SHA3-224", "SHA3-256", "SHA3-384",
        "SHA3-512",
    ];

    pub fn new(value: &str) -> Result<Self, BuildError> {
        Self::ALL
            .iter()
            .find(|token| **token == value)
            .map(|token| Self(token))
            .ok_or_else(|| BuildError::InvalidHashAlgorithm(value.to_string()))
    }

    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl AsRef<str> for HashAlgorithm {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl std::fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl TryFrom<&str> for HashAlgorithm {
    type Error = BuildError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for HashAlgorithm {
    type Error = BuildError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

/// A sequence's `elements` literal paired with its `encoding` IRI.
///
/// This is a convenience wrapper around the two fields a caller almost always
/// supplies together; the underlying `Sequence` stores them as separate
/// optional fields for serialization symmetry with the RDF model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceElements {
    pub elements: String,
    pub encoding: Iri,
}

impl SequenceElements {
    pub fn new(elements: impl Into<String>, encoding: Iri) -> Self {
        Self {
            elements: elements.into(),
            encoding,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_id_accepts_valid_form() {
        assert!(DisplayId::new("c").is_ok());
        assert!(DisplayId::new("_c").is_ok());
        assert!(DisplayId::new("my_component_1").is_ok());
    }

    #[test]
    fn display_id_rejects_invalid_form() {
        assert!(matches!(
            DisplayId::new("1bad"),
            Err(BuildError::InvalidDisplayId(_))
        ));
        assert!(matches!(
            DisplayId::new(""),
            Err(BuildError::InvalidDisplayId(_))
        ));
        assert!(matches!(
            DisplayId::new("has-hyphen"),
            Err(BuildError::InvalidDisplayId(_))
        ));
        assert!(matches!(
            DisplayId::new("has space"),
            Err(BuildError::InvalidDisplayId(_))
        ));
    }

    #[test]
    fn namespace_accepts_http_urls() {
        assert!(Namespace::new("https://example.org").is_ok());
        assert!(Namespace::new("https://example.org/lab").is_ok());
        assert!(Namespace::new("http://example.org/lab").is_ok());
    }

    #[test]
    fn namespace_rejects_bad_urls() {
        assert!(Namespace::new("").is_err());
        assert!(Namespace::new("example.org").is_err());
        assert!(Namespace::new("ftp://example.org").is_err());
        assert!(Namespace::new("https://example.org/").is_err());
        assert!(Namespace::new("https://").is_err());
    }

    #[test]
    fn sbol_identity_builds_compliant_url() {
        let id = SbolIdentity::new(
            Namespace::new("https://example.org/lab").unwrap(),
            DisplayId::new("c").unwrap(),
        );
        assert_eq!(id.to_iri().as_str(), "https://example.org/lab/c");
    }

    #[test]
    fn hash_algorithm_accepts_spec_tokens() {
        assert!(HashAlgorithm::new("SHA256").is_ok());
        assert!(HashAlgorithm::new("SHA3-256").is_ok());
        assert_eq!(HashAlgorithm::SHA256.as_str(), "SHA256");
    }

    #[test]
    fn hash_algorithm_rejects_unknown() {
        assert!(matches!(
            HashAlgorithm::new("MD5"),
            Err(BuildError::InvalidHashAlgorithm(_))
        ));
    }
}
