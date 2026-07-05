//! Hermetic per-rule cases: minimal in-memory SBOL 2 documents that isolate a
//! single rule, with a positive fixture that must not report it and a negative
//! fixture that must. These are fast and require no corpus download; they pin
//! the behavior of each implemented rule family.

use sbol2::validation::{ValidationConfig, ValidationReport};
use sbol2::{Document, RdfFormat};

fn doc(turtle: &str) -> Document {
    Document::read(turtle, RdfFormat::Turtle)
        .unwrap_or_else(|error| panic!("fixture failed to parse: {error}\n{turtle}"))
}

fn reports_rule(report: &ValidationReport, rule: &str) -> bool {
    report.errors().any(|issue| issue.rule == rule)
}

const PREAMBLE: &str = "@prefix sbol: <http://sbols.org/v2#> .\n\
     @prefix prov: <http://www.w3.org/ns/prov#> .\n\
     @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n";

// 10402: a Sequence requires exactly one elements value.
#[test]
fn sequence_elements_cardinality_10402() {
    let valid = doc(&format!(
        "{PREAMBLE}\
         <http://ex/s1> a sbol:Sequence ;\n\
           sbol:displayId \"s1\" ;\n\
           sbol:persistentIdentity <http://ex/s1> ;\n\
           sbol:elements \"ATCG\" ;\n\
           sbol:encoding <http://sbols.org/v2#IUPACDNA> .\n"
    ));
    assert!(!reports_rule(&valid.validate(), "sbol2-10402"));

    let invalid = doc(&format!(
        "{PREAMBLE}\
         <http://ex/s1> a sbol:Sequence ;\n\
           sbol:displayId \"s1\" ;\n\
           sbol:persistentIdentity <http://ex/s1> ;\n\
           sbol:encoding <http://sbols.org/v2#IUPACDNA> .\n"
    ));
    assert!(reports_rule(&invalid.validate(), "sbol2-10402"));
}

// 10204: displayId must not begin with a digit.
#[test]
fn display_id_syntax_10204() {
    let invalid = doc(&format!(
        "{PREAMBLE}\
         <http://ex/9bad> a sbol:Sequence ;\n\
           sbol:displayId \"9bad\" ;\n\
           sbol:persistentIdentity <http://ex/9bad> ;\n\
           sbol:elements \"A\" ;\n\
           sbol:encoding <http://sbols.org/v2#IUPACDNA> .\n"
    ));
    // A leading-digit displayId is caught regardless of the compliant flag.
    let report = invalid.validate_with_config(&ValidationConfig::default().with_compliant(false));
    assert!(reports_rule(&report, "sbol2-10204"));
}

// 10216: a compliant TopLevel persistentIdentity must end with a delimiter
// followed by its displayId. Gated on the compliant family.
#[test]
fn compliant_persistent_identity_10216() {
    let turtle = format!(
        "{PREAMBLE}\
         <http://ex/s1> a sbol:Sequence ;\n\
           sbol:displayId \"s1\" ;\n\
           sbol:persistentIdentity <http://ex/mismatch> ;\n\
           sbol:elements \"A\" ;\n\
           sbol:encoding <http://sbols.org/v2#IUPACDNA> .\n"
    );
    let document = doc(&turtle);
    // Off when the compliant family is disabled.
    let off = document.validate_with_config(&ValidationConfig::default().with_compliant(false));
    assert!(!reports_rule(&off, "sbol2-10216"));
    // On under the default (compliant) config.
    assert!(reports_rule(&document.validate(), "sbol2-10216"));
}

// 10217: a compliant child object's persistentIdentity must extend its
// parent's. Gated on the compliant family.
#[test]
fn compliant_child_persistent_identity_10217() {
    let turtle = format!(
        "{PREAMBLE}\
         <http://ex/cd/1> a sbol:ComponentDefinition ;\n\
           sbol:displayId \"cd\" ;\n\
           sbol:persistentIdentity <http://ex/cd> ;\n\
           sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;\n\
           sbol:component <http://ex/cd/c/1> .\n\
         <http://ex/cd/c/1> a sbol:Component ;\n\
           sbol:displayId \"c\" ;\n\
           sbol:persistentIdentity <http://ex/wrong/c> ;\n\
           sbol:access <http://sbols.org/v2#public> ;\n\
           sbol:definition <http://ex/other> .\n"
    );
    let document = doc(&turtle);
    let off = document.validate_with_config(&ValidationConfig::default().with_compliant(false));
    assert!(!reports_rule(&off, "sbol2-10217"));
    assert!(reports_rule(&document.validate(), "sbol2-10217"));
}

