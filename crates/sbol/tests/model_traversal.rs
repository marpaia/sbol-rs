use sbol::{Document, Resource, SbolClass, SbolObject};

const PREFIXES: &str = r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX EDAM: <https://identifiers.org/edam:>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX SO: <https://identifiers.org/SO:>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX sbol: <http://sbols.org/v3#>
"#;

#[test]
fn sbol_class_iris_and_hierarchy_match_public_contract() {
    use SbolClass::*;

    let classes = [
        (Identified, "Identified", false),
        (TopLevel, "TopLevel", true),
        (Attachment, "Attachment", true),
        (Collection, "Collection", true),
        (CombinatorialDerivation, "CombinatorialDerivation", true),
        (Component, "Component", true),
        (ComponentReference, "ComponentReference", false),
        (Constraint, "Constraint", false),
        (Cut, "Cut", false),
        (EntireSequence, "EntireSequence", false),
        (Experiment, "Experiment", true),
        (ExperimentalData, "ExperimentalData", true),
        (ExternallyDefined, "ExternallyDefined", false),
        (Feature, "Feature", false),
        (Implementation, "Implementation", true),
        (Interaction, "Interaction", false),
        (Interface, "Interface", false),
        (Location, "Location", false),
        (LocalSubComponent, "LocalSubComponent", false),
        (Model, "Model", true),
        (Participation, "Participation", false),
        (Range, "Range", false),
        (Sequence, "Sequence", true),
        (SequenceFeature, "SequenceFeature", false),
        (SubComponent, "SubComponent", false),
        (VariableFeature, "VariableFeature", false),
    ];

    for (class, local_name, is_top_level) in classes {
        let iri = sbol::Iri::new_unchecked(format!("http://sbols.org/v3#{local_name}"));
        assert_eq!(SbolClass::from_iri(&iri), Some(class));
        assert_eq!(class.iri(), iri.as_str());
        assert_eq!(class.local_name(), local_name);
        assert_eq!(class.is_top_level(), is_top_level);
        assert!(class.is_a(class));
    }

    for class in [
        Attachment,
        Collection,
        CombinatorialDerivation,
        Component,
        Experiment,
        ExperimentalData,
        Implementation,
        Model,
        Sequence,
    ] {
        assert!(class.is_a(TopLevel));
        assert!(class.is_a(Identified));
    }

    for class in [
        ComponentReference,
        ExternallyDefined,
        LocalSubComponent,
        SequenceFeature,
        SubComponent,
    ] {
        assert!(class.is_a(Feature));
        assert!(class.is_a(Identified));
    }

    for class in [Cut, EntireSequence, Range] {
        assert!(class.is_a(Location));
        assert!(class.is_a(Identified));
    }
}

