//! Ergonomic, mutable-feeling construction over the immutable core.
//!
//! [`Design`] is an arena: it owns every in-progress object and hands out
//! `Copy` handles. Composition happens through `&mut self` methods, so a parent
//! and its children never contend for a borrow. [`Design::finish`] lowers the
//! arena to an immutable [`Document`], patching each child's identity into its
//! parent's reference vector so the result is internally consistent by
//! construction.
//!
//! Construction is fluent and infallible at the call site: a bad display ID or
//! a missing required field is recorded against the arena and surfaced together
//! from [`Design::finish`], preserving the mutable-graph feel while still
//! reporting every problem.
//!
//! ```
//! use sbol3::design::Design;
//! use sbol3::constants::SO_PROMOTER;
//!
//! let mut d = Design::new("https://example.org/lab").unwrap();
//! let seq = d.sequence("j23119_seq").elements("ttgacagctagctcagtcctaggtataatgctagc").dna().add();
//! let _promoter = d.component("j23119").dna().role(SO_PROMOTER).sequence(seq).add();
//! let doc = d.finish().unwrap();
//! assert_eq!(doc.components().count(), 1);
//! ```

use std::fmt;

use crate::constants::{
    EDAM_IUPAC_DNA, EDAM_IUPAC_PROTEIN, EDAM_IUPAC_RNA, RESTRICTION_MEETS,
    ROLE_INTEGRATION_MERGE_ROLES, SBO_DNA, SBO_PROTEIN, SBO_RNA,
};
use crate::{
    BuildError, Component, Constraint, Document, Iri, Namespace, Resource, SbolObject,
    SbolTopLevel, Sequence, SubComponent,
};

// ---------------------------------------------------------------------------
// Handles
// ---------------------------------------------------------------------------

macro_rules! handle {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct $name(usize);
    };
}

handle!(
    /// Handle to a top-level `Component` in a [`Design`].
    ComponentId
);
handle!(
    /// Handle to a `Sequence` in a [`Design`].
    SequenceId
);
handle!(
    /// Handle to a feature (e.g. `SubComponent`) in a [`Design`].
    FeatureId
);
handle!(
    /// Handle to a `Constraint` in a [`Design`].
    ConstraintId
);

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// A single problem encountered while composing a [`Design`].
#[derive(Debug)]
#[non_exhaustive]
pub enum DesignProblem {
    /// An object failed to build (invalid display ID, missing required field).
    Build {
        /// The display ID of the object that failed.
        display_id: String,
        /// The underlying build error.
        source: BuildError,
    },
    /// A child referenced a parent or target that had itself failed to build.
    DanglingReference {
        /// The display ID of the child whose reference could not resolve.
        display_id: String,
    },
    /// Lowering to a [`Document`] failed (e.g. duplicate identity).
    Assembly(BuildError),
    /// A domain-level problem reported by a caller via [`Design::report`]
    /// (e.g. an extension layer's own precondition).
    Custom(String),
}

impl fmt::Display for DesignProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build { display_id, source } => {
                write!(f, "failed to build `{display_id}`: {source}")
            }
            Self::DanglingReference { display_id } => {
                write!(
                    f,
                    "`{display_id}` references an object that failed to build"
                )
            }
            Self::Assembly(source) => write!(f, "failed to assemble document: {source}"),
            Self::Custom(message) => write!(f, "{message}"),
        }
    }
}

/// The aggregate error returned by [`Design::new`] and [`Design::finish`],
/// carrying every problem recorded while the design was composed.
#[derive(Debug)]
pub struct DesignError {
    /// The problems recorded, in the order they occurred.
    pub problems: Vec<DesignProblem>,
}

impl fmt::Display for DesignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "design has {} problem(s):", self.problems.len())?;
        for problem in &self.problems {
            write!(f, "\n  - {problem}")?;
        }
        Ok(())
    }
}

impl std::error::Error for DesignError {}

// ---------------------------------------------------------------------------
// Arena
// ---------------------------------------------------------------------------

/// A mutable arena for composing SBOL objects, lowered to an immutable
/// [`Document`] by [`finish`](Design::finish). Each handle indexes one slot.
pub struct Design {
    namespace: Namespace,
    slots: Vec<Slot>,
    problems: Vec<DesignProblem>,
}

