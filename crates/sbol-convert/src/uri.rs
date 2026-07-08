//! SBOL 2 ↔ SBOL 3 IRI algebra.
//!
//! SBOL 2 encodes an object's version as the last path segment
//! (`<prefix>/<displayId>/<version>`); SBOL 3 places it before the
//! displayId (`<prefix>/<version>/<displayId>`). The conversion recomputes
//! identities by decomposing an IRI into its namespace, version, and
//! displayId parts and reassembling them for the target version.
//!
//! A segment counts as a version when it matches `[0-9]+[\p{L}0-9_.-]*` —
//! a leading digit followed by word characters. This mirrors the reference
//! converter so a given IRI decomposes identically in both implementations.

/// Whether `segment` is a valid SBOL version: empty, or a leading ASCII
/// digit followed by letters, digits, `_`, `.`, or `-`.
pub(crate) fn is_version_valid(segment: &str) -> bool {
    if segment.is_empty() {
        return true;
    }
    let mut chars = segment.chars();
    match chars.next() {
        Some(first) if first.is_ascii_digit() => {}
        _ => return false,
    }
    segment
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '-')
}

// === SBOL 3 IRI decomposition (`<prefix>/<version>/<displayId>`) ===

/// The displayId: the final path segment. `""` when there is no `/`.
pub(crate) fn display_id_sbol3(uri: &str) -> &str {
    match uri.rfind('/') {
        Some(i) => &uri[i + 1..],
        None => "",
    }
}

/// The version segment (the one immediately before the displayId), or `""`
/// when absent or not version-shaped.
pub(crate) fn version_sbol3(uri: &str) -> &str {
    let Some(last) = uri.rfind('/') else {
        return "";
    };
    let first_segment = &uri[..last];
    let Some(second_last) = first_segment.rfind('/') else {
        return "";
    };
    let version = &uri[second_last + 1..last];
    if is_version_valid(version) {
        version
    } else {
        ""
    }
}

/// The namespace/prefix: the IRI minus the displayId, and minus the version
/// segment when one is present.
pub(crate) fn namespace_sbol3(uri: &str) -> &str {
    let version = version_sbol3(uri);
    let Some(last) = uri.rfind('/') else {
        return "";
    };
    let namespace = &uri[..last];
    if version.is_empty() {
        return namespace;
    }
    match namespace.rfind('/') {
        Some(second_last) => &namespace[..second_last],
        None => "",
    }
}

// === SBOL 2 IRI decomposition (`<prefix>/<displayId>/<version>`) ===
//
// These return `None` for an empty IRI, one ending in `/`, or one with no
// `/` at all — the cases the reference treats as non-decomposable.

/// The version segment, or `Some("")` when the last segment is not
/// version-shaped.
pub(crate) fn version_sbol2(uri: &str) -> Option<&str> {
    if uri.is_empty() || uri.ends_with('/') {
        return None;
    }
    let last = uri.rfind('/')?;
    let last_segment = &uri[last + 1..];
    if is_version_valid(last_segment) {
        Some(last_segment)
    } else {
        Some("")
    }
}

/// The displayId: the segment before the version when the IRI is versioned,
/// otherwise the last segment.
pub(crate) fn display_id_sbol2(uri: &str) -> Option<&str> {
    if uri.is_empty() || uri.ends_with('/') {
        return None;
    }
    let last = uri.rfind('/')?;
    let last_segment = &uri[last + 1..];
    if is_version_valid(last_segment) {
        match uri[..last].rfind('/') {
            Some(second_last) => Some(&uri[second_last + 1..last]),
            None => Some(uri),
        }
    } else {
        Some(last_segment)
    }
}

/// The prefix: the IRI minus displayId (and version, when present).
pub(crate) fn uri_prefix_sbol2(uri: &str) -> Option<&str> {
    if uri.is_empty() || uri.ends_with('/') {
        return None;
    }
    let last = uri.rfind('/')?;
    let last_segment = &uri[last + 1..];
    if is_version_valid(last_segment) {
        match uri[..last].rfind('/') {
            Some(second_last) => Some(&uri[..second_last]),
            None => Some(uri),
        }
    } else {
        Some(&uri[..last])
    }
}

// === Cross-version identity construction ===