#[test]
fn document_typed_iterators_expose_supported_properties() {
    let document = read_document(COMPREHENSIVE_DOCUMENT);

    let component = document
        .components()
        .find(|c| c.identified.display_id.as_deref() == Some("component"))
        .expect("missing component");
    assert_eq!(component.identified.name.as_deref(), Some("Component name"));
    assert_eq!(
        component.identified.description.as_deref(),
        Some("Component description"),
    );
    assert_eq!(
        iris(&component.types),
        ["https://identifiers.org/SBO:0000251"]
    );
    assert_eq!(
        iris(&component.roles),
        ["https://identifiers.org/SO:0000167"]
    );
    assert_eq!(
        resources(&component.sequences),
        ["https://example.org/sequence"]
    );
    assert_eq!(
        resources(&component.features),
        [
            "https://example.org/component/subcomponent",
            "https://example.org/component/component_reference",
            "https://example.org/component/local_subcomponent",
            "https://example.org/component/externally_defined",
            "https://example.org/component/sequence_feature",
        ]
    );
    assert_eq!(
        resources(&component.constraints),
        ["https://example.org/component/constraint"]
    );
    assert_eq!(
        resources(&component.interactions),
        ["https://example.org/component/interaction"]
    );
    assert_eq!(
        resources(&component.interfaces),
        ["https://example.org/component/interface"]
    );
    assert_eq!(resources(&component.models), ["https://example.org/model"]);
    assert_eq!(
        resources(&component.identified.measures),
        ["https://example.org/measure"]
    );
    assert_eq!(
        resources(&component.identified.derived_from),
        ["https://example.org/source_component"]
    );
    assert_eq!(
        resources(&component.identified.generated_by),
        ["https://example.org/activity"]
    );

    assert_eq!(
        component
            .top_level
            .namespace
            .as_ref()
            .map(|iri| iri.as_str()),
        Some("https://example.org"),
    );
    assert_eq!(
        resources(&component.top_level.attachments),
        ["https://example.org/attachment"]
    );

    let mut top_level_ids: Vec<&str> = document
        .top_levels()
        .filter_map(top_level_display_id)
        .collect();
    top_level_ids.sort();
    assert_eq!(
        top_level_ids,
        [
            "attachment",
            "collection",
            "component",
            "derivation",
            "experiment",
            "experimental_data",
            "implementation",
            "model",
            "sequence",
            "source_component",
            "variant",
            "variant_derivation",
        ]
    );
    assert_eq!(document.attachments().count(), 1);
    // `collections()` iterates owned typed variants exactly; Experiment is a
    // separate variant even though SBOL classifies it as a Collection subclass.
    assert_eq!(document.collections().count(), 1);
    assert_eq!(document.experiments().count(), 1);
    assert_eq!(document.experimental_data().count(), 1);
    assert_eq!(document.combinatorial_derivations().count(), 2);
    assert_eq!(document.components().count(), 3);
    assert_eq!(document.sequences().count(), 1);
    assert_eq!(document.models().count(), 1);
    assert_eq!(document.implementations().count(), 1);

    let attachment = match find(&document, "attachment") {
        SbolObject::Attachment(a) => a,
        other => panic!("expected Attachment, got {other:?}"),
    };
    assert_eq!(
        attachment.source.as_ref().map(ToString::to_string),
        Some("https://example.org/file.txt".to_owned()),
    );
    assert_eq!(
        attachment.format.as_ref().map(|iri| iri.as_str()),
        Some("https://identifiers.org/edam:format_3752"),
    );
    assert_eq!(attachment.size, Some(12));
    assert_eq!(attachment.hash.as_deref(), Some("abcdef"));
    assert_eq!(attachment.hash_algorithm.as_deref(), Some("sha256"));

    let collection = match find(&document, "collection") {
        SbolObject::Collection(c) => c,
        other => panic!("expected Collection, got {other:?}"),
    };
    assert_eq!(
        resources(&collection.members),
        [
            "https://example.org/component",
            "https://example.org/variant"
        ]
    );

    let derivation = match find(&document, "derivation") {
        SbolObject::CombinatorialDerivation(d) => d,
        other => panic!("expected CombinatorialDerivation, got {other:?}"),
    };
    assert_eq!(
        derivation.template.as_ref().map(ToString::to_string),
        Some("https://example.org/component".to_owned()),
    );
    assert_eq!(
        derivation.strategy.as_ref().map(|iri| iri.as_str()),
        Some("http://sbols.org/v3#enumerate"),
    );
    assert_eq!(
        resources(&derivation.variable_features),
        ["https://example.org/derivation/variable_feature"]
    );

    let sub_component = match find(&document, "component/subcomponent") {
        SbolObject::SubComponent(s) => s,
        other => panic!("expected SubComponent, got {other:?}"),
    };
    assert_eq!(
        iris(&sub_component.feature.roles),
        ["https://identifiers.org/SO:0000316"]
    );
    assert_eq!(
        sub_component
            .feature
            .orientation
            .as_ref()
            .map(|iri| iri.as_str()),
        Some("http://sbols.org/v3#inline"),
    );
    assert_eq!(
        sub_component.instance_of.as_ref().map(ToString::to_string),
        Some("https://example.org/source_component".to_owned()),
    );
    assert_eq!(
        sub_component
            .role_integration
            .as_ref()
            .map(|iri| iri.as_str()),
        Some("http://sbols.org/v3#mergeRoles"),
    );
    assert_eq!(
        resources(&sub_component.locations),
        ["https://example.org/component/subcomponent/range"]
    );
    assert_eq!(
        resources(&sub_component.source_locations),
        ["https://example.org/component/subcomponent/source_range"]
    );

    let component_reference = match find(&document, "component/component_reference") {
        SbolObject::ComponentReference(r) => r,
        other => panic!("expected ComponentReference, got {other:?}"),
    };
    assert_eq!(
        component_reference
            .in_child_of
            .as_ref()
            .map(ToString::to_string),
        Some("https://example.org/component/subcomponent".to_owned()),
    );
    assert_eq!(
        component_reference
            .refers_to
            .as_ref()
            .map(ToString::to_string),
        Some("https://example.org/source_component/source_feature".to_owned()),
    );

    let local_sub_component = match find(&document, "component/local_subcomponent") {
        SbolObject::LocalSubComponent(l) => l,
        other => panic!("expected LocalSubComponent, got {other:?}"),
    };
    assert_eq!(
        iris(&local_sub_component.types),
        ["https://identifiers.org/SBO:0000251"]
    );
    assert_eq!(
        resources(&local_sub_component.locations),
        ["https://example.org/component/local_subcomponent/cut"]
    );

    let externally_defined = match find(&document, "component/externally_defined") {
        SbolObject::ExternallyDefined(e) => e,
        other => panic!("expected ExternallyDefined, got {other:?}"),
    };
    assert_eq!(
        externally_defined
            .definition
            .as_ref()
            .map(ToString::to_string),
        Some("https://identifiers.org/uniprot:P03023".to_owned()),
    );
    assert_eq!(
        iris(&externally_defined.types),
        ["https://identifiers.org/SBO:0000251"]
    );

    let sequence_feature = match find(&document, "component/sequence_feature") {
        SbolObject::SequenceFeature(s) => s,
        other => panic!("expected SequenceFeature, got {other:?}"),
    };
    assert_eq!(
        resources(&sequence_feature.locations),
        ["https://example.org/component/sequence_feature/entire_sequence"]
    );

    let range = match find(&document, "component/subcomponent/range") {
        SbolObject::Range(r) => r,
        other => panic!("expected Range, got {other:?}"),
    };
    assert_eq!(
        range.location.sequence.as_ref().map(ToString::to_string),
        Some("https://example.org/sequence".to_owned()),
    );
    assert_eq!(
        range.location.orientation.as_ref().map(|iri| iri.as_str()),
        Some("http://sbols.org/v3#inline"),
    );
    assert_eq!(range.location.order, Some(1));
    assert_eq!(range.start, Some(1));
    assert_eq!(range.end, Some(4));

    let cut = match find(&document, "component/local_subcomponent/cut") {
        SbolObject::Cut(c) => c,
        other => panic!("expected Cut, got {other:?}"),
    };
    assert_eq!(cut.at, Some(2));

    matches!(
        find(&document, "component/sequence_feature/entire_sequence"),
        SbolObject::EntireSequence(_)
    );

    let constraint = match find(&document, "component/constraint") {
        SbolObject::Constraint(c) => c,
        other => panic!("expected Constraint, got {other:?}"),
    };
    assert_eq!(
        constraint.subject.as_ref().map(ToString::to_string),
        Some("https://example.org/component/subcomponent".to_owned()),
    );
    assert_eq!(
        constraint
            .constrained_object
            .as_ref()
            .map(ToString::to_string),
        Some("https://example.org/component/local_subcomponent".to_owned()),
    );
    assert_eq!(
        constraint.restriction.as_ref().map(|iri| iri.as_str()),
        Some("http://sbols.org/v3#precedes"),
    );

    let interaction = match find(&document, "component/interaction") {
        SbolObject::Interaction(i) => i,
        other => panic!("expected Interaction, got {other:?}"),
    };
    assert_eq!(
        iris(&interaction.types),
        ["https://identifiers.org/SBO:0000177"]
    );
    assert_eq!(
        resources(&interaction.participations),
        ["https://example.org/component/interaction/participation"]
    );

    let participation = match find(&document, "component/interaction/participation") {
        SbolObject::Participation(p) => p,
        other => panic!("expected Participation, got {other:?}"),
    };
    assert_eq!(
        iris(&participation.roles),
        ["https://identifiers.org/SBO:0000645"]
    );
    assert_eq!(
        participation.participant.as_ref().map(ToString::to_string),
        Some("https://example.org/component/subcomponent".to_owned()),
    );
    assert_eq!(
        participation
            .higher_order_participant
            .as_ref()
            .map(ToString::to_string),
        Some("https://example.org/component/interaction".to_owned()),
    );

    let interface = match find(&document, "component/interface") {
        SbolObject::Interface(i) => i,
        other => panic!("expected Interface, got {other:?}"),
    };
    assert_eq!(
        resources(&interface.inputs),
        ["https://example.org/component/subcomponent"]
    );
    assert_eq!(
        resources(&interface.outputs),
        ["https://example.org/component/local_subcomponent"]
    );
    assert_eq!(
        resources(&interface.nondirectional),
        ["https://example.org/component/sequence_feature"]
    );

    let variable_feature = match find(&document, "derivation/variable_feature") {
        SbolObject::VariableFeature(v) => v,
        other => panic!("expected VariableFeature, got {other:?}"),
    };
    assert_eq!(
        variable_feature
            .cardinality
            .as_ref()
            .map(|iri| iri.as_str()),
        Some("http://sbols.org/v3#one"),
    );
    assert_eq!(
        variable_feature.variable.as_ref().map(ToString::to_string),
        Some("https://example.org/component/subcomponent".to_owned()),
    );
    assert_eq!(
        resources(&variable_feature.variants),
        ["https://example.org/variant"]
    );
    assert_eq!(
        resources(&variable_feature.variant_collections),
        ["https://example.org/collection"]
    );
    assert_eq!(
        resources(&variable_feature.variant_derivations),
        ["https://example.org/variant_derivation"]
    );
    assert_eq!(
        resources(&variable_feature.variant_measures),
        ["https://example.org/measure"]
    );

    let implementation = match find(&document, "implementation") {
        SbolObject::Implementation(i) => i,
        other => panic!("expected Implementation, got {other:?}"),
    };
    assert_eq!(
        implementation.built.as_ref().map(ToString::to_string),
        Some("https://example.org/component".to_owned()),
    );

    let model = match find(&document, "model") {
        SbolObject::Model(m) => m,
        other => panic!("expected Model, got {other:?}"),
    };
    assert_eq!(
        model.source.as_ref().map(ToString::to_string),
        Some("https://example.org/model.xml".to_owned()),
    );
    assert_eq!(
        model.language.as_ref().map(|iri| iri.as_str()),
        Some("https://identifiers.org/edam:format_2585"),
    );
    assert_eq!(
        model.framework.as_ref().map(|iri| iri.as_str()),
        Some("https://identifiers.org/SBO:0000062"),
    );

    let sequence = match find(&document, "sequence") {
        SbolObject::Sequence(s) => s,
        other => panic!("expected Sequence, got {other:?}"),
    };
    assert_eq!(sequence.elements.as_deref(), Some("ATGC"));
    assert_eq!(
        sequence.encoding.as_ref().map(|iri| iri.as_str()),
        Some("https://identifiers.org/edam:format_1207"),
    );

    matches!(find(&document, "experiment"), SbolObject::Experiment(_));
    matches!(
        find(&document, "experimental_data"),
        SbolObject::ExperimentalData(_)
    );
}

