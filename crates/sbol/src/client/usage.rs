use crate::client::IdentifiedData;
use crate::client::accessors::impl_sbol_identified;
use crate::client::builder::{
    ConstraintBuilder, InteractionBuilder, InterfaceBuilder, ParticipationBuilder,
    VariableFeatureBuilder,
};
use crate::client::shared::{iris, resources};
use crate::client::to_rdf::{Emitter, emit_identified, seed_triples};
use crate::client::{ToRdf, TryFromObject};
use crate::document::Document;
use crate::error::BuildError;
use sbol_core::error::BuildError as LexError;
use crate::identity::DisplayId;
use crate::vocab::*;
use crate::{Iri, Object, Resource, SbolClass, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Constraint {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub subject: Option<Resource>,
    pub constrained_object: Option<Resource>,
    pub restriction: Option<Iri>,
}

impl Constraint {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        subject: Resource,
        constrained_object: Resource,
        restriction: Iri,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?
            .subject(subject)
            .constrained_object(constrained_object)
            .restriction(restriction)
            .build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<ConstraintBuilder, BuildError> {
        ConstraintBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for Constraint {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::Constraint);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::Constraint);
        emit_identified(&mut e, &self.identified)?;
        e.resource(SBOL_SUBJECT, self.subject.as_ref())?;
        e.resource(SBOL_OBJECT, self.constrained_object.as_ref())?;
        e.iri(SBOL_RESTRICTION, self.restriction.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Constraint {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            subject: object.first_resource(SBOL_SUBJECT).cloned(),
            constrained_object: object.first_resource(SBOL_OBJECT).cloned(),
            restriction: object.first_iri(SBOL_RESTRICTION).cloned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Interaction {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub types: Vec<Iri>,
    pub participations: Vec<Resource>,
}

impl Interaction {
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
    ) -> Result<InteractionBuilder, BuildError> {
        InteractionBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for Interaction {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::Interaction);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::Interaction);
        emit_identified(&mut e, &self.identified)?;
        e.iris(SBOL_TYPE, &self.types)?;
        e.resources(SBOL_HAS_PARTICIPATION, &self.participations)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Interaction {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            types: iris(object, SBOL_TYPE),
            participations: resources(object, SBOL_HAS_PARTICIPATION),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Interface {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub inputs: Vec<Resource>,
    pub outputs: Vec<Resource>,
    pub nondirectional: Vec<Resource>,
}

impl Interface {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<InterfaceBuilder, BuildError> {
        InterfaceBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for Interface {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::Interface);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::Interface);
        emit_identified(&mut e, &self.identified)?;
        e.resources(SBOL_INPUT, &self.inputs)?;
        e.resources(SBOL_OUTPUT, &self.outputs)?;
        e.resources(SBOL_NONDIRECTIONAL, &self.nondirectional)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Interface {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            inputs: resources(object, SBOL_INPUT),
            outputs: resources(object, SBOL_OUTPUT),
            nondirectional: resources(object, SBOL_NONDIRECTIONAL),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Participation {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub roles: Vec<Iri>,
    pub participant: Option<Resource>,
    pub higher_order_participant: Option<Resource>,
}

impl Participation {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        roles: impl IntoIterator<Item = Iri>,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.roles(roles).build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<ParticipationBuilder, BuildError> {
        ParticipationBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for Participation {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::Participation);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::Participation);
        emit_identified(&mut e, &self.identified)?;
        e.iris(SBOL_ROLE, &self.roles)?;
        e.resource(SBOL_PARTICIPANT, self.participant.as_ref())?;
        e.resource(
            SBOL_HIGHER_ORDER_PARTICIPANT,
            self.higher_order_participant.as_ref(),
        )?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Participation {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            roles: iris(object, SBOL_ROLE),
            participant: object.first_resource(SBOL_PARTICIPANT).cloned(),
            higher_order_participant: object
                .first_resource(SBOL_HIGHER_ORDER_PARTICIPANT)
                .cloned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct VariableFeature {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub cardinality: Option<Iri>,
    pub variable: Option<Resource>,
    pub variants: Vec<Resource>,
    pub variant_collections: Vec<Resource>,
    pub variant_derivations: Vec<Resource>,
    pub variant_measures: Vec<Resource>,
}

impl VariableFeature {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        cardinality: Iri,
        variable: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?
            .cardinality(cardinality)
            .variable(variable)
            .build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<VariableFeatureBuilder, BuildError> {
        VariableFeatureBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for VariableFeature {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::VariableFeature);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::VariableFeature);
        emit_identified(&mut e, &self.identified)?;
        e.iri(SBOL_CARDINALITY, self.cardinality.as_ref())?;
        e.resource(SBOL_VARIABLE, self.variable.as_ref())?;
        e.resources(SBOL_VARIANT, &self.variants)?;
        e.resources(SBOL_VARIANT_COLLECTION, &self.variant_collections)?;
        e.resources(SBOL_VARIANT_DERIVATION, &self.variant_derivations)?;
        e.resources(SBOL_VARIANT_MEASURE, &self.variant_measures)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for VariableFeature {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            cardinality: object.first_iri(SBOL_CARDINALITY).cloned(),
            variable: object.first_resource(SBOL_VARIABLE).cloned(),
            variants: resources(object, SBOL_VARIANT),
            variant_collections: resources(object, SBOL_VARIANT_COLLECTION),
            variant_derivations: resources(object, SBOL_VARIANT_DERIVATION),
            variant_measures: resources(object, SBOL_VARIANT_MEASURE),
        })
    }
}

impl_sbol_identified!(
    Constraint,
    Interaction,
    Interface,
    Participation,
    VariableFeature
);

impl Participation {
    /// Returns the `Interaction` that owns this participation via
    /// `sbol:hasParticipation`, or `None` if no enclosing interaction is
    /// in the document. Participations must belong to exactly one
    /// interaction, so a `Some` result is unambiguous.
    ///
    /// Linear in the number of interactions in the document.
    pub fn parent_interaction<'a>(&self, doc: &'a Document) -> Option<&'a Interaction> {
        doc.interactions().find(|interaction| {
            interaction
                .participations
                .iter()
                .any(|p| p == &self.identity)
        })
    }
}
