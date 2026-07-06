//! Version-neutral read and write errors for SBOL documents.

use std::path::PathBuf;

use thiserror::Error;

/// Errors produced while reading an SBOL document.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ReadError {
    #[error("RDF parse error")]
    Rdf(#[from] sbol_rdf::ParseError),

    #[error("failed to read `{path}`")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(
        "unknown RDF format for `{path}` (extension: {})",
        extension.as_deref().unwrap_or("<none>")
    )]
    UnknownFormat {
        path: PathBuf,
        extension: Option<String>,
    },
}

/// Lexical construction errors for the validated identity newtypes.
///
/// These variants reject malformed identifiers, namespaces, and hash
/// algorithm tokens at construction time, before any version-specific
/// document assembly runs. A versioned crate's build error wraps these
/// so its builder surface reports a single error type.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum BuildError {
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

/// Errors produced while writing an SBOL document.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum WriteError {
    #[error("RDF write error")]
    Rdf(#[from] sbol_rdf::WriteError),

    #[error("failed to write `{path}`")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
