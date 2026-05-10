use sbol::constants::{RESTRICTION_SAME_ORIENTATION_AS, SBO_DNA};
use sbol::{BuildError, Component, Document, Iri, Resource, SbolObject};

fn minimal_component(namespace: &str, display_id: &str) -> Component {
    Component::new(namespace, display_id, [SBO_DNA.clone()]).expect("builder accepts inputs")
}

#[test]
fn typed_client_builds_valid_rdf_document() {
    let document = Document::from_objects(vec![SbolObject::Component(minimal_component(
        "https://example.org",
        "component",
    ))])
    .unwrap();

    assert!(document.validate().is_valid());
    assert!(document.write_turtle().unwrap().contains("Component"));
    assert_eq!(document.typed_objects().len(), 1);

    let component = document
        .components()
        .next()
        .expect("expected typed component");
    assert_eq!(
        component.identified.display_id.as_deref(),
        Some("component")
    );
    assert_eq!(
        component.types.iter().map(Iri::as_str).collect::<Vec<_>>(),
        [SBO_DNA.as_str()]
    );
    assert_eq!(
        component.top_level.namespace.as_ref().map(Iri::as_str),
        Some("https://example.org")
    );
}

#[test]
fn typed_document_lookup_and_class_iterators_filter_owned_objects() {
    let turtle = r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX sbol: <http://sbols.org/v3#>

:sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/subject>, <component/object>;
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :sequence;
    sbol:type SBO:0000251 .
<component/subject> a sbol:LocalSubComponent;
    sbol:displayId "subject";
    sbol:type SBO:0000251 .
<component/object> a sbol:LocalSubComponent;
    sbol:displayId "object";
    sbol:type SBO:0000251 .
<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/object>;
    sbol:restriction sbol:sameOrientationAs;
    sbol:subject <component/subject> .
"#;
    let document = Document::read_turtle(turtle).unwrap();

    assert_eq!(document.components().count(), 1);
    assert_eq!(document.sequences().count(), 1);
    assert_eq!(document.constraints().count(), 1);
    assert_eq!(document.local_sub_components().count(), 2);
    assert_eq!(
        document.components().next().unwrap().identity,
        Resource::iri("https://example.org/component")
    );

    let sequence_id = Resource::iri("https://example.org/sequence");
    assert!(matches!(
        document.resolve(&sequence_id),
        Some(SbolObject::Sequence(sequence)) if sequence.identity == sequence_id
    ));
    assert!(
        document
            .resolve(&Resource::iri("https://example.org/missing"))
            .is_none()
    );
    assert!(matches!(
        document.find_by_display_id("https://example.org", "sequence"),
        Some(SbolObject::Sequence(_))
    ));
    let _ = RESTRICTION_SAME_ORIENTATION_AS.clone();
}

#[test]
fn typed_client_rejects_duplicate_identities() {
    let error = Document::from_objects(vec![
        SbolObject::Component(minimal_component("https://example.org", "component")),
        SbolObject::Component(minimal_component("https://example.org", "component")),
    ])
    .unwrap_err();

    assert_eq!(
        error,
        BuildError::DuplicateIdentity(Resource::iri("https://example.org/component"))
    );
}

#[test]
fn typed_client_rejects_missing_required_fields_before_rdf_build() {
    let mut component = minimal_component("https://example.org", "component");
    component.top_level.namespace = None;

    let error = Document::from_objects(vec![SbolObject::Component(component)]).unwrap_err();
    assert!(matches!(
        error,
        BuildError::MissingRequired {
            property: "http://sbols.org/v3#hasNamespace",
            ..
        }
    ));
    assert!(error.to_string().contains("hasNamespace"));
}
