use crate::validation::resolver::*;

const PREFIXES: &str = r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX sbol: <http://sbols.org/v3#>
"#;

fn document(body: &str) -> Document {
    Document::read_turtle(&format!("{PREFIXES}\n{body}")).unwrap()
}

#[test]
fn ownership_index_supports_parent_and_child_lookup() {
    let document = document(
        r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:LocalSubComponent;
    sbol:displayId "feature";
    sbol:type SBO:0000251 .
"#,
    );
    let index = OwnershipIndex::new(&document);
    let component = Resource::iri("https://example.org/component");
    let feature = Resource::iri("https://example.org/component/feature");

    assert_eq!(
        index.single_parent(&feature, SBOL_HAS_FEATURE),
        Some(&component)
    );
    assert!(index.contains(&component, SBOL_HAS_FEATURE, &feature));
}

#[test]
fn ownership_index_reports_multiple_parents() {
    let document = document(
        r#":component_a a sbol:Component;
    sbol:displayId "component_a";
    sbol:hasFeature :shared_feature;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component_b a sbol:Component;
    sbol:displayId "component_b";
    sbol:hasFeature :shared_feature;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:shared_feature a sbol:LocalSubComponent;
    sbol:displayId "shared_feature";
    sbol:type SBO:0000251 .
"#,
    );
    let index = OwnershipIndex::new(&document);
    let feature = Resource::iri("https://example.org/shared_feature");
    let parents = index.parents(&feature, SBOL_HAS_FEATURE);

    assert_eq!(parents.len(), 2);
    assert!(parents.contains(&&Resource::iri("https://example.org/component_a")));
    assert!(parents.contains(&&Resource::iri("https://example.org/component_b")));
}

#[test]
fn component_reference_resolver_walks_nested_references() {
    let document = document(
        r#"<component/reference> a sbol:ComponentReference;
    sbol:displayId "reference";
    sbol:refersTo <definition/reference> .
<definition/reference> a sbol:ComponentReference;
    sbol:displayId "reference";
    sbol:refersTo <leaf_component/feature> .
<leaf_component/feature> a sbol:SequenceFeature;
    sbol:displayId "feature" .
"#,
    );
    let index = OwnershipIndex::new(&document);
    let resolver = ComponentReferenceResolver::new(&document, &index);
    let feature = Resource::iri("https://example.org/component/reference");
    let trace = resolver.trace_feature(&feature).unwrap();

    assert_eq!(
        trace.target,
        Resource::iri("https://example.org/leaf_component/feature")
    );
    assert_eq!(trace.path.len(), 2);
}

#[test]
fn component_reference_resolver_reports_missing_targets() {
    let document = document(
        r#"<component/reference> a sbol:ComponentReference;
    sbol:displayId "reference";
    sbol:refersTo <missing/feature> .
"#,
    );
    let index = OwnershipIndex::new(&document);
    let resolver = ComponentReferenceResolver::new(&document, &index);
    let feature = Resource::iri("https://example.org/component/reference");

    assert!(matches!(
        resolver.trace_feature(&feature),
        Err(FeatureResolveError::MissingObject(resource))
            if resource == Resource::iri("https://example.org/missing/feature")
    ));
}

#[test]
fn component_reference_resolver_reports_cycles() {
    let document = document(
        r#"<component/a> a sbol:ComponentReference;
    sbol:displayId "a";
    sbol:refersTo <component/b> .
<component/b> a sbol:ComponentReference;
    sbol:displayId "b";
    sbol:refersTo <component/a> .
"#,
    );
    let index = OwnershipIndex::new(&document);
    let resolver = ComponentReferenceResolver::new(&document, &index);
    let feature = Resource::iri("https://example.org/component/a");

    assert!(matches!(
        resolver.trace_feature(&feature),
        Err(FeatureResolveError::Cycle(_))
    ));
}

