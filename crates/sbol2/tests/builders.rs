//! Builder coverage for SBOL 2 classes absent from the fixture corpus, plus a
//! full builder → serialize → parse → typed → serialize round trip.

use std::collections::BTreeSet;

use sbol2::constants::{ACCESS_PUBLIC, BIOPAX_DNA, OPERATOR_ONE, ORIENTATION_INLINE, SO_PROMOTER};
use sbol2::prelude::*;

const NS: &str = "https://example.org/lab";

fn iri(value: &str) -> Iri {
    Iri::new_unchecked(value.to_string())
}

/// Serializes the objects, reparses, and asserts the typed round trip
/// reproduces the reparsed triple set exactly.
fn assert_objects_round_trip(objects: Vec<Sbol2Object>) {
    let document = Document::from_objects(objects).expect("objects build a document");
    let turtle = document.write_turtle().expect("serializes to turtle");
    let reparsed = Document::read_turtle(&turtle).expect("reparses");
    let rebuilt = Document::from_objects(reparsed.typed_objects().to_vec())
        .expect("reparsed typed objects rebuild");

    let a: BTreeSet<Triple> = reparsed
        .rdf_graph()
        .normalized_triples()
        .into_iter()
        .collect();
    let b: BTreeSet<Triple> = rebuilt
        .rdf_graph()
        .normalized_triples()
        .into_iter()
        .collect();
    assert_eq!(
        a, b,
        "builder output did not round-trip through the typed model"
    );
}

#[test]
fn generic_top_level_builds_and_round_trips() {
    let gtl =
        GenericTopLevel::new(NS, "custom", iri("https://example.org/ont#Widget")).expect("builds");
    assert_eq!(gtl.display_id(), Some("custom"));
    assert_eq!(gtl.version(), Some("1"));
    assert_eq!(
        gtl.identity.as_iri().unwrap().as_str(),
        "https://example.org/lab/custom/1"
    );
    assert_eq!(
        gtl.rdf_type.as_ref().unwrap().as_str(),
        "https://example.org/ont#Widget"
    );
    assert_objects_round_trip(vec![Sbol2Object::GenericTopLevel(gtl)]);
}

#[test]
fn implementation_and_generic_location_round_trip() {
    let cd = ComponentDefinition::new(NS, "device", [BIOPAX_DNA]).expect("cd");
    let implementation = Implementation::builder(NS, "build_1")
        .expect("impl builder")
        .built(cd.identity.clone())
        .name("Physical build")
        .build()
        .expect("impl");
    assert_eq!(implementation.name(), Some("Physical build"));

    // GenericLocation is a child; parent is a component's persistentIdentity.
    let component = Component::new(&cd.identity, "sub", cd.identity.clone()).expect("component");
    let location = GenericLocation::builder(component.persistent_identity().unwrap(), "whole")
        .expect("loc builder")
        .orientation(ORIENTATION_INLINE)
        .build()
        .expect("generic location");
    assert_eq!(
        location.location.orientation.as_ref().unwrap().as_str(),
        ORIENTATION_INLINE.as_str()
    );

    assert_objects_round_trip(vec![
        Sbol2Object::ComponentDefinition(cd),
        Sbol2Object::Implementation(implementation),
        Sbol2Object::Component(component),
        Sbol2Object::GenericLocation(location),
    ]);
}

#[test]
fn experiment_and_experimental_data_round_trip() {
    let data = ExperimentalData::new(NS, "reads").expect("experimental data");
    let experiment = Experiment::builder(NS, "run_1")
        .expect("experiment builder")
        .add_experimental_data(data.identity.clone())
        .build()
        .expect("experiment");
    assert_eq!(experiment.experimental_data.len(), 1);
    assert_objects_round_trip(vec![
        Sbol2Object::ExperimentalData(data),
        Sbol2Object::Experiment(experiment),
    ]);
}

#[test]
fn combinatorial_derivation_and_variable_component_round_trip() {
    let template = ComponentDefinition::new(NS, "template", [BIOPAX_DNA]).expect("template");
    let slot = Component::new(&template.identity, "slot", template.identity.clone()).expect("slot");
    let variant = ComponentDefinition::new(NS, "variant_a", [BIOPAX_DNA]).expect("variant");

    let derivation = CombinatorialDerivation::builder(NS, "library")
        .expect("cd builder")
        .template(template.identity.clone())
        .build()
        .expect("derivation");

    let variable = VariableComponent::builder(derivation.persistent_identity().unwrap(), "var")
        .expect("var builder")
        .variable(slot.identity.clone())
        .operator(OPERATOR_ONE)
        .add_variant(variant.identity.clone())
        .build()
        .expect("variable component");
    assert_eq!(variable.variants.len(), 1);

    assert_objects_round_trip(vec![
        Sbol2Object::ComponentDefinition(template),
        Sbol2Object::Component(slot),
        Sbol2Object::ComponentDefinition(variant),
        Sbol2Object::CombinatorialDerivation(derivation),
        Sbol2Object::VariableComponent(variable),
    ]);
}

