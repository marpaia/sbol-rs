//! Builders for the SBOL 2 TopLevel classes.

use super::{
    identified_seed, identified_setters, missing, top_level_seed, top_level_setters,
};
use crate::client::identity::{DEFAULT_VERSION, build_top_level_identity};
use crate::client::{
    Attachment, Collection, CombinatorialDerivation, ComponentDefinition, Experiment,
    ExperimentalData, IdentifiedData, Implementation, Model, ModuleDefinition, Sequence,
    TopLevelData,
};
use crate::error::BuildError;
use crate::identity::{DisplayId, Namespace};
use crate::{Iri, Resource, Sbol2Class, Term};
use sbol_core::error::BuildError as LexError;

fn seed(namespace: &Namespace, display_id: &DisplayId) -> (Resource, IdentifiedData, TopLevelData) {
    let (identity, persistent) = build_top_level_identity(namespace, display_id, DEFAULT_VERSION);
    (
        identity,
        identified_seed(display_id, persistent),
        top_level_seed(),
    )
}

macro_rules! top_level_builder {
    (
        $class:ident, $builder:ident, $sbol_class:ident,
        fields { $( $field:ident : $fty:ty = $default:expr ),* $(,)? }
        setters { $( $setter:item )* }
        build { $( $req:ident )* }
    ) => {
        /// Builder for the corresponding SBOL 2 TopLevel class.
        #[derive(Clone, Debug)]
        pub struct $builder {
            identity: Resource,
            identified: IdentifiedData,
            top_level: TopLevelData,
            $( $field: $fty, )*
        }

        impl $builder {
            pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
                let (identity, identified, top_level) = seed(&namespace, &display_id);
                Self { identity, identified, top_level, $( $field: $default, )* }
            }

            identified_setters!();
            top_level_setters!();
            $( $setter )*

            pub fn build(self) -> Result<$class, BuildError> {
                $(
                    if is_unset(&self.$req) {
                        return Err(missing(&self.identity, Sbol2Class::$sbol_class, stringify!($req)));
                    }
                )*
                Ok($class {
                    identity: self.identity,
                    identified: self.identified,
                    top_level: self.top_level,
                    $( $field: self.$field, )*
                })
            }
        }

        impl $class {
            pub fn builder(
                namespace: impl TryInto<Namespace, Error = LexError>,
                display_id: impl TryInto<DisplayId, Error = LexError>,
            ) -> Result<$builder, BuildError> {
                Ok($builder::seed(namespace.try_into()?, display_id.try_into()?))
            }
        }
    };
}

/// A field is "unset" if it is a `None` option or an empty collection.
trait Unset {
    fn is_unset(&self) -> bool;
}
impl<T> Unset for Option<T> {
    fn is_unset(&self) -> bool {
        self.is_none()
    }
}
impl<T> Unset for Vec<T> {
    fn is_unset(&self) -> bool {
        self.is_empty()
    }
}
fn is_unset(value: &impl Unset) -> bool {
    value.is_unset()
}

top_level_builder! {
    Sequence, SequenceBuilder, Sequence,
    fields { elements: Option<String> = None, encoding: Option<Iri> = None }
    setters {
        pub fn elements(mut self, value: impl Into<String>) -> Self {
            self.elements = Some(value.into());
            self
        }
        pub fn encoding(mut self, value: Iri) -> Self {
            self.encoding = Some(value);
            self
        }
    }
    build {}
}

top_level_builder! {
    ComponentDefinition, ComponentDefinitionBuilder, ComponentDefinition,
    fields {
        types: Vec<Iri> = Vec::new(),
        roles: Vec<Iri> = Vec::new(),
        sequences: Vec<Resource> = Vec::new(),
        components: Vec<Resource> = Vec::new(),
        sequence_annotations: Vec<Resource> = Vec::new(),
        sequence_constraints: Vec<Resource> = Vec::new(),
    }
    setters {
        pub fn types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
            self.types = values.into_iter().collect();
            self
        }
        pub fn add_type(mut self, value: Iri) -> Self { self.types.push(value); self }
        pub fn roles(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
            self.roles = values.into_iter().collect();
            self
        }
        pub fn add_role(mut self, value: Iri) -> Self { self.roles.push(value); self }
        pub fn sequences(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.sequences = values.into_iter().collect();
            self
        }
        pub fn add_sequence(mut self, value: Resource) -> Self { self.sequences.push(value); self }
        pub fn components(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.components = values.into_iter().collect();
            self
        }
        pub fn add_component(mut self, value: Resource) -> Self { self.components.push(value); self }
        pub fn sequence_annotations(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.sequence_annotations = values.into_iter().collect();
            self
        }
        pub fn add_sequence_annotation(mut self, value: Resource) -> Self {
            self.sequence_annotations.push(value);
            self
        }
        pub fn sequence_constraints(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.sequence_constraints = values.into_iter().collect();
            self
        }
        pub fn add_sequence_constraint(mut self, value: Resource) -> Self {
            self.sequence_constraints.push(value);
            self
        }
    }
    build { types }
}