/// One entry in a [`Design`]'s arena. `Object` is kept inline (the arena holds
/// one slot per object, dominated by the built-object case); boxing it to even
/// out variant sizes would add an allocation on the common path.
#[allow(clippy::large_enum_variant)]
enum Slot {
    /// A built object, ready to be lowered into the document.
    Object(SbolObject),
    /// A detached sub-component (created by
    /// [`Design::detached_sub_component`]) whose parent is not yet known.
    /// [`Design::place_feature`] turns it into a built [`SubComponent`] under a
    /// parent; if it is never placed it is dropped at [`finish`](Design::finish).
    Pending(SubComponentSpec),
    /// An object that failed to build, or a placeholder for a dangling
    /// reference; the reason is recorded in `problems`.
    Failed,
}

impl Design {
    /// Creates a design whose objects live under `namespace`. Fails if the
    /// namespace is not a valid SBOL namespace IRI.
    pub fn new(namespace: impl AsRef<str>) -> Result<Self, DesignError> {
        let namespace = Namespace::new(namespace.as_ref()).map_err(|source| DesignError {
            problems: vec![DesignProblem::Build {
                display_id: namespace.as_ref().to_string(),
                source: source.into(),
            }],
        })?;
        Ok(Self {
            namespace,
            slots: Vec::new(),
            problems: Vec::new(),
        })
    }

    /// Loads an existing [`Document`] into a fresh arena so it can be
    /// inspected and re-emitted, or extended with more objects. Every object
    /// keeps its original identity; the arena's namespace (used for minting
    /// identities of objects added afterward) is inferred from the document's
    /// top-level objects.
    ///
    /// This is a read/append path: objects come in intact and
    /// [`finish`](Self::finish) writes them back out unchanged. The arena does
    /// not yet support mutating or removing an imported object; use
    /// [`component_id`](Self::component_id) to obtain a handle for an imported
    /// component and add children under it.
    ///
    /// Fails if the document has no namespaced top-level object to root
    /// subsequently-added objects under.
    pub fn from_document(document: &Document) -> Result<Self, DesignError> {
        let namespace = imported_namespace(document).ok_or_else(|| DesignError {
            problems: vec![DesignProblem::Custom(
                "cannot import a document with no namespaced top-level object; \
                 the arena needs a namespace to root any objects added later"
                    .to_string(),
            )],
        })?;
        let slots = document
            .typed_objects()
            .iter()
            .cloned()
            .map(Slot::Object)
            .collect();
        Ok(Self {
            namespace,
            slots,
            problems: Vec::new(),
        })
    }

    /// Returns a handle to an imported (or built) `Component` by its identity,
    /// so children can be added under it after [`from_document`](Self::from_document).
    pub fn component_id(&self, identity: &Resource) -> Option<ComponentId> {
        self.slots
            .iter()
            .enumerate()
            .find_map(|(index, slot)| match slot {
                Slot::Object(SbolObject::Component(component))
                    if &component.identity == identity =>
                {
                    Some(ComponentId(index))
                }
                _ => None,
            })
    }

