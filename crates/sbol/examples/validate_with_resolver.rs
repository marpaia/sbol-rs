//! Validate a document that references an external attachment, using a
//! `FileResolver` to dereference the attachment from a local fixture
//! directory. Then walk the typed document to inspect what was validated.
//!
//! Run with `cargo run -p sbol --example validate_with_resolver`.

use std::env;
use std::fs;
use std::path::PathBuf;

use sbol::prelude::*;
use sbol::{FileResolver, ValidationContext};

const TURTLE: &str = r#"
PREFIX sbol: <http://sbols.org/v3#>

<https://example.org/lab/attached> a sbol:Attachment;
    sbol:displayId "attached";
    sbol:hasNamespace <https://example.org/lab>;
    sbol:source <file:///fixture-payload.txt> .
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Stage a temporary "payload" the resolver can dereference.
    let tmp = env::temp_dir().join("sbol-rs-validate-with-resolver");
    fs::create_dir_all(&tmp)?;
    let payload_path = tmp.join("fixture-payload.txt");
    fs::write(&payload_path, b"hello sbol-rs")?;

    let document = Document::read_turtle(TURTLE)?;

    let resolver = FileResolver::new().with_root(PathBuf::from("/"));
    let context = ValidationContext::new().with_content_resolver(&resolver);

    let report = document.validate_with_context(context);
    if report.is_valid() {
        println!("document is valid against the resolver-aware context");
    } else {
        println!("validation reported {} issues:", report.issues().len());
        for issue in report.issues() {
            println!(
                "  [{:?}] {} — {}",
                issue.severity, issue.rule, issue.message
            );
        }
    }

    // Use the resolver API to walk the typed surface.
    for attachment in document.attachments() {
        let display_id = attachment.display_id().unwrap_or("?");
        let source = attachment
            .source
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "<no source>".to_owned());
        println!("Attachment {display_id}: source = {source}");
        if let Some(parent) = SbolObject::Attachment(attachment.clone()).parent_identity() {
            // Top-level objects have no parent identity; this branch is
            // primarily useful for child objects (Range, Constraint, ...).
            println!("  parent_identity: {parent}");
        }
    }

    fs::remove_file(&payload_path).ok();
    Ok(())
}
