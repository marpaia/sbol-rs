//! End-to-end coverage for layering ontology extensions onto validation.

use sbol3::{Document, Ontology, ValidationContext, ValidationOptions};

const SYNTHETIC_EXTENSION: &str = concat!(
    "# format_version: 1\n",
    "# kind\tid\tiri\tlabel\taliases\tparents\tontology\trole\tcomponent_family\tsequence_family\ttable1\ttable2\n",
    "term\tCL:9999999\thttps://identifiers.org/CL:9999999\tlab-only synthetic cell\t-\tCL:0000540\tCL\tcomponent_type\t-\t-\tfalse\tfalse\n",
    "branch\tCL:9999999\tCL:0000000\n",
);

#[test]
fn extension_terms_are_visible_to_validation_context() {
    let extension = Ontology::from_tsv_str(SYNTHETIC_EXTENSION).unwrap();
    let options = ValidationOptions::default().with_ontology_extension(extension);
    let context = ValidationContext::with_options(options);

    let ontology = context.ontology();
    assert!(ontology.contains_term("CL:9999999"));
    assert_eq!(ontology.is_cell_type_term("CL:9999999"), Some(true));
    // Bundled CL terms still resolve.
    assert_eq!(ontology.is_cell_type_term("CL:0000540"), Some(true));
}

#[test]
fn default_validation_does_not_see_extensions() {
    let context = ValidationContext::with_options(ValidationOptions::default());
    let ontology = context.ontology();
    assert!(!ontology.contains_term("CL:9999999"));
}

#[test]
fn validate_with_extension_still_produces_clean_report() {
    // A normal valid Component should still validate cleanly when an
    // extension is attached. The extension only adds new known terms;
    // it cannot make a previously-valid document invalid.
    let document = Document::read_turtle(
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

    let extension = Ontology::from_tsv_str(SYNTHETIC_EXTENSION).unwrap();
    let options = ValidationOptions::default().with_ontology_extension(extension);
    let report = document.validate_with(options);
    assert!(report.is_valid(), "report: {report:?}");
}