    /// Begins a top-level `Component`. Terminate with
    /// [`ComponentDraft::add`].
    pub fn component(&mut self, display_id: &str) -> ComponentDraft<'_> {
        ComponentDraft {
            design: self,
            display_id: display_id.to_string(),
            types: Vec::new(),
            roles: Vec::new(),
            sequences: Vec::new(),
            name: None,
            description: None,
        }
    }

    /// Begins a `Sequence`. Terminate with [`SequenceDraft::add`].
    pub fn sequence(&mut self, display_id: &str) -> SequenceDraft<'_> {
        SequenceDraft {
            design: self,
            display_id: display_id.to_string(),
            elements: None,
            encoding: None,
            name: None,
            description: None,
        }
    }

    /// Begins a `SubComponent` under `parent`. Terminate with
    /// [`SubComponentDraft::add`].
    pub fn sub_component(
        &mut self,
        parent: ComponentId,
        display_id: &str,
    ) -> SubComponentDraft<'_> {
        SubComponentDraft {
            design: self,
            parent: Some(parent),
            display_id: display_id.to_string(),
            instance_of: None,
            roles: Vec::new(),
            role_integration: None,
            name: None,
            description: None,
        }
    }

    /// Begins a `SubComponent` with no parent yet. Its `.add()` registers a
    /// *detached* feature, returning a [`FeatureId`] that can later be placed
    /// under a parent with [`place_feature`](Self::place_feature) (or handed to
    /// an extension verb that does so). This is the arena's analogue of
    /// building a `SubComponent` object before appending it to a component, and
    /// is how features carrying their own roles/`roleIntegration` are supplied
    /// to helpers like `engineered_region`. A detached feature that is never
    /// placed is dropped at [`finish`](Self::finish).
    pub fn detached_sub_component(&mut self, display_id: &str) -> SubComponentDraft<'_> {
        SubComponentDraft {
            design: self,
            parent: None,
            display_id: display_id.to_string(),
            instance_of: None,
            roles: Vec::new(),
            role_integration: None,
            name: None,
            description: None,
        }
    }

    /// Adds a `meets` ordering constraint (`left` immediately precedes
    /// `right`) under `parent`.
    pub fn meets(
        &mut self,
        parent: ComponentId,
        left: FeatureId,
        right: FeatureId,
    ) -> ConstraintId {
        self.constrain(parent, left, RESTRICTION_MEETS, right)
    }

    /// Adds a general topological constraint under `parent`.
    pub fn constrain(
        &mut self,
        parent: ComponentId,
        subject: FeatureId,
        restriction: Iri,
        object: FeatureId,
    ) -> ConstraintId {
        let display_id = format!("constraint_{}", self.parent_constraint_count(parent));

        let (parent_iri, subject_iri, object_iri) = match (
            self.identity_of(parent.0),
            self.identity_of(subject.0),
            self.identity_of(object.0),
        ) {
            (Some(p), Some(s), Some(o)) => (p, s, o),
            _ => return ConstraintId(self.push_dangling(display_id)),
        };

        let built = Constraint::builder(&parent_iri, display_id.as_str()).and_then(|b| {
            b.subject(subject_iri)
                .restriction(restriction)
                .constrained_object(object_iri)
                .build()
        });
        match built {
            Ok(constraint) => {
                let child = constraint.identity.clone();
                let index = self.push(SbolObject::Constraint(constraint));
                self.attach_constraint(parent, child);
                ConstraintId(index)
            }
            Err(source) => ConstraintId(self.push_failed(display_id, source)),
        }
    }

    /// Returns the `Component` behind a handle, or `None` if the handle refers
    /// to an object that failed to build. Read-only; useful for extension
    /// layers that need to inspect a referenced component (e.g. to copy its
    /// roles onto a sub-component).
    pub fn resolve_component(&self, component: ComponentId) -> Option<&Component> {
        match &self.slots[component.0] {
            Slot::Object(SbolObject::Component(component)) => Some(component),
            _ => None,
        }
    }

    /// Records a domain-level problem to be surfaced from [`finish`](Self::finish).
    /// Lets extension layers report their own preconditions into the same
    /// aggregated error.
    pub fn report(&mut self, message: impl Into<String>) {
        self.problems.push(DesignProblem::Custom(message.into()));
    }

    /// Places a detached feature (from [`detached_sub_component`](Self::detached_sub_component))
    /// under `parent`, building it and returning a handle to the built
    /// sub-component. The feature is used as configured — its own roles and
    /// `roleIntegration` are kept, not copied from the instantiated component.
    ///
    /// Reports a problem (and returns the original handle) if `feature` is not
    /// a detached feature — e.g. it already belongs to a parent or failed to
    /// build. Extension layers use this to adopt caller-supplied features.
    pub fn place_feature(&mut self, parent: ComponentId, feature: FeatureId) -> FeatureId {
        let spec = match std::mem::replace(&mut self.slots[feature.0], Slot::Failed) {
            Slot::Pending(spec) => spec,
            other => {
                // Restore the slot; the handle was not a detached feature.
                self.slots[feature.0] = other;
                self.report(
                    "place_feature expects a detached feature from \
                     `detached_sub_component`; the handle is already placed or invalid",
                );
                return feature;
            }
        };
        self.finish_sub_component(parent, spec)
    }

    /// Lowers the arena to an immutable [`Document`]. Returns every recorded
    /// problem if the design is not well-formed. Does not validate the result;
    /// call [`Document::check`] or [`Document::check_complete`] for that.
    pub fn finish(self) -> Result<Document, DesignError> {
        if !self.problems.is_empty() {
            return Err(DesignError {
                problems: self.problems,
            });
        }
        let objects = self
            .slots
            .into_iter()
            .filter_map(|slot| match slot {
                Slot::Object(object) => Some(object),
                Slot::Pending(_) | Slot::Failed => None,
            })
            .collect();
        Document::from_objects(objects).map_err(|source| DesignError {
            problems: vec![DesignProblem::Assembly(source)],
        })
    }

    // -- internal --------------------------------------------------------

    fn push(&mut self, object: SbolObject) -> usize {
        self.slots.push(Slot::Object(object));
        self.slots.len() - 1
    }

    fn push_pending(&mut self, spec: SubComponentSpec) -> usize {
        self.slots.push(Slot::Pending(spec));
        self.slots.len() - 1
    }

    fn push_failed(&mut self, display_id: String, source: BuildError) -> usize {
        self.problems
            .push(DesignProblem::Build { display_id, source });
        self.slots.push(Slot::Failed);
        self.slots.len() - 1
    }

    fn push_dangling(&mut self, display_id: String) -> usize {
        self.problems
            .push(DesignProblem::DanglingReference { display_id });
        self.slots.push(Slot::Failed);
        self.slots.len() - 1
    }

    fn identity_of(&self, index: usize) -> Option<Resource> {
        match &self.slots[index] {
            Slot::Object(object) => Some(object.identity().clone()),
            Slot::Pending(_) | Slot::Failed => None,
        }
    }

    fn parent_constraint_count(&self, parent: ComponentId) -> usize {
        match &self.slots[parent.0] {
            Slot::Object(SbolObject::Component(component)) => component.constraints.len(),
            _ => 0,
        }
    }

    fn attach_feature(&mut self, parent: ComponentId, child: Resource) {
        if let Slot::Object(SbolObject::Component(component)) = &mut self.slots[parent.0] {
            component.features.push(child);
        }
    }

    fn attach_constraint(&mut self, parent: ComponentId, child: Resource) {
        if let Slot::Object(SbolObject::Component(component)) = &mut self.slots[parent.0] {
            component.constraints.push(child);
        }
    }

    fn finish_component(
        &mut self,
        display_id: String,
        types: Vec<Iri>,
        roles: Vec<Iri>,
        sequences: Vec<SequenceId>,
        name: Option<String>,
        description: Option<String>,
    ) -> ComponentId {
        let sequence_iris: Vec<Resource> = sequences
            .into_iter()
            .filter_map(|id| self.identity_of(id.0))
            .collect();

        let built =
            Component::builder(self.namespace.as_str(), display_id.as_str()).and_then(|b| {
                let mut b = b
                    .types(types)
                    .component_roles(roles)
                    .sequences(sequence_iris);
                if let Some(name) = name {
                    b = b.name(name);
                }
                if let Some(description) = description {
                    b = b.description(description);
                }
                b.build()
            });
        match built {
            Ok(component) => ComponentId(self.push(SbolObject::Component(component))),
            Err(source) => ComponentId(self.push_failed(display_id, source)),
        }
    }

    fn finish_sequence(
        &mut self,
        display_id: String,
        elements: Option<String>,
        encoding: Option<Iri>,
        name: Option<String>,
        description: Option<String>,
    ) -> SequenceId {
        let built = Sequence::builder(self.namespace.as_str(), display_id.as_str()).and_then(|b| {
            let mut b = b;
            if let Some(elements) = elements {
                b = b.elements(elements);
            }
            if let Some(encoding) = encoding {
                b = b.encoding(encoding);
            }
            if let Some(name) = name {
                b = b.name(name);
            }
            if let Some(description) = description {
                b = b.description(description);
            }
            b.build()
        });
        match built {
            Ok(sequence) => SequenceId(self.push(SbolObject::Sequence(sequence))),
            Err(source) => SequenceId(self.push_failed(display_id, source)),
        }
    }

    fn finish_sub_component(&mut self, parent: ComponentId, spec: SubComponentSpec) -> FeatureId {
        let SubComponentSpec {
            display_id,
            instance_of,
            roles,
            role_integration,
            name,
            description,
        } = spec;
        let parent_iri = match self.identity_of(parent.0) {
            Some(iri) => iri,
            None => return FeatureId(self.push_dangling(display_id)),
        };
        let instance_iri = instance_of.and_then(|id| self.identity_of(id.0));
        // A sub-component carrying roles must declare a roleIntegration
        // (rule sbol3-10802); default to merging roles so callers copying SO
        // roles don't trip validation.
        let role_integration = match role_integration {
            Some(value) => Some(value),
            None if !roles.is_empty() => Some(ROLE_INTEGRATION_MERGE_ROLES),
            None => None,
        };

        let built = SubComponent::builder(&parent_iri, display_id.as_str()).and_then(|b| {
            let mut b = b.roles(roles);
            if let Some(instance_iri) = instance_iri {
                b = b.instance_of(instance_iri);
            }
            if let Some(role_integration) = role_integration {
                b = b.role_integration(role_integration);
            }
            if let Some(name) = name {
                b = b.name(name);
            }
            if let Some(description) = description {
                b = b.description(description);
            }
            b.build()
        });
        match built {
            Ok(sub) => {
                let child = sub.identity.clone();
                let index = self.push(SbolObject::SubComponent(sub));
                self.attach_feature(parent, child);
                FeatureId(index)
            }
            Err(source) => FeatureId(self.push_failed(display_id, source)),
        }
    }
}