// 10604: a FunctionalComponent definition must resolve in a complete document.
// Gated on the completeness family.
#[test]
fn completeness_definition_10604() {
    let turtle = format!(
        "{PREAMBLE}\
         <http://ex/md/1> a sbol:ModuleDefinition ;\n\
           sbol:displayId \"md\" ;\n\
           sbol:persistentIdentity <http://ex/md> ;\n\
           sbol:functionalComponent <http://ex/md/fc/1> .\n\
         <http://ex/md/fc/1> a sbol:FunctionalComponent ;\n\
           sbol:displayId \"fc\" ;\n\
           sbol:persistentIdentity <http://ex/md/fc> ;\n\
           sbol:access <http://sbols.org/v2#public> ;\n\
           sbol:direction <http://sbols.org/v2#inout> ;\n\
           sbol:definition <http://ex/absent_cd> .\n"
    );
    let document = doc(&turtle);
    let off = document.validate_with_config(&ValidationConfig::default().with_complete(false));
    assert!(!reports_rule(&off, "sbol2-10604"));
    assert!(reports_rule(&document.validate(), "sbol2-10604"));
}

// 10501: a ComponentDefinition must not carry an unrecognized SBOL property.
#[test]
fn closed_property_set_10501() {
    let invalid = doc(&format!(
        "{PREAMBLE}\
         <http://ex/cd/1> a sbol:ComponentDefinition ;\n\
           sbol:displayId \"cd\" ;\n\
           sbol:persistentIdentity <http://ex/cd> ;\n\
           sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;\n\
           sbol:bogus \"x\" .\n"
    ));
    assert!(reports_rule(&invalid.validate(), "sbol2-10501"));
}

// 11104: a Range end must be greater than or equal to its start.
#[test]
fn range_bounds_11104() {
    let invalid = doc(&format!(
        "{PREAMBLE}\
         <http://ex/r/1> a sbol:Range ;\n\
           sbol:displayId \"r\" ;\n\
           sbol:persistentIdentity <http://ex/r> ;\n\
           sbol:start 10 ;\n\
           sbol:end 5 .\n"
    ));
    assert!(reports_rule(&invalid.validate(), "sbol2-11104"));
}

// 10303: a TopLevel must not derive from itself.
#[test]
fn self_derivation_10303() {
    let invalid = doc(&format!(
        "{PREAMBLE}\
         <http://ex/cd/1> a sbol:ComponentDefinition ;\n\
           sbol:displayId \"cd\" ;\n\
           sbol:persistentIdentity <http://ex/cd> ;\n\
           sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;\n\
           prov:wasDerivedFrom <http://ex/cd/1> .\n"
    ));
    assert!(reports_rule(&invalid.validate(), "sbol2-10303"));
}

// 10503: a ComponentDefinition must not contain more than one Table 2 type.
#[test]
fn multiple_biopax_types_10503() {
    let invalid = doc(&format!(
        "{PREAMBLE}\
         <http://ex/cd/1> a sbol:ComponentDefinition ;\n\
           sbol:displayId \"cd\" ;\n\
           sbol:persistentIdentity <http://ex/cd> ;\n\
           sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;\n\
           sbol:type <http://www.biopax.org/release/biopax-level3.owl#Protein> .\n"
    ));
    let on = invalid.validate_with_config(&ValidationConfig::default().with_best_practice(true));
    assert!(reports_rule(&on, "sbol2-10503"));
    // Off under the default (best-practice off) config.
    assert!(!reports_rule(&invalid.validate(), "sbol2-10503"));
}

