//! Biology-first construction helpers for sbol-rs, in the spirit of the Python
//! `sbol_utilities` package.
//!
//! The verbs read as inherent methods on [`sbol3::design::Design`] via the
//! [`ComponentVerbs`] extension trait. Import the [`prelude`] to light them up:
//!
//! ```
//! use sbol3::design::Design;
//! use sbol_utilities::prelude::*;
//!
//! let mut d = Design::new("https://example.org/lab").unwrap();
//! let plac = d.promoter("pLac", "caatacg").description("LacI-repressible").add();
//! let b0034 = d.rbs("B0034", "ttgaac").add();
//! d.engineered_region("pLac_tu", [plac, b0034]).add();
//! let doc = d.finish().unwrap();
//! assert_eq!(doc.components().count(), 3); // pLac, B0034, pLac_tu
//! ```

#![forbid(unsafe_code)]

use sbol3::Iri;
use sbol3::constants::{SO_CDS, SO_ENGINEERED_REGION, SO_PROMOTER, SO_RBS, SO_TERMINATOR};
use sbol3::design::{ComponentId, Design, FeatureId};
use sbol3::prelude::SbolIdentified;

/// Common imports for authoring designs with the biology verbs.
pub mod prelude {
    pub use crate::{ComponentVerbs, Part, PartDraft, RegionDraft};
}

/// A member of an [`engineered_region`](ComponentVerbs::engineered_region):
/// either a `Component` (wrapped in a fresh sub-component with its roles
/// copied) or an already-built feature.
#[derive(Clone, Copy, Debug)]
pub enum Part {
    /// A component to instantiate as a new sub-component.
    Component(ComponentId),
    /// An existing feature, used as-is.
    Feature(FeatureId),
}

impl From<ComponentId> for Part {
    fn from(value: ComponentId) -> Self {
        Part::Component(value)
    }
}

impl From<FeatureId> for Part {
    fn from(value: FeatureId) -> Self {
        Part::Feature(value)
    }
}

/// Biology-first construction verbs, implemented on [`Design`].
pub trait ComponentVerbs {
    /// A promoter (`SO:0000167`) DNA part with its sequence.
    fn promoter<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d>;
    /// A ribosome entry site (`SO:0000139`) DNA part with its sequence.
    fn rbs<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d>;
    /// A coding sequence (`SO:0000316`) DNA part with its sequence.
    fn cds<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d>;
    /// A terminator (`SO:0000141`) DNA part with its sequence.
    fn terminator<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d>;
    /// An engineered region (`SO:0000804`) whose parts are chained head-to-tail
    /// with `meets` constraints, each part's roles copied onto its
    /// sub-component.
    fn engineered_region<'d, I>(&'d mut self, display_id: &str, parts: I) -> RegionDraft<'d>
    where
        I: IntoIterator,
        I::Item: Into<Part>;
}

impl ComponentVerbs for Design {
    fn promoter<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d> {
        PartDraft::new(self, display_id, SO_PROMOTER, elements)
    }

    fn rbs<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d> {
        PartDraft::new(self, display_id, SO_RBS, elements)
    }

    fn cds<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d> {
        PartDraft::new(self, display_id, SO_CDS, elements)
    }

    fn terminator<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d> {
        PartDraft::new(self, display_id, SO_TERMINATOR, elements)
    }

    fn engineered_region<'d, I>(&'d mut self, display_id: &str, parts: I) -> RegionDraft<'d>
    where
        I: IntoIterator,
        I::Item: Into<Part>,
    {
        RegionDraft {
            design: self,
            display_id: display_id.to_string(),
            parts: parts.into_iter().map(Into::into).collect(),
            name: None,
            description: None,
        }
    }
}

/// In-progress DNA part (a `Component` plus its `Sequence`), created by the
/// part verbs. The sequence is registered automatically; `.add()` returns only
/// the component handle.
#[must_use = "call `.add()` to register the part in the design"]
pub struct PartDraft<'d> {
    design: &'d mut Design,
    display_id: String,
    role: Iri,
    elements: String,
    name: Option<String>,
    description: Option<String>,
}

impl<'d> PartDraft<'d> {
    fn new(design: &'d mut Design, display_id: &str, role: Iri, elements: &str) -> Self {
        Self {
            design,
            display_id: display_id.to_string(),
            role,
            elements: elements.to_string(),
            name: None,
            description: None,
        }
    }

    /// Sets the human-readable name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Registers the part (component + sequence) and returns the component.
    pub fn add(self) -> ComponentId {
        let Self {
            design,
            display_id,
            role,
            elements,
            name,
            description,
        } = self;

        let sequence = design
            .sequence(&format!("{display_id}_seq"))
            .elements(elements)
            .dna()
            .add();

        let mut component = design
            .component(&display_id)
            .dna()
            .role(role)
            .sequence(sequence);
        if let Some(name) = name {
            component = component.name(name);
        }
        if let Some(description) = description {
            component = component.description(description);
        }
        component.add()
    }
}

/// In-progress engineered region, created by
/// [`engineered_region`](ComponentVerbs::engineered_region).
#[must_use = "call `.add()` to register the region in the design"]
pub struct RegionDraft<'d> {
    design: &'d mut Design,
    display_id: String,
    parts: Vec<Part>,
    name: Option<String>,
    description: Option<String>,
}

impl RegionDraft<'_> {
    /// Sets the human-readable name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Registers the region, its sub-components, and their `meets` ordering.
    /// Returns the region component.
    pub fn add(self) -> ComponentId {
        let Self {
            design,
            display_id,
            parts,
            name,
            description,
        } = self;

        let mut region = design
            .component(&display_id)
            .dna()
            .role(SO_ENGINEERED_REGION);
        if let Some(name) = name {
            region = region.name(name);
        }
        if let Some(description) = description {
            region = region.description(description);
        }
        let region = region.add();

        let mut features = Vec::with_capacity(parts.len());
        for (index, part) in parts.into_iter().enumerate() {
            match part {
                Part::Component(component) => {
                    let (roles, source_name) = match design.resolve_component(component) {
                        Some(component) => (
                            component.roles.clone(),
                            component.display_id().map(str::to_string),
                        ),
                        None => (Vec::new(), None),
                    };
                    if roles.is_empty() {
                        design.report(format!(
                            "engineered_region `{display_id}` includes a part with no roles; \
                             DNAplotlib requires roles on the sub-component"
                        ));
                    }
                    let mut sub = design
                        .sub_component(region, &format!("{display_id}_sub{index}"))
                        .instance_of(component)
                        .roles(roles);
                    if let Some(source_name) = source_name {
                        sub = sub.name(source_name);
                    }
                    features.push(sub.add());
                }
                Part::Feature(_) => {
                    design.report(format!(
                        "engineered_region `{display_id}`: passing an existing feature is not \
                         yet supported; pass the Component instead"
                    ));
                }
            }
        }

        for pair in features.windows(2) {
            design.meets(region, pair[0], pair[1]);
        }

        region
    }
}

#[cfg(test)]
mod tests;
