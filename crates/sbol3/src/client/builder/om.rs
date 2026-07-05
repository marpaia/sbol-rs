//! Builders for the OM classes adopted by SBOL 3 (Appendix A.2) and the
//! `IdentifiedExtension` catch-all: `Measure`, the `Unit` hierarchy, the
//! `Prefix` hierarchy, and `IdentifiedExtension`.

use super::{
    child_seed, identified_seed, identified_setters, missing, prefix_seed, prefix_setters,
    top_level_seed, top_level_setters, unit_seed, unit_setters,
};
use crate::client::identity::build_top_level_identity;
use crate::client::{
    BinaryPrefix, CompoundUnit, ExtensionTriple, IdentifiedData, IdentifiedExtension, Measure,
    Prefix, PrefixData, PrefixedUnit, SIPrefix, SingularUnit, TopLevelData, Unit, UnitData,
    UnitDivision, UnitExponentiation, UnitMultiplication,
};
use crate::error::BuildError;
use crate::identity::{DisplayId, Namespace};
use crate::{Iri, Resource, SbolClass, Term};

/// Builder for [`Measure`].
#[derive(Clone, Debug)]
pub struct MeasureBuilder {
    identity: Resource,
    identified: IdentifiedData,
    types: Vec<Iri>,
    has_unit: Option<Resource>,
    has_numerical_value: Option<String>,
}

impl MeasureBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            types: Vec::new(),
            has_unit: None,
            has_numerical_value: None,
        })
    }

    identified_setters!();

    pub fn types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.types = values.into_iter().collect();
        self
    }

    pub fn add_type(mut self, value: Iri) -> Self {
        self.types.push(value);
        self
    }

    pub fn has_unit(mut self, value: Resource) -> Self {
        self.has_unit = Some(value);
        self
    }

    pub fn has_numerical_value(mut self, value: f64) -> Self {
        self.has_numerical_value = Some(value.to_string());
        self
    }

    pub fn build(self) -> Result<Measure, BuildError> {
        let has_unit = self
            .has_unit
            .ok_or_else(|| missing(&self.identity, SbolClass::OmMeasure, "hasUnit"))?;
        let has_numerical_value = self
            .has_numerical_value
            .ok_or_else(|| missing(&self.identity, SbolClass::OmMeasure, "hasNumericalValue"))?;
        Ok(Measure {
            identity: self.identity,
            identified: self.identified,
            types: self.types,
            has_unit: Some(has_unit),
            has_numerical_value: Some(has_numerical_value),
        })
    }
}

/// Builder for [`Unit`].
#[derive(Clone, Debug)]
pub struct UnitBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
}

impl UnitBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        let unit = unit_seed(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit,
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn build(self) -> Result<Unit, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnit, "label"))?;
        let symbol = self
            .unit
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnit, "symbol"))?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(Unit {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
        })
    }
}

/// Builder for [`SingularUnit`].
#[derive(Clone, Debug)]
pub struct SingularUnitBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
    has_unit: Option<Resource>,
    has_factor: Option<String>,
}

impl SingularUnitBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit: UnitData::default(),
            has_unit: None,
            has_factor: None,
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn has_unit(mut self, value: Resource) -> Self {
        self.has_unit = Some(value);
        self
    }

    pub fn has_factor(mut self, value: f64) -> Self {
        self.has_factor = Some(value.to_string());
        self
    }

    pub fn build(self) -> Result<SingularUnit, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmSingularUnit, "label"))?;
        let symbol = self
            .unit
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmSingularUnit, "symbol"))?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(SingularUnit {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
            has_unit: self.has_unit,
            has_factor: self.has_factor,
        })
    }
}

/// Builder for [`CompoundUnit`].
#[derive(Clone, Debug)]
pub struct CompoundUnitBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
}

impl CompoundUnitBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit: UnitData::default(),
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn build(self) -> Result<CompoundUnit, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmCompoundUnit, "label"))?;
        let symbol = self
            .unit
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmCompoundUnit, "symbol"))?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(CompoundUnit {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
        })
    }
}

/// Builder for [`UnitDivision`].
#[derive(Clone, Debug)]
pub struct UnitDivisionBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
    has_numerator: Option<Resource>,
    has_denominator: Option<Resource>,
}