/// Builds the SBOL 2 IRI for an SBOL 3 IRI: `<namespace>/<displayId>` with an
/// optional trailing `/<version>`. Returns the input unchanged when it has no
/// decomposable structure (mirrors the reference's `"/"` guard).
pub(crate) fn create_sbol2_uri(sbol3_uri: &str) -> String {
    let version = version_sbol3(sbol3_uri);
    let mut sbol2 = format!(
        "{}/{}",
        namespace_sbol3(sbol3_uri),
        display_id_sbol3(sbol3_uri)
    );
    if !version.is_empty() {
        sbol2.push('/');
        sbol2.push_str(version);
    }
    if sbol2 == "/" {
        return sbol3_uri.to_owned();
    }
    sbol2
}

/// Builds the SBOL 3 IRI for an SBOL 2 IRI: `<prefix>/<version>/<displayId>`,
/// omitting the version segment when the source is unversioned.
pub(crate) fn create_sbol3_uri(sbol2_uri: &str) -> String {
    let Some(prefix) = uri_prefix_sbol2(sbol2_uri) else {
        return sbol2_uri.to_owned();
    };
    let mut sbol3 = prefix.to_owned();
    if let Some(version) = version_sbol2(sbol2_uri)
        && !version.is_empty()
    {
        sbol3.push('/');
        sbol3.push_str(version);
    }
    if let Some(display_id) = display_id_sbol2(sbol2_uri)
        && !display_id.is_empty()
    {
        sbol3.push('/');
        sbol3.push_str(display_id);
    }
    sbol3
}

// === Ontology URI canonicalization (SO / SBO / EDAM) ===
//
// SBOL 2 and SBOL 3 spell the same ontology terms differently. The conversion
// is dispatched by the field the term appears in — Sequence Ontology terms on
// roles, Systems Biology Ontology terms on interaction/participation/model
// fields, EDAM terms on model languages — and each conversion also toggles the
// URI scheme (`http` ⇄ `https`), unconditionally, matching the reference. A
// term that carries no recognized ontology token therefore still gets its
// scheme toggled when it flows through the corresponding field.
//
// SBOL 3 spellings: `https://identifiers.org/SO:<id>`,
// `https://identifiers.org/SBO:<id>`, `https://identifiers.org/edam:<id>`.
// SBOL 2 spellings: `http://identifiers.org/so/SO:<id>`,
// `http://identifiers.org/biomodels.sbo/SBO:<id>`,
// `http://identifiers.org/edam/<id>`.

fn to_https(uri: &str) -> String {
    match uri.strip_prefix("http://") {
        Some(rest) => format!("https://{rest}"),
        None => uri.to_owned(),
    }
}

fn to_http(uri: &str) -> String {
    match uri.strip_prefix("https://") {
        Some(rest) => format!("http://{rest}"),
        None => uri.to_owned(),
    }
}

/// Sequence Ontology term, SBOL 2 → SBOL 3: drop the `so/` path segment.
pub(crate) fn convert_so_2_to_3(uri: &str) -> String {
    to_https(&uri.replace("so/", ""))
}

/// Sequence Ontology term, SBOL 3 → SBOL 2: reinstate the `so/` path segment.
pub(crate) fn convert_so_3_to_2(uri: &str) -> String {
    to_http(&uri.replace("SO:", "so/SO:"))
}

/// Systems Biology Ontology term, SBOL 2 → SBOL 3: drop `biomodels.sbo/`.
pub(crate) fn convert_sbo_2_to_3(uri: &str) -> String {
    to_https(&uri.replace("biomodels.sbo/", ""))
}

/// Systems Biology Ontology term, SBOL 3 → SBOL 2: reinstate `biomodels.sbo/`.
pub(crate) fn convert_sbo_3_to_2(uri: &str) -> String {
    to_http(&uri.replace("SBO:", "biomodels.sbo/SBO:"))
}

/// EDAM term, SBOL 2 → SBOL 3: rewrite the `/edam/` path segment to `/edam:`.
pub(crate) fn convert_edam_2_to_3(uri: &str) -> String {
    to_https(&uri.replace("/edam/", "/edam:"))
}

/// EDAM term, SBOL 3 → SBOL 2: rewrite `/edam:` back to `/edam/`.
pub(crate) fn convert_edam_3_to_2(uri: &str) -> String {
    to_http(&uri.replace("/edam:", "/edam/"))
}

/// Component type, SBOL 2 → SBOL 3, for a type that is not one of the BioPAX
/// terms with a dedicated SBO mapping: an SO term is canonicalized, anything
/// else passes through unchanged (no scheme toggle).
pub(crate) fn component_type_2_to_3(uri: &str) -> String {
    if uri.to_ascii_lowercase().contains("so/so:") {
        convert_so_2_to_3(uri)
    } else {
        uri.to_owned()
    }
}

