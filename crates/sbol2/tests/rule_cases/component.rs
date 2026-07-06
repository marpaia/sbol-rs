//! ComponentDefinition, Component, SequenceAnnotation, and SequenceConstraint
//! semantic rules (105xx, 107xx, 109xx, 114xx).

use super::common::{PositiveCase, RuleCase};
use sbol2::validation::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "ComponentDefinition with more than one BioPAX type",
            rule: "sbol2-10503",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion>,
              <http://www.biopax.org/release/biopax-level3.owl#Protein> .
"#,
        },
        RuleCase {
            name: "sequence-feature role without a DNA or RNA type",
            rule: "sbol2-10511",
            severity: Warning,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#Protein> ;
    sbol:role <http://identifiers.org/so/SO:0000167> .
"#,
        },
        RuleCase {
            name: "ComponentDefinition sequence reference does not resolve",
            rule: "sbol2-10513",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:sequence <http://ex/missing-sequence> .
"#,
        },
        RuleCase {
            name: "DNA ComponentDefinition carries only a protein Sequence",
            rule: "sbol2-10516",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:sequence <http://ex/s> .
<http://ex/s> a sbol:Sequence ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/s> ;
    sbol:elements "MKV" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iupac/AminoAcid/> .
"#,
        },
        RuleCase {
            name: "two nucleic Sequences of one ComponentDefinition differ in length",
            rule: "sbol2-10518",
            severity: Warning,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:sequence <http://ex/s1>, <http://ex/s2> .
<http://ex/s1> a sbol:Sequence ;
    sbol:displayId "s1" ;
    sbol:persistentIdentity <http://ex/s1> ;
    sbol:elements "ACGT" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
<http://ex/s2> a sbol:Sequence ;
    sbol:displayId "s2" ;
    sbol:persistentIdentity <http://ex/s2> ;
    sbol:elements "ACG" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
"#,
        },
        RuleCase {
            name: "sub-component Sequence length disagrees with its annotated span",
            rule: "sbol2-10520",
            severity: Warning,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:sequence <http://ex/parent-seq> ;
    sbol:component <http://ex/cd/c> ;
    sbol:sequenceAnnotation <http://ex/cd/sa> .
<http://ex/parent-seq> a sbol:Sequence ;
    sbol:displayId "parent" ;
    sbol:persistentIdentity <http://ex/parent-seq> ;
    sbol:elements "ACGTACGT" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
<http://ex/cd/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/cd/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/child> .
<http://ex/cd/sa> a sbol:SequenceAnnotation ;
    sbol:displayId "sa" ;
    sbol:persistentIdentity <http://ex/cd/sa> ;
    sbol:component <http://ex/cd/c> ;
    sbol:location <http://ex/cd/sa/l> .
<http://ex/cd/sa/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa/l> ;
    sbol:start 1 ;
    sbol:end 4 .
<http://ex/child> a sbol:ComponentDefinition ;
    sbol:displayId "child" ;
    sbol:persistentIdentity <http://ex/child> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:sequence <http://ex/child-seq> .
<http://ex/child-seq> a sbol:Sequence ;
    sbol:displayId "childseq" ;
    sbol:persistentIdentity <http://ex/child-seq> ;
    sbol:elements "ACGTACGT" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
"#,
        },
        RuleCase {
            name: "two SequenceAnnotations refer to the same Component",
            rule: "sbol2-10522",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/c> ;
    sbol:sequenceAnnotation <http://ex/cd/sa1>, <http://ex/cd/sa2> .
<http://ex/cd/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/cd/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> .
<http://ex/cd/sa1> a sbol:SequenceAnnotation ;
    sbol:displayId "sa1" ;
    sbol:persistentIdentity <http://ex/cd/sa1> ;
    sbol:component <http://ex/cd/c> ;
    sbol:location <http://ex/cd/sa1/l> .
<http://ex/cd/sa1/l> a sbol:GenericLocation ;
    sbol:displayId "l1" ;
    sbol:persistentIdentity <http://ex/cd/sa1/l> .
<http://ex/cd/sa2> a sbol:SequenceAnnotation ;
    sbol:displayId "sa2" ;
    sbol:persistentIdentity <http://ex/cd/sa2> ;
    sbol:component <http://ex/cd/c> ;
    sbol:location <http://ex/cd/sa2/l> .
<http://ex/cd/sa2/l> a sbol:GenericLocation ;
    sbol:displayId "l2" ;
    sbol:persistentIdentity <http://ex/cd/sa2/l> .
"#,
        },
        RuleCase {
            name: "SequenceAnnotation position lies outside the sequence",
            rule: "sbol2-10523",
            severity: Warning,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:sequence <http://ex/s> ;
    sbol:sequenceAnnotation <http://ex/cd/sa> .
<http://ex/s> a sbol:Sequence ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/s> ;
    sbol:elements "ACGT" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
<http://ex/cd/sa> a sbol:SequenceAnnotation ;
    sbol:displayId "sa" ;
    sbol:persistentIdentity <http://ex/cd/sa> ;
    sbol:location <http://ex/cd/sa/l> .
<http://ex/cd/sa/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa/l> ;
    sbol:start 1 ;
    sbol:end 10 .
"#,
        },
        RuleCase {
            name: "ComponentDefinition without a BioPAX type",
            rule: "sbol2-10525",
            severity: Warning,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://example.org/custom-type> .
"#,
        },
        RuleCase {
            name: "two useRemote MapsTos of a ComponentDefinition share a local",
            rule: "sbol2-10526",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/local>, <http://ex/cd/c1>, <http://ex/cd/c2> .
<http://ex/cd/local> a sbol:Component ;
    sbol:displayId "local" ;
    sbol:persistentIdentity <http://ex/cd/local> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> .
<http://ex/cd/c1> a sbol:Component ;
    sbol:displayId "c1" ;
    sbol:persistentIdentity <http://ex/cd/c1> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:mapsTo <http://ex/cd/c1/m> .
<http://ex/cd/c1/m> a sbol:MapsTo ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/cd/c1/m> ;
    sbol:refinement <http://sbols.org/v2#useRemote> ;
    sbol:local <http://ex/cd/local> ;
    sbol:remote <http://ex/d/r> .
<http://ex/cd/c2> a sbol:Component ;
    sbol:displayId "c2" ;
    sbol:persistentIdentity <http://ex/cd/c2> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:mapsTo <http://ex/cd/c2/m> .
<http://ex/cd/c2/m> a sbol:MapsTo ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/cd/c2/m> ;
    sbol:refinement <http://sbols.org/v2#useRemote> ;
    sbol:local <http://ex/cd/local> ;
    sbol:remote <http://ex/d/r> .
"#,
        },
        RuleCase {
            name: "DNA ComponentDefinition without exactly one sequence-feature role",
            rule: "sbol2-10527",
            severity: Warning,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
        RuleCase {
            name: "DNA ComponentDefinition with two topology types",
            rule: "sbol2-10528",
            severity: Warning,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion>,
              <http://identifiers.org/so/SO:0000987>,
              <http://identifiers.org/so/SO:0000988> .
"#,
        },
        RuleCase {
            name: "topology type without a DNA or RNA type",
            rule: "sbol2-10529",
            severity: Warning,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#Protein>,
              <http://identifiers.org/so/SO:0000987> .
"#,
        },
        RuleCase {
            name: "Component definition refers to its own containing ComponentDefinition",
            rule: "sbol2-10603",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/c> .
<http://ex/cd/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/cd/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/cd> .
"#,
        },
        RuleCase {
            name: "Component definition is not a ComponentDefinition",
            rule: "sbol2-10604",
            severity: Error,
            body: r#"<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/not-cd> .
<http://ex/not-cd> a sbol:Collection ;
    sbol:displayId "notcd" ;
    sbol:persistentIdentity <http://ex/not-cd> .
"#,
        },
        RuleCase {
            name: "ComponentDefinition cycle through Component definitions",
            rule: "sbol2-10605",
            severity: Error,
            body: r#"<http://ex/a> a sbol:ComponentDefinition ;
    sbol:displayId "a" ;
    sbol:persistentIdentity <http://ex/a> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/a/c> .
<http://ex/a/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/a/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/b> .
<http://ex/b> a sbol:ComponentDefinition ;
    sbol:displayId "b" ;
    sbol:persistentIdentity <http://ex/b> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/b/c> .
<http://ex/b/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/b/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/a> .
"#,
        },
        RuleCase {
            name: "Component measure does not resolve to an om:Measure",
            rule: "sbol2-10608",
            severity: Error,
            body: r#"<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:measure <http://ex/not-measure> .
<http://ex/not-measure> a sbol:Collection ;
    sbol:displayId "nm" ;
    sbol:persistentIdentity <http://ex/not-measure> .
"#,
        },
        RuleCase {
            name: "Component role is a sequence-feature term but its definition is not DNA/RNA",
            rule: "sbol2-10706",
            severity: Warning,
            body: r#"<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/prot> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    sbol:roleIntegration <http://sbols.org/v2#mergeRoles> .
<http://ex/prot> a sbol:ComponentDefinition ;
    sbol:displayId "prot" ;
    sbol:persistentIdentity <http://ex/prot> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#Protein> .
"#,
        },
        RuleCase {
            name: "DNA Component with two sequence-feature roles",
            rule: "sbol2-10707",
            severity: Warning,
            body: r#"<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/dna> ;
    sbol:role <http://identifiers.org/so/SO:0000167>, <http://identifiers.org/so/SO:0000316> ;
    sbol:roleIntegration <http://sbols.org/v2#mergeRoles> .
<http://ex/dna> a sbol:ComponentDefinition ;
    sbol:displayId "dna" ;
    sbol:persistentIdentity <http://ex/dna> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
        RuleCase {
            name: "Component with roles but no roleIntegration",
            rule: "sbol2-10709",
            severity: Error,
            body: r#"<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:role <http://identifiers.org/so/SO:0000167> .
"#,
        },
        RuleCase {
            name: "Component source Locations overlap",
            rule: "sbol2-10711",
            severity: Warning,
            body: r#"<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:sourceLocation <http://ex/c/l1>, <http://ex/c/l2> .
<http://ex/c/l1> a sbol:Range ;
    sbol:displayId "l1" ;
    sbol:persistentIdentity <http://ex/c/l1> ;
    sbol:start 1 ;
    sbol:end 10 .
<http://ex/c/l2> a sbol:Range ;
    sbol:displayId "l2" ;
    sbol:persistentIdentity <http://ex/c/l2> ;
    sbol:start 5 ;
    sbol:end 15 .
"#,
        },
        RuleCase {
            name: "Component source length disagrees with its annotation length",
            rule: "sbol2-10712",
            severity: Warning,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/c> ;
    sbol:sequenceAnnotation <http://ex/sa> .
<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:sourceLocation <http://ex/c/src> .
<http://ex/c/src> a sbol:Range ;
    sbol:displayId "src" ;
    sbol:persistentIdentity <http://ex/c/src> ;
    sbol:start 1 ;
    sbol:end 10 .
<http://ex/sa> a sbol:SequenceAnnotation ;
    sbol:displayId "sa" ;
    sbol:persistentIdentity <http://ex/sa> ;
    sbol:component <http://ex/c> ;
    sbol:location <http://ex/sa/l> .
<http://ex/sa/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/sa/l> ;
    sbol:start 1 ;
    sbol:end 5 .
"#,
        },
        RuleCase {
            name: "SequenceAnnotation Locations overlap",
            rule: "sbol2-10903",
            severity: Warning,
            body: r#"<http://ex/sa> a sbol:SequenceAnnotation ;
    sbol:displayId "sa" ;
    sbol:persistentIdentity <http://ex/sa> ;
    sbol:location <http://ex/sa/l1>, <http://ex/sa/l2> .
<http://ex/sa/l1> a sbol:Range ;
    sbol:displayId "l1" ;
    sbol:persistentIdentity <http://ex/sa/l1> ;
    sbol:start 1 ;
    sbol:end 10 .
<http://ex/sa/l2> a sbol:Range ;
    sbol:displayId "l2" ;
    sbol:persistentIdentity <http://ex/sa/l2> ;
    sbol:start 5 ;
    sbol:end 15 .
"#,
        },
        RuleCase {
            name: "SequenceAnnotation Component is not in the containing ComponentDefinition",
            rule: "sbol2-10905",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:sequenceAnnotation <http://ex/cd/sa> .
<http://ex/cd/sa> a sbol:SequenceAnnotation ;
    sbol:displayId "sa" ;
    sbol:persistentIdentity <http://ex/cd/sa> ;
    sbol:component <http://ex/stray-component> ;
    sbol:location <http://ex/cd/sa/l> .
<http://ex/cd/sa/l> a sbol:GenericLocation ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa/l> .
<http://ex/stray-component> a sbol:Component ;
    sbol:displayId "stray" ;
    sbol:persistentIdentity <http://ex/stray-component> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> .
"#,
        },
        RuleCase {
            name: "SequenceAnnotation carries both a component and roles",
            rule: "sbol2-10909",
            severity: Error,
            body: r#"<http://ex/sa> a sbol:SequenceAnnotation ;
    sbol:displayId "sa" ;
    sbol:persistentIdentity <http://ex/sa> ;
    sbol:component <http://ex/c> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    sbol:location <http://ex/sa/l> .
<http://ex/sa/l> a sbol:GenericLocation ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/sa/l> .
"#,
        },
        RuleCase {
            name: "SequenceConstraint subject is not a Component of the ComponentDefinition",
            rule: "sbol2-11403",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/o> ;
    sbol:sequenceConstraint <http://ex/cd/sc> .
<http://ex/cd/o> a sbol:Component ;
    sbol:displayId "o" ;
    sbol:persistentIdentity <http://ex/cd/o> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/do> .
<http://ex/cd/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/cd/sc> ;
    sbol:restriction <http://sbols.org/v2#precedes> ;
    sbol:subject <http://ex/ghost> ;
    sbol:object <http://ex/cd/o> .
"#,
        },
        RuleCase {
            name: "SequenceConstraint object is not a Component of the ComponentDefinition",
            rule: "sbol2-11405",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/s> ;
    sbol:sequenceConstraint <http://ex/cd/sc> .
<http://ex/cd/s> a sbol:Component ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/cd/s> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/ds> .
<http://ex/cd/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/cd/sc> ;
    sbol:restriction <http://sbols.org/v2#precedes> ;
    sbol:subject <http://ex/cd/s> ;
    sbol:object <http://ex/ghost> .
"#,
        },
        RuleCase {
            name: "SequenceConstraint subject equals its object",
            rule: "sbol2-11406",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/s> ;
    sbol:sequenceConstraint <http://ex/cd/sc> .
<http://ex/cd/s> a sbol:Component ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/cd/s> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/ds> .
<http://ex/cd/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/cd/sc> ;
    sbol:restriction <http://sbols.org/v2#precedes> ;
    sbol:subject <http://ex/cd/s> ;
    sbol:object <http://ex/cd/s> .
"#,
        },
        RuleCase {
            name: "precedes SequenceConstraint with subject positioned after object",
            rule: "sbol2-11409",
            severity: Error,
            body: PRECEDES_VIOLATION,
        },
        RuleCase {
            name: "sameOrientationAs SequenceConstraint with differing orientations",
            rule: "sbol2-11410",
            severity: Error,
            body: SAME_ORIENTATION_VIOLATION,
        },
        RuleCase {
            name: "oppositeOrientationAs SequenceConstraint with matching orientations",
            rule: "sbol2-11411",
            severity: Error,
            body: OPPOSITE_ORIENTATION_VIOLATION,
        },
        RuleCase {
            name: "SequenceConstraint restriction is not a Table 7 URI",
            rule: "sbol2-11412",
            severity: Warning,
            body: r#"<http://ex/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/sc> ;
    sbol:restriction <http://example.org/custom-restriction> ;
    sbol:subject <http://ex/s> ;
    sbol:object <http://ex/o> .
"#,
        },
        RuleCase {
            name: "differentFrom SequenceConstraint whose Components share a definition URI",
            rule: "sbol2-11413",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/s>, <http://ex/cd/o> ;
    sbol:sequenceConstraint <http://ex/cd/sc> .
<http://ex/cd/s> a sbol:Component ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/cd/s> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/shared-def> .
<http://ex/cd/o> a sbol:Component ;
    sbol:displayId "o" ;
    sbol:persistentIdentity <http://ex/cd/o> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/shared-def> .
<http://ex/cd/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/cd/sc> ;
    sbol:restriction <http://sbols.org/v2#differentFrom> ;
    sbol:subject <http://ex/cd/s> ;
    sbol:object <http://ex/cd/o> .
"#,
        },
        RuleCase {
            name: "differentFrom SequenceConstraint whose Components resolve to one definition",
            rule: "sbol2-11414",
            severity: Error,
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/s>, <http://ex/cd/o> ;
    sbol:sequenceConstraint <http://ex/cd/sc> .
<http://ex/cd/s> a sbol:Component ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/cd/s> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/def> .
<http://ex/cd/o> a sbol:Component ;
    sbol:displayId "o" ;
    sbol:persistentIdentity <http://ex/cd/o> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/def/1> .
<http://ex/cd/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/cd/sc> ;
    sbol:restriction <http://sbols.org/v2#differentFrom> ;
    sbol:subject <http://ex/cd/s> ;
    sbol:object <http://ex/cd/o> .
<http://ex/def/1> a sbol:ComponentDefinition ;
    sbol:displayId "def" ;
    sbol:persistentIdentity <http://ex/def> ;
    sbol:version "1" ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
    ]
}

const PRECEDES_VIOLATION: &str = r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/s>, <http://ex/cd/o> ;
    sbol:sequenceAnnotation <http://ex/cd/sa_s>, <http://ex/cd/sa_o> ;
    sbol:sequenceConstraint <http://ex/cd/sc> .
<http://ex/cd/s> a sbol:Component ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/cd/s> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/ds> .
<http://ex/cd/o> a sbol:Component ;
    sbol:displayId "o" ;
    sbol:persistentIdentity <http://ex/cd/o> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/do> .
<http://ex/cd/sa_s> a sbol:SequenceAnnotation ;
    sbol:displayId "sa_s" ;
    sbol:persistentIdentity <http://ex/cd/sa_s> ;
    sbol:component <http://ex/cd/s> ;
    sbol:location <http://ex/cd/sa_s/l> .
<http://ex/cd/sa_s/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa_s/l> ;
    sbol:start 100 ; sbol:end 200 .
<http://ex/cd/sa_o> a sbol:SequenceAnnotation ;
    sbol:displayId "sa_o" ;
    sbol:persistentIdentity <http://ex/cd/sa_o> ;
    sbol:component <http://ex/cd/o> ;
    sbol:location <http://ex/cd/sa_o/l> .
<http://ex/cd/sa_o/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa_o/l> ;
    sbol:start 1 ; sbol:end 50 .
<http://ex/cd/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/cd/sc> ;
    sbol:restriction <http://sbols.org/v2#precedes> ;
    sbol:subject <http://ex/cd/s> ;
    sbol:object <http://ex/cd/o> .
"#;

const SAME_ORIENTATION_VIOLATION: &str = r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/s>, <http://ex/cd/o> ;
    sbol:sequenceAnnotation <http://ex/cd/sa_s>, <http://ex/cd/sa_o> ;
    sbol:sequenceConstraint <http://ex/cd/sc> .
<http://ex/cd/s> a sbol:Component ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/cd/s> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/ds> .
<http://ex/cd/o> a sbol:Component ;
    sbol:displayId "o" ;
    sbol:persistentIdentity <http://ex/cd/o> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/do> .
<http://ex/cd/sa_s> a sbol:SequenceAnnotation ;
    sbol:displayId "sa_s" ;
    sbol:persistentIdentity <http://ex/cd/sa_s> ;
    sbol:component <http://ex/cd/s> ;
    sbol:location <http://ex/cd/sa_s/l> .
<http://ex/cd/sa_s/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa_s/l> ;
    sbol:start 1 ; sbol:end 50 ;
    sbol:orientation <http://sbols.org/v2#inline> .
<http://ex/cd/sa_o> a sbol:SequenceAnnotation ;
    sbol:displayId "sa_o" ;
    sbol:persistentIdentity <http://ex/cd/sa_o> ;
    sbol:component <http://ex/cd/o> ;
    sbol:location <http://ex/cd/sa_o/l> .
<http://ex/cd/sa_o/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa_o/l> ;
    sbol:start 100 ; sbol:end 200 ;
    sbol:orientation <http://sbols.org/v2#reverseComplement> .
<http://ex/cd/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/cd/sc> ;
    sbol:restriction <http://sbols.org/v2#sameOrientationAs> ;
    sbol:subject <http://ex/cd/s> ;
    sbol:object <http://ex/cd/o> .
"#;

const OPPOSITE_ORIENTATION_VIOLATION: &str = r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/s>, <http://ex/cd/o> ;
    sbol:sequenceAnnotation <http://ex/cd/sa_s>, <http://ex/cd/sa_o> ;
    sbol:sequenceConstraint <http://ex/cd/sc> .
<http://ex/cd/s> a sbol:Component ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/cd/s> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/ds> .
<http://ex/cd/o> a sbol:Component ;
    sbol:displayId "o" ;
    sbol:persistentIdentity <http://ex/cd/o> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/do> .
<http://ex/cd/sa_s> a sbol:SequenceAnnotation ;
    sbol:displayId "sa_s" ;
    sbol:persistentIdentity <http://ex/cd/sa_s> ;
    sbol:component <http://ex/cd/s> ;
    sbol:location <http://ex/cd/sa_s/l> .
<http://ex/cd/sa_s/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa_s/l> ;
    sbol:start 1 ; sbol:end 50 ;
    sbol:orientation <http://sbols.org/v2#inline> .
<http://ex/cd/sa_o> a sbol:SequenceAnnotation ;
    sbol:displayId "sa_o" ;
    sbol:persistentIdentity <http://ex/cd/sa_o> ;
    sbol:component <http://ex/cd/o> ;
    sbol:location <http://ex/cd/sa_o/l> .
<http://ex/cd/sa_o/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa_o/l> ;
    sbol:start 100 ; sbol:end 200 ;
    sbol:orientation <http://sbols.org/v2#inline> .
<http://ex/cd/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/cd/sc> ;
    sbol:restriction <http://sbols.org/v2#oppositeOrientationAs> ;
    sbol:subject <http://ex/cd/s> ;
    sbol:object <http://ex/cd/o> .
"#;

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "ComponentDefinition with a single BioPAX type",
            rule: "sbol2-10503",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
        PositiveCase {
            name: "sequence-feature role on a DNA ComponentDefinition",
            rule: "sbol2-10511",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:role <http://identifiers.org/so/SO:0000167> .
"#,
        },
        PositiveCase {
            name: "ComponentDefinition sequence reference resolves",
            rule: "sbol2-10513",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:sequence <http://ex/s> .
<http://ex/s> a sbol:Sequence ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/s> ;
    sbol:elements "ACGT" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
"#,
        },
        PositiveCase {
            name: "DNA ComponentDefinition with a nucleic Sequence",
            rule: "sbol2-10516",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:sequence <http://ex/s> .
<http://ex/s> a sbol:Sequence ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/s> ;
    sbol:elements "ACGT" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
"#,
        },
        PositiveCase {
            name: "two nucleic Sequences of one ComponentDefinition share a length",
            rule: "sbol2-10518",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:sequence <http://ex/s1>, <http://ex/s2> .
<http://ex/s1> a sbol:Sequence ;
    sbol:displayId "s1" ;
    sbol:persistentIdentity <http://ex/s1> ;
    sbol:elements "ACGT" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
<http://ex/s2> a sbol:Sequence ;
    sbol:displayId "s2" ;
    sbol:persistentIdentity <http://ex/s2> ;
    sbol:elements "TGCA" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
"#,
        },
        PositiveCase {
            name: "sub-component Sequence length matches its annotated span",
            rule: "sbol2-10520",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:sequence <http://ex/parent-seq> ;
    sbol:component <http://ex/cd/c> ;
    sbol:sequenceAnnotation <http://ex/cd/sa> .
<http://ex/parent-seq> a sbol:Sequence ;
    sbol:displayId "parent" ;
    sbol:persistentIdentity <http://ex/parent-seq> ;
    sbol:elements "ACGTACGT" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
<http://ex/cd/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/cd/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/child> .
<http://ex/cd/sa> a sbol:SequenceAnnotation ;
    sbol:displayId "sa" ;
    sbol:persistentIdentity <http://ex/cd/sa> ;
    sbol:component <http://ex/cd/c> ;
    sbol:location <http://ex/cd/sa/l> .
<http://ex/cd/sa/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa/l> ;
    sbol:start 1 ;
    sbol:end 4 .
<http://ex/child> a sbol:ComponentDefinition ;
    sbol:displayId "child" ;
    sbol:persistentIdentity <http://ex/child> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:sequence <http://ex/child-seq> .
<http://ex/child-seq> a sbol:Sequence ;
    sbol:displayId "childseq" ;
    sbol:persistentIdentity <http://ex/child-seq> ;
    sbol:elements "ACGT" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
"#,
        },
        PositiveCase {
            name: "SequenceAnnotations refer to distinct Components",
            rule: "sbol2-10522",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/c1>, <http://ex/cd/c2> ;
    sbol:sequenceAnnotation <http://ex/cd/sa1>, <http://ex/cd/sa2> .
<http://ex/cd/c1> a sbol:Component ;
    sbol:displayId "c1" ;
    sbol:persistentIdentity <http://ex/cd/c1> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> .
<http://ex/cd/c2> a sbol:Component ;
    sbol:displayId "c2" ;
    sbol:persistentIdentity <http://ex/cd/c2> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> .
<http://ex/cd/sa1> a sbol:SequenceAnnotation ;
    sbol:displayId "sa1" ;
    sbol:persistentIdentity <http://ex/cd/sa1> ;
    sbol:component <http://ex/cd/c1> ;
    sbol:location <http://ex/cd/sa1/l> .
<http://ex/cd/sa1/l> a sbol:GenericLocation ;
    sbol:displayId "l1" ;
    sbol:persistentIdentity <http://ex/cd/sa1/l> .
<http://ex/cd/sa2> a sbol:SequenceAnnotation ;
    sbol:displayId "sa2" ;
    sbol:persistentIdentity <http://ex/cd/sa2> ;
    sbol:component <http://ex/cd/c2> ;
    sbol:location <http://ex/cd/sa2/l> .
<http://ex/cd/sa2/l> a sbol:GenericLocation ;
    sbol:displayId "l2" ;
    sbol:persistentIdentity <http://ex/cd/sa2/l> .
"#,
        },
        PositiveCase {
            name: "SequenceAnnotation position lies within the sequence",
            rule: "sbol2-10523",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:sequence <http://ex/s> ;
    sbol:sequenceAnnotation <http://ex/cd/sa> .
<http://ex/s> a sbol:Sequence ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/s> ;
    sbol:elements "ACGTACGT" ;
    sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .
<http://ex/cd/sa> a sbol:SequenceAnnotation ;
    sbol:displayId "sa" ;
    sbol:persistentIdentity <http://ex/cd/sa> ;
    sbol:location <http://ex/cd/sa/l> .
<http://ex/cd/sa/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa/l> ;
    sbol:start 1 ;
    sbol:end 4 .
"#,
        },
        PositiveCase {
            name: "ComponentDefinition with a BioPAX type",
            rule: "sbol2-10525",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
        PositiveCase {
            name: "useRemote MapsTos of a ComponentDefinition use distinct locals",
            rule: "sbol2-10526",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/l1>, <http://ex/cd/l2>, <http://ex/cd/c1>, <http://ex/cd/c2> .
<http://ex/cd/l1> a sbol:Component ;
    sbol:displayId "l1" ;
    sbol:persistentIdentity <http://ex/cd/l1> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> .
<http://ex/cd/l2> a sbol:Component ;
    sbol:displayId "l2" ;
    sbol:persistentIdentity <http://ex/cd/l2> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> .
<http://ex/cd/c1> a sbol:Component ;
    sbol:displayId "c1" ;
    sbol:persistentIdentity <http://ex/cd/c1> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:mapsTo <http://ex/cd/c1/m> .
<http://ex/cd/c1/m> a sbol:MapsTo ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/cd/c1/m> ;
    sbol:refinement <http://sbols.org/v2#useRemote> ;
    sbol:local <http://ex/cd/l1> ;
    sbol:remote <http://ex/d/r> .
<http://ex/cd/c2> a sbol:Component ;
    sbol:displayId "c2" ;
    sbol:persistentIdentity <http://ex/cd/c2> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:mapsTo <http://ex/cd/c2/m> .
<http://ex/cd/c2/m> a sbol:MapsTo ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/cd/c2/m> ;
    sbol:refinement <http://sbols.org/v2#useRemote> ;
    sbol:local <http://ex/cd/l2> ;
    sbol:remote <http://ex/d/r> .
"#,
        },
        PositiveCase {
            name: "DNA ComponentDefinition with exactly one sequence-feature role",
            rule: "sbol2-10527",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:role <http://identifiers.org/so/SO:0000167> .
"#,
        },
        PositiveCase {
            name: "DNA ComponentDefinition with a single topology type",
            rule: "sbol2-10528",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion>,
              <http://identifiers.org/so/SO:0000987> .
"#,
        },
        PositiveCase {
            name: "topology type accompanied by a DNA type",
            rule: "sbol2-10529",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion>,
              <http://identifiers.org/so/SO:0000987> .
"#,
        },
        PositiveCase {
            name: "Component definition refers to a distinct ComponentDefinition",
            rule: "sbol2-10603",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/c> .
<http://ex/cd/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/cd/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/other> .
"#,
        },
        PositiveCase {
            name: "Component definition resolves to a ComponentDefinition",
            rule: "sbol2-10604",
            body: r#"<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/cd> .
<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
        PositiveCase {
            name: "acyclic ComponentDefinition-Component hierarchy",
            rule: "sbol2-10605",
            body: r#"<http://ex/a> a sbol:ComponentDefinition ;
    sbol:displayId "a" ;
    sbol:persistentIdentity <http://ex/a> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/a/c> .
<http://ex/a/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/a/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/b> .
<http://ex/b> a sbol:ComponentDefinition ;
    sbol:displayId "b" ;
    sbol:persistentIdentity <http://ex/b> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
        PositiveCase {
            name: "Component measure resolves to an om:Measure",
            rule: "sbol2-10608",
            body: r#"<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:measure <http://ex/m> .
<http://ex/m> a om:Measure ;
    sbol:displayId "m" ;
    sbol:persistentIdentity <http://ex/m> ;
    om:hasNumericalValue 1.0 ;
    om:hasUnit <http://ex/unit> .
"#,
        },
        PositiveCase {
            name: "Component without a sequence-feature role for a non-DNA definition",
            rule: "sbol2-10706",
            body: r#"<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/prot> .
<http://ex/prot> a sbol:ComponentDefinition ;
    sbol:displayId "prot" ;
    sbol:persistentIdentity <http://ex/prot> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#Protein> .
"#,
        },
        PositiveCase {
            name: "DNA Component with a single sequence-feature role",
            rule: "sbol2-10707",
            body: r#"<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/dna> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    sbol:roleIntegration <http://sbols.org/v2#mergeRoles> .
<http://ex/dna> a sbol:ComponentDefinition ;
    sbol:displayId "dna" ;
    sbol:persistentIdentity <http://ex/dna> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#,
        },
        PositiveCase {
            name: "Component with a role also carries a roleIntegration",
            rule: "sbol2-10709",
            body: r#"<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:role <http://identifiers.org/so/SO:0000167> ;
    sbol:roleIntegration <http://sbols.org/v2#mergeRoles> .
"#,
        },
        PositiveCase {
            name: "Component source Locations do not overlap",
            rule: "sbol2-10711",
            body: r#"<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:sourceLocation <http://ex/c/l1>, <http://ex/c/l2> .
<http://ex/c/l1> a sbol:Range ;
    sbol:displayId "l1" ;
    sbol:persistentIdentity <http://ex/c/l1> ;
    sbol:start 1 ;
    sbol:end 10 .
<http://ex/c/l2> a sbol:Range ;
    sbol:displayId "l2" ;
    sbol:persistentIdentity <http://ex/c/l2> ;
    sbol:start 11 ;
    sbol:end 20 .
"#,
        },
        PositiveCase {
            name: "Component source length matches its annotation length",
            rule: "sbol2-10712",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/c> ;
    sbol:sequenceAnnotation <http://ex/sa> .
<http://ex/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> ;
    sbol:sourceLocation <http://ex/c/src> .
<http://ex/c/src> a sbol:Range ;
    sbol:displayId "src" ;
    sbol:persistentIdentity <http://ex/c/src> ;
    sbol:start 1 ;
    sbol:end 10 .
<http://ex/sa> a sbol:SequenceAnnotation ;
    sbol:displayId "sa" ;
    sbol:persistentIdentity <http://ex/sa> ;
    sbol:component <http://ex/c> ;
    sbol:location <http://ex/sa/l> .
<http://ex/sa/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/sa/l> ;
    sbol:start 1 ;
    sbol:end 10 .
"#,
        },
        PositiveCase {
            name: "SequenceAnnotation Locations do not overlap",
            rule: "sbol2-10903",
            body: r#"<http://ex/sa> a sbol:SequenceAnnotation ;
    sbol:displayId "sa" ;
    sbol:persistentIdentity <http://ex/sa> ;
    sbol:location <http://ex/sa/l1>, <http://ex/sa/l2> .
<http://ex/sa/l1> a sbol:Range ;
    sbol:displayId "l1" ;
    sbol:persistentIdentity <http://ex/sa/l1> ;
    sbol:start 1 ;
    sbol:end 10 .
<http://ex/sa/l2> a sbol:Range ;
    sbol:displayId "l2" ;
    sbol:persistentIdentity <http://ex/sa/l2> ;
    sbol:start 11 ;
    sbol:end 20 .
"#,
        },
        PositiveCase {
            name: "SequenceAnnotation Component belongs to the containing ComponentDefinition",
            rule: "sbol2-10905",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/c> ;
    sbol:sequenceAnnotation <http://ex/cd/sa> .
<http://ex/cd/c> a sbol:Component ;
    sbol:displayId "c" ;
    sbol:persistentIdentity <http://ex/cd/c> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/d> .
<http://ex/cd/sa> a sbol:SequenceAnnotation ;
    sbol:displayId "sa" ;
    sbol:persistentIdentity <http://ex/cd/sa> ;
    sbol:component <http://ex/cd/c> ;
    sbol:location <http://ex/cd/sa/l> .
<http://ex/cd/sa/l> a sbol:GenericLocation ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa/l> .
"#,
        },
        PositiveCase {
            name: "SequenceAnnotation with a component but no roles",
            rule: "sbol2-10909",
            body: r#"<http://ex/sa> a sbol:SequenceAnnotation ;
    sbol:displayId "sa" ;
    sbol:persistentIdentity <http://ex/sa> ;
    sbol:component <http://ex/c> ;
    sbol:location <http://ex/sa/l> .
<http://ex/sa/l> a sbol:GenericLocation ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/sa/l> .
"#,
        },
        PositiveCase {
            name: "SequenceConstraint subject is a Component of the ComponentDefinition",
            rule: "sbol2-11403",
            body: SC_VALID,
        },
        PositiveCase {
            name: "SequenceConstraint object is a Component of the ComponentDefinition",
            rule: "sbol2-11405",
            body: SC_VALID,
        },
        PositiveCase {
            name: "SequenceConstraint subject differs from its object",
            rule: "sbol2-11406",
            body: SC_VALID,
        },
        PositiveCase {
            name: "precedes SequenceConstraint with subject before object",
            rule: "sbol2-11409",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/s>, <http://ex/cd/o> ;
    sbol:sequenceAnnotation <http://ex/cd/sa_s>, <http://ex/cd/sa_o> ;
    sbol:sequenceConstraint <http://ex/cd/sc> .
<http://ex/cd/s> a sbol:Component ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/cd/s> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/ds> .
<http://ex/cd/o> a sbol:Component ;
    sbol:displayId "o" ;
    sbol:persistentIdentity <http://ex/cd/o> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/do> .
<http://ex/cd/sa_s> a sbol:SequenceAnnotation ;
    sbol:displayId "sa_s" ;
    sbol:persistentIdentity <http://ex/cd/sa_s> ;
    sbol:component <http://ex/cd/s> ;
    sbol:location <http://ex/cd/sa_s/l> .
<http://ex/cd/sa_s/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa_s/l> ;
    sbol:start 1 ; sbol:end 50 .
<http://ex/cd/sa_o> a sbol:SequenceAnnotation ;
    sbol:displayId "sa_o" ;
    sbol:persistentIdentity <http://ex/cd/sa_o> ;
    sbol:component <http://ex/cd/o> ;
    sbol:location <http://ex/cd/sa_o/l> .
<http://ex/cd/sa_o/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa_o/l> ;
    sbol:start 100 ; sbol:end 200 .
<http://ex/cd/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/cd/sc> ;
    sbol:restriction <http://sbols.org/v2#precedes> ;
    sbol:subject <http://ex/cd/s> ;
    sbol:object <http://ex/cd/o> .
"#,
        },
        PositiveCase {
            name: "sameOrientationAs SequenceConstraint with matching orientations",
            rule: "sbol2-11410",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/s>, <http://ex/cd/o> ;
    sbol:sequenceAnnotation <http://ex/cd/sa_s>, <http://ex/cd/sa_o> ;
    sbol:sequenceConstraint <http://ex/cd/sc> .
<http://ex/cd/s> a sbol:Component ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/cd/s> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/ds> .
<http://ex/cd/o> a sbol:Component ;
    sbol:displayId "o" ;
    sbol:persistentIdentity <http://ex/cd/o> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/do> .
<http://ex/cd/sa_s> a sbol:SequenceAnnotation ;
    sbol:displayId "sa_s" ;
    sbol:persistentIdentity <http://ex/cd/sa_s> ;
    sbol:component <http://ex/cd/s> ;
    sbol:location <http://ex/cd/sa_s/l> .
<http://ex/cd/sa_s/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa_s/l> ;
    sbol:start 1 ; sbol:end 50 ;
    sbol:orientation <http://sbols.org/v2#inline> .
<http://ex/cd/sa_o> a sbol:SequenceAnnotation ;
    sbol:displayId "sa_o" ;
    sbol:persistentIdentity <http://ex/cd/sa_o> ;
    sbol:component <http://ex/cd/o> ;
    sbol:location <http://ex/cd/sa_o/l> .
<http://ex/cd/sa_o/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa_o/l> ;
    sbol:start 100 ; sbol:end 200 ;
    sbol:orientation <http://sbols.org/v2#inline> .
<http://ex/cd/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/cd/sc> ;
    sbol:restriction <http://sbols.org/v2#sameOrientationAs> ;
    sbol:subject <http://ex/cd/s> ;
    sbol:object <http://ex/cd/o> .
"#,
        },
        PositiveCase {
            name: "oppositeOrientationAs SequenceConstraint with differing orientations",
            rule: "sbol2-11411",
            body: r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/s>, <http://ex/cd/o> ;
    sbol:sequenceAnnotation <http://ex/cd/sa_s>, <http://ex/cd/sa_o> ;
    sbol:sequenceConstraint <http://ex/cd/sc> .
<http://ex/cd/s> a sbol:Component ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/cd/s> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/ds> .
<http://ex/cd/o> a sbol:Component ;
    sbol:displayId "o" ;
    sbol:persistentIdentity <http://ex/cd/o> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/do> .
<http://ex/cd/sa_s> a sbol:SequenceAnnotation ;
    sbol:displayId "sa_s" ;
    sbol:persistentIdentity <http://ex/cd/sa_s> ;
    sbol:component <http://ex/cd/s> ;
    sbol:location <http://ex/cd/sa_s/l> .
<http://ex/cd/sa_s/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa_s/l> ;
    sbol:start 1 ; sbol:end 50 ;
    sbol:orientation <http://sbols.org/v2#inline> .
<http://ex/cd/sa_o> a sbol:SequenceAnnotation ;
    sbol:displayId "sa_o" ;
    sbol:persistentIdentity <http://ex/cd/sa_o> ;
    sbol:component <http://ex/cd/o> ;
    sbol:location <http://ex/cd/sa_o/l> .
<http://ex/cd/sa_o/l> a sbol:Range ;
    sbol:displayId "l" ;
    sbol:persistentIdentity <http://ex/cd/sa_o/l> ;
    sbol:start 100 ; sbol:end 200 ;
    sbol:orientation <http://sbols.org/v2#reverseComplement> .
<http://ex/cd/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/cd/sc> ;
    sbol:restriction <http://sbols.org/v2#oppositeOrientationAs> ;
    sbol:subject <http://ex/cd/s> ;
    sbol:object <http://ex/cd/o> .
"#,
        },
        PositiveCase {
            name: "SequenceConstraint restriction is a Table 7 URI",
            rule: "sbol2-11412",
            body: SC_VALID,
        },
        PositiveCase {
            name: "differentFrom SequenceConstraint whose Components have distinct definitions",
            rule: "sbol2-11413",
            body: DIFFERENT_FROM_VALID,
        },
        PositiveCase {
            name: "differentFrom SequenceConstraint whose Components resolve to distinct definitions",
            rule: "sbol2-11414",
            body: DIFFERENT_FROM_VALID,
        },
    ]
}

const SC_VALID: &str = r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/s>, <http://ex/cd/o> ;
    sbol:sequenceConstraint <http://ex/cd/sc> .
<http://ex/cd/s> a sbol:Component ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/cd/s> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/ds> .
<http://ex/cd/o> a sbol:Component ;
    sbol:displayId "o" ;
    sbol:persistentIdentity <http://ex/cd/o> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/do> .
<http://ex/cd/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/cd/sc> ;
    sbol:restriction <http://sbols.org/v2#precedes> ;
    sbol:subject <http://ex/cd/s> ;
    sbol:object <http://ex/cd/o> .
"#;

const DIFFERENT_FROM_VALID: &str = r#"<http://ex/cd> a sbol:ComponentDefinition ;
    sbol:displayId "cd" ;
    sbol:persistentIdentity <http://ex/cd> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;
    sbol:component <http://ex/cd/s>, <http://ex/cd/o> ;
    sbol:sequenceConstraint <http://ex/cd/sc> .
<http://ex/cd/s> a sbol:Component ;
    sbol:displayId "s" ;
    sbol:persistentIdentity <http://ex/cd/s> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/ds> .
<http://ex/cd/o> a sbol:Component ;
    sbol:displayId "o" ;
    sbol:persistentIdentity <http://ex/cd/o> ;
    sbol:access <http://sbols.org/v2#public> ;
    sbol:definition <http://ex/do> .
<http://ex/cd/sc> a sbol:SequenceConstraint ;
    sbol:displayId "sc" ;
    sbol:persistentIdentity <http://ex/cd/sc> ;
    sbol:restriction <http://sbols.org/v2#differentFrom> ;
    sbol:subject <http://ex/cd/s> ;
    sbol:object <http://ex/cd/o> .
<http://ex/ds> a sbol:ComponentDefinition ;
    sbol:displayId "ds" ;
    sbol:persistentIdentity <http://ex/ds> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
<http://ex/do> a sbol:ComponentDefinition ;
    sbol:displayId "do" ;
    sbol:persistentIdentity <http://ex/do> ;
    sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> .
"#;