fn read_document(body: &str) -> Document {
    Document::read_turtle(&format!("{PREFIXES}\n{body}")).expect("test document parses")
}

fn find<'a>(document: &'a Document, local_name: &str) -> &'a SbolObject {
    document
        .resolve(&resource(local_name))
        .unwrap_or_else(|| panic!("missing object {local_name}"))
}

fn resource(local_name: &str) -> Resource {
    Resource::iri(format!("https://example.org/{local_name}"))
}

fn resources(values: &[Resource]) -> Vec<String> {
    values.iter().map(ToString::to_string).collect()
}

fn iris(values: &[sbol::Iri]) -> Vec<&str> {
    values.iter().map(sbol::Iri::as_str).collect()
}

fn top_level_display_id(object: &SbolObject) -> Option<&str> {
    let identified = match object {
        SbolObject::Attachment(o) => &o.identified,
        SbolObject::Collection(o) => &o.identified,
        SbolObject::CombinatorialDerivation(o) => &o.identified,
        SbolObject::Component(o) => &o.identified,
        SbolObject::Experiment(o) => &o.identified,
        SbolObject::ExperimentalData(o) => &o.identified,
        SbolObject::Implementation(o) => &o.identified,
        SbolObject::Model(o) => &o.identified,
        SbolObject::Sequence(o) => &o.identified,
        _ => return None,
    };
    identified.display_id.as_deref()
}

