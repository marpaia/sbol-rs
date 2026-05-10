//! Builder coverage: required-field constructors, builder chains,
//! invalid-input rejection across every owned SBOL class.
//!
//! Confirms that:
//! - `Class::new(...)` produces a compliant identity URL.
//! - `Class::builder(...).build()` returns `MissingRequired` when a cardinality-required
//!   field is never set.
//! - Invalid `displayId` is rejected at construction.

use sbol::constants::{
    CARDINALITY_ONE, ORIENTATION_INLINE, RESTRICTION_PRECEDES, SBO_DNA, SBO_PROTEIN, SO_PROMOTER,
};
use sbol::{
    Attachment, BuildError, CombinatorialDerivation, Component, Constraint, Cut, Iri, Model,
    Participation, Range, Resource, SbolClass, Sequence, SequenceFeature, SubComponent,
    VariableFeature,
};

fn r(s: &str) -> Resource {
    Resource::iri(s)
}

#[test]
fn component_new_builds_compliant_identity() {
    let c = Component::new("https://example.org/lab", "my_component", [SBO_DNA.clone()]).unwrap();
    assert_eq!(
        c.identity.to_string(),
        "https://example.org/lab/my_component"
    );
    assert_eq!(c.identified.display_id.as_deref(), Some("my_component"));
    assert_eq!(c.types, vec![SBO_DNA.clone()]);
}

#[test]
fn component_builder_chains_optional_fields() {
    let c = Component::builder("https://example.org/lab", "my_component")
        .unwrap()
        .types([SBO_DNA.clone()])
        .add_component_role(SO_PROMOTER.clone())
        .name("My promoter")
        .description("A test promoter")
        .build()
        .unwrap();
    assert_eq!(c.roles, vec![SO_PROMOTER.clone()]);
    assert_eq!(c.identified.name.as_deref(), Some("My promoter"));
    assert_eq!(c.identified.description.as_deref(), Some("A test promoter"));
}

#[test]
fn component_build_without_types_reports_missing_required() {
    let err = Component::builder("https://example.org/lab", "my_component")
        .unwrap()
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        BuildError::MissingRequired {
            class: SbolClass::Component,
            property: "type",
            ..
        }
    ));
}

#[test]
fn invalid_display_id_is_rejected_at_construction() {
    let err = Component::builder("https://example.org/lab", "1bad").unwrap_err();
    assert!(matches!(err, BuildError::InvalidDisplayId(_)));
}

#[test]
fn invalid_namespace_is_rejected_at_construction() {
    let err = Component::builder("not-a-url", "c").unwrap_err();
    assert!(matches!(err, BuildError::InvalidNamespace(_)));
}

#[test]
fn sequence_new_works_without_required_fields() {
    let s = Sequence::new("https://example.org/lab", "seq1").unwrap();
    assert_eq!(s.identity.to_string(), "https://example.org/lab/seq1");
    assert!(s.elements.is_none());
}

#[test]
fn attachment_requires_source() {
    let err = Attachment::builder("https://example.org/lab", "att")
        .unwrap()
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        BuildError::MissingRequired {
            class: SbolClass::Attachment,
            property: "source",
            ..
        }
    ));

    let a = Attachment::new(
        "https://example.org/lab",
        "att",
        r("https://example.org/blob"),
    )
    .unwrap();
    assert_eq!(a.source.unwrap().to_string(), "https://example.org/blob");
}

#[test]
fn model_requires_all_three_iris() {
    let err = Model::builder("https://example.org/lab", "m")
        .unwrap()
        .source(r("https://example.org/m.sbml"))
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        BuildError::MissingRequired {
            class: SbolClass::Model,
            property: "language",
            ..
        }
    ));
}

#[test]
fn combinatorial_derivation_requires_template() {
    let err = CombinatorialDerivation::builder("https://example.org/lab", "cd")
        .unwrap()
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        BuildError::MissingRequired {
            class: SbolClass::CombinatorialDerivation,
            property: "template",
            ..
        }
    ));
}