// 11905: an Interaction should carry exactly one occurring-entity SBO type.
// Best-practice recommendation, emitted at warning severity.
#[test]
fn interaction_sbo_type_11905() {
    let turtle = format!(
        "{PREAMBLE}\
         <http://ex/i/1> a sbol:Interaction ;\n\
           sbol:displayId \"i\" ;\n\
           sbol:persistentIdentity <http://ex/i> ;\n\
           sbol:type <http://identifiers.org/so/SO:0000167> .\n"
    );
    let document = doc(&turtle);
    let on = document.validate_with_config(&ValidationConfig::default().with_best_practice(true));
    assert!(on.warnings().any(|issue| issue.rule == "sbol2-11905"));
    // Silent when best-practice checking is off.
    assert!(!document.validate().warnings().any(|i| i.rule == "sbol2-11905"));
}

// 10225: an object generated by a build Activity should be an Implementation.
// Gated on the best-practice family.
#[test]
fn provenance_build_role_10225() {
    let turtle = format!(
        "{PREAMBLE}\
         <http://ex/cd/1> a sbol:ComponentDefinition ;\n\
           sbol:displayId \"cd\" ;\n\
           sbol:persistentIdentity <http://ex/cd> ;\n\
           sbol:type <http://www.biopax.org/release/biopax-level3.owl#DnaRegion> ;\n\
           prov:wasGeneratedBy <http://ex/act/1> .\n\
         <http://ex/act/1> a prov:Activity ;\n\
           sbol:displayId \"act\" ;\n\
           sbol:persistentIdentity <http://ex/act> ;\n\
           prov:qualifiedAssociation <http://ex/act/assoc/1> .\n\
         <http://ex/act/assoc/1> a prov:Association ;\n\
           sbol:displayId \"assoc\" ;\n\
           sbol:persistentIdentity <http://ex/act/assoc> ;\n\
           prov:agent <http://ex/agent> ;\n\
           prov:hadRole <http://sbols.org/v2#build> .\n"
    );
    let document = doc(&turtle);
    // Off when best-practice checking is disabled (the default).
    assert!(!reports_rule(&document.validate(), "sbol2-10225"));
    // On when best-practice checking is enabled.
    let on = document.validate_with_config(&ValidationConfig::default().with_best_practice(true));
    assert!(reports_rule(&on, "sbol2-10225"));
}

const DNA_REGION: &str = "<http://www.biopax.org/release/biopax-level3.owl#DnaRegion>";

fn warns(report: &ValidationReport, rule: &str) -> bool {
    report.warnings().any(|issue| issue.rule == rule)
}

fn all_on() -> ValidationConfig {
    ValidationConfig::all_on()
}

// 10101: a document whose SBOL prefix is misbound uses no SBOL 2 namespace term.
#[test]
fn document_namespace_10101() {
    // Every type and property lands in a look-alike namespace, so the document
    // declares no SBOL 2 class or property.
    let misbound = "@prefix wrong: <http://not-sbols.example/v2#> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         <http://ex/cd> a wrong:ComponentDefinition ;\n\
           wrong:displayId \"cd\" .\n";
    let document = doc(misbound);
    assert!(reports_rule(&document.validate(), "sbol2-10101"));

    // A genuine SBOL 2 document names SBOL 2 properties.
    let valid = doc(&format!(
        "{PREAMBLE}\
         <http://ex/cd> a sbol:ComponentDefinition ;\n\
           sbol:displayId \"cd\" ;\n\
           sbol:persistentIdentity <http://ex/cd> ;\n\
           sbol:type {DNA_REGION} .\n"
    ));
    assert!(!reports_rule(&valid.validate(), "sbol2-10101"));
}

// 10405: a Sequence's elements must be consistent with its encoding.
#[test]
fn sequence_encoding_10405() {
    let invalid = doc(&format!(
        "{PREAMBLE}\
         <http://ex/s> a sbol:Sequence ;\n\
           sbol:displayId \"s\" ;\n\
           sbol:persistentIdentity <http://ex/s> ;\n\
           sbol:elements \"ACGTFZ\" ;\n\
           sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .\n"
    ));
    assert!(reports_rule(&invalid.validate(), "sbol2-10405"));

    let valid = doc(&format!(
        "{PREAMBLE}\
         <http://ex/s> a sbol:Sequence ;\n\
           sbol:displayId \"s\" ;\n\
           sbol:persistentIdentity <http://ex/s> ;\n\
           sbol:elements \"ACGTUN\" ;\n\
           sbol:encoding <http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html> .\n"
    ));
    assert!(!reports_rule(&valid.validate(), "sbol2-10405"));
}

