//! Schema consistency gate for the A.2 invariant.
//!
//! For every typed `SbolObject` variant, this test asserts two things:
//!
//! 1. The predicate set emitted by `ToRdf::to_rdf_triples` for a fully
//!    populated instance equals the predicate set declared in the class's
//!    `FieldDescriptor` array (walking parent classes), modulo `rdf:type`.
//!    A failure means `to_rdf` skipped a descriptor field, or `to_rdf`
//!    emits a predicate the descriptor never declared. The Emitter
//!    panics on the second case at runtime; this test catches the first.
//!
//! 2. The full to_rdf → Turtle → from_rdf chain preserves every populated
//!    field by `PartialEq`. A failure means `from_rdf` ignores a
//!    descriptor field that `to_rdf` writes, or vice versa.
//!
//! These checks gate the larger PROV/OM typed-struct work so future
//! drift between the three descriptor / serializer / deserializer
//! surfaces is caught at the next `cargo test`.

use std::collections::BTreeSet;

use sbol3::constants::{
    CARDINALITY_ONE, EDAM_IUPAC_DNA, ORIENTATION_INLINE, RESTRICTION_PRECEDES,
    ROLE_INTEGRATION_MERGE_ROLES, SBO_DNA, SBO_NON_COVALENT_BINDING, SBO_PROTEIN, SBO_REACTANT,
    SO_PROMOTER, STRATEGY_ENUMERATE,
};
use sbol3::prelude::*;
use sbol3::schema::class_descriptor;

const RDF_TYPE_IRI: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const NS: &str = "https://example.org/lab";

fn r(value: &str) -> Resource {
    Resource::iri(value)
}

fn parent(name: &str) -> Resource {
    r(&format!("{NS}/{name}"))
}

fn descriptor_predicates(class_iri: &str) -> BTreeSet<String> {
    fn walk(class_iri: &str, out: &mut BTreeSet<String>) {
        if let Some(descriptor) = class_descriptor(class_iri) {
            for parent in descriptor.parents {
                walk(parent, out);
            }
            for field in descriptor.fields {
                out.insert(field.predicate.to_string());
            }
        }
    }
    let mut out = BTreeSet::new();
    walk(class_iri, &mut out);
    out
}

fn emitted_predicates(object: &SbolObject) -> BTreeSet<String> {
    object
        .to_rdf_triples()
        .expect("to_rdf must succeed for a fully populated instance")
        .into_iter()
        .map(|triple| triple.predicate.as_str().to_string())
        .filter(|predicate| predicate != RDF_TYPE_IRI)
        .collect()
}

