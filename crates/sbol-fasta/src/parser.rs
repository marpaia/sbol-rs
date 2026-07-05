//! Minimal FASTA parser.
//!
//! FASTA is shaped like:
//!
//! ```text
//! >id description...
//! sequence line 1
//! sequence line 2
//! ;optional comment line (ignored)
//! >id2 description...
//! sequence...
//! ```
//!
//! The parser is intentionally permissive: it strips whitespace from
//! sequence lines, ignores comment lines (`;…` and blank), and treats
//! everything between two `>` markers as one record. Embedded
//! whitespace and ASCII control characters inside the sequence are
//! stripped; non-ASCII bytes are passed through verbatim and caught
//! by the alphabet check downstream.

/// One parsed FASTA record.
#[derive(Clone, Debug)]
pub(crate) struct Record {
    /// First whitespace-delimited token of the header line (the `>`
    /// is stripped). Empty if the header was just `>` with nothing
    /// after it.
    pub id: String,
    /// Everything on the header line after the first whitespace, with
    /// leading/trailing whitespace trimmed. `None` if the header had
    /// no description.
    pub description: Option<String>,
    /// Concatenated sequence with internal whitespace stripped, in
    /// the case the source file came from. Callers normalize case
    /// downstream (lowercase for DNA/RNA per SBOL 3 convention).
    pub sequence: String,
}

/// Parses every record in the input. Returns an empty vector when the
/// input has no `>` lines.
pub(crate) fn parse_records(input: &str) -> Vec<Record> {
    let mut records: Vec<Record> = Vec::new();
    let mut current: Option<RecordBuilder> = None;

    for raw_line in input.lines() {
        let line = raw_line.trim_end_matches('\r');
        if line.is_empty() {
            continue;
        }
        let first = line.as_bytes()[0];
        match first {
            b'>' => {
                if let Some(rb) = current.take() {
                    records.push(rb.build());
                }
                let header = &line[1..];
                let (id, description) = split_header(header);
                current = Some(RecordBuilder {
                    id,
                    description,
                    sequence: String::new(),
                });
            }
            b';' => {
                // Comment line; ignore.
            }
            _ => {
                if let Some(rb) = current.as_mut() {
                    append_sequence(&mut rb.sequence, line);
                } else {
                    // Sequence data before any `>` header. The most
                    // tolerant move is to skip it.
                }
            }
        }
    }

    if let Some(rb) = current.take() {
        records.push(rb.build());
    }

    records
}

struct RecordBuilder {
    id: String,
    description: Option<String>,
    sequence: String,
}

impl RecordBuilder {
    fn build(self) -> Record {
        Record {
            id: self.id,
            description: self.description,
            sequence: self.sequence,
        }
    }
}

fn split_header(header: &str) -> (String, Option<String>) {
    let trimmed = header.trim();
    if trimmed.is_empty() {
        return (String::new(), None);
    }
    match trimmed.split_once(char::is_whitespace) {
        Some((id, rest)) => {
            let description = rest.trim();
            let description = if description.is_empty() {
                None
            } else {
                Some(description.to_owned())
            };
            (id.to_owned(), description)
        }
        None => (trimmed.to_owned(), None),
    }
}

fn append_sequence(buffer: &mut String, line: &str) {
    for byte in line.bytes() {
        // Strip all ASCII whitespace and control characters; keep
        // everything else (incl. ambiguity codes and gap symbols).
        if byte > b' ' {
            buffer.push(byte as char);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_record() {
        let records = parse_records(">id1 description\nACGT\n");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, "id1");
        assert_eq!(records[0].description.as_deref(), Some("description"));
        assert_eq!(records[0].sequence, "ACGT");
    }

    #[test]
    fn parses_multi_line_sequence() {
        let records = parse_records(">id\nACGT\nACGT\nACGT\n");
        assert_eq!(records[0].sequence, "ACGTACGTACGT");
    }

    #[test]
    fn parses_multiple_records() {
        let input = ">a\nAA\n>b desc\nBB\n>c\nCC\n";
        let records = parse_records(input);
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].id, "a");
        assert_eq!(records[1].id, "b");
        assert_eq!(records[1].description.as_deref(), Some("desc"));
        assert_eq!(records[2].sequence, "CC");
    }

    #[test]
    fn strips_internal_whitespace() {
        let records = parse_records(">id\nAC GT\n\tAC\tGT\n");
        assert_eq!(records[0].sequence, "ACGTACGT");
    }

    #[test]
    fn ignores_comment_lines() {
        let input = ">id\n;this is a comment\nACGT\n;more comments\nACGT\n";
        let records = parse_records(input);
        assert_eq!(records[0].sequence, "ACGTACGT");
    }

    #[test]
    fn handles_empty_description() {
        let records = parse_records(">just_id\nACGT\n");
        assert_eq!(records[0].id, "just_id");
        assert!(records[0].description.is_none());
    }

    #[test]
    fn handles_crlf_line_endings() {
        let records = parse_records(">id description\r\nACGT\r\n");
        assert_eq!(records[0].id, "id");
        assert_eq!(records[0].description.as_deref(), Some("description"));
        assert_eq!(records[0].sequence, "ACGT");
    }

    #[test]
    fn empty_input_returns_no_records() {
        assert!(parse_records("").is_empty());
    }

    #[test]
    fn sequence_before_first_header_is_skipped() {
        let records = parse_records("orphan\nACGT\n>id\nGGGG\n");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].sequence, "GGGG");
    }
}
