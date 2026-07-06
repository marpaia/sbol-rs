//! The OM `Prefix` hierarchy: `Prefix` and its concrete subclasses
//! (`SIPrefix`, `BinaryPrefix`). Each is a TopLevel carrying the shared
//! [`PrefixData`] symbol/label/factor fields.

use super::{PrefixData, emit_prefix_fields};
use crate::client::accessors::{impl_sbol_identified, impl_sbol_top_level};
use crate::client::to_rdf::{Emitter, emit_identified, emit_top_level, seed_triples};
use crate::client::{IdentifiedData, ToRdf, TopLevelData, TryFromObject};
use crate::{Object, Resource, Sbol2Class, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Prefix {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub prefix: PrefixData,
}

impl ToRdf for Prefix {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::OmPrefix);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::OmPrefix);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        emit_prefix_fields(&mut e, &self.prefix)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Prefix {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            prefix: PrefixData::from_object(object),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SIPrefix {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub prefix: PrefixData,
}

impl ToRdf for SIPrefix {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::OmSiPrefix);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::OmSiPrefix);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        emit_prefix_fields(&mut e, &self.prefix)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for SIPrefix {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            prefix: PrefixData::from_object(object),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinaryPrefix {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub prefix: PrefixData,
}

impl ToRdf for BinaryPrefix {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::OmBinaryPrefix);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::OmBinaryPrefix);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        emit_prefix_fields(&mut e, &self.prefix)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for BinaryPrefix {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            prefix: PrefixData::from_object(object),
        })
    }
}

impl_sbol_identified!(Prefix, SIPrefix, BinaryPrefix);
impl_sbol_top_level!(Prefix, SIPrefix, BinaryPrefix);