top_level_builder! {
    ModuleDefinition, ModuleDefinitionBuilder, ModuleDefinition,
    fields {
        roles: Vec<Iri> = Vec::new(),
        modules: Vec<Resource> = Vec::new(),
        functional_components: Vec<Resource> = Vec::new(),
        interactions: Vec<Resource> = Vec::new(),
        models: Vec<Resource> = Vec::new(),
    }
    setters {
        pub fn roles(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
            self.roles = values.into_iter().collect();
            self
        }
        pub fn add_role(mut self, value: Iri) -> Self { self.roles.push(value); self }
        pub fn modules(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.modules = values.into_iter().collect();
            self
        }
        pub fn add_module(mut self, value: Resource) -> Self { self.modules.push(value); self }
        pub fn functional_components(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.functional_components = values.into_iter().collect();
            self
        }
        pub fn add_functional_component(mut self, value: Resource) -> Self {
            self.functional_components.push(value);
            self
        }
        pub fn interactions(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.interactions = values.into_iter().collect();
            self
        }
        pub fn add_interaction(mut self, value: Resource) -> Self { self.interactions.push(value); self }
        pub fn models(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.models = values.into_iter().collect();
            self
        }
        pub fn add_model(mut self, value: Resource) -> Self { self.models.push(value); self }
    }
    build {}
}

top_level_builder! {
    Model, ModelBuilder, Model,
    fields {
        source: Option<Resource> = None,
        language: Option<Iri> = None,
        framework: Option<Iri> = None,
    }
    setters {
        pub fn source(mut self, value: Resource) -> Self { self.source = Some(value); self }
        pub fn language(mut self, value: Iri) -> Self { self.language = Some(value); self }
        pub fn framework(mut self, value: Iri) -> Self { self.framework = Some(value); self }
    }
    build { source language framework }
}

top_level_builder! {
    Collection, CollectionBuilder, Collection,
    fields { members: Vec<Resource> = Vec::new() }
    setters {
        pub fn members(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.members = values.into_iter().collect();
            self
        }
        pub fn add_member(mut self, value: Resource) -> Self { self.members.push(value); self }
    }
    build {}
}

top_level_builder! {
    CombinatorialDerivation, CombinatorialDerivationBuilder, CombinatorialDerivation,
    fields {
        template: Option<Resource> = None,
        variable_components: Vec<Resource> = Vec::new(),
        strategy: Option<Iri> = None,
    }
    setters {
        pub fn template(mut self, value: Resource) -> Self { self.template = Some(value); self }
        pub fn variable_components(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.variable_components = values.into_iter().collect();
            self
        }
        pub fn add_variable_component(mut self, value: Resource) -> Self {
            self.variable_components.push(value);
            self
        }
        pub fn strategy(mut self, value: Iri) -> Self { self.strategy = Some(value); self }
    }
    build { template }
}

top_level_builder! {
    Implementation, ImplementationBuilder, Implementation,
    fields { built: Option<Resource> = None }
    setters {
        pub fn built(mut self, value: Resource) -> Self { self.built = Some(value); self }
    }
    build {}
}

top_level_builder! {
    Attachment, AttachmentBuilder, Attachment,
    fields {
        source: Option<Resource> = None,
        format: Option<Iri> = None,
        size: Option<i64> = None,
        hash: Option<String> = None,
    }
    setters {
        pub fn source(mut self, value: Resource) -> Self { self.source = Some(value); self }
        pub fn format(mut self, value: Iri) -> Self { self.format = Some(value); self }
        pub fn size(mut self, value: i64) -> Self { self.size = Some(value); self }
        pub fn hash(mut self, value: impl Into<String>) -> Self { self.hash = Some(value.into()); self }
    }
    build { source }
}

top_level_builder! {
    ExperimentalData, ExperimentalDataBuilder, ExperimentalData,
    fields {}
    setters {}
    build {}
}

top_level_builder! {
    Experiment, ExperimentBuilder, Experiment,
    fields { experimental_data: Vec<Resource> = Vec::new() }
    setters {
        pub fn experimental_data(mut self, values: impl IntoIterator<Item = Resource>) -> Self {
            self.experimental_data = values.into_iter().collect();
            self
        }
        pub fn add_experimental_data(mut self, value: Resource) -> Self {
            self.experimental_data.push(value);
            self
        }
    }
    build {}
}

impl Sequence {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.build()
    }
}

impl ComponentDefinition {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        types: impl IntoIterator<Item = Iri>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.types(types).build()
    }
}

impl ModuleDefinition {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.build()
    }
}

impl Model {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        source: Resource,
        language: Iri,
        framework: Iri,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .source(source)
            .language(language)
            .framework(framework)
            .build()
    }
}

impl Collection {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.build()
    }
}

impl CombinatorialDerivation {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        template: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.template(template).build()
    }
}

impl Implementation {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.build()
    }
}

impl Attachment {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        source: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.source(source).build()
    }
}

impl ExperimentalData {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.build()
    }
}

impl Experiment {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.build()
    }
}
