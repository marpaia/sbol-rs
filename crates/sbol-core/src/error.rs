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