const COMPREHENSIVE_DOCUMENT: &str = r#":attachment a sbol:Attachment;
    sbol:displayId "attachment";
    sbol:format EDAM:format_3752;
    sbol:hash "abcdef";
    sbol:hashAlgorithm "sha256";
    sbol:hasNamespace <https://example.org>;
    sbol:size "12";
    sbol:source <https://example.org/file.txt> .

:collection a sbol:Collection;
    sbol:displayId "collection";
    sbol:hasNamespace <https://example.org>;
    sbol:member :component, :variant .

:sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "ATGC";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .

:source_component a sbol:Component;
    sbol:displayId "source_component";
    sbol:hasFeature <source_component/source_feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .

<source_component/source_feature> a sbol:SequenceFeature;
    sbol:displayId "source_feature" .

:component a sbol:Component;
    sbol:description "Component description";
    sbol:displayId "component";
    sbol:hasAttachment :attachment;
    sbol:hasConstraint <component/constraint>;
    sbol:hasFeature <component/subcomponent>,
        <component/component_reference>,
        <component/local_subcomponent>,
        <component/externally_defined>,
        <component/sequence_feature>;
    sbol:hasInteraction <component/interaction>;
    sbol:hasInterface <component/interface>;
    sbol:hasMeasure :measure;
    sbol:hasModel :model;
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :sequence;
    sbol:name "Component name";
    sbol:role SO:0000167;
    sbol:type SBO:0000251;
    prov:wasDerivedFrom :source_component;
    prov:wasGeneratedBy :activity .