/// Component type, SBOL 3 → SBOL 2, mirror of [`component_type_2_to_3`].
pub(crate) fn component_type_3_to_2(uri: &str) -> String {
    if uri.to_ascii_lowercase().contains("so:") {
        convert_so_3_to_2(uri)
    } else {
        uri.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ontology_round_trips() {
        assert_eq!(
            convert_so_2_to_3("http://identifiers.org/so/SO:0000141"),
            "https://identifiers.org/SO:0000141"
        );
        assert_eq!(
            convert_so_3_to_2("https://identifiers.org/SO:0000141"),
            "http://identifiers.org/so/SO:0000141"
        );
        assert_eq!(
            convert_sbo_2_to_3("http://identifiers.org/biomodels.sbo/SBO:0000020"),
            "https://identifiers.org/SBO:0000020"
        );
        assert_eq!(
            convert_sbo_3_to_2("https://identifiers.org/SBO:0000020"),
            "http://identifiers.org/biomodels.sbo/SBO:0000020"
        );
        assert_eq!(
            convert_edam_2_to_3("http://identifiers.org/edam/format_1207"),
            "https://identifiers.org/edam:format_1207"
        );
        assert_eq!(
            convert_edam_3_to_2("https://identifiers.org/edam:format_1207"),
            "http://identifiers.org/edam/format_1207"
        );
        // A role that is not an ontology term still gets its scheme toggled.
        assert_eq!(
            convert_so_2_to_3("http://parts.igem.org/cgi/pgroup.cgi"),
            "https://parts.igem.org/cgi/pgroup.cgi"
        );
        assert_eq!(
            convert_so_3_to_2("https://parts.igem.org/cgi/pgroup.cgi"),
            "http://parts.igem.org/cgi/pgroup.cgi"
        );
    }

    #[test]
    fn version_validity() {
        assert!(is_version_valid(""));
        assert!(is_version_valid("1"));
        assert!(is_version_valid("1.2.3"));
        assert!(is_version_valid("1a"));
        assert!(!is_version_valid("BBa_E0040"));
        assert!(!is_version_valid("v1"));
    }

    #[test]
    fn sbol2_decomposition_versioned() {
        let uri = "https://synbiohub.org/public/igem/BBa_E0040/1";
        assert_eq!(
            uri_prefix_sbol2(uri),
            Some("https://synbiohub.org/public/igem")
        );
        assert_eq!(display_id_sbol2(uri), Some("BBa_E0040"));
        assert_eq!(version_sbol2(uri), Some("1"));
    }

    #[test]
    fn sbol2_decomposition_unversioned() {
        let uri = "https://example.org/lab/foo";
        assert_eq!(uri_prefix_sbol2(uri), Some("https://example.org/lab"));
        assert_eq!(display_id_sbol2(uri), Some("foo"));
        assert_eq!(version_sbol2(uri), Some(""));
    }

    #[test]
    fn sbol3_decomposition_versioned() {
        // SBOL 3 places the version before the displayId.
        let uri = "https://example.org/lab/1/foo";
        assert_eq!(namespace_sbol3(uri), "https://example.org/lab");
        assert_eq!(version_sbol3(uri), "1");
        assert_eq!(display_id_sbol3(uri), "foo");
    }

    #[test]
    fn sbol3_decomposition_unversioned() {
        let uri = "https://example.org/lab/foo";
        assert_eq!(namespace_sbol3(uri), "https://example.org/lab");
        assert_eq!(version_sbol3(uri), "");
        assert_eq!(display_id_sbol3(uri), "foo");
    }

    #[test]
    fn round_trips_versioned() {
        let sbol2 = "https://synbiohub.org/public/igem/BBa_E0040/1";
        let sbol3 = create_sbol3_uri(sbol2);
        assert_eq!(sbol3, "https://synbiohub.org/public/igem/1/BBa_E0040");
        assert_eq!(create_sbol2_uri(&sbol3), sbol2);
    }

    #[test]
    fn round_trips_unversioned() {
        let sbol2 = "https://example.org/lab/foo";
        let sbol3 = create_sbol3_uri(sbol2);
        assert_eq!(sbol3, "https://example.org/lab/foo");
        assert_eq!(create_sbol2_uri(&sbol3), sbol2);
    }
}
