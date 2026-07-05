#[test]
fn targets_sbol_3_1_0() {
    assert_eq!(sbol::SPEC_VERSION, "3.1.0");
    assert!(sbol::SPECIFICATION_URL.ends_with("/version-3.1.0/"));
}

#[test]
fn public_docs_do_not_claim_sbol_3_0_target() {
    let readme = include_str!("../../../README.md");
    let manifest = include_str!("../Cargo.toml");
    let crate_docs = include_str!("../src/lib.rs");

    assert!(!readme.contains("currently targets SBOL 3.0.1"));
    assert!(!readme.contains("SBOL 3.0.1 Turtle parsing"));
    assert!(!manifest.contains("SBOL 3.0 specification"));
    assert!(!crate_docs.contains("SBOL 3.0\n//! specification"));
}

#[test]
fn reads_and_validates_minimal_component() {
    let document = sbol::Document::read_turtle(
        r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX sbol: <http://sbols.org/v3#>

:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
    )
    .unwrap();

    assert!(document.validate().is_valid());
    assert!(document.check().is_ok());
    assert_eq!(document.objects().len(), 1);
}

#[test]
fn rejects_invalid_display_id() {
    let document = sbol::Document::read_turtle(
        r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX sbol: <http://sbols.org/v3#>

:component a sbol:Component;
    sbol:displayId "bad id";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
    )
    .unwrap();

    let report = document.validate();
    assert!(report.has_errors());
    assert!(report.errors().any(|issue| issue.rule == "sbol3-10201"));
    assert!(document.check().is_err());
    assert!(report.to_string().contains("errors"));
}

#[test]
fn read_errors_report_rdf_parse_failures() {
    let error = sbol::Document::read_turtle("@prefix broken").unwrap_err();
    assert!(error.to_string().contains("RDF parse error"));
}

#[test]
fn read_path_reports_missing_file_via_io_error() {
    let error = sbol::Document::read_path("/tmp/sbol-rs-does-not-exist.ttl").unwrap_err();
    assert!(matches!(error, sbol::ReadError::Io { .. }));
    assert!(error.to_string().contains("failed to read"));
}

#[test]
fn read_path_rejects_unknown_extension() {
    let tmp = std::env::temp_dir().join("sbol-rs-unknown-extension.xml");
    std::fs::write(&tmp, b"<not-really-rdf/>").unwrap();
    let error = sbol::Document::read_path(&tmp).unwrap_err();
    let _ = std::fs::remove_file(&tmp);
    match error {
        sbol::ReadError::UnknownFormat {
            extension: Some(ext),
            ..
        } => assert_eq!(ext, "xml"),
        other => panic!("expected UnknownFormat for .xml; got {other:?}"),
    }
}

#[test]
fn read_path_round_trips_through_every_format() {
    let document = sbol::Document::read_turtle(
        r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX sbol: <http://sbols.org/v3#>

:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
    )
    .unwrap();

    let dir = std::env::temp_dir();
    for &format in sbol::RdfFormat::ALL {
        let path = dir.join(format!("sbol-rs-path-roundtrip.{}", format.extension()));
        document
            .write_path(&path, format)
            .unwrap_or_else(|error| panic!("write_path {format} failed: {error}"));
        let reparsed = sbol::Document::read_path(&path)
            .unwrap_or_else(|error| panic!("read_path {format} failed: {error}"));
        let _ = std::fs::remove_file(&path);

        assert_eq!(
            reparsed.rdf_graph().normalized_triples(),
            document.rdf_graph().normalized_triples(),
            "read_path/write_path round trip differed for {format}"
        );
    }
}

#[test]
fn validation_report_separates_warnings_from_errors() {
    let document = sbol::Document::read_turtle(
        r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX sbol: <http://sbols.org/v3#>

:experiment a sbol:Experiment, sbol:Collection;
    sbol:displayId "experiment";
    sbol:hasNamespace <https://example.org> .
"#,
    )
    .unwrap();

    let report = document.validate();
    assert!(!report.has_errors());
    assert!(report.is_valid());
    assert_eq!(report.errors().count(), 0);
    assert_eq!(report.warnings().count(), 1);
    assert!(report.warnings().any(|issue| issue.rule == "sbol3-10107"));
    assert!(document.check().is_ok());
}

#[test]
fn object_helpers_expose_typed_property_values() {
    let document = sbol::Document::read_turtle(
        r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX sbol: <http://sbols.org/v3#>

:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:name "Component";
    sbol:type SBO:0000251 .
"#,
    )
    .unwrap();
    let object = document
        .get(&sbol::Resource::iri("https://example.org/component"))
        .unwrap();

    use sbol::ObjectClasses;

    assert!(object.has_class(sbol::SbolClass::Component));
    assert!(object.is_top_level());
    assert_eq!(object.identified().display_id.as_deref(), Some("component"));
    assert!(object.top_level().is_some());
    assert_eq!(object.values("http://sbols.org/v3#name").len(), 1);
    assert_eq!(
        object.first_literal_value("http://sbols.org/v3#name"),
        Some("Component")
    );
    assert_eq!(
        object
            .first_resource("http://sbols.org/v3#hasNamespace")
            .map(ToString::to_string),
        Some("https://example.org".to_owned())
    );
    assert_eq!(
        object
            .first_iri("http://sbols.org/v3#type")
            .map(sbol::Iri::as_str),
        Some("https://identifiers.org/SBO:0000251")
    );
    assert_eq!(object.literals("http://sbols.org/v3#name").count(), 1);
    assert_eq!(object.iris("http://sbols.org/v3#type").count(), 1);
    assert_eq!(
        object.classes().iter().copied().collect::<Vec<_>>(),
        vec![sbol::SbolClass::Component]
    );
}
