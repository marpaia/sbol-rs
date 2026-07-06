//! The OM `Unit` hierarchy: `Unit` and its concrete subclasses. Each is a
//! TopLevel carrying the shared [`UnitData`] symbol/label fields plus its own
//! structural references.

use super::{UnitData, emit_unit_fields};
use crate::client::accessors::{impl_sbol_identified, impl_sbol_top_level};
use crate::client::shared::first_i64;
use crate::client::to_rdf::{Emitter, emit_identified, emit_top_level, seed_triples};
use crate::client::{IdentifiedData, ToRdf, TopLevelData, TryFromObject};
use crate::vocab::*;
use crate::{Object, Resource, Sbol2Class, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Unit {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub unit: UnitData,
}

impl ToRdf for Unit {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::OmUnit);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::OmUnit);
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

impl ToRdf for SingularUnit {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::OmSingularUnit);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::OmSingularUnit);
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

impl ToRdf for CompoundUnit {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::OmCompoundUnit);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::OmCompoundUnit);
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
pub struct UnitMultiplication {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub unit: UnitData,
    pub has_term1: Option<Resource>,
    pub has_term2: Option<Resource>,
}

impl ToRdf for UnitMultiplication {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::OmUnitMultiplication);
        let mut e = Emitter::new(
            &mut triples,
            &self.identity,
            Sbol2Class::OmUnitMultiplication,
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
pub struct UnitDivision {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub unit: UnitData,
    pub has_numerator: Option<Resource>,
    pub has_denominator: Option<Resource>,
}

impl ToRdf for UnitDivision {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::OmUnitDivision);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::OmUnitDivision);
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

impl ToRdf for UnitExponentiation {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::OmUnitExponentiation);
        let mut e = Emitter::new(
            &mut triples,
            &self.identity,
            Sbol2Class::OmUnitExponentiation,
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
            has_exponent: first_i64(object, OM_HAS_EXPONENT),
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

impl ToRdf for PrefixedUnit {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::OmPrefixedUnit);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::OmPrefixedUnit);
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

impl_sbol_identified!(
    Unit,
    SingularUnit,
    CompoundUnit,
    UnitMultiplication,
    UnitDivision,
    UnitExponentiation,
    PrefixedUnit,
);
impl_sbol_top_level!(
    Unit,
    SingularUnit,
    CompoundUnit,
    UnitMultiplication,
    UnitDivision,
    UnitExponentiation,
    PrefixedUnit,
);
