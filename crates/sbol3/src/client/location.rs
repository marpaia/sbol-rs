use crate::client::accessors::impl_sbol_identified;
use crate::client::builder::{CutBuilder, EntireSequenceBuilder, RangeBuilder};
use crate::client::shared::first_i64;
use crate::client::to_rdf::{Emitter, emit_identified, emit_location, seed_triples};
use crate::client::{IdentifiedData, LocationData, ToRdf, TryFromObject};
use crate::error::BuildError;
use crate::identity::DisplayId;
use crate::vocab::*;
use crate::{Object, Resource, SbolClass, Triple};
use sbol_core::error::BuildError as LexError;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Cut {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub location: LocationData,
    pub at: Option<i64>,
}

impl Cut {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        at: i64,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.at(at).build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<CutBuilder, BuildError> {
        CutBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for Cut {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::Cut);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::Cut);
        emit_identified(&mut e, &self.identified)?;
        emit_location(&mut e, &self.location)?;
        e.i64(SBOL_AT, self.at)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Cut {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            location: LocationData::from_object(object),
            at: first_i64(object, SBOL_AT),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct EntireSequence {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub location: LocationData,
}

impl EntireSequence {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?.build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<EntireSequenceBuilder, BuildError> {
        EntireSequenceBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for EntireSequence {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::EntireSequence);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::EntireSequence);
        emit_identified(&mut e, &self.identified)?;
        emit_location(&mut e, &self.location)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for EntireSequence {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            location: LocationData::from_object(object),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Range {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub location: LocationData,
    pub start: Option<i64>,
    pub end: Option<i64>,
}

impl Range {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        start: i64,
        end: i64,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?
            .start(start)
            .end(end)
            .build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<RangeBuilder, BuildError> {
        RangeBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for Range {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::Range);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::Range);
        emit_identified(&mut e, &self.identified)?;
        emit_location(&mut e, &self.location)?;
        e.i64(SBOL_START, self.start)?;
        e.i64(SBOL_END, self.end)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Range {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            location: LocationData::from_object(object),
            start: first_i64(object, SBOL_START),
            end: first_i64(object, SBOL_END),
        })
    }
}

impl_sbol_identified!(Cut, EntireSequence, Range);