impl UnitDivisionBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit: UnitData::default(),
            has_numerator: None,
            has_denominator: None,
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn has_numerator(mut self, value: Resource) -> Self {
        self.has_numerator = Some(value);
        self
    }

    pub fn has_denominator(mut self, value: Resource) -> Self {
        self.has_denominator = Some(value);
        self
    }

    pub fn build(self) -> Result<UnitDivision, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitDivision, "label"))?;
        let symbol = self
            .unit
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitDivision, "symbol"))?;
        let has_numerator = self
            .has_numerator
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitDivision, "hasNumerator"))?;
        let has_denominator = self
            .has_denominator
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitDivision, "hasDenominator"))?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(UnitDivision {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
            has_numerator: Some(has_numerator),
            has_denominator: Some(has_denominator),
        })
    }
}

/// Builder for [`UnitExponentiation`].
#[derive(Clone, Debug)]
pub struct UnitExponentiationBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
    has_base: Option<Resource>,
    has_exponent: Option<i64>,
}

impl UnitExponentiationBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit: UnitData::default(),
            has_base: None,
            has_exponent: None,
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn has_base(mut self, value: Resource) -> Self {
        self.has_base = Some(value);
        self
    }

    pub fn has_exponent(mut self, value: i64) -> Self {
        self.has_exponent = Some(value);
        self
    }

    pub fn build(self) -> Result<UnitExponentiation, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitExponentiation, "label"))?;
        let symbol =
            self.unit.symbol.clone().ok_or_else(|| {
                missing(&self.identity, SbolClass::OmUnitExponentiation, "symbol")
            })?;
        let has_base = self
            .has_base
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitExponentiation, "hasBase"))?;
        let has_exponent = self.has_exponent.ok_or_else(|| {
            missing(
                &self.identity,
                SbolClass::OmUnitExponentiation,
                "hasExponent",
            )
        })?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(UnitExponentiation {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
            has_base: Some(has_base),
            has_exponent: Some(has_exponent),
        })
    }
}

/// Builder for [`UnitMultiplication`].
#[derive(Clone, Debug)]
pub struct UnitMultiplicationBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
    has_term1: Option<Resource>,
    has_term2: Option<Resource>,
}

impl UnitMultiplicationBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit: UnitData::default(),
            has_term1: None,
            has_term2: None,
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn has_term1(mut self, value: Resource) -> Self {
        self.has_term1 = Some(value);
        self
    }

    pub fn has_term2(mut self, value: Resource) -> Self {
        self.has_term2 = Some(value);
        self
    }

    pub fn build(self) -> Result<UnitMultiplication, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitMultiplication, "label"))?;
        let symbol =
            self.unit.symbol.clone().ok_or_else(|| {
                missing(&self.identity, SbolClass::OmUnitMultiplication, "symbol")
            })?;
        let has_term1 = self
            .has_term1
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitMultiplication, "hasTerm1"))?;
        let has_term2 = self
            .has_term2
            .ok_or_else(|| missing(&self.identity, SbolClass::OmUnitMultiplication, "hasTerm2"))?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(UnitMultiplication {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
            has_term1: Some(has_term1),
            has_term2: Some(has_term2),
        })
    }
}

/// Builder for [`PrefixedUnit`].
#[derive(Clone, Debug)]
pub struct PrefixedUnitBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    unit: UnitData,
    has_unit: Option<Resource>,
    has_prefix: Option<Resource>,
}

impl PrefixedUnitBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            unit: UnitData::default(),
            has_unit: None,
            has_prefix: None,
        }
    }

    identified_setters!();
    top_level_setters!();
    unit_setters!();

    pub fn has_unit(mut self, value: Resource) -> Self {
        self.has_unit = Some(value);
        self
    }

    pub fn has_prefix(mut self, value: Resource) -> Self {
        self.has_prefix = Some(value);
        self
    }

    pub fn build(self) -> Result<PrefixedUnit, BuildError> {
        let label = self
            .unit
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefixedUnit, "label"))?;
        let symbol = self
            .unit
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefixedUnit, "symbol"))?;
        let has_unit = self
            .has_unit
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefixedUnit, "hasUnit"))?;
        let has_prefix = self
            .has_prefix
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefixedUnit, "hasPrefix"))?;
        let mut unit = self.unit;
        unit.label = Some(label);
        unit.symbol = Some(symbol);
        Ok(PrefixedUnit {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            unit,
            has_unit: Some(has_unit),
            has_prefix: Some(has_prefix),
        })
    }
}

