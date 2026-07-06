use crate::client::accessors::impl_sbol_identified;
use crate::client::builder::{
    ComponentReferenceBuilder, ExternallyDefinedBuilder, LocalSubComponentBuilder,
    SequenceFeatureBuilder, SubComponentBuilder,
};
use crate::client::shared::{iris, resources};
use crate::client::to_rdf::{Emitter, emit_feature, emit_identified, seed_triples};
use crate::client::top_level::Component;
use crate::client::{FeatureData, IdentifiedData, ToRdf, TryFromObject};
use crate::document::Document;
use crate::error::BuildError;
use crate::identity::DisplayId;
use crate::vocab::*;
use crate::{Iri, Object, Resource, SbolClass, Triple};
use sbol_core::error::BuildError as LexError;

/// Looks up the `Component` that owns `feature` via `sbol:hasFeature`.
///
/// Returns `None` if no enclosing component is present in the document.
/// Linear in the number of components, sufficient for typical-sized
/// SBOL documents; a cached reverse index can be layered on later if
/// large-document workflows demand it.
fn parent_component<'a>(doc: &'a Document, feature: &Resource) -> Option<&'a Component> {
    doc.components()
        .find(|component| component.features.iter().any(|f| f == feature))
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ComponentReference {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub feature: FeatureData,
    pub in_child_of: Option<Resource>,
    pub refers_to: Option<Resource>,
}

impl ComponentReference {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        in_child_of: Resource,
        refers_to: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?
            .in_child_of(in_child_of)
            .refers_to(refers_to)
            .build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<ComponentReferenceBuilder, BuildError> {
        ComponentReferenceBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for ComponentReference {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::ComponentReference);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::ComponentReference);
        emit_identified(&mut e, &self.identified)?;
        emit_feature(&mut e, &self.feature)?;
        e.resource(SBOL_IN_CHILD_OF, self.in_child_of.as_ref())?;
        e.resource(SBOL_REFERS_TO, self.refers_to.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for ComponentReference {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            feature: FeatureData::from_object(object),
            in_child_of: object.first_resource(SBOL_IN_CHILD_OF).cloned(),
            refers_to: object.first_resource(SBOL_REFERS_TO).cloned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExternallyDefined {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub feature: FeatureData,
    pub definition: Option<Resource>,
    pub types: Vec<Iri>,
}

impl ExternallyDefined {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        definition: Resource,
        types: impl IntoIterator<Item = Iri>,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?
            .definition(definition)
            .types(types)
            .build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<ExternallyDefinedBuilder, BuildError> {
        ExternallyDefinedBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for ExternallyDefined {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::ExternallyDefined);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::ExternallyDefined);
        emit_identified(&mut e, &self.identified)?;
        emit_feature(&mut e, &self.feature)?;
        e.resource(SBOL_DEFINITION, self.definition.as_ref())?;
        e.iris(SBOL_TYPE, &self.types)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for ExternallyDefined {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            feature: FeatureData::from_object(object),
            definition: object.first_resource(SBOL_DEFINITION).cloned(),
            types: iris(object, SBOL_TYPE),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LocalSubComponent {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub feature: FeatureData,
    pub types: Vec<Iri>,
    pub locations: Vec<Resource>,
}

impl LocalSubComponent {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        types: impl IntoIterator<Item = Iri>,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.types(types).build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<LocalSubComponentBuilder, BuildError> {
        LocalSubComponentBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for LocalSubComponent {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::LocalSubComponent);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::LocalSubComponent);
        emit_identified(&mut e, &self.identified)?;
        emit_feature(&mut e, &self.feature)?;
        e.iris(SBOL_TYPE, &self.types)?;
        e.resources(SBOL_HAS_LOCATION, &self.locations)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for LocalSubComponent {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            feature: FeatureData::from_object(object),
            types: iris(object, SBOL_TYPE),
            locations: resources(object, SBOL_HAS_LOCATION),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SequenceFeature {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub feature: FeatureData,
    pub locations: Vec<Resource>,
}

impl SequenceFeature {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        locations: impl IntoIterator<Item = Resource>,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?
            .locations(locations)
            .build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<SequenceFeatureBuilder, BuildError> {
        SequenceFeatureBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for SequenceFeature {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::SequenceFeature);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::SequenceFeature);
        emit_identified(&mut e, &self.identified)?;
        emit_feature(&mut e, &self.feature)?;
        e.resources(SBOL_HAS_LOCATION, &self.locations)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for SequenceFeature {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            feature: FeatureData::from_object(object),
            locations: resources(object, SBOL_HAS_LOCATION),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SubComponent {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub feature: FeatureData,
    pub instance_of: Option<Resource>,
    pub role_integration: Option<Iri>,
    pub locations: Vec<Resource>,
    pub source_locations: Vec<Resource>,
}

impl SubComponent {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        instance_of: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?
            .instance_of(instance_of)
            .build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<SubComponentBuilder, BuildError> {
        SubComponentBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for SubComponent {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::SubComponent);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::SubComponent);
        emit_identified(&mut e, &self.identified)?;
        emit_feature(&mut e, &self.feature)?;
        e.resource(SBOL_INSTANCE_OF, self.instance_of.as_ref())?;
        e.iri(SBOL_ROLE_INTEGRATION, self.role_integration.as_ref())?;
        e.resources(SBOL_HAS_LOCATION, &self.locations)?;
        e.resources(SBOL_SOURCE_LOCATION, &self.source_locations)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for SubComponent {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            feature: FeatureData::from_object(object),
            instance_of: object.first_resource(SBOL_INSTANCE_OF).cloned(),
            role_integration: object.first_iri(SBOL_ROLE_INTEGRATION).cloned(),
            locations: resources(object, SBOL_HAS_LOCATION),
            source_locations: resources(object, SBOL_SOURCE_LOCATION),
        })
    }
}

impl_sbol_identified!(
    ComponentReference,
    ExternallyDefined,
    LocalSubComponent,
    SequenceFeature,
    SubComponent,
);

macro_rules! impl_parent_component {
    ($($ty:ident),+ $(,)?) => {
        $(
            impl $ty {
                /// Returns the `Component` that owns this feature via
                /// `sbol:hasFeature`, or `None` if no enclosing component is
                /// in the document. SBOL features must belong to exactly one
                /// component, so a `Some` result is unambiguous.
                pub fn parent_component<'a>(&self, doc: &'a Document) -> Option<&'a Component> {
                    parent_component(doc, &self.identity)
                }
            }
        )+
    };
}

impl_parent_component!(
    ComponentReference,
    ExternallyDefined,
    LocalSubComponent,
    SequenceFeature,
    SubComponent,
);