#[test]
fn sub_component_uses_parent_identity_and_requires_instance_of() {
    let parent = r("https://example.org/lab/parent");
    let display_id = "child";
    let sc = SubComponent::new(&parent, display_id, r("https://example.org/lab/def")).unwrap();
    assert_eq!(
        sc.identity.to_string(),
        "https://example.org/lab/parent/child"
    );

    let err = SubComponent::builder(&parent, "child2")
        .unwrap()
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        BuildError::MissingRequired {
            class: SbolClass::SubComponent,
            property: "instanceOf",
            ..
        }
    ));
}

#[test]
fn range_requires_start_and_end() {
    let parent = r("https://example.org/lab/feature");
    let r1 = Range::new(&parent, "r1", 1, 100).unwrap();
    assert_eq!(r1.start, Some(1));
    assert_eq!(r1.end, Some(100));

    let err = Range::builder(&parent, "r2")
        .unwrap()
        .start(5)
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        BuildError::MissingRequired {
            class: SbolClass::Range,
            property: "end",
            ..
        }
    ));
}

#[test]
fn cut_requires_at() {
    let parent = r("https://example.org/lab/feature");
    let c = Cut::new(&parent, "c", 42).unwrap();
    assert_eq!(c.at, Some(42));

    let err = Cut::builder(&parent, "c2").unwrap().build().unwrap_err();
    assert!(matches!(
        err,
        BuildError::MissingRequired {
            class: SbolClass::Cut,
            property: "at",
            ..
        }
    ));
}

#[test]
fn constraint_requires_subject_object_restriction() {
    let parent = r("https://example.org/lab/c");
    let constraint = Constraint::new(
        &parent,
        "k1",
        r("https://example.org/lab/c/f1"),
        r("https://example.org/lab/c/f2"),
        RESTRICTION_PRECEDES.clone(),
    )
    .unwrap();
    assert!(constraint.subject.is_some());
    assert!(constraint.constrained_object.is_some());
    assert_eq!(
        constraint.restriction.unwrap(),
        RESTRICTION_PRECEDES.clone()
    );

    let err = Constraint::builder(&parent, "k2")
        .unwrap()
        .subject(r("https://example.org/lab/c/f1"))
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        BuildError::MissingRequired {
            class: SbolClass::Constraint,
            property: "object",
            ..
        }
    ));
}

#[test]
fn variable_feature_requires_cardinality_and_variable() {
    let parent = r("https://example.org/lab/cd");
    let vf = VariableFeature::new(
        &parent,
        "vf",
        CARDINALITY_ONE.clone(),
        r("https://example.org/lab/template/v"),
    )
    .unwrap();
    assert_eq!(vf.cardinality.unwrap(), CARDINALITY_ONE.clone());

    let err = VariableFeature::builder(&parent, "vf2")
        .unwrap()
        .cardinality(CARDINALITY_ONE.clone())
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        BuildError::MissingRequired {
            class: SbolClass::VariableFeature,
            property: "variable",
            ..
        }
    ));
}

#[test]
fn participation_requires_roles() {
    let parent = r("https://example.org/lab/i");
    let err = Participation::builder(&parent, "p")
        .unwrap()
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        BuildError::MissingRequired {
            class: SbolClass::Participation,
            property: "role",
            ..
        }
    ));
}

#[test]
fn sequence_feature_requires_locations() {
    let parent = r("https://example.org/lab/c");
    let err = SequenceFeature::builder(&parent, "sf")
        .unwrap()
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        BuildError::MissingRequired {
            class: SbolClass::SequenceFeature,
            property: "hasLocation",
            ..
        }
    ));
}

#[test]
fn unused_iri_keeps_borrow_simple() {
    // Compile-only check: ORIENTATION_INLINE, SBO_PROTEIN, and a bare Iri all coexist.
    let _ = ORIENTATION_INLINE.clone();
    let _ = SBO_PROTEIN.clone();
    let _ = Iri::from_static("https://example.org/x");
}