// 10605: a ComponentDefinition must not cycle through its Components' definitions.
#[test]
fn component_definition_cycle_10605() {
    let turtle = format!(
        "{PREAMBLE}\
         <http://ex/a> a sbol:ComponentDefinition ; sbol:displayId \"a\" ;\n\
           sbol:persistentIdentity <http://ex/a> ; sbol:type {DNA_REGION} ;\n\
           sbol:component <http://ex/a/c> .\n\
         <http://ex/a/c> a sbol:Component ; sbol:displayId \"c\" ;\n\
           sbol:persistentIdentity <http://ex/a/c> ; sbol:access <http://sbols.org/v2#public> ;\n\
           sbol:definition <http://ex/b> .\n\
         <http://ex/b> a sbol:ComponentDefinition ; sbol:displayId \"b\" ;\n\
           sbol:persistentIdentity <http://ex/b> ; sbol:type {DNA_REGION} ;\n\
           sbol:component <http://ex/b/c> .\n\
         <http://ex/b/c> a sbol:Component ; sbol:displayId \"c\" ;\n\
           sbol:persistentIdentity <http://ex/b/c> ; sbol:access <http://sbols.org/v2#public> ;\n\
           sbol:definition <http://ex/a> .\n"
    );
    assert!(reports_rule(&doc(&turtle).validate(), "sbol2-10605"));
}

// 13015: a CombinatorialDerivation must not cycle through variantDerivations.
#[test]
fn combinatorial_derivation_cycle_13015() {
    let turtle = format!(
        "{PREAMBLE}\
         <http://ex/cda> a sbol:CombinatorialDerivation ; sbol:displayId \"cda\" ;\n\
           sbol:persistentIdentity <http://ex/cda> ; sbol:template <http://ex/t> ;\n\
           sbol:variableComponent <http://ex/cda/v> .\n\
         <http://ex/cda/v> a sbol:VariableComponent ; sbol:displayId \"v\" ;\n\
           sbol:persistentIdentity <http://ex/cda/v> ; sbol:variantDerivation <http://ex/cdb> .\n\
         <http://ex/cdb> a sbol:CombinatorialDerivation ; sbol:displayId \"cdb\" ;\n\
           sbol:persistentIdentity <http://ex/cdb> ; sbol:template <http://ex/t> ;\n\
           sbol:variableComponent <http://ex/cdb/v> .\n\
         <http://ex/cdb/v> a sbol:VariableComponent ; sbol:displayId \"v\" ;\n\
           sbol:persistentIdentity <http://ex/cdb/v> ; sbol:variantDerivation <http://ex/cda> .\n\
         <http://ex/t> a sbol:ComponentDefinition ; sbol:displayId \"t\" ;\n\
           sbol:persistentIdentity <http://ex/t> ; sbol:type {DNA_REGION} .\n"
    );
    assert!(reports_rule(&doc(&turtle).validate(), "sbol2-13015"));
}