// ---------------------------------------------------------------------------
// Drafts
// ---------------------------------------------------------------------------

/// In-progress `Component`, created by [`Design::component`].
#[must_use = "call `.add()` to register the component in the design"]
pub struct ComponentDraft<'d> {
    design: &'d mut Design,
    display_id: String,
    types: Vec<Iri>,
    roles: Vec<Iri>,
    sequences: Vec<SequenceId>,
    name: Option<String>,
    description: Option<String>,
}

impl ComponentDraft<'_> {
    /// Types this component as DNA (`SBO_DNA`).
    pub fn dna(mut self) -> Self {
        self.types.push(SBO_DNA);
        self
    }

    /// Types this component as RNA (`SBO_RNA`).
    pub fn rna(mut self) -> Self {
        self.types.push(SBO_RNA);
        self
    }

    /// Types this component as protein (`SBO_PROTEIN`).
    pub fn protein(mut self) -> Self {
        self.types.push(SBO_PROTEIN);
        self
    }

    /// Adds an arbitrary type IRI.
    pub fn type_(mut self, type_: Iri) -> Self {
        self.types.push(type_);
        self
    }

    /// Adds a role IRI (typically a Sequence Ontology term).
    pub fn role(mut self, role: Iri) -> Self {
        self.roles.push(role);
        self
    }

    /// References a `Sequence` created earlier in this design.
    pub fn sequence(mut self, sequence: SequenceId) -> Self {
        self.sequences.push(sequence);
        self
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

    /// Registers the component and returns its handle.
    pub fn add(self) -> ComponentId {
        let Self {
            design,
            display_id,
            types,
            roles,
            sequences,
            name,
            description,
        } = self;
        design.finish_component(display_id, types, roles, sequences, name, description)
    }
}

