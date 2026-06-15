#![allow(dead_code)]

use std::fs;

use tempfile::TempDir;

pub const TTL_VALID: &str = r#"
PREFIX sbol: <http://sbols.org/v3#>
PREFIX SBO: <https://identifiers.org/SBO:>

<https://example.org/lab/c> a sbol:Component;
    sbol:displayId "c";
    sbol:hasNamespace <https://example.org/lab>;
    sbol:type SBO:0000251 .
"#;

pub const TTL_INVALID: &str = r#"
PREFIX sbol: <http://sbols.org/v3#>

<https://example.org/lab/c> a sbol:Component;
    sbol:displayId "1bad" ;
    sbol:hasNamespace <https://example.org/lab> .
"#;

pub fn write_fixture(dir: &TempDir, name: &str, contents: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, contents).unwrap();
    path
}
