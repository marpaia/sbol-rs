//! Owned typed structs for the OM classes adopted by SBOL 2 (§13.2).
//!
//! `Measure` is a bare Identified attached to other objects through
//! `sbol2:measure`. The `Unit` and `Prefix` hierarchies are TopLevels carrying
//! symbols and labels; SBOL 2 records the human-readable label and comment as
//! `rdfs:label` and `rdfs:comment` rather than OM predicates. Their shared
//! fields live in [`UnitData`] and [`PrefixData`].

mod prefixes;
mod units;

pub use prefixes::{BinaryPrefix, Prefix, SIPrefix};
pub use units::{
    CompoundUnit, PrefixedUnit, SingularUnit, Unit, UnitDivision, UnitExponentiation,
    UnitMultiplication,
};

use crate::client::accessors::impl_sbol_identified;
use crate::client::shared::{iris, literals};
use crate::client::to_rdf::{Emitter, emit_identified, seed_triples};
use crate::client::{IdentifiedData, ToRdf, TryFromObject};
use crate::vocab::*;
use crate::{Iri, Object, Resource, Sbol2Class, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Measure {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub has_numerical_value: Option<String>,
    pub has_unit: Option<Resource>,
    pub types: Vec<Iri>,
}

impl ToRdf for Measure {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::OmMeasure);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::OmMeasure);
        emit_identified(&mut e, &self.identified)?;
        e.literal(OM_HAS_NUMERICAL_VALUE, self.has_numerical_value.as_deref())?;
        e.resource(OM_HAS_UNIT, self.has_unit.as_ref())?;
        e.iris(SBOL2_TYPE, &self.types)?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for Measure {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            has_numerical_value: object
                .first_literal_value(OM_HAS_NUMERICAL_VALUE)
                .map(ToOwned::to_owned),
            has_unit: object.first_resource(OM_HAS_UNIT).cloned(),
            types: iris(object, SBOL2_TYPE),
        })
    }
}

/// Shared OM Unit fields, carried by every concrete Unit subclass.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnitData {
    pub symbol: Option<String>,
    pub label: Option<String>,
    pub alternative_symbols: Vec<String>,
    pub alternative_labels: Vec<String>,
    pub comment: Option<String>,
    pub long_comment: Option<String>,
}

impl UnitData {
    pub(crate) fn from_object(object: &Object) -> Self {
        Self {
            symbol: object.first_literal_value(OM_SYMBOL).map(ToOwned::to_owned),
            label: object.first_literal_value(RDFS_LABEL).map(ToOwned::to_owned),
            alternative_symbols: literals(object, OM_ALTERNATIVE_SYMBOL),
            alternative_labels: literals(object, OM_ALTERNATIVE_LABEL),
            comment: object
                .first_literal_value(RDFS_COMMENT)
                .map(ToOwned::to_owned),
            long_comment: object
                .first_literal_value(OM_LONG_COMMENT)
                .map(ToOwned::to_owned),
        }
    }
}

pub(crate) fn emit_unit_fields(e: &mut Emitter<'_>, data: &UnitData) -> Result<(), crate::BuildError> {
    e.literal(OM_SYMBOL, data.symbol.as_deref())?;
    e.literal(RDFS_LABEL, data.label.as_deref())?;
    e.literals(OM_ALTERNATIVE_SYMBOL, &data.alternative_symbols)?;
    e.literals(OM_ALTERNATIVE_LABEL, &data.alternative_labels)?;
    e.literal(RDFS_COMMENT, data.comment.as_deref())?;
    e.literal(OM_LONG_COMMENT, data.long_comment.as_deref())?;
    Ok(())
}

/// Shared OM Prefix fields, carried by every concrete Prefix subclass.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct PrefixData {
    pub symbol: Option<String>,
    pub label: Option<String>,
    pub alternative_symbols: Vec<String>,
    pub alternative_labels: Vec<String>,
    pub comment: Option<String>,
    pub long_comment: Option<String>,
    pub has_factor: Option<String>,
}

impl PrefixData {
    pub(crate) fn from_object(object: &Object) -> Self {
        Self {
            symbol: object.first_literal_value(OM_SYMBOL).map(ToOwned::to_owned),
            label: object.first_literal_value(RDFS_LABEL).map(ToOwned::to_owned),
            alternative_symbols: literals(object, OM_ALTERNATIVE_SYMBOL),
            alternative_labels: literals(object, OM_ALTERNATIVE_LABEL),
            comment: object
                .first_literal_value(RDFS_COMMENT)
                .map(ToOwned::to_owned),
            long_comment: object
                .first_literal_value(OM_LONG_COMMENT)
                .map(ToOwned::to_owned),
            has_factor: object
                .first_literal_value(OM_HAS_FACTOR)
                .map(ToOwned::to_owned),
        }
    }
}

pub(crate) fn emit_prefix_fields(
    e: &mut Emitter<'_>,
    data: &PrefixData,
) -> Result<(), crate::BuildError> {
    e.literal(OM_SYMBOL, data.symbol.as_deref())?;
    e.literal(RDFS_LABEL, data.label.as_deref())?;
    e.literals(OM_ALTERNATIVE_SYMBOL, &data.alternative_symbols)?;
    e.literals(OM_ALTERNATIVE_LABEL, &data.alternative_labels)?;
    e.literal(RDFS_COMMENT, data.comment.as_deref())?;
    e.literal(OM_LONG_COMMENT, data.long_comment.as_deref())?;
    e.literal(OM_HAS_FACTOR, data.has_factor.as_deref())?;
    Ok(())
}

impl_sbol_identified!(Measure);
