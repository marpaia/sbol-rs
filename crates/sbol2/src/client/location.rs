use crate::client::accessors::impl_sbol_identified;
use crate::client::shared::first_i64;
use crate::client::to_rdf::{Emitter, emit_identified, emit_location, seed_triples};
use crate::client::{IdentifiedData, LocationData, ToRdf, TryFromObject};
use crate::vocab::*;
use crate::{Object, Resource, Sbol2Class, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Range {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub location: LocationData,
    pub start: Option<i64>,
    pub end: Option<i64>,
}

impl ToRdf for Range {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::Range);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::Range);
        emit_identified(&mut e, &self.identified)?;
        emit_location(&mut e, &self.location)?;
        e.i64(SBOL2_START, self.start)?;
        e.i64(SBOL2_END, self.end)?;
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
            start: first_i64(object, SBOL2_START),
            end: first_i64(object, SBOL2_END),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Cut {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub location: LocationData,
    pub at: Option<i64>,
}

impl ToRdf for Cut {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::Cut);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::Cut);
        emit_identified(&mut e, &self.identified)?;
        emit_location(&mut e, &self.location)?;
        e.i64(SBOL2_AT, self.at)?;
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
            at: first_i64(object, SBOL2_AT),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct GenericLocation {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub location: LocationData,
}

impl ToRdf for GenericLocation {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::GenericLocation);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::GenericLocation);
        emit_identified(&mut e, &self.identified)?;
        emit_location(&mut e, &self.location)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for GenericLocation {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            location: LocationData::from_object(object),
        })
    }
}

impl_sbol_identified!(Range, Cut, GenericLocation);
