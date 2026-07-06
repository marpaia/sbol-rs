//! Coverage for the typed document API: accessors, lookups, class
//! classification, and the extension catch-all.

use sbol2::constants::BIOPAX_DNA;
use sbol2::prelude::*;

const NS: &str = "https://example.org/lab";

#[test]
fn accessors_expose_identified_and_version_metadata() {
    let cd = ComponentDefinition::builder(NS, "j23100")
        .expect("builder")
        .types([BIOPAX_DNA])
        .name("Anderson promoter")
        .description("Constitutive promoter")
        .build()
        .expect("cd");
    assert_eq!(cd.display_id(), Some("j23100"));
    assert_eq!(cd.name(), Some("Anderson promoter"));
    assert_eq!(cd.description(), Some("Constitutive promoter"));
    assert_eq!(cd.version(), Some("1"));
    assert_eq!(
        cd.persistent_identity().unwrap().as_iri().unwrap().as_str(),
        "https://example.org/lab/j23100"
    );
}

#[test]
fn document_exposes_typed_accessors_and_lookup() {
    let cd = ComponentDefinition::new(NS, "cd", [BIOPAX_DNA]).expect("cd");
    let seq = Sequence::builder(NS, "seq")
        .expect("builder")
        .elements("atcg")
        .encoding(Iri::new_unchecked(
            "http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html".to_string(),
        ))
        .build()
        .expect("seq");

    let document = Document::from_objects(vec![
        Sbol2Object::ComponentDefinition(cd),
        Sbol2Object::Sequence(seq),
    ])
    .expect("document");

    assert_eq!(document.component_definitions().count(), 1);
    assert_eq!(document.sequences().count(), 1);
    assert_eq!(document.top_levels().count(), 2);

    let found = document
        .find_by_display_id(NS, "cd")
        .expect("finds versioned identity");
    assert_eq!(found.class(), Sbol2Class::ComponentDefinition);
    assert_eq!(
        found.identity().as_iri().unwrap().as_str(),
        "https://example.org/lab/cd/1"
    );

    let namespaces: Vec<String> = document
        .namespaces()
        .iter()
        .map(|iri| iri.as_str().to_string())
        .collect();
    assert_eq!(namespaces, vec![NS.to_string()]);
}

#[test]
fn child_reports_parent_identity() {
    let cd = ComponentDefinition::new(NS, "device", [BIOPAX_DNA]).expect("cd");
    let component = Component::new(&cd.identity, "sub", cd.identity.clone()).expect("component");
    let object = Sbol2Object::Component(component);
    assert!(!object.is_top_level_object());
    assert_eq!(
        object.parent_identity().unwrap().as_iri().unwrap().as_str(),
        "https://example.org/lab/device/1"
    );
}

#[test]
fn object_classes_honor_the_hierarchy() {
    let cd = ComponentDefinition::new(NS, "cd", [BIOPAX_DNA]).expect("cd");
    let document = Document::from_objects(vec![Sbol2Object::ComponentDefinition(cd.clone())])
        .expect("document");
    let object = document.get(&cd.identity).expect("raw object");
    assert!(object.has_class(Sbol2Class::ComponentDefinition));
    assert!(object.has_class(Sbol2Class::TopLevel));
    assert!(object.has_class(Sbol2Class::Identified));
    assert!(!object.has_class(Sbol2Class::Component));
}

#[test]
fn unknown_predicates_round_trip_as_extension_triples() {
    let cd = ComponentDefinition::builder(NS, "annotated")
        .expect("builder")
        .types([BIOPAX_DNA])
        .extension(
            Iri::new_unchecked("http://parts.igem.org/partStatus".to_string()),
            Term::Literal(Literal::simple("Available")),
        )
        .build()
        .expect("cd");

    let document =
        Document::from_objects(vec![Sbol2Object::ComponentDefinition(cd)]).expect("document");
    let turtle = document.write_turtle().expect("turtle");
    let reparsed = Document::read_turtle(&turtle).expect("reparse");
    let cd = reparsed
        .component_definitions()
        .next()
        .expect("component definition");
    assert!(
        cd.extensions()
            .iter()
            .any(|ext| ext.predicate.as_str() == "http://parts.igem.org/partStatus")
    );
}