/// Builder for [`Prefix`].
#[derive(Clone, Debug)]
pub struct PrefixBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    prefix: PrefixData,
}

impl PrefixBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            prefix: prefix_seed(&namespace, &display_id),
        }
    }

    identified_setters!();
    top_level_setters!();
    prefix_setters!();

    pub fn build(self) -> Result<Prefix, BuildError> {
        let label = self
            .prefix
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefix, "label"))?;
        let symbol = self
            .prefix
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefix, "symbol"))?;
        let has_factor = self
            .prefix
            .has_factor
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmPrefix, "hasFactor"))?;
        let mut prefix = self.prefix;
        prefix.label = Some(label);
        prefix.symbol = Some(symbol);
        prefix.has_factor = Some(has_factor);
        Ok(Prefix {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            prefix,
        })
    }
}

/// Builder for [`SIPrefix`].
#[derive(Clone, Debug)]
pub struct SIPrefixBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    prefix: PrefixData,
}

impl SIPrefixBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            prefix: PrefixData::default(),
        }
    }

    identified_setters!();
    top_level_setters!();
    prefix_setters!();

    pub fn build(self) -> Result<SIPrefix, BuildError> {
        let label = self
            .prefix
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmSiPrefix, "label"))?;
        let symbol = self
            .prefix
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmSiPrefix, "symbol"))?;
        let has_factor = self
            .prefix
            .has_factor
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmSiPrefix, "hasFactor"))?;
        let mut prefix = self.prefix;
        prefix.label = Some(label);
        prefix.symbol = Some(symbol);
        prefix.has_factor = Some(has_factor);
        Ok(SIPrefix {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            prefix,
        })
    }
}

/// Builder for [`BinaryPrefix`].
#[derive(Clone, Debug)]
pub struct BinaryPrefixBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    prefix: PrefixData,
}

impl BinaryPrefixBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let identity = build_top_level_identity(&namespace, &display_id);
        Self {
            identity,
            identified: identified_seed(&display_id),
            top_level: top_level_seed(&namespace),
            prefix: PrefixData::default(),
        }
    }

    identified_setters!();
    top_level_setters!();
    prefix_setters!();

    pub fn build(self) -> Result<BinaryPrefix, BuildError> {
        let label = self
            .prefix
            .label
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmBinaryPrefix, "label"))?;
        let symbol = self
            .prefix
            .symbol
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmBinaryPrefix, "symbol"))?;
        let has_factor = self
            .prefix
            .has_factor
            .clone()
            .ok_or_else(|| missing(&self.identity, SbolClass::OmBinaryPrefix, "hasFactor"))?;
        let mut prefix = self.prefix;
        prefix.label = Some(label);
        prefix.symbol = Some(symbol);
        prefix.has_factor = Some(has_factor);
        Ok(BinaryPrefix {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            prefix,
        })
    }
}

// ---------------------------------------------------------------------------
// IdentifiedExtension (catch-all for bare sbol:Identified subjects)
// ---------------------------------------------------------------------------

/// Builder for [`IdentifiedExtension`].
#[derive(Clone, Debug)]
pub struct IdentifiedExtensionBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: Option<TopLevelData>,
    rdf_types: Vec<Iri>,
}

impl IdentifiedExtensionBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            top_level: None,
            rdf_types: Vec::new(),
        })
    }

    identified_setters!();

    pub fn top_level(mut self, top_level: TopLevelData) -> Self {
        self.top_level = Some(top_level);
        self
    }

    pub fn rdf_types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.rdf_types = values.into_iter().collect();
        self
    }

    pub fn add_rdf_type(mut self, value: Iri) -> Self {
        self.rdf_types.push(value);
        self
    }

    pub fn build(self) -> Result<IdentifiedExtension, BuildError> {
        Ok(IdentifiedExtension {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            rdf_types: self.rdf_types,
        })
    }
}