#[allow(clippy::vec_init_then_push)]
fn populated_instances() -> Vec<SbolObject> {
    let component_parent = parent("parent_component");
    let combderiv_parent = parent("parent_combderiv");
    let sequence_feature_parent = parent("parent_component/sf");
    let sub_component_parent = parent("parent_component/sc");
    let interaction_parent = parent("parent_component/x");

    let mut out = Vec::new();

    out.push(SbolObject::Attachment(
        Attachment::builder(NS, "att")
            .unwrap()
            .source(r("https://example.org/source"))
            .format(Iri::from_static("https://identifiers.org/edam:format_2547"))
            .size(123)
            .hash("deadbeef")
            .hash_algorithm(HashAlgorithm::SHA256)
            .name("attachment name")
            .description("attachment description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att2"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Collection(
        Collection::builder(NS, "col")
            .unwrap()
            .add_member(r("https://example.org/member"))
            .name("collection name")
            .description("collection description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::CombinatorialDerivation(
        CombinatorialDerivation::builder(NS, "cd")
            .unwrap()
            .template(component_parent.clone())
            .strategy(STRATEGY_ENUMERATE)
            .add_variable_feature(r("https://example.org/lab/cd/vf"))
            .name("cd name")
            .description("cd description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Component(
        Component::builder(NS, "comp")
            .unwrap()
            .types([SBO_DNA])
            .add_component_role(SO_PROMOTER)
            .add_sequence(r("https://example.org/lab/seq"))
            .add_feature(r("https://example.org/lab/comp/f"))
            .add_constraint(r("https://example.org/lab/comp/cn"))
            .add_interaction(r("https://example.org/lab/comp/x"))
            .add_interface(r("https://example.org/lab/comp/iface"))
            .add_model(r("https://example.org/lab/model"))
            .name("component name")
            .description("component description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::ComponentReference(
        ComponentReference::builder(&component_parent, "cref")
            .unwrap()
            .in_child_of(r("https://example.org/lab/parent_component/parent_sc"))
            .refers_to(r("https://example.org/lab/parent_component/target_feature"))
            .add_role(SO_PROMOTER)
            .orientation(ORIENTATION_INLINE)
            .name("cref name")
            .description("cref description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Constraint(
        Constraint::builder(&component_parent, "cn")
            .unwrap()
            .subject(r("https://example.org/lab/parent_component/sub"))
            .constrained_object(r("https://example.org/lab/parent_component/obj"))
            .restriction(RESTRICTION_PRECEDES)
            .name("constraint name")
            .description("constraint description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Cut(
        Cut::builder(&sequence_feature_parent, "cut")
            .unwrap()
            .at(5)
            .sequence(r("https://example.org/lab/seq"))
            .orientation(ORIENTATION_INLINE)
            .order(1)
            .name("cut name")
            .description("cut description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::EntireSequence(
        EntireSequence::builder(&sequence_feature_parent, "es")
            .unwrap()
            .sequence(r("https://example.org/lab/seq"))
            .orientation(ORIENTATION_INLINE)
            .order(1)
            .name("es name")
            .description("es description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Experiment(
        Experiment::builder(NS, "exp")
            .unwrap()
            .add_member(r("https://example.org/member"))
            .name("experiment name")
            .description("experiment description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::ExperimentalData(
        ExperimentalData::builder(NS, "expdata")
            .unwrap()
            .name("expdata name")
            .description("expdata description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::ExternallyDefined(
        ExternallyDefined::builder(&component_parent, "ed")
            .unwrap()
            .definition(r("https://identifiers.org/CHEBI:15422"))
            .types([SBO_NON_COVALENT_BINDING])
            .add_role(SO_PROMOTER)
            .orientation(ORIENTATION_INLINE)
            .name("ed name")
            .description("ed description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Implementation(
        Implementation::builder(NS, "impl")
            .unwrap()
            .built(r("https://example.org/lab/comp"))
            .name("impl name")
            .description("impl description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Interaction(
        Interaction::builder(&component_parent, "x")
            .unwrap()
            .types([SBO_NON_COVALENT_BINDING])
            .add_participation(r("https://example.org/lab/parent_component/x/p"))
            .name("x name")
            .description("x description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Interface(
        Interface::builder(&component_parent, "iface")
            .unwrap()
            .add_input(r("https://example.org/lab/parent_component/in"))
            .add_output(r("https://example.org/lab/parent_component/out"))
            .add_nondirectional(r("https://example.org/lab/parent_component/nd"))
            .name("iface name")
            .description("iface description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::LocalSubComponent(
        LocalSubComponent::builder(&component_parent, "lsc")
            .unwrap()
            .types([SBO_PROTEIN])
            .add_location(r("https://example.org/lab/parent_component/lsc/loc"))
            .add_role(SO_PROMOTER)
            .orientation(ORIENTATION_INLINE)
            .name("lsc name")
            .description("lsc description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Model(
        Model::builder(NS, "model")
            .unwrap()
            .source(r("https://example.org/model.xml"))
            .language(Iri::from_static("https://identifiers.org/edam:format_2585"))
            .framework(Iri::from_static("https://identifiers.org/SBO:0000293"))
            .name("model name")
            .description("model description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Participation(
        Participation::builder(&interaction_parent, "p")
            .unwrap()
            .roles([SBO_REACTANT])
            .participant(r("https://example.org/lab/parent_component/sub"))
            .higher_order_participant(r("https://example.org/lab/parent_component/x2"))
            .name("p name")
            .description("p description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Range(
        Range::builder(&sequence_feature_parent, "r")
            .unwrap()
            .start(1)
            .end(10)
            .sequence(r("https://example.org/lab/seq"))
            .orientation(ORIENTATION_INLINE)
            .order(1)
            .name("r name")
            .description("r description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Sequence(
        Sequence::builder(NS, "seq")
            .unwrap()
            .elements("ACGT")
            .encoding(EDAM_IUPAC_DNA)
            .name("seq name")
            .description("seq description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::SequenceFeature(
        SequenceFeature::builder(&component_parent, "sf")
            .unwrap()
            .add_location(r("https://example.org/lab/parent_component/sf/loc"))
            .add_role(SO_PROMOTER)
            .orientation(ORIENTATION_INLINE)
            .name("sf name")
            .description("sf description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::SubComponent(
        SubComponent::builder(&component_parent, "sc")
            .unwrap()
            .instance_of(r("https://example.org/lab/comp"))
            .role_integration(ROLE_INTEGRATION_MERGE_ROLES)
            .add_location(r("https://example.org/lab/parent_component/sc/loc"))
            .add_source_location(r("https://example.org/lab/parent_component/sc/src"))
            .add_role(SO_PROMOTER)
            .orientation(ORIENTATION_INLINE)
            .name("sc name")
            .description("sc description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::VariableFeature(
        VariableFeature::builder(&combderiv_parent, "vf")
            .unwrap()
            .cardinality(CARDINALITY_ONE)
            .variable(r("https://example.org/lab/parent_component/sub"))
            .add_variant(r("https://example.org/lab/variant"))
            .add_variant_collection(r("https://example.org/lab/variant_collection"))
            .add_variant_derivation(r("https://example.org/lab/variant_derivation"))
            .add_variant_measure(r("https://example.org/lab/variant_measure"))
            .name("vf name")
            .description("vf description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    // PROV-O classes (Appendix A.1).

    let activity_parent = parent("act");
    out.push(SbolObject::Activity(
        Activity::builder(NS, "act")
            .unwrap()
            .add_type(Iri::from_static("https://identifiers.org/SBO:0000004"))
            .started_at_time("2026-01-01T00:00:00Z")
            .ended_at_time("2026-01-01T01:00:00Z")
            .add_was_informed_by(r("https://example.org/other_activity"))
            .add_qualified_usage(r("https://example.org/lab/act/u"))
            .add_qualified_association(r("https://example.org/lab/act/a"))
            .name("activity name")
            .description("activity description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Agent(
        Agent::builder(NS, "agent")
            .unwrap()
            .name("agent name")
            .description("agent description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Plan(
        Plan::builder(NS, "plan")
            .unwrap()
            .name("plan name")
            .description("plan description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Association(
        Association::builder(&activity_parent, "a")
            .unwrap()
            .agent(r("https://example.org/lab/agent"))
            .add_had_role(Iri::from_static("https://example.org/role"))
            .had_plan(r("https://example.org/lab/plan"))
            .name("association name")
            .description("association description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Usage(
        Usage::builder(&activity_parent, "u")
            .unwrap()
            .entity(r("https://example.org/lab/comp"))
            .add_had_role(Iri::from_static("https://example.org/role"))
            .name("usage name")
            .description("usage description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    // OM classes (Appendix A.2).

    let measure_parent = parent("comp");
    out.push(SbolObject::Measure(
        Measure::builder(&measure_parent, "m_volts")
            .unwrap()
            .has_unit(r("https://example.org/lab/unit_volt"))
            .has_numerical_value(3.3)
            .add_type(Iri::from_static("https://identifiers.org/SBO:0000545"))
            .name("measure name")
            .description("measure description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Unit(
        Unit::builder(NS, "unit_abstract")
            .unwrap()
            .label("abstract")
            .symbol("a")
            .add_alternative_label("alt".to_string())
            .add_alternative_symbol("alt_s".to_string())
            .comment("c")
            .long_comment("long")
            .name("unit name")
            .description("unit description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::SingularUnit(
        SingularUnit::builder(NS, "unit_meter")
            .unwrap()
            .label("meter")
            .symbol("m")
            .add_alternative_label("metre".to_string())
            .add_alternative_symbol("m.".to_string())
            .comment("c")
            .long_comment("long")
            .has_unit(r("https://example.org/lab/unit_abstract"))
            .has_factor(1.0)
            .name("singular unit name")
            .description("singular unit description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::CompoundUnit(
        CompoundUnit::builder(NS, "unit_compound")
            .unwrap()
            .label("compound")
            .symbol("c")
            .add_alternative_label("comp".to_string())
            .add_alternative_symbol("c.".to_string())
            .comment("c")
            .long_comment("long")
            .name("compound name")
            .description("compound description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::UnitDivision(
        UnitDivision::builder(NS, "unit_division")
            .unwrap()
            .label("meter per second")
            .symbol("m/s")
            .add_alternative_label("velocity".to_string())
            .add_alternative_symbol("ms-1".to_string())
            .comment("c")
            .long_comment("long")
            .has_numerator(r("https://example.org/lab/unit_meter"))
            .has_denominator(r("https://example.org/lab/unit_second"))
            .name("unit division name")
            .description("unit division description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::UnitExponentiation(
        UnitExponentiation::builder(NS, "unit_exponentiation")
            .unwrap()
            .label("meter squared")
            .symbol("m^2")
            .add_alternative_label("area".to_string())
            .add_alternative_symbol("m2".to_string())
            .comment("c")
            .long_comment("long")
            .has_base(r("https://example.org/lab/unit_meter"))
            .has_exponent(2)
            .name("unit exponentiation name")
            .description("unit exponentiation description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::UnitMultiplication(
        UnitMultiplication::builder(NS, "unit_multiplication")
            .unwrap()
            .label("newton meter")
            .symbol("N.m")
            .add_alternative_label("torque".to_string())
            .add_alternative_symbol("Nm".to_string())
            .comment("c")
            .long_comment("long")
            .has_term1(r("https://example.org/lab/unit_newton"))
            .has_term2(r("https://example.org/lab/unit_meter"))
            .name("unit multiplication name")
            .description("unit multiplication description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::PrefixedUnit(
        PrefixedUnit::builder(NS, "unit_kilometer")
            .unwrap()
            .label("kilometer")
            .symbol("km")
            .add_alternative_label("kilometre".to_string())
            .add_alternative_symbol("k.m".to_string())
            .comment("c")
            .long_comment("long")
            .has_unit(r("https://example.org/lab/unit_meter"))
            .has_prefix(r("https://example.org/lab/prefix_kilo"))
            .name("prefixed unit name")
            .description("prefixed unit description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::Prefix(
        Prefix::builder(NS, "prefix_abstract")
            .unwrap()
            .label("abstract prefix")
            .symbol("ap")
            .has_factor(1.0)
            .add_alternative_label("alt".to_string())
            .add_alternative_symbol("a.".to_string())
            .comment("c")
            .long_comment("long")
            .name("prefix name")
            .description("prefix description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::SIPrefix(
        SIPrefix::builder(NS, "prefix_kilo")
            .unwrap()
            .label("kilo")
            .symbol("k")
            .has_factor(1000.0)
            .add_alternative_label("alt".to_string())
            .add_alternative_symbol("ki".to_string())
            .comment("c")
            .long_comment("long")
            .name("si prefix name")
            .description("si prefix description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    out.push(SbolObject::BinaryPrefix(
        BinaryPrefix::builder(NS, "prefix_kibi")
            .unwrap()
            .label("kibi")
            .symbol("Ki")
            .has_factor(1024.0)
            .add_alternative_label("alt".to_string())
            .add_alternative_symbol("KiB".to_string())
            .comment("c")
            .long_comment("long")
            .name("binary prefix name")
            .description("binary prefix description")
            .add_derived_from(r("https://example.org/d"))
            .add_generated_by(r("https://example.org/g"))
            .add_measure(r("https://example.org/m"))
            .add_attachment(r("https://example.org/att"))
            .build()
            .unwrap(),
    ));

    // Suppress unused warnings for parents that aren't referenced again.
    let _ = sub_component_parent;
    out
}

#[test]
fn descriptor_predicates_match_emitted_for_every_typed_class() {
    let instances = populated_instances();
    let mut seen_classes = BTreeSet::new();
    for object in &instances {
        let class_iri = object.class().iri();
        assert!(
            seen_classes.insert(class_iri),
            "duplicate populated instance for {class_iri}"
        );
        let descriptor = descriptor_predicates(class_iri);
        let emitted = emitted_predicates(object);
        assert_eq!(
            descriptor, emitted,
            "predicate-set drift for {class_iri}: descriptor={descriptor:?} emitted={emitted:?}",
        );
    }
}

#[test]
fn bare_sbol_identified_subject_survives_typed_round_trip() {
    // Regression: prior to A.1, RDF subjects whose only `rdf:type` was
    // `sbol:Identified` dropped on the typed round trip. The
    // `IdentifiedExtension` variant captures them and preserves
    // displayId / name / extension triples / hasMeasure references so
    // `Document::from_objects` -> `write_turtle` -> `read_turtle`
    // reproduces the original graph.
    let turtle = r#"
@prefix sbol: <http://sbols.org/v3#> .
@prefix lab: <https://example.org/lab/> .
@prefix igem: <http://parts.igem.org/> .

<https://example.org/lab/bare> a sbol:Identified ;
    sbol:displayId "bare" ;
    sbol:name "bare identified" ;
    sbol:description "a subject typed only as sbol:Identified" ;
    igem:partType "Promoter" ;
    igem:status "Available" .
"#;

    let document = Document::read_turtle(turtle).expect("turtle must parse");
    let identity = r("https://example.org/lab/bare");
    let bare = document
        .typed_objects()
        .iter()
        .find(|o| o.identity() == &identity)
        .expect("bare-Identified subject must round-trip as a typed object");
    let SbolObject::IdentifiedExtension(extension) = bare else {
        panic!("bare-Identified subject should land on IdentifiedExtension, got {bare:?}");
    };
    assert_eq!(extension.identified.display_id.as_deref(), Some("bare"));
    assert_eq!(
        extension.identified.name.as_deref(),
        Some("bare identified")
    );
    let extension_predicates: Vec<&str> = extension
        .identified
        .extensions
        .iter()
        .map(|triple| triple.predicate.as_str())
        .collect();
    assert!(
        extension_predicates.contains(&"http://parts.igem.org/partType"),
        "iGEM extension triple must survive parsing; got {extension_predicates:?}",
    );
    assert!(
        extension_predicates.contains(&"http://parts.igem.org/status"),
        "second iGEM extension triple must survive parsing; got {extension_predicates:?}",
    );

    // Round-trip through `from_objects` -> Turtle -> reparse. The graph
    // must be byte-identical (modulo whitespace) and the typed cache
    // must contain the same IdentifiedExtension.
    let rebuilt = Document::from_objects(document.typed_objects().to_vec()).expect("typed rebuild");
    let serialized = rebuilt.write_turtle().expect("serialize");
    let reparsed = Document::read_turtle(&serialized).expect("reparse");
    assert_eq!(
        document.rdf_graph().normalized_triples(),
        reparsed.rdf_graph().normalized_triples(),
        "extension triples on bare sbol:Identified subjects must round-trip"
    );
    let round_tripped = reparsed
        .typed_objects()
        .iter()
        .find(|o| o.identity() == &identity)
        .expect("identity preserved after round trip");
    assert_eq!(bare, round_tripped);
}

#[test]
fn round_trip_preserves_every_populated_field_in_every_format() {
    for original in populated_instances() {
        let class_iri = original.class().iri();
        let document = Document::from_objects(vec![original.clone()])
            .unwrap_or_else(|error| panic!("from_objects failed for {class_iri}: {error}"));

        for &format in sbol3::RdfFormat::ALL {
            let serialized = document
                .write(format)
                .unwrap_or_else(|error| panic!("write {format} failed for {class_iri}: {error}"));
            let reparsed = Document::read(&serialized, format)
                .unwrap_or_else(|error| panic!("read {format} failed for {class_iri}: {error}"));
            let round_tripped = reparsed
                .typed_objects()
                .iter()
                .find(|object| object.identity() == original.identity())
                .unwrap_or_else(|| {
                    panic!("identity dropped after {format} round trip for {class_iri}")
                })
                .clone();
            assert_eq!(
                original, round_tripped,
                "{format} round-trip changed object for {class_iri}",
            );
        }
    }
}
