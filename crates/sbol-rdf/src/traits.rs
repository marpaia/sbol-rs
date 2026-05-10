use crate::{ParseError, RdfFormat, Triple, WriteError};

/// Read-only RDF graph behavior exposed independently of the concrete backend.
pub trait RdfGraph {
    fn triples(&self) -> &[Triple];

    /// Returns the deduplicated, deterministically sorted triple set for
    /// equality checks and tests.
    ///
    /// RDF graphs are semantically *sets* of triples, but specific
    /// serializations (notably RDF/XML, when a resource appears both
    /// inline and at the top level) may parse into a `Vec<Triple>` with
    /// repeated rows. This method collapses those duplicates so equality
    /// comparisons match RDF set semantics.
    ///
    /// Blank node identifiers are compared as parsed. This is sufficient for
    /// the current SBOL fixture corpus, which uses URI-identified SBOL
    /// objects.
    fn normalized_triples(&self) -> Vec<Triple> {
        let mut triples = self.triples().to_vec();
        triples.sort();
        triples.dedup();
        triples
    }
}

/// RDF parsing and serialization across all supported formats.
pub trait RdfIo: RdfGraph + Sized {
    fn parse(input: &str, format: RdfFormat) -> Result<Self, ParseError>;

    fn write(&self, format: RdfFormat) -> Result<String, WriteError>;
}

/// Turtle-specific shorthand for [`RdfIo`].
///
/// Kept for source-compatibility with the original single-format API; new
/// callers should prefer [`RdfIo::parse`] / [`RdfIo::write`] with an explicit
/// [`RdfFormat`].
pub trait TurtleGraph: RdfIo {
    fn parse_turtle(input: &str) -> Result<Self, ParseError> {
        <Self as RdfIo>::parse(input, RdfFormat::Turtle)
    }

    fn write_turtle(&self) -> Result<String, WriteError> {
        <Self as RdfIo>::write(self, RdfFormat::Turtle)
    }
}

impl<T: RdfIo> TurtleGraph for T {}