#[test]
fn location_resolver_normalizes_entire_sequence() {
    let document = document(
        r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:hasNamespace <https://example.org> .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/feature> a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <component/feature/location> .
<component/feature/location> a sbol:EntireSequence;
    sbol:displayId "location";
    sbol:hasSequence :sequence .
"#,
    );
    let index = OwnershipIndex::new(&document);
    let resolver = LocationResolver::new(&document, &index);
    let feature = Resource::iri("https://example.org/component/feature");
    let locations = resolver.locations_for_feature(&feature);

    assert_eq!(locations.len(), 1);
    assert_eq!(locations[0].start, 0);
    assert_eq!(locations[0].end, 4);
}

#[test]
fn location_resolver_follows_component_references_to_target_locations() {
    let document = document(
        r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:hasNamespace <https://example.org> .
<definition/feature> a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <definition/feature/location> .
<definition/feature/location> a sbol:Range;
    sbol:displayId "location";
    sbol:end "3";
    sbol:hasSequence :sequence;
    sbol:start "2" .
<component/reference> a sbol:ComponentReference;
    sbol:displayId "reference";
    sbol:refersTo <definition/feature> .
"#,
    );
    let index = OwnershipIndex::new(&document);
    let resolver = LocationResolver::new(&document, &index);
    let reference = Resource::iri("https://example.org/component/reference");
    let locations = resolver.locations_for_feature(&reference);

    assert_eq!(locations.len(), 1);
    assert_eq!(locations[0].start, 1);
    assert_eq!(locations[0].end, 3);
}

#[test]
fn location_resolver_leaves_entire_sequence_without_length_unresolved() {
    let document = document(
        r#":sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:hasNamespace <https://example.org> .
<component/feature> a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <component/feature/location> .
<component/feature/location> a sbol:EntireSequence;
    sbol:displayId "location";
    sbol:hasSequence :sequence .
"#,
    );
    let index = OwnershipIndex::new(&document);
    let resolver = LocationResolver::new(&document, &index);
    let feature = Resource::iri("https://example.org/component/feature");

    assert!(resolver.locations_for_feature(&feature).is_empty());
}

#[test]
fn constraint_engine_detects_direct_identity_contradictions() {
    let document = document(
        r#":definition_a a sbol:Component;
    sbol:displayId "definition_a";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:definition_b a sbol:Component;
    sbol:displayId "definition_b";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/a>, <component/b>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/a> a sbol:SubComponent;
    sbol:displayId "a";
    sbol:instanceOf :definition_a .
<component/b> a sbol:SubComponent;
    sbol:displayId "b";
    sbol:instanceOf :definition_b .
"#,
    );
    let index = OwnershipIndex::new(&document);
    let engine = ConstraintEngine::new(&document, &index);
    let subject = Resource::iri("https://example.org/component/a");
    let object = Resource::iri("https://example.org/component/b");

    assert!(matches!(
        engine.table8_relation(SBOL_VERIFY_IDENTICAL, &subject, &object),
        RelationOutcome::Contradicted { .. }
    ));
}

#[test]
fn constraint_engine_keeps_replaces_and_spatial_table9_relations_undecided() {
    let document = document(
        r#":component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/a>, <component/b>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
<component/a> a sbol:LocalSubComponent;
    sbol:displayId "a";
    sbol:type SBO:0000251 .
<component/b> a sbol:LocalSubComponent;
    sbol:displayId "b";
    sbol:type SBO:0000251 .
"#,
    );
    let index = OwnershipIndex::new(&document);
    let engine = ConstraintEngine::new(&document, &index);
    let subject = Resource::iri("https://example.org/component/a");
    let object = Resource::iri("https://example.org/component/b");

    assert_eq!(
        engine.table8_relation(SBOL_REPLACES, &subject, &object),
        RelationOutcome::Unknown
    );
    assert_eq!(
        engine.table8_relation(SBOL_COVERS, &subject, &object),
        RelationOutcome::Unsupported
    );
}
