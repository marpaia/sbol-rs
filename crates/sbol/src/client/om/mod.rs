//! Owned typed structs for the OM (Ontology of units of Measure) classes
//! adopted by SBOL 3 (Appendix A.2).
//!
//! `Measure` is a bare Identified attached to other SBOL objects through
//! `sbol:hasMeasure`. The `Unit` hierarchy (`SingularUnit`,
//! `CompoundUnit`, `UnitDivision`, `UnitExponentiation`,
//! `UnitMultiplication`, `PrefixedUnit`, in [`units`]) and the `Prefix`
//! hierarchy (`Prefix`, `SIPrefix`, `BinaryPrefix`, in [`prefixes`]) are
//! TopLevels carrying labels, symbols, and structural references that the
//! descriptor-driven serializer emits through the shared `Emitter`. Their
//! shared label/symbol fields live in [`UnitData`] and [`PrefixData`].

mod prefixes;
mod units;

pub use prefixes::{BinaryPrefix, Prefix, SIPrefix};
pub use units::{
    CompoundUnit, PrefixedUnit, SingularUnit, Unit, UnitDivision, UnitExponentiation,
    UnitMultiplication,
};

use crate::client::accessors::{impl_sbol_identified, impl_sbol_top_level};
use crate::client::builder::MeasureBuilder;
use crate::client::shared::{iris, literals};
use crate::client::to_rdf::{Emitter, emit_identified, seed_triples};
use crate::client::{IdentifiedData, ToRdf, TryFromObject};
use crate::error::BuildError;
use crate::identity::DisplayId;
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
