//! Provenance Activity, Usage, and Association semantic rules (124xx, 125xx,
//! 126xx).

use super::common::{PositiveCase, RuleCase};
use sbol2::validation::Severity::{Error, Warning};

pub fn cases() -> Vec<RuleCase> {
    vec![
        RuleCase {
            name: "Activity wasInformedBy reference does not resolve",
            rule: "sbol2-12407",
            severity: Error,
            body: r#"<http://ex/act> a prov:Activity ;
    sbol:displayId "act" ;
    sbol:persistentIdentity <http://ex/act> ;
    prov:wasInformedBy <http://ex/missing-activity> .
"#,
        },
        RuleCase {
            name: "design Activity with a build Usage",
            rule: "sbol2-12408",
            severity: Warning,
            body: activity_roles("http://sbols.org/v2#design", "http://sbols.org/v2#build"),
        },
        RuleCase {
            name: "build Activity with a test Usage",
            rule: "sbol2-12409",
            severity: Warning,
            body: activity_roles("http://sbols.org/v2#build", "http://sbols.org/v2#test"),
        },
        RuleCase {
            name: "test Activity with a design Usage",
            rule: "sbol2-12410",
            severity: Warning,
            body: activity_roles("http://sbols.org/v2#test", "http://sbols.org/v2#design"),
        },
        RuleCase {
            name: "learn Activity with a design Usage",
            rule: "sbol2-12411",
            severity: Warning,
            body: activity_roles("http://sbols.org/v2#learn", "http://sbols.org/v2#design"),
        },
        RuleCase {
            name: "design Usage refers to an Implementation",
            rule: "sbol2-12504",
            severity: Warning,
            body: usage_entity("http://sbols.org/v2#design", "sbol:Implementation"),
        },
        RuleCase {
            name: "build Usage refers to a non-Implementation",
            rule: "sbol2-12505",
            severity: Warning,
            body: usage_entity("http://sbols.org/v2#build", "sbol:ComponentDefinition"),
        },
        RuleCase {
            name: "test Usage refers to neither Attachment nor Collection",
            rule: "sbol2-12506",
            severity: Warning,
            body: usage_entity("http://sbols.org/v2#test", "sbol:ComponentDefinition"),
        },
        RuleCase {
            name: "learn Usage refers to an Implementation",
            rule: "sbol2-12507",
            severity: Warning,
            body: usage_entity("http://sbols.org/v2#learn", "sbol:Implementation"),
        },
        RuleCase {
            name: "Association agent reference does not resolve",
            rule: "sbol2-12604",
            severity: Error,
            body: r#"<http://ex/assoc> a prov:Association ;
    sbol:displayId "assoc" ;
    sbol:persistentIdentity <http://ex/assoc> ;
    prov:agent <http://ex/missing-agent> .
"#,
        },
        RuleCase {
            name: "Association agent refers to a non-Agent object",
            rule: "sbol2-12606",
            severity: Error,
            body: r#"<http://ex/assoc> a prov:Association ;
    sbol:displayId "assoc" ;
    sbol:persistentIdentity <http://ex/assoc> ;
    prov:agent <http://ex/not-agent> .
<http://ex/not-agent> a sbol:Collection ;
    sbol:displayId "na" ;
    sbol:persistentIdentity <http://ex/not-agent> .
"#,
        },
    ]
}

/// An Activity carrying an Association with `assoc_role` and a Usage with
/// `usage_role`.
fn activity_roles(assoc_role: &str, usage_role: &str) -> &'static str {
    match (assoc_role, usage_role) {
        ("http://sbols.org/v2#design", "http://sbols.org/v2#build") => ACT_DESIGN_BUILD,
        ("http://sbols.org/v2#build", "http://sbols.org/v2#test") => ACT_BUILD_TEST,
        ("http://sbols.org/v2#test", "http://sbols.org/v2#design") => ACT_TEST_DESIGN,
        ("http://sbols.org/v2#learn", "http://sbols.org/v2#design") => ACT_LEARN_DESIGN,
        _ => unreachable!(),
    }
}

fn usage_entity(role: &str, class: &str) -> &'static str {
    match (role, class) {
        ("http://sbols.org/v2#design", "sbol:Implementation") => USAGE_DESIGN_IMPL,
        ("http://sbols.org/v2#build", "sbol:ComponentDefinition") => USAGE_BUILD_CD,
        ("http://sbols.org/v2#test", "sbol:ComponentDefinition") => USAGE_TEST_CD,
        ("http://sbols.org/v2#learn", "sbol:Implementation") => USAGE_LEARN_IMPL,
        _ => unreachable!(),
    }
}

macro_rules! activity_body {
    ($assoc:literal, $usage:literal) => {
        concat!(
            r#"<http://ex/act> a prov:Activity ;
    sbol:displayId "act" ;
    sbol:persistentIdentity <http://ex/act> ;
    prov:qualifiedAssociation <http://ex/act/assoc> ;
    prov:qualifiedUsage <http://ex/act/usage> .
<http://ex/act/assoc> a prov:Association ;
    sbol:displayId "assoc" ;
    sbol:persistentIdentity <http://ex/act/assoc> ;
    prov:agent <http://ex/agent> ;
    prov:hadRole <"#,
            $assoc,
            r#"> .
<http://ex/act/usage> a prov:Usage ;
    sbol:displayId "usage" ;
    sbol:persistentIdentity <http://ex/act/usage> ;
    prov:entity <http://ex/ent> ;
    prov:hadRole <"#,
            $usage,
            r#"> .
"#
        )
    };
}

