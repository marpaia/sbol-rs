use std::path::PathBuf;

use thiserror::Error;

use crate::{Resource, SbolClass};

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
