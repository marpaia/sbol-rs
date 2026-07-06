use thiserror::Error;

use crate::{Resource, SbolClass};

pub use sbol_core::error::{ReadError, WriteError};

/// Errors produced while building an SBOL document from owned typed objects.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum BuildError {
    #[error("duplicate SBOL object identity `{0}`")]
    DuplicateIdentity(Resource),

    #[error("{class} `{identity}` is missing required property `{property}`",
            class = .class.local_name())]
    MissingRequired {
        identity: Resource,
        class: SbolClass,
        property: &'static str,
    },

    #[error(
        "invalid SBOL displayId `{0}` — must start with a letter or underscore and contain only ASCII alphanumerics or underscores"
    )]
    InvalidDisplayId(String),

    #[error("invalid SBOL namespace `{0}` — must be an http(s) URL with no trailing slash")]
    InvalidNamespace(String),

    #[error(
        "invalid SBOL hash algorithm `{0}` — must be one of the spec-defined tokens (SHA1, SHA256, SHA3-256, ...)"
    )]
    InvalidHashAlgorithm(String),
}

impl From<sbol_core::error::BuildError> for BuildError {
    fn from(error: sbol_core::error::BuildError) -> Self {
        use sbol_core::error::BuildError as Lex;
        match error {
            Lex::InvalidDisplayId(value) => BuildError::InvalidDisplayId(value),
            Lex::InvalidNamespace(value) => BuildError::InvalidNamespace(value),
            Lex::InvalidHashAlgorithm(value) => BuildError::InvalidHashAlgorithm(value),
        }
    }
}
