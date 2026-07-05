//! Generated-style structural rule cases: per-property cardinality,
//! value-kind, local-reference, and closed-property-set rules driven by
//! the SBOL 2 schema. A negative violates exactly one property of a
//! minimal carrier object; a positive supplies one valid value.

use super::common::{PositiveCase, RuleCase};
use sbol2::validation::Severity::Error;

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "sbol:persistentIdentity appears twice",
            rule: "sbol2-10203",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    sbol:persistentIdentity <http://ex/ref1>, <http://ex/refB> .
"#,
        },
        RuleCase {
            name: "sbol:displayId appears twice",
            rule: "sbol2-10204",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "aa", "bb" .
"#,
        },
        RuleCase {
            name: "sbol:version appears twice",
            rule: "sbol2-10206",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    sbol:version "1", "2" .
"#,
        },
        RuleCase {
            name: "dcterms:title appears twice",
            rule: "sbol2-10212",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    dcterms:title "text", "other" .
"#,
        },
        RuleCase {
            name: "dcterms:description appears twice",
            rule: "sbol2-10213",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    dcterms:description "text", "other" .
"#,
        },
        RuleCase {
            name: "prov:wasDerivedFrom wrong value kind",
            rule: "sbol2-10208",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    prov:wasDerivedFrom "not-a-uri" .
"#,
        },
        RuleCase {
            name: "prov:wasGeneratedBy wrong value kind",
            rule: "sbol2-10221",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    prov:wasGeneratedBy "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:attachment wrong value kind",
            rule: "sbol2-10306",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    sbol:attachment "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:definition missing",
            rule: "sbol2-10602",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:access missing",
            rule: "sbol2-10607",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:mapsTo wrong value kind",
            rule: "sbol2-10606",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" ;
    sbol:mapsTo "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:orientation appears twice",
            rule: "sbol2-11002",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:GenericLocation ;
    sbol:displayId "x" ;
    sbol:orientation <http://ex/ref1>, <http://ex/refB> .
"#,
        },
        RuleCase {
            name: "sbol:sequence appears twice",
            rule: "sbol2-11003",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:GenericLocation ;
    sbol:displayId "x" ;
    sbol:sequence <http://ex/ref1>, <http://ex/refB> .
"#,
        },
        RuleCase {
            name: "sbol:elements missing",
            rule: "sbol2-10402",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Sequence ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:encoding missing",
            rule: "sbol2-10403",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Sequence ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:type missing",
            rule: "sbol2-10502",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:role wrong value kind",
            rule: "sbol2-10507",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" ;
    sbol:role "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:sequence wrong value kind",
            rule: "sbol2-10512",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" ;
    sbol:sequence "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:component wrong value kind",
            rule: "sbol2-10519",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" ;
    sbol:component "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:sequenceAnnotation wrong value kind",
            rule: "sbol2-10521",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" ;
    sbol:sequenceAnnotation "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:sequenceConstraint wrong value kind",
            rule: "sbol2-10524",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" ;
    sbol:sequenceConstraint "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:role wrong value kind",
            rule: "sbol2-11602",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ModuleDefinition ;
    sbol:displayId "x" ;
    sbol:role "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:module wrong value kind",
            rule: "sbol2-11604",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ModuleDefinition ;
    sbol:displayId "x" ;
    sbol:module "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:functionalComponent wrong value kind",
            rule: "sbol2-11606",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ModuleDefinition ;
    sbol:displayId "x" ;
    sbol:functionalComponent "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:interaction wrong value kind",
            rule: "sbol2-11605",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ModuleDefinition ;
    sbol:displayId "x" ;
    sbol:interaction "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:model wrong value kind",
            rule: "sbol2-11607",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ModuleDefinition ;
    sbol:displayId "x" ;
    sbol:model "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:source missing",
            rule: "sbol2-11502",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Model ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:language missing",
            rule: "sbol2-11504",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Model ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:framework missing",
            rule: "sbol2-11508",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Model ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:member wrong value kind",
            rule: "sbol2-12102",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    sbol:member "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:template missing",
            rule: "sbol2-12904",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:CombinatorialDerivation ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:variableComponent wrong value kind",
            rule: "sbol2-12906",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:CombinatorialDerivation ;
    sbol:displayId "x" ;
    sbol:variableComponent "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:strategy appears twice",
            rule: "sbol2-12914",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:CombinatorialDerivation ;
    sbol:displayId "x" ;
    sbol:strategy <http://ex/ref1>, <http://ex/refB> .
"#,
        },
        RuleCase {
            name: "sbol:built appears twice",
            rule: "sbol2-13102",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Implementation ;
    sbol:displayId "x" ;
    sbol:built <http://ex/ref1>, <http://ex/refB> .
"#,
        },
        RuleCase {
            name: "sbol:source missing",
            rule: "sbol2-13202",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Attachment ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:format appears twice",
            rule: "sbol2-13204",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Attachment ;
    sbol:displayId "x" ;
    sbol:format <http://ex/ref1>, <http://ex/refB> .
"#,
        },
        RuleCase {
            name: "sbol:size appears twice",
            rule: "sbol2-13207",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Attachment ;
    sbol:displayId "x" ;
    sbol:size 1, 2 .
"#,
        },
        RuleCase {
            name: "sbol:hash appears twice",
            rule: "sbol2-13208",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Attachment ;
    sbol:displayId "x" ;
    sbol:hash "text", "other" .
"#,
        },
        RuleCase {
            name: "sbol:experimentalData wrong value kind",
            rule: "sbol2-13402",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Experiment ;
    sbol:displayId "x" ;
    sbol:experimentalData "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:rdfType missing",
            rule: "sbol2-12302",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:GenericTopLevel ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:role wrong value kind",
            rule: "sbol2-10702",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" ;
    sbol:role "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:roleIntegration appears twice",
            rule: "sbol2-10708",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" ;
    sbol:roleIntegration <http://ex/ref1>, <http://ex/refB> .
"#,
        },
        RuleCase {
            name: "sbol:sourceLocation wrong value kind",
            rule: "sbol2-10710",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" ;
    sbol:sourceLocation "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:direction missing",
            rule: "sbol2-11802",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:FunctionalComponent ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:definition missing",
            rule: "sbol2-11702",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Module ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:mapsTo wrong value kind",
            rule: "sbol2-11706",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Module ;
    sbol:displayId "x" ;
    sbol:mapsTo "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:local missing",
            rule: "sbol2-10802",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:MapsTo ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:remote missing",
            rule: "sbol2-10805",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:MapsTo ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:refinement missing",
            rule: "sbol2-10810",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:MapsTo ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:location missing",
            rule: "sbol2-10902",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:SequenceAnnotation ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:component appears twice",
            rule: "sbol2-10904",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:SequenceAnnotation ;
    sbol:displayId "x" ;
    sbol:component <http://ex/ref1>, <http://ex/refB> .
"#,
        },
        RuleCase {
            name: "sbol:role wrong value kind",
            rule: "sbol2-10906",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:SequenceAnnotation ;
    sbol:displayId "x" ;
    sbol:role "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:subject missing",
            rule: "sbol2-11402",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:SequenceConstraint ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:object missing",
            rule: "sbol2-11404",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:SequenceConstraint ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:restriction missing",
            rule: "sbol2-11407",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:SequenceConstraint ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:start missing",
            rule: "sbol2-11102",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Range ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:end missing",
            rule: "sbol2-11103",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Range ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:at missing",
            rule: "sbol2-11202",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Cut ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:type missing",
            rule: "sbol2-11902",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Interaction ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:participation wrong value kind",
            rule: "sbol2-11906",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Interaction ;
    sbol:displayId "x" ;
    sbol:participation "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:role missing",
            rule: "sbol2-12004",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Participation ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:participant missing",
            rule: "sbol2-12002",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Participation ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:variable missing",
            rule: "sbol2-13004",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:VariableComponent ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:variant wrong value kind",
            rule: "sbol2-13007",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:VariableComponent ;
    sbol:displayId "x" ;
    sbol:variant "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:variantCollection wrong value kind",
            rule: "sbol2-13009",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:VariableComponent ;
    sbol:displayId "x" ;
    sbol:variantCollection "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:variantDerivation wrong value kind",
            rule: "sbol2-13013",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:VariableComponent ;
    sbol:displayId "x" ;
    sbol:variantDerivation "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:operator missing",
            rule: "sbol2-13002",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:VariableComponent ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:type wrong value kind",
            rule: "sbol2-12412",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" ;
    sbol:type "not-a-uri" .
"#,
        },
        RuleCase {
            name: "prov:startedAtTime appears twice",
            rule: "sbol2-12402",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" ;
    prov:startedAtTime "2020-01-01T00:00:00Z"^^xsd:dateTime, "2021-01-01T00:00:00Z"^^xsd:dateTime .
"#,
        },
        RuleCase {
            name: "prov:endedAtTime appears twice",
            rule: "sbol2-12403",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" ;
    prov:endedAtTime "2020-01-01T00:00:00Z"^^xsd:dateTime, "2021-01-01T00:00:00Z"^^xsd:dateTime .
"#,
        },
        RuleCase {
            name: "prov:qualifiedAssociation wrong value kind",
            rule: "sbol2-12404",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" ;
    prov:qualifiedAssociation "not-a-uri" .
"#,
        },
        RuleCase {
            name: "prov:qualifiedUsage wrong value kind",
            rule: "sbol2-12405",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" ;
    prov:qualifiedUsage "not-a-uri" .
"#,
        },
        RuleCase {
            name: "prov:wasInformedBy wrong value kind",
            rule: "sbol2-12406",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" ;
    prov:wasInformedBy "not-a-uri" .
"#,
        },
        RuleCase {
            name: "prov:agent missing",
            rule: "sbol2-12605",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Association ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "prov:hadRole wrong value kind",
            rule: "sbol2-12602",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Association ;
    sbol:displayId "x" ;
    prov:hadRole "not-a-uri" .
"#,
        },
        RuleCase {
            name: "prov:hadPlan appears twice",
            rule: "sbol2-12603",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Association ;
    sbol:displayId "x" ;
    prov:hadPlan <http://ex/ref1>, <http://ex/refB> .
"#,
        },
        RuleCase {
            name: "prov:entity missing",
            rule: "sbol2-12502",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Usage ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "prov:hadRole wrong value kind",
            rule: "sbol2-12503",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Usage ;
    sbol:displayId "x" ;
    prov:hadRole "not-a-uri" .
"#,
        },
        RuleCase {
            name: "om:hasNumericalValue missing",
            rule: "sbol2-13502",
            severity: Error,
            body: r#"<http://ex/subj> a om:Measure ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "om:hasUnit missing",
            rule: "sbol2-13503",
            severity: Error,
            body: r#"<http://ex/subj> a om:Measure ;
    sbol:displayId "x" .
"#,
        },
        RuleCase {
            name: "sbol:type wrong value kind",
            rule: "sbol2-13504",
            severity: Error,
            body: r#"<http://ex/subj> a om:Measure ;
    sbol:displayId "x" ;
    sbol:type "not-a-uri" .
"#,
        },
        RuleCase {
            name: "sbol:Sequence carries a disallowed SBOL property",
            rule: "sbol2-10401",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Sequence ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:ComponentDefinition carries a disallowed SBOL property",
            rule: "sbol2-10501",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:Component carries a disallowed SBOL property",
            rule: "sbol2-10701",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:MapsTo carries a disallowed SBOL property",
            rule: "sbol2-10801",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:MapsTo ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:SequenceAnnotation carries a disallowed SBOL property",
            rule: "sbol2-10901",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:SequenceAnnotation ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:Range carries a disallowed SBOL property",
            rule: "sbol2-11101",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Range ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:Cut carries a disallowed SBOL property",
            rule: "sbol2-11201",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Cut ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:GenericLocation carries a disallowed SBOL property",
            rule: "sbol2-11301",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:GenericLocation ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:SequenceConstraint carries a disallowed SBOL property",
            rule: "sbol2-11401",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:SequenceConstraint ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:Model carries a disallowed SBOL property",
            rule: "sbol2-11501",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Model ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:ModuleDefinition carries a disallowed SBOL property",
            rule: "sbol2-11601",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ModuleDefinition ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:Module carries a disallowed SBOL property",
            rule: "sbol2-11701",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Module ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:FunctionalComponent carries a disallowed SBOL property",
            rule: "sbol2-11801",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:FunctionalComponent ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:Interaction carries a disallowed SBOL property",
            rule: "sbol2-11901",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Interaction ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:Participation carries a disallowed SBOL property",
            rule: "sbol2-12001",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Participation ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:Collection carries a disallowed SBOL property",
            rule: "sbol2-12101",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:GenericTopLevel carries a disallowed SBOL property",
            rule: "sbol2-12301",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:GenericTopLevel ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "prov:Activity carries a disallowed SBOL property",
            rule: "sbol2-12401",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "prov:Usage carries a disallowed SBOL property",
            rule: "sbol2-12501",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Usage ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "prov:Association carries a disallowed SBOL property",
            rule: "sbol2-12601",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Association ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "prov:Plan carries a disallowed SBOL property",
            rule: "sbol2-12701",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Plan ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "prov:Agent carries a disallowed SBOL property",
            rule: "sbol2-12801",
            severity: Error,
            body: r#"<http://ex/subj> a prov:Agent ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:CombinatorialDerivation carries a disallowed SBOL property",
            rule: "sbol2-12901",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:CombinatorialDerivation ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:VariableComponent carries a disallowed SBOL property",
            rule: "sbol2-13001",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:VariableComponent ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:Implementation carries a disallowed SBOL property",
            rule: "sbol2-13101",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Implementation ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:Attachment carries a disallowed SBOL property",
            rule: "sbol2-13201",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Attachment ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:ExperimentalData carries a disallowed SBOL property",
            rule: "sbol2-13301",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:ExperimentalData ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "sbol:Experiment carries a disallowed SBOL property",
            rule: "sbol2-13401",
            severity: Error,
            body: r#"<http://ex/subj> a sbol:Experiment ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
        RuleCase {
            name: "om:Measure carries a disallowed SBOL property",
            rule: "sbol2-13501",
            severity: Error,
            body: r#"<http://ex/subj> a om:Measure ;
    sbol:displayId "x" ;
    sbol:notARealProperty "x" .
"#,
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "sbol:persistentIdentity present and valid",
            rule: "sbol2-10203",
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    sbol:persistentIdentity <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:displayId present and valid",
            rule: "sbol2-10204",
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "validId" .
"#,
        },
        PositiveCase {
            name: "sbol:version present and valid",
            rule: "sbol2-10206",
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    sbol:version "1" .
"#,
        },
        PositiveCase {
            name: "dcterms:title present and valid",
            rule: "sbol2-10212",
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    dcterms:title "text" .
"#,
        },
        PositiveCase {
            name: "dcterms:description present and valid",
            rule: "sbol2-10213",
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    dcterms:description "text" .
"#,
        },
        PositiveCase {
            name: "prov:wasDerivedFrom present and valid",
            rule: "sbol2-10208",
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    prov:wasDerivedFrom <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "prov:wasGeneratedBy present and valid",
            rule: "sbol2-10221",
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    prov:wasGeneratedBy <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:attachment present and valid",
            rule: "sbol2-10306",
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    sbol:attachment <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:definition present and valid",
            rule: "sbol2-10602",
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" ;
    sbol:definition <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:access present and valid",
            rule: "sbol2-10607",
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" ;
    sbol:access <http://sbols.org/v2#public> .
"#,
        },
        PositiveCase {
            name: "sbol:mapsTo present and valid",
            rule: "sbol2-10606",
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" ;
    sbol:mapsTo <http://ex/ref1> .
<http://ex/ref1> a sbol:MapsTo ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "sbol:orientation present and valid",
            rule: "sbol2-11002",
            body: r#"<http://ex/subj> a sbol:GenericLocation ;
    sbol:displayId "x" ;
    sbol:orientation <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:sequence present and valid",
            rule: "sbol2-11003",
            body: r#"<http://ex/subj> a sbol:GenericLocation ;
    sbol:displayId "x" ;
    sbol:sequence <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:elements present and valid",
            rule: "sbol2-10402",
            body: r#"<http://ex/subj> a sbol:Sequence ;
    sbol:displayId "x" ;
    sbol:elements "text" .
"#,
        },
        PositiveCase {
            name: "sbol:encoding present and valid",
            rule: "sbol2-10403",
            body: r#"<http://ex/subj> a sbol:Sequence ;
    sbol:displayId "x" ;
    sbol:encoding <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:type present and valid",
            rule: "sbol2-10502",
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" ;
    sbol:type <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:role present and valid",
            rule: "sbol2-10507",
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" ;
    sbol:role <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:sequence present and valid",
            rule: "sbol2-10512",
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" ;
    sbol:sequence <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:component present and valid",
            rule: "sbol2-10519",
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" ;
    sbol:component <http://ex/ref1> .
<http://ex/ref1> a sbol:Component ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "sbol:sequenceAnnotation present and valid",
            rule: "sbol2-10521",
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" ;
    sbol:sequenceAnnotation <http://ex/ref1> .
<http://ex/ref1> a sbol:SequenceAnnotation ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "sbol:sequenceConstraint present and valid",
            rule: "sbol2-10524",
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" ;
    sbol:sequenceConstraint <http://ex/ref1> .
<http://ex/ref1> a sbol:SequenceConstraint ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "sbol:role present and valid",
            rule: "sbol2-11602",
            body: r#"<http://ex/subj> a sbol:ModuleDefinition ;
    sbol:displayId "x" ;
    sbol:role <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:module present and valid",
            rule: "sbol2-11604",
            body: r#"<http://ex/subj> a sbol:ModuleDefinition ;
    sbol:displayId "x" ;
    sbol:module <http://ex/ref1> .
<http://ex/ref1> a sbol:Module ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "sbol:functionalComponent present and valid",
            rule: "sbol2-11606",
            body: r#"<http://ex/subj> a sbol:ModuleDefinition ;
    sbol:displayId "x" ;
    sbol:functionalComponent <http://ex/ref1> .
<http://ex/ref1> a sbol:FunctionalComponent ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "sbol:interaction present and valid",
            rule: "sbol2-11605",
            body: r#"<http://ex/subj> a sbol:ModuleDefinition ;
    sbol:displayId "x" ;
    sbol:interaction <http://ex/ref1> .
<http://ex/ref1> a sbol:Interaction ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "sbol:model present and valid",
            rule: "sbol2-11607",
            body: r#"<http://ex/subj> a sbol:ModuleDefinition ;
    sbol:displayId "x" ;
    sbol:model <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:source present and valid",
            rule: "sbol2-11502",
            body: r#"<http://ex/subj> a sbol:Model ;
    sbol:displayId "x" ;
    sbol:source <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:language present and valid",
            rule: "sbol2-11504",
            body: r#"<http://ex/subj> a sbol:Model ;
    sbol:displayId "x" ;
    sbol:language <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:framework present and valid",
            rule: "sbol2-11508",
            body: r#"<http://ex/subj> a sbol:Model ;
    sbol:displayId "x" ;
    sbol:framework <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:member present and valid",
            rule: "sbol2-12102",
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" ;
    sbol:member <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:template present and valid",
            rule: "sbol2-12904",
            body: r#"<http://ex/subj> a sbol:CombinatorialDerivation ;
    sbol:displayId "x" ;
    sbol:template <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:variableComponent present and valid",
            rule: "sbol2-12906",
            body: r#"<http://ex/subj> a sbol:CombinatorialDerivation ;
    sbol:displayId "x" ;
    sbol:variableComponent <http://ex/ref1> .
<http://ex/ref1> a sbol:VariableComponent ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "sbol:strategy present and valid",
            rule: "sbol2-12914",
            body: r#"<http://ex/subj> a sbol:CombinatorialDerivation ;
    sbol:displayId "x" ;
    sbol:strategy <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:built present and valid",
            rule: "sbol2-13102",
            body: r#"<http://ex/subj> a sbol:Implementation ;
    sbol:displayId "x" ;
    sbol:built <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:source present and valid",
            rule: "sbol2-13202",
            body: r#"<http://ex/subj> a sbol:Attachment ;
    sbol:displayId "x" ;
    sbol:source <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:format present and valid",
            rule: "sbol2-13204",
            body: r#"<http://ex/subj> a sbol:Attachment ;
    sbol:displayId "x" ;
    sbol:format <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:size present and valid",
            rule: "sbol2-13207",
            body: r#"<http://ex/subj> a sbol:Attachment ;
    sbol:displayId "x" ;
    sbol:size 1 .
"#,
        },
        PositiveCase {
            name: "sbol:hash present and valid",
            rule: "sbol2-13208",
            body: r#"<http://ex/subj> a sbol:Attachment ;
    sbol:displayId "x" ;
    sbol:hash "text" .
"#,
        },
        PositiveCase {
            name: "sbol:experimentalData present and valid",
            rule: "sbol2-13402",
            body: r#"<http://ex/subj> a sbol:Experiment ;
    sbol:displayId "x" ;
    sbol:experimentalData <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:rdfType present and valid",
            rule: "sbol2-12302",
            body: r#"<http://ex/subj> a sbol:GenericTopLevel ;
    sbol:displayId "x" ;
    sbol:rdfType <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:role present and valid",
            rule: "sbol2-10702",
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" ;
    sbol:role <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:roleIntegration present and valid",
            rule: "sbol2-10708",
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" ;
    sbol:roleIntegration <http://sbols.org/v2#mergeRoles> .
"#,
        },
        PositiveCase {
            name: "sbol:sourceLocation present and valid",
            rule: "sbol2-10710",
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" ;
    sbol:sourceLocation <http://ex/ref1> .
<http://ex/ref1> a sbol:GenericLocation ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "sbol:direction present and valid",
            rule: "sbol2-11802",
            body: r#"<http://ex/subj> a sbol:FunctionalComponent ;
    sbol:displayId "x" ;
    sbol:direction <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:definition present and valid",
            rule: "sbol2-11702",
            body: r#"<http://ex/subj> a sbol:Module ;
    sbol:displayId "x" ;
    sbol:definition <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:mapsTo present and valid",
            rule: "sbol2-11706",
            body: r#"<http://ex/subj> a sbol:Module ;
    sbol:displayId "x" ;
    sbol:mapsTo <http://ex/ref1> .
<http://ex/ref1> a sbol:MapsTo ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "sbol:local present and valid",
            rule: "sbol2-10802",
            body: r#"<http://ex/subj> a sbol:MapsTo ;
    sbol:displayId "x" ;
    sbol:local <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:remote present and valid",
            rule: "sbol2-10805",
            body: r#"<http://ex/subj> a sbol:MapsTo ;
    sbol:displayId "x" ;
    sbol:remote <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:refinement present and valid",
            rule: "sbol2-10810",
            body: r#"<http://ex/subj> a sbol:MapsTo ;
    sbol:displayId "x" ;
    sbol:refinement <http://sbols.org/v2#verifyIdentical> .
"#,
        },
        PositiveCase {
            name: "sbol:location present and valid",
            rule: "sbol2-10902",
            body: r#"<http://ex/subj> a sbol:SequenceAnnotation ;
    sbol:displayId "x" ;
    sbol:location <http://ex/ref1> .
<http://ex/ref1> a sbol:GenericLocation ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "sbol:component present and valid",
            rule: "sbol2-10904",
            body: r#"<http://ex/subj> a sbol:SequenceAnnotation ;
    sbol:displayId "x" ;
    sbol:component <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:role present and valid",
            rule: "sbol2-10906",
            body: r#"<http://ex/subj> a sbol:SequenceAnnotation ;
    sbol:displayId "x" ;
    sbol:role <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:subject present and valid",
            rule: "sbol2-11402",
            body: r#"<http://ex/subj> a sbol:SequenceConstraint ;
    sbol:displayId "x" ;
    sbol:subject <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:object present and valid",
            rule: "sbol2-11404",
            body: r#"<http://ex/subj> a sbol:SequenceConstraint ;
    sbol:displayId "x" ;
    sbol:object <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:restriction present and valid",
            rule: "sbol2-11407",
            body: r#"<http://ex/subj> a sbol:SequenceConstraint ;
    sbol:displayId "x" ;
    sbol:restriction <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:start present and valid",
            rule: "sbol2-11102",
            body: r#"<http://ex/subj> a sbol:Range ;
    sbol:displayId "x" ;
    sbol:start 1 .
"#,
        },
        PositiveCase {
            name: "sbol:end present and valid",
            rule: "sbol2-11103",
            body: r#"<http://ex/subj> a sbol:Range ;
    sbol:displayId "x" ;
    sbol:end 1 .
"#,
        },
        PositiveCase {
            name: "sbol:at present and valid",
            rule: "sbol2-11202",
            body: r#"<http://ex/subj> a sbol:Cut ;
    sbol:displayId "x" ;
    sbol:at 1 .
"#,
        },
        PositiveCase {
            name: "sbol:type present and valid",
            rule: "sbol2-11902",
            body: r#"<http://ex/subj> a sbol:Interaction ;
    sbol:displayId "x" ;
    sbol:type <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:participation present and valid",
            rule: "sbol2-11906",
            body: r#"<http://ex/subj> a sbol:Interaction ;
    sbol:displayId "x" ;
    sbol:participation <http://ex/ref1> .
<http://ex/ref1> a sbol:Participation ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "sbol:role present and valid",
            rule: "sbol2-12004",
            body: r#"<http://ex/subj> a sbol:Participation ;
    sbol:displayId "x" ;
    sbol:role <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:participant present and valid",
            rule: "sbol2-12002",
            body: r#"<http://ex/subj> a sbol:Participation ;
    sbol:displayId "x" ;
    sbol:participant <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:variable present and valid",
            rule: "sbol2-13004",
            body: r#"<http://ex/subj> a sbol:VariableComponent ;
    sbol:displayId "x" ;
    sbol:variable <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:variant present and valid",
            rule: "sbol2-13007",
            body: r#"<http://ex/subj> a sbol:VariableComponent ;
    sbol:displayId "x" ;
    sbol:variant <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:variantCollection present and valid",
            rule: "sbol2-13009",
            body: r#"<http://ex/subj> a sbol:VariableComponent ;
    sbol:displayId "x" ;
    sbol:variantCollection <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:variantDerivation present and valid",
            rule: "sbol2-13013",
            body: r#"<http://ex/subj> a sbol:VariableComponent ;
    sbol:displayId "x" ;
    sbol:variantDerivation <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:operator present and valid",
            rule: "sbol2-13002",
            body: r#"<http://ex/subj> a sbol:VariableComponent ;
    sbol:displayId "x" ;
    sbol:operator <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:type present and valid",
            rule: "sbol2-12412",
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" ;
    sbol:type <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "prov:startedAtTime present and valid",
            rule: "sbol2-12402",
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" ;
    prov:startedAtTime "2020-01-01T00:00:00Z"^^xsd:dateTime .
"#,
        },
        PositiveCase {
            name: "prov:endedAtTime present and valid",
            rule: "sbol2-12403",
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" ;
    prov:endedAtTime "2020-01-01T00:00:00Z"^^xsd:dateTime .
"#,
        },
        PositiveCase {
            name: "prov:qualifiedAssociation present and valid",
            rule: "sbol2-12404",
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" ;
    prov:qualifiedAssociation <http://ex/ref1> .
<http://ex/ref1> a prov:Association ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "prov:qualifiedUsage present and valid",
            rule: "sbol2-12405",
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" ;
    prov:qualifiedUsage <http://ex/ref1> .
<http://ex/ref1> a prov:Usage ;
    sbol:displayId "c" .
"#,
        },
        PositiveCase {
            name: "prov:wasInformedBy present and valid",
            rule: "sbol2-12406",
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" ;
    prov:wasInformedBy <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "prov:agent present and valid",
            rule: "sbol2-12605",
            body: r#"<http://ex/subj> a prov:Association ;
    sbol:displayId "x" ;
    prov:agent <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "prov:hadRole present and valid",
            rule: "sbol2-12602",
            body: r#"<http://ex/subj> a prov:Association ;
    sbol:displayId "x" ;
    prov:hadRole <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "prov:hadPlan present and valid",
            rule: "sbol2-12603",
            body: r#"<http://ex/subj> a prov:Association ;
    sbol:displayId "x" ;
    prov:hadPlan <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "prov:entity present and valid",
            rule: "sbol2-12502",
            body: r#"<http://ex/subj> a prov:Usage ;
    sbol:displayId "x" ;
    prov:entity <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "prov:hadRole present and valid",
            rule: "sbol2-12503",
            body: r#"<http://ex/subj> a prov:Usage ;
    sbol:displayId "x" ;
    prov:hadRole <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "om:hasNumericalValue present and valid",
            rule: "sbol2-13502",
            body: r#"<http://ex/subj> a om:Measure ;
    sbol:displayId "x" ;
    om:hasNumericalValue 1.0 .
"#,
        },
        PositiveCase {
            name: "om:hasUnit present and valid",
            rule: "sbol2-13503",
            body: r#"<http://ex/subj> a om:Measure ;
    sbol:displayId "x" ;
    om:hasUnit <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:type present and valid",
            rule: "sbol2-13504",
            body: r#"<http://ex/subj> a om:Measure ;
    sbol:displayId "x" ;
    sbol:type <http://ex/ref1> .
"#,
        },
        PositiveCase {
            name: "sbol:Sequence with only permitted properties",
            rule: "sbol2-10401",
            body: r#"<http://ex/subj> a sbol:Sequence ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:ComponentDefinition with only permitted properties",
            rule: "sbol2-10501",
            body: r#"<http://ex/subj> a sbol:ComponentDefinition ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:Component with only permitted properties",
            rule: "sbol2-10701",
            body: r#"<http://ex/subj> a sbol:Component ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:MapsTo with only permitted properties",
            rule: "sbol2-10801",
            body: r#"<http://ex/subj> a sbol:MapsTo ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:SequenceAnnotation with only permitted properties",
            rule: "sbol2-10901",
            body: r#"<http://ex/subj> a sbol:SequenceAnnotation ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:Range with only permitted properties",
            rule: "sbol2-11101",
            body: r#"<http://ex/subj> a sbol:Range ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:Cut with only permitted properties",
            rule: "sbol2-11201",
            body: r#"<http://ex/subj> a sbol:Cut ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:GenericLocation with only permitted properties",
            rule: "sbol2-11301",
            body: r#"<http://ex/subj> a sbol:GenericLocation ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:SequenceConstraint with only permitted properties",
            rule: "sbol2-11401",
            body: r#"<http://ex/subj> a sbol:SequenceConstraint ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:Model with only permitted properties",
            rule: "sbol2-11501",
            body: r#"<http://ex/subj> a sbol:Model ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:ModuleDefinition with only permitted properties",
            rule: "sbol2-11601",
            body: r#"<http://ex/subj> a sbol:ModuleDefinition ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:Module with only permitted properties",
            rule: "sbol2-11701",
            body: r#"<http://ex/subj> a sbol:Module ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:FunctionalComponent with only permitted properties",
            rule: "sbol2-11801",
            body: r#"<http://ex/subj> a sbol:FunctionalComponent ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:Interaction with only permitted properties",
            rule: "sbol2-11901",
            body: r#"<http://ex/subj> a sbol:Interaction ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:Participation with only permitted properties",
            rule: "sbol2-12001",
            body: r#"<http://ex/subj> a sbol:Participation ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:Collection with only permitted properties",
            rule: "sbol2-12101",
            body: r#"<http://ex/subj> a sbol:Collection ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:GenericTopLevel with only permitted properties",
            rule: "sbol2-12301",
            body: r#"<http://ex/subj> a sbol:GenericTopLevel ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "prov:Activity with only permitted properties",
            rule: "sbol2-12401",
            body: r#"<http://ex/subj> a prov:Activity ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "prov:Usage with only permitted properties",
            rule: "sbol2-12501",
            body: r#"<http://ex/subj> a prov:Usage ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "prov:Association with only permitted properties",
            rule: "sbol2-12601",
            body: r#"<http://ex/subj> a prov:Association ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "prov:Plan with only permitted properties",
            rule: "sbol2-12701",
            body: r#"<http://ex/subj> a prov:Plan ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "prov:Agent with only permitted properties",
            rule: "sbol2-12801",
            body: r#"<http://ex/subj> a prov:Agent ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:CombinatorialDerivation with only permitted properties",
            rule: "sbol2-12901",
            body: r#"<http://ex/subj> a sbol:CombinatorialDerivation ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:VariableComponent with only permitted properties",
            rule: "sbol2-13001",
            body: r#"<http://ex/subj> a sbol:VariableComponent ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:Implementation with only permitted properties",
            rule: "sbol2-13101",
            body: r#"<http://ex/subj> a sbol:Implementation ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:Attachment with only permitted properties",
            rule: "sbol2-13201",
            body: r#"<http://ex/subj> a sbol:Attachment ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:ExperimentalData with only permitted properties",
            rule: "sbol2-13301",
            body: r#"<http://ex/subj> a sbol:ExperimentalData ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "sbol:Experiment with only permitted properties",
            rule: "sbol2-13401",
            body: r#"<http://ex/subj> a sbol:Experiment ;
    sbol:displayId "x" .
"#,
        },
        PositiveCase {
            name: "om:Measure with only permitted properties",
            rule: "sbol2-13501",
            body: r#"<http://ex/subj> a om:Measure ;
    sbol:displayId "x" .
"#,
        },
    ]
}