const ACT_DESIGN_BUILD: &str = activity_body!("http://sbols.org/v2#design", "http://sbols.org/v2#build");
const ACT_BUILD_TEST: &str = activity_body!("http://sbols.org/v2#build", "http://sbols.org/v2#test");
const ACT_TEST_DESIGN: &str = activity_body!("http://sbols.org/v2#test", "http://sbols.org/v2#design");
const ACT_LEARN_DESIGN: &str = activity_body!("http://sbols.org/v2#learn", "http://sbols.org/v2#design");

macro_rules! usage_body {
    ($role:literal, $class:literal) => {
        concat!(
            r#"<http://ex/usage> a prov:Usage ;
    sbol:displayId "usage" ;
    sbol:persistentIdentity <http://ex/usage> ;
    prov:entity <http://ex/ent> ;
    prov:hadRole <"#,
            $role,
            r#"> .
<http://ex/ent> a "#,
            $class,
            r#" ;
    sbol:displayId "ent" ;
    sbol:persistentIdentity <http://ex/ent> .
"#
        )
    };
}

const USAGE_DESIGN_IMPL: &str = usage_body!("http://sbols.org/v2#design", "sbol:Implementation");
const USAGE_BUILD_CD: &str = usage_body!("http://sbols.org/v2#build", "sbol:ComponentDefinition");
const USAGE_TEST_CD: &str = usage_body!("http://sbols.org/v2#test", "sbol:ComponentDefinition");
const USAGE_LEARN_IMPL: &str = usage_body!("http://sbols.org/v2#learn", "sbol:Implementation");

pub fn positives() -> Vec<PositiveCase> {
    vec![
        PositiveCase {
            name: "Activity wasInformedBy resolves to an Activity",
            rule: "sbol2-12407",
            body: r#"<http://ex/act> a prov:Activity ;
    sbol:displayId "act" ;
    sbol:persistentIdentity <http://ex/act> ;
    prov:wasInformedBy <http://ex/other-act> .
<http://ex/other-act> a prov:Activity ;
    sbol:displayId "other" ;
    sbol:persistentIdentity <http://ex/other-act> .
"#,
        },
        PositiveCase {
            name: "design Activity without conflicting Usage roles",
            rule: "sbol2-12408",
            body: ACT_DESIGN_ONLY,
        },
        PositiveCase {
            name: "build Activity without conflicting Usage roles",
            rule: "sbol2-12409",
            body: ACT_BUILD_ONLY,
        },
        PositiveCase {
            name: "test Activity without conflicting Usage roles",
            rule: "sbol2-12410",
            body: ACT_TEST_ONLY,
        },
        PositiveCase {
            name: "learn Activity without conflicting Usage roles",
            rule: "sbol2-12411",
            body: ACT_LEARN_ONLY,
        },
        PositiveCase {
            name: "design Usage refers to a non-Implementation TopLevel",
            rule: "sbol2-12504",
            body: usage_entity_positive("http://sbols.org/v2#design", "sbol:ComponentDefinition"),
        },
        PositiveCase {
            name: "build Usage refers to an Implementation",
            rule: "sbol2-12505",
            body: usage_entity_positive("http://sbols.org/v2#build", "sbol:Implementation"),
        },
        PositiveCase {
            name: "test Usage refers to an Attachment",
            rule: "sbol2-12506",
            body: usage_entity_positive("http://sbols.org/v2#test", "sbol:Attachment"),
        },
        PositiveCase {
            name: "learn Usage refers to a non-Implementation",
            rule: "sbol2-12507",
            body: usage_entity_positive("http://sbols.org/v2#learn", "sbol:ComponentDefinition"),
        },
        PositiveCase {
            name: "Association agent resolves to an Agent",
            rule: "sbol2-12604",
            body: ASSOC_VALID,
        },
        PositiveCase {
            name: "Association agent refers to an Agent",
            rule: "sbol2-12606",
            body: ASSOC_VALID,
        },
    ]
}

fn usage_entity_positive(role: &str, class: &str) -> &'static str {
    match (role, class) {
        ("http://sbols.org/v2#design", "sbol:ComponentDefinition") => USAGE_DESIGN_CD,
        ("http://sbols.org/v2#build", "sbol:Implementation") => USAGE_BUILD_IMPL,
        ("http://sbols.org/v2#test", "sbol:Attachment") => USAGE_TEST_ATTACHMENT,
        ("http://sbols.org/v2#learn", "sbol:ComponentDefinition") => USAGE_LEARN_CD,
        _ => unreachable!(),
    }
}

const USAGE_DESIGN_CD: &str = usage_body!("http://sbols.org/v2#design", "sbol:ComponentDefinition");
const USAGE_BUILD_IMPL: &str = usage_body!("http://sbols.org/v2#build", "sbol:Implementation");
const USAGE_TEST_ATTACHMENT: &str = usage_body!("http://sbols.org/v2#test", "sbol:Attachment");
const USAGE_LEARN_CD: &str = usage_body!("http://sbols.org/v2#learn", "sbol:ComponentDefinition");

const ACT_DESIGN_ONLY: &str = activity_body!("http://sbols.org/v2#design", "http://sbols.org/v2#design");
const ACT_BUILD_ONLY: &str = activity_body!("http://sbols.org/v2#build", "http://sbols.org/v2#build");
const ACT_TEST_ONLY: &str = activity_body!("http://sbols.org/v2#test", "http://sbols.org/v2#test");
const ACT_LEARN_ONLY: &str = activity_body!("http://sbols.org/v2#learn", "http://sbols.org/v2#learn");

const ASSOC_VALID: &str = r#"<http://ex/assoc> a prov:Association ;
    sbol:displayId "assoc" ;
    sbol:persistentIdentity <http://ex/assoc> ;
    prov:agent <http://ex/agent> .
<http://ex/agent> a prov:Agent ;
    sbol:displayId "agent" ;
    sbol:persistentIdentity <http://ex/agent> .
"#;
