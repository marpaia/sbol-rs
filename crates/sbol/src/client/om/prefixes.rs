//! The OM `Prefix` hierarchy: `Prefix` and its concrete subclasses
//! (`SIPrefix`, `BinaryPrefix`). Each is a TopLevel carrying the shared
//! [`PrefixData`] label/symbol/factor fields.

use super::{PrefixData, emit_prefix_fields};
use crate::client::builder::{BinaryPrefixBuilder, PrefixBuilder, SIPrefixBuilder};
use crate::client::to_rdf::{Emitter, emit_identified, emit_top_level, seed_triples};
use crate::client::{IdentifiedData, ToRdf, TopLevelData, TryFromObject};
use crate::error::BuildError;
use sbol_core::error::BuildError as LexError;
use crate::identity::{DisplayId, Namespace};
use crate::{Object, Resource, SbolClass, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Prefix {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub prefix: PrefixData,
}

impl Prefix {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        label: impl Into<String>,
        symbol: impl Into<String>,
        has_factor: f64,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .label(label)
            .symbol(symbol)
            .has_factor(has_factor)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<PrefixBuilder, BuildError> {
        Ok(PrefixBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for Prefix {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::OmPrefix);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::OmPrefix);
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

impl SIPrefix {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        label: impl Into<String>,
        symbol: impl Into<String>,
        has_factor: f64,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .label(label)
            .symbol(symbol)
            .has_factor(has_factor)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<SIPrefixBuilder, BuildError> {
        Ok(SIPrefixBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for SIPrefix {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::OmSiPrefix);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::OmSiPrefix);
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

impl BinaryPrefix {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        label: impl Into<String>,
        symbol: impl Into<String>,
        has_factor: f64,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .label(label)
            .symbol(symbol)
            .has_factor(has_factor)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<BinaryPrefixBuilder, BuildError> {
        Ok(BinaryPrefixBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for BinaryPrefix {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::OmBinaryPrefix);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::OmBinaryPrefix);
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
