//! Tables 20 and 21 (Appendix A.1): design-build-test-learn workflow
//! constraints over `prov:Activity`, `prov:Association`, `prov:Usage`,
//! and Identified objects with `prov:wasGeneratedBy`.

use crate::Object;
use crate::validation::validator::Validator;
use crate::vocab::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkflowStage {
    Design,
    Build,
    Test,
    Learn,
}

impl WorkflowStage {
    fn from_iri(iri: &str) -> Option<Self> {
        match iri {
            SBOL_DESIGN => Some(Self::Design),
            SBOL_BUILD => Some(Self::Build),
            SBOL_TEST => Some(Self::Test),
            SBOL_LEARN => Some(Self::Learn),
            _ => None,
        }
    }

    /// Table 21: the preceding stage for each workflow stage.
    fn preceding(self) -> Self {
        match self {
            Self::Design => Self::Learn,
            Self::Build => Self::Design,
            Self::Test => Self::Build,
            Self::Learn => Self::Test,
        }
    }

    /// Table 21: whether the target object's SBOL `rdf:type` set is
    /// compatible with this stage's "referred object type" column.
    fn matches_target(self, target: &Object) -> bool {
        let types: Vec<&str> = target.rdf_types().iter().map(|iri| iri.as_str()).collect();
        let is_implementation = types.contains(&SBOL_IMPLEMENTATION_CLASS);
        let is_experimental_data = types.contains(&SBOL_EXPERIMENTAL_DATA_CLASS);
        match self {
            // design → TopLevel other than Implementation
            Self::Design => !is_implementation,
            // build → Implementation
            Self::Build => is_implementation,
            // test → ExperimentalData
            Self::Test => is_experimental_data,
            // learn → Identified other than Implementation
            Self::Learn => !is_implementation,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Design => "design",
            Self::Build => "build",
            Self::Test => "test",
            Self::Learn => "learn",
        }
    }
}

impl<'a> Validator<'a> {
    pub(crate) fn validate_workflow_rules(&mut self, object: &Object) {
        if object
            .rdf_types()
            .iter()
            .any(|iri| iri.as_str() == PROV_ACTIVITY)
        {
            self.validate_activity_workflow(object);
        }
        if object
            .rdf_types()
            .iter()
            .any(|iri| iri.as_str() == PROV_USAGE)
        {
            self.validate_usage_workflow(object);
        }
        self.validate_was_generated_by_workflow(object);
    }

    /// sbol3-12901 + sbol3-12902 on `prov:Activity`.
    fn validate_activity_workflow(&mut self, activity: &Object) {
        let activity_stages: Vec<WorkflowStage> = activity
            .iris(SBOL_TYPE)
            .filter_map(|iri| WorkflowStage::from_iri(iri.as_str()))
            .collect();
        if activity_stages.is_empty() {
            return;
        }

        // sbol3-12901: child Usage roles MUST NOT be Table 20 stages
        // other than same or preceding stage.
        for usage in activity.resources(PROV_QUALIFIED_USAGE) {
            let Some(usage_object) = self.document.get(usage) else {
                continue;
            };
            for role in usage_object.iris(PROV_HAD_ROLE) {
                let Some(role_stage) = WorkflowStage::from_iri(role.as_str()) else {
                    continue;
                };
                let allowed = activity_stages
                    .iter()
                    .any(|stage| *stage == role_stage || stage.preceding() == role_stage);
                if !allowed {
                    self.warning(
                        "sbol3-12901",
                        usage_object,
                        Some(PROV_HAD_ROLE),
                        format!(
                            "Usage role `{}` is not the same as or preceding any of \
                             the Activity's workflow stages",
                            role_stage.label()
                        ),
                    );
                }
            }
        }

        // sbol3-12902: every Association's prov:hadRole SHOULD equal the
        // Activity's Table 20 type.
        for association in activity.resources(PROV_QUALIFIED_ASSOCIATION) {
            let Some(association_object) = self.document.get(association) else {
                continue;
            };
            for role in association_object.iris(PROV_HAD_ROLE) {
                let Some(role_stage) = WorkflowStage::from_iri(role.as_str()) else {
                    continue;
                };
                if !activity_stages.contains(&role_stage) {
                    self.warning(
                        "sbol3-12902",
                        association_object,
                        Some(PROV_HAD_ROLE),
                        format!(
                            "Association role `{}` does not match any of the Activity's \
                             workflow stages",
                            role_stage.label()
                        ),
                    );
                }
            }
        }
    }

    /// sbol3-13001 on `prov:Usage`.
    fn validate_usage_workflow(&mut self, usage: &Object) {
        for role in usage.iris(PROV_HAD_ROLE) {
            let Some(stage) = WorkflowStage::from_iri(role.as_str()) else {
                continue;
            };
            for entity in usage.resources(PROV_ENTITY) {
                let Some(entity_object) = self.document.get(entity) else {
                    continue;
                };
                if !stage.matches_target(entity_object) {
                    self.warning(
                        "sbol3-13001",
                        usage,
                        Some(PROV_ENTITY),
                        format!(
                            "Usage role `{}` requires entity to be {}",
                            stage.label(),
                            stage_referred_type_label(stage)
                        ),
                    );
                }
            }
        }
    }

    /// sbol3-10205 on any Identified with `prov:wasGeneratedBy`.
    fn validate_was_generated_by_workflow(&mut self, object: &Object) {
        for activity_id in object.resources(PROV_WAS_GENERATED_BY) {
            let Some(activity) = self.document.get(activity_id) else {
                continue;
            };
            for association in activity.resources(PROV_QUALIFIED_ASSOCIATION) {
                let Some(association_object) = self.document.get(association) else {
                    continue;
                };
                for role in association_object.iris(PROV_HAD_ROLE) {
                    let Some(stage) = WorkflowStage::from_iri(role.as_str()) else {
                        continue;
                    };
                    if !stage.matches_target(object) {
                        self.warning(
                            "sbol3-10205",
                            object,
                            Some(PROV_WAS_GENERATED_BY),
                            format!(
                                "wasGeneratedBy Association role `{}` requires the generated \
                                 object to be {}",
                                stage.label(),
                                stage_referred_type_label(stage)
                            ),
                        );
                    }
                }
            }
        }
    }
}

fn stage_referred_type_label(stage: WorkflowStage) -> &'static str {
    match stage {
        WorkflowStage::Design => "a TopLevel other than Implementation",
        WorkflowStage::Build => "an Implementation",
        WorkflowStage::Test => "an ExperimentalData",
        WorkflowStage::Learn => "an Identified other than Implementation",
    }
}
