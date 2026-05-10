use std::path::Path;

/// RDF serialization format.
///
/// SBOL 3.1.0 documents may be exchanged in any of four RDF serializations.
/// libSBOLj3 defaults to RDF/XML; pySBOL3 defaults to sorted Turtle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RdfFormat {
    /// Turtle (`.ttl`).
    Turtle,
    /// RDF/XML (`.rdf`).
    RdfXml,
    /// JSON-LD (`.jsonld`).
    JsonLd,
    /// N-Triples (`.nt`).
    NTriples,
}

impl RdfFormat {
    /// Maps a filename extension (without the leading dot) to a format.
    ///
    /// Matching is case-insensitive. Ambiguous extensions like `.xml`
    /// and `.json` return `None`; callers should require the
    /// unambiguous spec-listed extensions instead.
    pub fn from_extension(extension: &str) -> Option<Self> {
        let normalized = extension.trim_start_matches('.').to_ascii_lowercase();
        Some(match normalized.as_str() {
            "ttl" => Self::Turtle,
            "rdf" => Self::RdfXml,
            "jsonld" => Self::JsonLd,
            "nt" => Self::NTriples,
            _ => return None,
        })
    }

    /// Maps a path's extension to a format. See [`from_extension`].
    ///
    /// [`from_extension`]: Self::from_extension
    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        let extension = path.as_ref().extension()?.to_str()?;
        Self::from_extension(extension)
    }

    /// Human-readable name of the format.
    pub fn name(self) -> &'static str {
        match self {
            Self::Turtle => "Turtle",
            Self::RdfXml => "RDF/XML",
            Self::JsonLd => "JSON-LD",
            Self::NTriples => "N-Triples",
        }
    }

    /// Canonical filename extension (without the leading dot) for the format.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Turtle => "ttl",
            Self::RdfXml => "rdf",
            Self::JsonLd => "jsonld",
            Self::NTriples => "nt",
        }
    }

    /// All filename extensions sbol-rs accepts on read for this format,
    /// canonical extension first.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Turtle => &["ttl"],
            Self::RdfXml => &["rdf"],
            Self::JsonLd => &["jsonld"],
            Self::NTriples => &["nt"],
        }
    }

    /// Every supported format in canonical iteration order.
    pub const ALL: &'static [RdfFormat] =
        &[Self::Turtle, Self::RdfXml, Self::JsonLd, Self::NTriples];
}

impl std::fmt::Display for RdfFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name())
    }
}
