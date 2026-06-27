//! sbol3-10205, sbol3-12901, sbol3-12902, sbol3-13001 — Appendix A.1
//! design-build-test-learn workflow constraints (Tables 20 and 21).

use super::{PositiveCase, RuleCase};
use sbol::Severity::Warning;

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "wasGeneratedBy build role with non-Implementation target",
            rule: "sbol3-10205",
            severity: Warning,
            body: r#"<https://example.org/artifact> a sbol:Component ;
    sbol:displayId "artifact" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:type SBO:0000251 ;
    prov:wasGeneratedBy <https://example.org/build_run> .

<https://example.org/build_run> a prov:Activity, sbol:TopLevel ;
    sbol:displayId "build_run" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:type sbol:build ;
    prov:qualifiedAssociation <https://example.org/build_run/build_assoc> .

<https://example.org/build_run/build_assoc> a prov:Association ;
    sbol:displayId "build_assoc" ;
    prov:hadRole sbol:build .
"#,
        },
        RuleCase {
            name: "Activity Usage role skips the workflow stage order",
            rule: "sbol3-12901",
            severity: Warning,
            body: r#"<https://example.org/mix_run> a prov:Activity, sbol:TopLevel ;
    sbol:displayId "mix_run" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:type sbol:build ;
    prov:qualifiedUsage <https://example.org/mix_run/wrong_stage> .

<https://example.org/mix_run/wrong_stage> a prov:Usage ;
    sbol:displayId "wrong_stage" ;
    prov:hadRole sbol:learn ;
    prov:entity <https://example.org/leftover> .

<https://example.org/leftover> a sbol:Implementation ;
    sbol:displayId "leftover" ;
    sbol:hasNamespace <https://example.org> .
"#,
        },
        RuleCase {
            name: "Association role does not match Activity stage",
            rule: "sbol3-12902",
            severity: Warning,
            body: r#"<https://example.org/run> a prov:Activity, sbol:TopLevel ;
    sbol:displayId "run" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:type sbol:test ;
    prov:qualifiedAssociation <https://example.org/run/assoc> .

<https://example.org/run/assoc> a prov:Association ;
    sbol:displayId "assoc" ;
    prov:hadRole sbol:build .
"#,
        },
        RuleCase {
            name: "Usage build role with non-Implementation entity",
            rule: "sbol3-13001",
            severity: Warning,
            body: r#"<https://example.org/use> a prov:Usage, sbol:TopLevel ;
    sbol:displayId "use" ;
    sbol:hasNamespace <https://example.org> ;
    prov:hadRole sbol:build ;
    prov:entity <https://example.org/not_an_implementation> .

<https://example.org/not_an_implementation> a sbol:Sequence ;
    sbol:displayId "not_an_implementation" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:elements "A" ;
    sbol:encoding EDAM:format_1207 .
"#,
        },
    ]
}

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "wasGeneratedBy build role with Implementation target",
            rule: "sbol3-10205",
            body: r#"<https://example.org/artifact> a sbol:Implementation ;
    sbol:displayId "artifact" ;
    sbol:hasNamespace <https://example.org> ;
    prov:wasGeneratedBy <https://example.org/build_run> .

<https://example.org/build_run> a prov:Activity, sbol:TopLevel ;
    sbol:displayId "build_run" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:type sbol:build ;
    prov:qualifiedAssociation <https://example.org/build_run/build_assoc> .

<https://example.org/build_run/build_assoc> a prov:Association ;
    sbol:displayId "build_assoc" ;
    prov:hadRole sbol:build .
"#,
        },
        PositiveCase {
            name: "Activity Usage role matches the workflow stage order",
            rule: "sbol3-12901",
            body: r#"<https://example.org/mix_run> a prov:Activity, sbol:TopLevel ;
    sbol:displayId "mix_run" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:type sbol:build ;
    prov:qualifiedUsage <https://example.org/mix_run/build_stage> .

<https://example.org/mix_run/build_stage> a prov:Usage ;
    sbol:displayId "build_stage" ;
    prov:hadRole sbol:build ;
    prov:entity <https://example.org/implementation> .

<https://example.org/implementation> a sbol:Implementation ;
    sbol:displayId "implementation" ;
    sbol:hasNamespace <https://example.org> .
"#,
        },
        PositiveCase {
            name: "Association role matches Activity stage",
            rule: "sbol3-12902",
            body: r#"<https://example.org/run> a prov:Activity, sbol:TopLevel ;
    sbol:displayId "run" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:type sbol:test ;
    prov:qualifiedAssociation <https://example.org/run/assoc> .

<https://example.org/run/assoc> a prov:Association ;
    sbol:displayId "assoc" ;
    prov:hadRole sbol:test .
"#,
        },
        PositiveCase {
            name: "Usage build role with Implementation entity",
            rule: "sbol3-13001",
            body: r#"<https://example.org/use> a prov:Usage, sbol:TopLevel ;
    sbol:displayId "use" ;
    sbol:hasNamespace <https://example.org> ;
    prov:hadRole sbol:build ;
    prov:entity <https://example.org/an_implementation> .

<https://example.org/an_implementation> a sbol:Implementation ;
    sbol:displayId "an_implementation" ;
    sbol:hasNamespace <https://example.org> .
"#,
        },
    ]
}