/// In-progress `Sequence`, created by [`Design::sequence`].
#[must_use = "call `.add()` to register the sequence in the design"]
pub struct SequenceDraft<'d> {
    design: &'d mut Design,
    display_id: String,
    elements: Option<String>,
    encoding: Option<Iri>,
    name: Option<String>,
    description: Option<String>,
}

impl SequenceDraft<'_> {
    /// Sets the raw sequence string.
    pub fn elements(mut self, elements: impl Into<String>) -> Self {
        self.elements = Some(elements.into());
        self
    }

    /// Encodes the elements as IUPAC DNA.
    pub fn dna(mut self) -> Self {
        self.encoding = Some(EDAM_IUPAC_DNA);
        self
    }

    /// Encodes the elements as IUPAC RNA. SBOL uses one nucleic-acid encoding
    /// for both DNA and RNA (see [`EDAM_IUPAC_RNA`]); the RNA-ness is carried by
    /// the `Component` type, not the sequence.
    pub fn rna(mut self) -> Self {
        self.encoding = Some(EDAM_IUPAC_RNA);
        self
    }

    /// Encodes the elements as IUPAC protein.
    pub fn protein(mut self) -> Self {
        self.encoding = Some(EDAM_IUPAC_PROTEIN);
        self
    }

    /// Sets an arbitrary encoding IRI.
    pub fn encoding(mut self, encoding: Iri) -> Self {
        self.encoding = Some(encoding);
        self
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

    /// Registers the sequence and returns its handle.
    pub fn add(self) -> SequenceId {
        let Self {
            design,
            display_id,
            elements,
            encoding,
            name,
            description,
        } = self;
        design.finish_sequence(display_id, elements, encoding, name, description)
    }
}