// A ComponentDefinition holding two annotated Components under a SequenceConstraint.
fn constraint_cd(restriction: &str, subject_end: u32, subject_orientation: &str,
                 object_orientation: &str) -> String {
    format!(
        "{PREAMBLE}\
         <http://ex/cd> a sbol:ComponentDefinition ; sbol:displayId \"cd\" ;\n\
           sbol:persistentIdentity <http://ex/cd> ; sbol:type {DNA_REGION} ;\n\
           sbol:component <http://ex/cd/s> , <http://ex/cd/o> ;\n\
           sbol:sequenceAnnotation <http://ex/cd/sa_s> , <http://ex/cd/sa_o> ;\n\
           sbol:sequenceConstraint <http://ex/cd/sc> .\n\
         <http://ex/cd/s> a sbol:Component ; sbol:displayId \"s\" ;\n\
           sbol:persistentIdentity <http://ex/cd/s> ; sbol:access <http://sbols.org/v2#public> ;\n\
           sbol:definition <http://ex/ds> .\n\
         <http://ex/cd/o> a sbol:Component ; sbol:displayId \"o\" ;\n\
           sbol:persistentIdentity <http://ex/cd/o> ; sbol:access <http://sbols.org/v2#public> ;\n\
           sbol:definition <http://ex/do> .\n\
         <http://ex/cd/sa_s> a sbol:SequenceAnnotation ; sbol:displayId \"sa_s\" ;\n\
           sbol:persistentIdentity <http://ex/cd/sa_s> ; sbol:component <http://ex/cd/s> ;\n\
           sbol:location <http://ex/cd/sa_s/l> .\n\
         <http://ex/cd/sa_s/l> a sbol:Range ; sbol:displayId \"l\" ;\n\
           sbol:persistentIdentity <http://ex/cd/sa_s/l> ; sbol:start 100 ; sbol:end {subject_end} ;\n\
           sbol:orientation <{subject_orientation}> .\n\
         <http://ex/cd/sa_o> a sbol:SequenceAnnotation ; sbol:displayId \"sa_o\" ;\n\
           sbol:persistentIdentity <http://ex/cd/sa_o> ; sbol:component <http://ex/cd/o> ;\n\
           sbol:location <http://ex/cd/sa_o/l> .\n\
         <http://ex/cd/sa_o/l> a sbol:Range ; sbol:displayId \"l\" ;\n\
           sbol:persistentIdentity <http://ex/cd/sa_o/l> ; sbol:start 1 ; sbol:end 50 ;\n\
           sbol:orientation <{object_orientation}> .\n\
         <http://ex/cd/sc> a sbol:SequenceConstraint ; sbol:displayId \"sc\" ;\n\
           sbol:persistentIdentity <http://ex/cd/sc> ; sbol:restriction <{restriction}> ;\n\
           sbol:subject <http://ex/cd/s> ; sbol:object <http://ex/cd/o> .\n\
         <http://ex/ds> a sbol:ComponentDefinition ; sbol:displayId \"ds\" ;\n\
           sbol:persistentIdentity <http://ex/ds> ; sbol:type {DNA_REGION} .\n\
         <http://ex/do> a sbol:ComponentDefinition ; sbol:displayId \"do\" ;\n\
           sbol:persistentIdentity <http://ex/do> ; sbol:type {DNA_REGION} .\n",
        subject_orientation = subject_orientation,
        object_orientation = object_orientation,
    )
}

// 11409: a precedes constraint whose subject is positioned after its object.
#[test]
fn sequence_constraint_precedes_11409() {
    let inline = "http://sbols.org/v2#inline";
    let invalid = constraint_cd("http://sbols.org/v2#precedes", 200, inline, inline);
    assert!(reports_rule(&doc(&invalid).validate(), "sbol2-11409"));
}

// 11410: a sameOrientationAs constraint whose annotations differ in orientation.
#[test]
fn sequence_constraint_same_orientation_11410() {
    let invalid = constraint_cd(
        "http://sbols.org/v2#sameOrientationAs",
        200,
        "http://sbols.org/v2#inline",
        "http://sbols.org/v2#reverseComplement",
    );
    assert!(reports_rule(&doc(&invalid).validate(), "sbol2-11410"));
}

// 11411: an oppositeOrientationAs constraint whose annotations share orientation.
#[test]
fn sequence_constraint_opposite_orientation_11411() {
    let inline = "http://sbols.org/v2#inline";
    let invalid = constraint_cd("http://sbols.org/v2#oppositeOrientationAs", 200, inline, inline);
    assert!(reports_rule(&doc(&invalid).validate(), "sbol2-11411"));
}

