//! IRI string helpers shared across SBOL versions and the conversion layer.

/// Returns the trailing segment of an IRI, after the rightmost `/`,
/// `#`, or `:`. URN identities (`urn:sbol:foo:bar:1`) use `:` as the
/// separator; HTTP IRIs use `/` or `#`. A plain `rsplit('/')` would
/// return the entire URN, which then concatenates into a malformed
/// nested IRI when used as a child segment.
pub fn last_iri_segment(iri: &str) -> &str {
    let split = iri.rfind('/').max(iri.rfind('#')).max(iri.rfind(':'));
    match split {
        Some(idx) if idx + 1 < iri.len() => &iri[idx + 1..],
        _ => iri,
    }
}

#[cfg(test)]
mod tests {
    use super::last_iri_segment;

    #[test]
    fn handles_http_paths() {
        assert_eq!(
            last_iri_segment("https://example.org/lab/promoter"),
            "promoter"
        );
    }

    #[test]
    fn handles_hash_fragments() {
        assert_eq!(last_iri_segment("http://example.org/ns#frag"), "frag");
    }

    #[test]
    fn handles_pure_urns() {
        // A `rsplit('/')` would return the entire URN, which then
        // concatenates into a malformed nested IRI at the call site.
        assert_eq!(last_iri_segment("urn:sbol:design:promoter:1"), "1");
    }

    #[test]
    fn handles_mixed_urns() {
        assert_eq!(last_iri_segment("urn:sbol:design:promoter/1"), "1");
    }
}
