//! Further biology-first construction verbs that complement [`ComponentVerbs`](crate::ComponentVerbs),
//! covering RNA, protein, and abstract functional components. Like the core
//! verbs, they read as inherent methods on [`sbol3::design::Design`]; import the
//! crate [`prelude`](crate::prelude) to bring them into scope.
//!
//! ```
//! use sbol3::design::Design;
//! use sbol_utilities::prelude::*;
//!
//! let mut d = Design::new("https://example.org/lab").unwrap();
//! let _araC = d.gene("araC", "atggtgaaacag").add();
//! let _rna = d.mrna("gfp_mrna", "auggugagcaag").add();
//! let _laci = d.functional_component("LacI").description("LacI tetramer").add();
//! let doc = d.finish().unwrap();
//! assert_eq!(doc.components().count(), 3);
//! ```

use sbol3::Iri;
use sbol3::constants::{SBO_FUNCTIONAL_ENTITY, SO_GENE, SO_MRNA, SO_OPERATOR};
use sbol3::design::{ComponentId, Design};

use crate::{Molecule, PartDraft};

/// Transcription-factor role (`SO:0003700`, DNA-binding transcription factor),
/// mirroring pySBOL3's `SO_TRANSCRIPTION_FACTOR`.
const SO_TRANSCRIPTION_FACTOR: Iri = Iri::from_static("https://identifiers.org/SO:0003700");

/// Biology-first verbs for parts and components beyond DNA-only regions,
/// implemented on [`Design`]. Complements [`ComponentVerbs`](crate::ComponentVerbs).
pub trait MoleculeVerbs {
    /// A gene (`SO:0000704`) DNA part with its sequence.
    fn gene<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d>;
    /// An operator (`SO:0000057`) DNA part with its sequence.
    fn operator<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d>;
    /// An mRNA (`SO:0000234`) RNA part: an `SBO_RNA` component with its RNA
    /// sequence.
    fn mrna<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d>;
    /// A transcription factor (`SO:0003700`): an `SBO_PROTEIN` component with its
    /// amino-acid sequence.
    fn transcription_factor<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d>;
    /// A functional-entity `Component` (`SBO:0000241`) with no sequence, used for
    /// proteins, complexes, and other abstract species that a design references
    /// but does not spell out at the sequence level.
    fn functional_component<'d>(&'d mut self, display_id: &str) -> FunctionalComponentDraft<'d>;
}

impl MoleculeVerbs for Design {
    fn gene<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d> {
        PartDraft::new(self, display_id, SO_GENE, Molecule::Dna, elements)
    }

    fn operator<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d> {
        PartDraft::new(self, display_id, SO_OPERATOR, Molecule::Dna, elements)
    }

    fn mrna<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d> {
        PartDraft::new(self, display_id, SO_MRNA, Molecule::Rna, elements)
    }

    fn transcription_factor<'d>(&'d mut self, display_id: &str, elements: &str) -> PartDraft<'d> {
        PartDraft::new(
            self,
            display_id,
            SO_TRANSCRIPTION_FACTOR,
            Molecule::Protein,
            elements,
        )
    }

    fn functional_component<'d>(&'d mut self, display_id: &str) -> FunctionalComponentDraft<'d> {
        FunctionalComponentDraft::new(self, display_id)
    }
}

/// In-progress functional-entity `Component`, created by
/// [`functional_component`](MoleculeVerbs::functional_component). Unlike a part,
/// it has no sequence; `.add()` returns the component handle.
#[must_use = "call `.add()` to register the component in the design"]
pub struct FunctionalComponentDraft<'d> {
    design: &'d mut Design,
    display_id: String,
    name: Option<String>,
    description: Option<String>,
}

impl<'d> FunctionalComponentDraft<'d> {
    fn new(design: &'d mut Design, display_id: &str) -> Self {
        Self {
            design,
            display_id: display_id.to_string(),
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

    /// Registers the functional-entity component and returns its handle.
    pub fn add(self) -> ComponentId {
        let Self {
            design,
            display_id,
            name,
            description,
        } = self;

        let mut component = design.component(&display_id).type_(SBO_FUNCTIONAL_ENTITY);
        if let Some(name) = name {
            component = component.name(name);
        }
        if let Some(description) = description {
            component = component.description(description);
        }
        component.add()
    }
}

#[cfg(test)]
mod tests;