<component/subcomponent> a sbol:SubComponent;
    sbol:displayId "subcomponent";
    sbol:hasLocation <component/subcomponent/range>;
    sbol:instanceOf :source_component;
    sbol:orientation sbol:inline;
    sbol:role SO:0000316;
    sbol:roleIntegration sbol:mergeRoles;
    sbol:sourceLocation <component/subcomponent/source_range> .

<component/subcomponent/range> a sbol:Range;
    sbol:displayId "range";
    sbol:end "4";
    sbol:hasSequence :sequence;
    sbol:order "1";
    sbol:orientation sbol:inline;
    sbol:start "1" .

<component/subcomponent/source_range> a sbol:Range;
    sbol:displayId "source_range";
    sbol:end "4";
    sbol:hasSequence :sequence;
    sbol:start "1" .

<component/component_reference> a sbol:ComponentReference;
    sbol:displayId "component_reference";
    sbol:inChildOf <component/subcomponent>;
    sbol:refersTo <source_component/source_feature> .

<component/local_subcomponent> a sbol:LocalSubComponent;
    sbol:displayId "local_subcomponent";
    sbol:hasLocation <component/local_subcomponent/cut>;
    sbol:type SBO:0000251 .

<component/local_subcomponent/cut> a sbol:Cut;
    sbol:at "2";
    sbol:displayId "cut";
    sbol:hasSequence :sequence .

<component/externally_defined> a sbol:ExternallyDefined;
    sbol:definition <https://identifiers.org/uniprot:P03023>;
    sbol:displayId "externally_defined";
    sbol:type SBO:0000251 .

<component/sequence_feature> a sbol:SequenceFeature;
    sbol:displayId "sequence_feature";
    sbol:hasLocation <component/sequence_feature/entire_sequence> .

<component/sequence_feature/entire_sequence> a sbol:EntireSequence;
    sbol:displayId "entire_sequence";
    sbol:hasSequence :sequence .

<component/constraint> a sbol:Constraint;
    sbol:displayId "constraint";
    sbol:object <component/local_subcomponent>;
    sbol:restriction sbol:precedes;
    sbol:subject <component/subcomponent> .

<component/interaction> a sbol:Interaction;
    sbol:displayId "interaction";
    sbol:hasParticipation <component/interaction/participation>;
    sbol:type SBO:0000177 .

<component/interaction/participation> a sbol:Participation;
    sbol:displayId "participation";
    sbol:higherOrderParticipant <component/interaction>;
    sbol:participant <component/subcomponent>;
    sbol:role SBO:0000645 .

<component/interface> a sbol:Interface;
    sbol:displayId "interface";
    sbol:input <component/subcomponent>;
    sbol:nondirectional <component/sequence_feature>;
    sbol:output <component/local_subcomponent> .

:variant a sbol:Component;
    sbol:displayId "variant";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .

:variant_derivation a sbol:CombinatorialDerivation;
    sbol:displayId "variant_derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:template :variant .

:derivation a sbol:CombinatorialDerivation;
    sbol:displayId "derivation";
    sbol:hasNamespace <https://example.org>;
    sbol:hasVariableFeature <derivation/variable_feature>;
    sbol:strategy sbol:enumerate;
    sbol:template :component .

<derivation/variable_feature> a sbol:VariableFeature;
    sbol:cardinality sbol:one;
    sbol:displayId "variable_feature";
    sbol:variable <component/subcomponent>;
    sbol:variant :variant;
    sbol:variantCollection :collection;
    sbol:variantDerivation :variant_derivation;
    sbol:variantMeasure :measure .

:implementation a sbol:Implementation;
    sbol:built :component;
    sbol:displayId "implementation";
    sbol:hasNamespace <https://example.org> .

:model a sbol:Model;
    sbol:displayId "model";
    sbol:framework SBO:0000062;
    sbol:hasNamespace <https://example.org>;
    sbol:language EDAM:format_2585;
    sbol:source <https://example.org/model.xml> .

:experimental_data a sbol:ExperimentalData;
    sbol:displayId "experimental_data";
    sbol:hasAttachment :attachment;
    sbol:hasNamespace <https://example.org> .

:experiment a sbol:Experiment;
    sbol:displayId "experiment";
    sbol:hasNamespace <https://example.org>;
    sbol:member :experimental_data .
"#;