#[test]
fn provenance_hierarchy_round_trips() {
    let agent = Agent::new(NS, "designer").expect("agent");
    let plan = Plan::new(NS, "protocol").expect("plan");
    let activity = Activity::builder(NS, "assembly")
        .expect("activity builder")
        .started_at_time("2020-01-01T00:00:00Z")
        .ended_at_time("2020-01-02T00:00:00Z")
        .build()
        .expect("activity");

    let association = Association::builder(activity.persistent_identity().unwrap(), "assoc")
        .expect("assoc builder")
        .agent(agent.identity.clone())
        .had_plan(plan.identity.clone())
        .build()
        .expect("association");
    let usage = Usage::builder(activity.persistent_identity().unwrap(), "use")
        .expect("usage builder")
        .entity(Resource::iri("https://example.org/lab/thing/1"))
        .build()
        .expect("usage");

    assert_eq!(association.agent.as_ref().unwrap(), &agent.identity);
    assert_eq!(
        activity.started_at_time.as_deref(),
        Some("2020-01-01T00:00:00Z")
    );

    assert_objects_round_trip(vec![
        Sbol2Object::Agent(agent),
        Sbol2Object::Plan(plan),
        Sbol2Object::Activity(activity),
        Sbol2Object::Association(association),
        Sbol2Object::Usage(usage),
    ]);
}

#[test]
fn om_unit_and_prefix_hierarchy_round_trips() {
    let second = Unit::new(NS, "second", "second", "s").expect("unit");
    let metre = SingularUnit::builder(NS, "metre")
        .expect("singular builder")
        .label("metre")
        .symbol("m")
        .has_factor(1.0)
        .build()
        .expect("singular unit");
    let milli = Prefix::new(NS, "milli", "milli", "m", 0.001).expect("prefix");
    let si_kilo = SIPrefix::new(NS, "kilo", "kilo", "k", 1000.0).expect("si prefix");
    let binary_kibi = BinaryPrefix::new(NS, "kibi", "kibi", "Ki", 1024.0).expect("binary prefix");

    let per_second = UnitDivision::builder(NS, "per_second")
        .expect("division builder")
        .label("per second")
        .symbol("1/s")
        .has_numerator(metre.identity.clone())
        .has_denominator(second.identity.clone())
        .build()
        .expect("division");
    let squared = UnitExponentiation::builder(NS, "metre_squared")
        .expect("exponent builder")
        .label("square metre")
        .symbol("m^2")
        .has_base(metre.identity.clone())
        .has_exponent(2)
        .build()
        .expect("exponentiation");
    let product = UnitMultiplication::builder(NS, "metre_second")
        .expect("mult builder")
        .label("metre second")
        .symbol("m*s")
        .has_term1(metre.identity.clone())
        .has_term2(second.identity.clone())
        .build()
        .expect("multiplication");
    let prefixed = PrefixedUnit::builder(NS, "kilometre")
        .expect("prefixed builder")
        .label("kilometre")
        .symbol("km")
        .has_unit(metre.identity.clone())
        .has_prefix(si_kilo.identity.clone())
        .build()
        .expect("prefixed unit");
    let compound = CompoundUnit::new(NS, "compound", "compound", "c").expect("compound");

    // A Measure attaches to a parent via sbol2:measure.
    let cd = ComponentDefinition::new(NS, "measured_cd", [BIOPAX_DNA]).expect("cd");
    let measure =
        Measure::new(&cd.identity, "one_metre", metre.identity.clone(), 1.0).expect("measure");
    assert_eq!(measure.has_numerical_value.as_deref(), Some("1"));

    assert_objects_round_trip(vec![
        Sbol2Object::Unit(second),
        Sbol2Object::SingularUnit(metre),
        Sbol2Object::Prefix(milli),
        Sbol2Object::SIPrefix(si_kilo),
        Sbol2Object::BinaryPrefix(binary_kibi),
        Sbol2Object::UnitDivision(per_second),
        Sbol2Object::UnitExponentiation(squared),
        Sbol2Object::UnitMultiplication(product),
        Sbol2Object::PrefixedUnit(prefixed),
        Sbol2Object::CompoundUnit(compound),
        Sbol2Object::ComponentDefinition(cd),
        Sbol2Object::Measure(measure),
    ]);
}

#[test]
fn missing_required_property_is_reported() {
    let error = ComponentDefinition::builder(NS, "no_types")
        .expect("builder")
        .build()
        .unwrap_err();
    assert!(matches!(
        error,
        BuildError::MissingRequired {
            property: "types",
            ..
        }
    ));
}

#[test]
fn version_setter_rewrites_compliant_identity() {
    let cd = ComponentDefinition::builder(NS, "versioned")
        .expect("builder")
        .types([BIOPAX_DNA])
        .add_role(SO_PROMOTER)
        .version("2")
        .build()
        .expect("cd");
    assert_eq!(cd.version(), Some("2"));
    assert_eq!(
        cd.identity.as_iri().unwrap().as_str(),
        "https://example.org/lab/versioned/2"
    );
    assert_eq!(
        cd.persistent_identity().unwrap().as_iri().unwrap().as_str(),
        "https://example.org/lab/versioned"
    );
}

#[test]
fn access_is_optional_on_component_instances() {
    let cd = ComponentDefinition::new(NS, "def", [BIOPAX_DNA]).expect("cd");
    let fc = FunctionalComponent::builder(&cd.identity, "fc")
        .expect("builder")
        .definition(cd.identity.clone())
        .access(ACCESS_PUBLIC)
        .build()
        .expect("functional component");
    assert_eq!(
        fc.component_instance.access.as_ref().unwrap().as_str(),
        ACCESS_PUBLIC.as_str()
    );
}
