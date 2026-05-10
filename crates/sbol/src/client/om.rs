//! Owned typed structs for the OM (Ontology of units of Measure) classes
//! adopted by SBOL 3 (Appendix A.2).
//!
//! `Measure` is a bare Identified attached to other SBOL objects through
//! `sbol:hasMeasure`. The `Unit` hierarchy (`SingularUnit`,
//! `CompoundUnit`, `UnitDivision`, `UnitExponentiation`,
//! `UnitMultiplication`, `PrefixedUnit`) and the `Prefix` hierarchy
//! (`SIPrefix`, `BinaryPrefix`) are TopLevels carrying labels, symbols,
//! and structural references that the descriptor-driven serializer
//! emits through the shared `Emitter`.

use crate::client::accessors::{impl_sbol_identified, impl_sbol_top_level};
use crate::client::builder::{
    BinaryPrefixBuilder, CompoundUnitBuilder, MeasureBuilder, PrefixBuilder, PrefixedUnitBuilder,
    SIPrefixBuilder, SingularUnitBuilder, UnitBuilder, UnitDivisionBuilder,
    UnitExponentiationBuilder, UnitMultiplicationBuilder,
};
use crate::client::shared::{iris, literals};
use crate::client::to_rdf::{Emitter, emit_identified, emit_top_level, seed_triples};
use crate::client::{IdentifiedData, ToRdf, TopLevelData, TryFromObject};
use crate::error::BuildError;
use crate::identity::{DisplayId, Namespace};
use crate::vocab::*;
use crate::{Iri, Object, Resource, SbolClass, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Measure {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub types: Vec<Iri>,
    pub has_unit: Option<Resource>,
    pub has_numerical_value: Option<String>,
}

impl Measure {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
        has_unit: Resource,
        has_numerical_value: f64,
    ) -> Result<Self, BuildError> {
        Self::builder(parent, display_id)?
            .has_unit(has_unit)
            .has_numerical_value(has_numerical_value)
            .build()
    }

    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
    ) -> Result<MeasureBuilder, BuildError> {
        MeasureBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for Measure {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = seed_triples(&self.identity, SbolClass::OmMeasure);
        let mut e = Emitter::new(&mut triples, &self.identity, SbolClass::OmMeasure);
        emit_identified(&mut e, &self.identified)?;
        e.iris(SBOL_TYPE, &self.types)?;
        e.resource(OM_HAS_UNIT, self.has_unit.as_ref())?;
        e.literal(OM_HAS_NUMERICAL_VALUE, self.has_numerical_value.as_deref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Measure {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            types: iris(object, SBOL_TYPE),
            has_unit: object.first_resource(OM_HAS_UNIT).cloned(),
            has_numerical_value: object
                .first_literal_value(OM_HAS_NUMERICAL_VALUE)
                .map(ToOwned::to_owned),
        })
    }
}

/// Shared OM Unit fields. Carried by every concrete Unit subclass so
/// the label/symbol/comment serializers can be reused.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnitData {
    pub label: Option<String>,
    pub symbol: Option<String>,
    pub alternative_labels: Vec<String>,
    pub alternative_symbols: Vec<String>,
    pub comment: Option<String>,
    pub long_comment: Option<String>,
}

impl UnitData {
    pub(crate) fn from_object(object: &Object) -> Self {
        Self {
            label: object.first_literal_value(OM_LABEL).map(ToOwned::to_owned),
            symbol: object.first_literal_value(OM_SYMBOL).map(ToOwned::to_owned),
            alternative_labels: literals(object, OM_ALTERNATIVE_LABEL),
            alternative_symbols: literals(object, OM_ALTERNATIVE_SYMBOL),
            comment: object
                .first_literal_value(OM_COMMENT)
                .map(ToOwned::to_owned),
            long_comment: object
                .first_literal_value(OM_LONG_COMMENT)
                .map(ToOwned::to_owned),
        }
    }
}

pub(crate) fn emit_unit_fields(e: &mut Emitter<'_>, data: &UnitData) -> Result<(), BuildError> {
    e.literal(OM_LABEL, data.label.as_deref())?;
    e.literal(OM_SYMBOL, data.symbol.as_deref())?;
    e.literals(OM_ALTERNATIVE_LABEL, &data.alternative_labels)?;
    e.literals(OM_ALTERNATIVE_SYMBOL, &data.alternative_symbols)?;
    e.literal(OM_COMMENT, data.comment.as_deref())?;
    e.literal(OM_LONG_COMMENT, data.long_comment.as_deref())?;
    Ok(())
}

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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
        label: impl Into<String>,
        symbol: impl Into<String>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .label(label)
            .symbol(symbol)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
        label: impl Into<String>,
        symbol: impl Into<String>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .label(label)
            .symbol(symbol)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
        label: impl Into<String>,
        symbol: impl Into<String>,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?
            .label(label)
            .symbol(symbol)
            .build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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

/// Shared OM Prefix fields. Carried by every concrete Prefix subclass
/// so the label/symbol/factor serializers can be reused.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct PrefixData {
    pub label: Option<String>,
    pub symbol: Option<String>,
    pub has_factor: Option<String>,
    pub alternative_labels: Vec<String>,
    pub alternative_symbols: Vec<String>,
    pub comment: Option<String>,
    pub long_comment: Option<String>,
}

impl PrefixData {
    pub(crate) fn from_object(object: &Object) -> Self {
        Self {
            label: object.first_literal_value(OM_LABEL).map(ToOwned::to_owned),
            symbol: object.first_literal_value(OM_SYMBOL).map(ToOwned::to_owned),
            has_factor: object
                .first_literal_value(OM_HAS_FACTOR)
                .map(ToOwned::to_owned),
            alternative_labels: literals(object, OM_ALTERNATIVE_LABEL),
            alternative_symbols: literals(object, OM_ALTERNATIVE_SYMBOL),
            comment: object
                .first_literal_value(OM_COMMENT)
                .map(ToOwned::to_owned),
            long_comment: object
                .first_literal_value(OM_LONG_COMMENT)
                .map(ToOwned::to_owned),
        }
    }
}

pub(crate) fn emit_prefix_fields(e: &mut Emitter<'_>, data: &PrefixData) -> Result<(), BuildError> {
    e.literal(OM_LABEL, data.label.as_deref())?;
    e.literal(OM_SYMBOL, data.symbol.as_deref())?;
    e.literal(OM_HAS_FACTOR, data.has_factor.as_deref())?;
    e.literals(OM_ALTERNATIVE_LABEL, &data.alternative_labels)?;
    e.literals(OM_ALTERNATIVE_SYMBOL, &data.alternative_symbols)?;
    e.literal(OM_COMMENT, data.comment.as_deref())?;
    e.literal(OM_LONG_COMMENT, data.long_comment.as_deref())?;
    Ok(())
}

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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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
        namespace: impl TryInto<Namespace, Error = BuildError>,
        display_id: impl TryInto<DisplayId, Error = BuildError>,
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

impl_sbol_identified!(
    Measure,
    Unit,
    SingularUnit,
    CompoundUnit,
    UnitDivision,
    UnitExponentiation,
    UnitMultiplication,
    PrefixedUnit,
    Prefix,
    SIPrefix,
    BinaryPrefix,
);
impl_sbol_top_level!(
    Unit,
    SingularUnit,
    CompoundUnit,
    UnitDivision,
    UnitExponentiation,
    UnitMultiplication,
    PrefixedUnit,
    Prefix,
    SIPrefix,
    BinaryPrefix,
);