/// In-progress `SubComponent`, created by [`Design::sub_component`].
#[must_use = "call `.add()` to register the sub-component in the design"]
pub struct SubComponentDraft<'d> {
    design: &'d mut Design,
    parent: Option<ComponentId>,
    display_id: String,
    instance_of: Option<ComponentId>,
    roles: Vec<Iri>,
    role_integration: Option<Iri>,
    name: Option<String>,
    description: Option<String>,
}

impl SubComponentDraft<'_> {
    /// Sets the `Component` this sub-component instantiates (required).
    pub fn instance_of(mut self, component: ComponentId) -> Self {
        self.instance_of = Some(component);
        self
    }

    /// Sets `roleIntegration` explicitly. When roles are present and this is
    /// left unset, the design defaults it to `mergeRoles`.
    pub fn role_integration(mut self, role_integration: Iri) -> Self {
        self.role_integration = Some(role_integration);
        self
    }

    /// Adds a role IRI carried by the sub-component.
    pub fn role(mut self, role: Iri) -> Self {
        self.roles.push(role);
        self
    }

    /// Sets all roles carried by the sub-component, replacing any set so far.
    pub fn roles(mut self, roles: impl IntoIterator<Item = Iri>) -> Self {
        self.roles = roles.into_iter().collect();
        self
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

    /// Registers the sub-component and returns its handle. A sub-component with
    /// a parent (from [`Design::sub_component`]) is built immediately; a
    /// detached one (from [`Design::detached_sub_component`]) is held until
    /// [`Design::place_feature`] gives it a parent.
    pub fn add(self) -> FeatureId {
        let Self {
            design,
            parent,
            display_id,
            instance_of,
            roles,
            role_integration,
            name,
            description,
        } = self;
        let spec = SubComponentSpec {
            display_id,
            instance_of,
            roles,
            role_integration,
            name,
            description,
        };
        match parent {
            Some(parent) => design.finish_sub_component(parent, spec),
            None => FeatureId(design.push_pending(spec)),
        }
    }
}

/// The fields needed to build a `SubComponent`, bundled so the internal
/// constructor stays single-argument.
struct SubComponentSpec {
    display_id: String,
    instance_of: Option<ComponentId>,
    roles: Vec<Iri>,
    role_integration: Option<Iri>,
    name: Option<String>,
    description: Option<String>,
}

// ---------------------------------------------------------------------------
// Display ID sanitization
// ---------------------------------------------------------------------------

/// Infers a namespace for an imported document from the first namespaced
/// top-level object it can find.
fn imported_namespace(document: &Document) -> Option<Namespace> {
    let iri = document
        .components()
        .find_map(|component| component.namespace())
        .or_else(|| {
            document
                .sequences()
                .find_map(|sequence| sequence.namespace())
        })
        .or_else(|| {
            document
                .collections()
                .find_map(|collection| collection.namespace())
        })?;
    Namespace::new(iri.as_str()).ok()
}

/// Turns free-text into a valid SBOL `displayId`: the first character must be
/// an ASCII letter or underscore and the rest ASCII alphanumeric or
/// underscore (rule sbol3-10201). Invalid characters become `_`; a leading
/// digit is prefixed with `_`; empty input becomes `_`.
pub fn sanitize_display_id(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    match out.chars().next() {
        None => "_".to_string(),
        Some(first) if first.is_ascii_digit() => format!("_{out}"),
        Some(_) => out,
    }
}

#[cfg(test)]
mod tests;
