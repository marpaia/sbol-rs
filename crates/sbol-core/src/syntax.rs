//! Lexical checks for IUPAC sequence characters, URLs/IRIs, hash digests, and display IDs.
//!
//! These are pure string predicates shared by both SBOL versions and their
//! validators; they encode lexical forms that SBOL 2 and SBOL 3 hold in common.

pub fn is_iupac_nucleotide(character: char) -> bool {
    matches!(
        character.to_ascii_uppercase(),
        'A' | 'C'
            | 'G'
            | 'T'
            | 'U'
            | 'R'
            | 'Y'
            | 'S'
            | 'W'
            | 'K'
            | 'M'
            | 'B'
            | 'D'
            | 'H'
            | 'V'
            | 'N'
    )
}

pub fn is_iupac_protein(character: char) -> bool {
    matches!(
        character.to_ascii_uppercase(),
        'A' | 'C'
            | 'D'
            | 'E'
            | 'F'
            | 'G'
            | 'H'
            | 'I'
            | 'K'
            | 'L'
            | 'M'
            | 'N'
            | 'P'
            | 'Q'
            | 'R'
            | 'S'
            | 'T'
            | 'V'
            | 'W'
            | 'Y'
            | 'B'
            | 'Z'
            | 'J'
            | 'X'
            | 'U'
            | 'O'
            | '*'
    )
}

pub fn is_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

pub fn is_namespace_url(value: &str) -> bool {
    is_url(value) || value.starts_with("urn:")
}

pub fn is_hex_digest(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|character| character.is_ascii_hexdigit())
}

pub fn is_known_hash_algorithm(value: &str) -> bool {
    matches!(
        value,
        "sha2-256" | "sha3-256" | "blake3" | "sha2-512" | "sha3-512"
    )
}

pub fn is_hash_algorithm_token(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphanumeric()
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
}

pub fn hex_digest(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push_str(&format!("{byte:02x}"));
    }
    value
}

pub fn url_is_child_of(candidate: &str, parent: &str) -> bool {
    candidate
        .strip_prefix(parent.trim_end_matches('/'))
        .is_some_and(|suffix| suffix.starts_with('/'))
}

pub fn url_matches_namespace_display_id(identity: &str, namespace: &str, display_id: &str) -> bool {
    let namespace = namespace.trim_end_matches('/');
    let Some(rest) = identity.strip_prefix(namespace) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix('/') else {
        return false;
    };
    !rest.is_empty()
        && !rest.split('/').any(str::is_empty)
        && rest.rsplit('/').next() == Some(display_id)
}

pub fn is_valid_display_id(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
}