// 11414: a differentFrom constraint whose Components resolve to one definition.
#[test]
fn sequence_constraint_11414() {
    // Subject's definition names the CD by persistentIdentity, object's by
    // identity: both resolve to the one definition <http://ex/def/1>.
    let turtle = format!(
        "{PREAMBLE}\
         <http://ex/cd> a sbol:ComponentDefinition ; sbol:displayId \"cd\" ;\n\
           sbol:persistentIdentity <http://ex/cd> ; sbol:type {DNA_REGION} ;\n\
           sbol:component <http://ex/cd/s> , <http://ex/cd/o> ;\n\
           sbol:sequenceConstraint <http://ex/cd/sc> .\n\
         <http://ex/cd/s> a sbol:Component ; sbol:displayId \"s\" ;\n\
           sbol:persistentIdentity <http://ex/cd/s> ; sbol:access <http://sbols.org/v2#public> ;\n\
           sbol:definition <http://ex/def> .\n\
         <http://ex/cd/o> a sbol:Component ; sbol:displayId \"o\" ;\n\
           sbol:persistentIdentity <http://ex/cd/o> ; sbol:access <http://sbols.org/v2#public> ;\n\
           sbol:definition <http://ex/def/1> .\n\
         <http://ex/cd/sc> a sbol:SequenceConstraint ; sbol:displayId \"sc\" ;\n\
           sbol:persistentIdentity <http://ex/cd/sc> ;\n\
           sbol:restriction <http://sbols.org/v2#differentFrom> ;\n\
           sbol:subject <http://ex/cd/s> ; sbol:object <http://ex/cd/o> .\n\
         <http://ex/def/1> a sbol:ComponentDefinition ; sbol:displayId \"def\" ;\n\
           sbol:persistentIdentity <http://ex/def> ; sbol:version \"1\" ; sbol:type {DNA_REGION} .\n"
    );
    assert!(reports_rule(&doc(&turtle).validate(), "sbol2-11414"));
}

// 10526: two useRemote MapsTos of a ComponentDefinition sharing a local.
#[test]
fn maps_to_use_remote_uniqueness_10526() {
    let turtle = format!(
        "{PREAMBLE}\
         <http://ex/cd> a sbol:ComponentDefinition ; sbol:displayId \"cd\" ;\n\
           sbol:persistentIdentity <http://ex/cd> ; sbol:type {DNA_REGION} ;\n\
           sbol:component <http://ex/cd/local> , <http://ex/cd/c1> , <http://ex/cd/c2> .\n\
         <http://ex/cd/local> a sbol:Component ; sbol:displayId \"local\" ;\n\
           sbol:persistentIdentity <http://ex/cd/local> ; sbol:access <http://sbols.org/v2#public> ;\n\
           sbol:definition <http://ex/d> .\n\
         <http://ex/cd/c1> a sbol:Component ; sbol:displayId \"c1\" ;\n\
           sbol:persistentIdentity <http://ex/cd/c1> ; sbol:access <http://sbols.org/v2#public> ;\n\
           sbol:definition <http://ex/d> ; sbol:mapsTo <http://ex/cd/c1/m> .\n\
         <http://ex/cd/c1/m> a sbol:MapsTo ; sbol:displayId \"m\" ;\n\
           sbol:persistentIdentity <http://ex/cd/c1/m> ; sbol:refinement <http://sbols.org/v2#useRemote> ;\n\
           sbol:local <http://ex/cd/local> ; sbol:remote <http://ex/d/r> .\n\
         <http://ex/cd/c2> a sbol:Component ; sbol:displayId \"c2\" ;\n\
           sbol:persistentIdentity <http://ex/cd/c2> ; sbol:access <http://sbols.org/v2#public> ;\n\
           sbol:definition <http://ex/d> ; sbol:mapsTo <http://ex/cd/c2/m> .\n\
         <http://ex/cd/c2/m> a sbol:MapsTo ; sbol:displayId \"m\" ;\n\
           sbol:persistentIdentity <http://ex/cd/c2/m> ; sbol:refinement <http://sbols.org/v2#useRemote> ;\n\
           sbol:local <http://ex/cd/local> ; sbol:remote <http://ex/d/r> .\n\
         <http://ex/d> a sbol:ComponentDefinition ; sbol:displayId \"d\" ;\n\
           sbol:persistentIdentity <http://ex/d> ; sbol:type {DNA_REGION} .\n"
    );
    assert!(reports_rule(&doc(&turtle).validate(), "sbol2-10526"));
}

