//! The OM `Unit` hierarchy: `Unit` and its concrete subclasses
//! (`SingularUnit`, `CompoundUnit`, `UnitDivision`, `UnitExponentiation`,
//! `UnitMultiplication`, `PrefixedUnit`). Each is a TopLevel carrying the
//! shared [`UnitData`] label/symbol fields plus its own structural
//! references.

use super::{UnitData, emit_unit_fields};
use crate::client::builder::{
    CompoundUnitBuilder, PrefixedUnitBuilder, SingularUnitBuilder, UnitBuilder,
    UnitDivisionBuilder, UnitExponentiationBuilder, UnitMultiplicationBuilder,
};
use crate::client::to_rdf::{Emitter, emit_identified, emit_top_level, seed_triples};
use crate::client::{IdentifiedData, ToRdf, TopLevelData, TryFromObject};
use crate::error::BuildError;
use crate::identity::{DisplayId, Namespace};
use crate::vocab::*;
use crate::{Object, Resource, SbolClass, Triple};
use sbol_core::error::BuildError as LexError;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Unit {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub unit: UnitData,
}

impl Unit {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        label: impl Into<String>,
        symbol: impl Into<String>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .label(label)
            .symbol(symbol)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<UnitBuilder, BuildError> {
        Ok(UnitBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for Unit {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::OmUnit);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::OmUnit);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        emit_unit_fields(&mut e, &self.unit)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Unit {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            unit: UnitData::from_object(object),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SingularUnit {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub unit: UnitData,
    pub has_unit: Option<Resource>,
    pub has_factor: Option<String>,
}

impl SingularUnit {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        label: impl Into<String>,
        symbol: impl Into<String>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .label(label)
            .symbol(symbol)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<SingularUnitBuilder, BuildError> {
        Ok(SingularUnitBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for SingularUnit {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::OmSingularUnit);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::OmSingularUnit);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        emit_unit_fields(&mut e, &self.unit)?;
        e.resource(OM_HAS_UNIT, self.has_unit.as_ref())?;
        e.literal(OM_HAS_FACTOR, self.has_factor.as_deref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for SingularUnit {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            unit: UnitData::from_object(object),
            has_unit: object.first_resource(OM_HAS_UNIT).cloned(),
            has_factor: object
                .first_literal_value(OM_HAS_FACTOR)
                .map(ToOwned::to_owned),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct CompoundUnit {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub unit: UnitData,
}

impl CompoundUnit {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        label: impl Into<String>,
        symbol: impl Into<String>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .label(label)
            .symbol(symbol)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<CompoundUnitBuilder, BuildError> {
        Ok(CompoundUnitBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for CompoundUnit {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::OmCompoundUnit);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::OmCompoundUnit);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        emit_unit_fields(&mut e, &self.unit)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for CompoundUnit {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            unit: UnitData::from_object(object),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnitDivision {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub unit: UnitData,
    pub has_numerator: Option<Resource>,
    pub has_denominator: Option<Resource>,
}

impl UnitDivision {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        label: impl Into<String>,
        symbol: impl Into<String>,
        has_numerator: Resource,
        has_denominator: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .label(label)
            .symbol(symbol)
            .has_numerator(has_numerator)
            .has_denominator(has_denominator)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<UnitDivisionBuilder, BuildError> {
        Ok(UnitDivisionBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for UnitDivision {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::OmUnitDivision);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::OmUnitDivision);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        emit_unit_fields(&mut e, &self.unit)?;
        e.resource(OM_HAS_NUMERATOR, self.has_numerator.as_ref())?;
        e.resource(OM_HAS_DENOMINATOR, self.has_denominator.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for UnitDivision {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            unit: UnitData::from_object(object),
            has_numerator: object.first_resource(OM_HAS_NUMERATOR).cloned(),
            has_denominator: object.first_resource(OM_HAS_DENOMINATOR).cloned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnitExponentiation {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub unit: UnitData,
    pub has_base: Option<Resource>,
    pub has_exponent: Option<i64>,
}

impl UnitExponentiation {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        label: impl Into<String>,
        symbol: impl Into<String>,
        has_base: Resource,
        has_exponent: i64,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .label(label)
            .symbol(symbol)
            .has_base(has_base)
            .has_exponent(has_exponent)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<UnitExponentiationBuilder, BuildError> {
        Ok(UnitExponentiationBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for UnitExponentiation {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::OmUnitExponentiation);
        let mut e = Emitter::new(
            &mut triples,
            &self.identity,
            SbolClass::OmUnitExponentiation,
        );
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        emit_unit_fields(&mut e, &self.unit)?;
        e.resource(OM_HAS_BASE, self.has_base.as_ref())?;
        e.i64(OM_HAS_EXPONENT, self.has_exponent)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for UnitExponentiation {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            unit: UnitData::from_object(object),
            has_base: object.first_resource(OM_HAS_BASE).cloned(),
            has_exponent: object
                .first_literal_value(OM_HAS_EXPONENT)
                .and_then(|v| v.parse().ok()),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnitMultiplication {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub unit: UnitData,
    pub has_term1: Option<Resource>,
    pub has_term2: Option<Resource>,
}

impl UnitMultiplication {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        label: impl Into<String>,
        symbol: impl Into<String>,
        has_term1: Resource,
        has_term2: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .label(label)
            .symbol(symbol)
            .has_term1(has_term1)
            .has_term2(has_term2)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<UnitMultiplicationBuilder, BuildError> {
        Ok(UnitMultiplicationBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for UnitMultiplication {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::OmUnitMultiplication);
        let mut e = Emitter::new(
            &mut triples,
            &self.identity,
            SbolClass::OmUnitMultiplication,
        );
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        emit_unit_fields(&mut e, &self.unit)?;
        e.resource(OM_HAS_TERM1, self.has_term1.as_ref())?;
        e.resource(OM_HAS_TERM2, self.has_term2.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for UnitMultiplication {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            unit: UnitData::from_object(object),
            has_term1: object.first_resource(OM_HAS_TERM1).cloned(),
            has_term2: object.first_resource(OM_HAS_TERM2).cloned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PrefixedUnit {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub unit: UnitData,
    pub has_unit: Option<Resource>,
    pub has_prefix: Option<Resource>,
}

impl PrefixedUnit {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        label: impl Into<String>,
        symbol: impl Into<String>,
        has_unit: Resource,
        has_prefix: Resource,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .label(label)
            .symbol(symbol)
            .has_unit(has_unit)
            .has_prefix(has_prefix)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<PrefixedUnitBuilder, BuildError> {
        Ok(PrefixedUnitBuilder::seed(
            namespace.try_into()?,
            display_id.try_into()?,
        ))
    }
}

impl ToRdf for PrefixedUnit {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::OmPrefixedUnit);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::OmPrefixedUnit);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        emit_unit_fields(&mut e, &self.unit)?;
        e.resource(OM_HAS_UNIT, self.has_unit.as_ref())?;
        e.resource(OM_HAS_PREFIX, self.has_prefix.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for PrefixedUnit {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            unit: UnitData::from_object(object),
            has_unit: object.first_resource(OM_HAS_UNIT).cloned(),
            has_prefix: object.first_resource(OM_HAS_PREFIX).cloned(),
        })
    }
}