// 10516: a DNA ComponentDefinition with a protein-only Sequence lacks a nucleic one.
#[test]
fn cd_sequence_category_10516() {
    let turtle = format!(
        "{PREAMBLE}\
         <http://ex/cd> a sbol:ComponentDefinition ; sbol:displayId \"cd\" ;\n\
           sbol:persistentIdentity <http://ex/cd> ; sbol:type {DNA_REGION} ;\n\
           sbol:sequence <http://ex/s> .\n\
         <http://ex/s> a sbol:Sequence ; sbol:displayId \"s\" ;\n\
           sbol:persistentIdentity <http://ex/s> ; sbol:elements \"MKV\" ;\n\
           sbol:encoding <http://www.chem.qmul.ac.uk/iupac/AminoAcid/> .\n"
    );
    let document = doc(&turtle);
    // Gated on the best-practice family; silent by default.
    assert!(!reports_rule(&document.validate(), "sbol2-10516"));
    assert!(reports_rule(&document.validate_with_config(&all_on()), "sbol2-10516"));
}

// 10518: two nucleic Sequences of one ComponentDefinition differing in length.
#[test]
fn cd_sequence_length_10518() {
    let nucleic = "http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html";
    let turtle = format!(
        "{PREAMBLE}\
         <http://ex/cd> a sbol:ComponentDefinition ; sbol:displayId \"cd\" ;\n\
           sbol:persistentIdentity <http://ex/cd> ; sbol:type {DNA_REGION} ;\n\
           sbol:sequence <http://ex/s1> , <http://ex/s2> .\n\
         <http://ex/s1> a sbol:Sequence ; sbol:displayId \"s1\" ;\n\
           sbol:persistentIdentity <http://ex/s1> ; sbol:elements \"ACGT\" ;\n\
           sbol:encoding <{nucleic}> .\n\
         <http://ex/s2> a sbol:Sequence ; sbol:displayId \"s2\" ;\n\
           sbol:persistentIdentity <http://ex/s2> ; sbol:elements \"ACG\" ;\n\
           sbol:encoding <{nucleic}> .\n",
        nucleic = nucleic,
    );
    let document = doc(&turtle);
    assert!(warns(&document.validate_with_config(&all_on()), "sbol2-10518"));
}

// 11907: an inhibition Interaction with a product-role Participation.
#[test]
fn interaction_participation_role_11907() {
    let turtle = format!(
        "{PREAMBLE}\
         <http://ex/i> a sbol:Interaction ; sbol:displayId \"i\" ;\n\
           sbol:persistentIdentity <http://ex/i> ;\n\
           sbol:type <http://identifiers.org/biomodels.sbo/SBO:0000169> ;\n\
           sbol:participation <http://ex/i/p> .\n\
         <http://ex/i/p> a sbol:Participation ; sbol:displayId \"p\" ;\n\
           sbol:persistentIdentity <http://ex/i/p> ;\n\
           sbol:role <http://identifiers.org/biomodels.sbo/SBO:0000011> ;\n\
           sbol:participant <http://ex/i/fc> .\n"
    );
    let document = doc(&turtle);
    assert!(warns(&document.validate_with_config(&all_on()), "sbol2-11907"));
}

// 12909: a CombinatorialDerivation whose template has no Components.
#[test]
fn combinatorial_empty_template_12909() {
    let turtle = format!(
        "{PREAMBLE}\
         <http://ex/cda> a sbol:CombinatorialDerivation ; sbol:displayId \"cda\" ;\n\
           sbol:persistentIdentity <http://ex/cda> ; sbol:template <http://ex/t> .\n\
         <http://ex/t> a sbol:ComponentDefinition ; sbol:displayId \"t\" ;\n\
           sbol:persistentIdentity <http://ex/t> ; sbol:type {DNA_REGION} .\n"
    );
    let document = doc(&turtle);
    assert!(warns(&document.validate_with_config(&all_on()), "sbol2-12909"));
}
