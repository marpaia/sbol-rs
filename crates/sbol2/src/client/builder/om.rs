//! Builders for the OM classes adopted by SBOL 2, plus the `GenericTopLevel`
//! and `IdentifiedExtension` extension carriers.

use super::{
    child_seed, identified_seed, identified_setters, missing, prefix_seed, prefix_setters,
    top_level_seed, top_level_setters, unit_seed, unit_setters,
};
use crate::client::identity::{DEFAULT_VERSION, build_top_level_identity};
use crate::client::{
    BinaryPrefix, CompoundUnit, GenericTopLevel, IdentifiedData, IdentifiedExtension, Measure,
    Prefix, PrefixData, PrefixedUnit, SIPrefix, SingularUnit, TopLevelData, Unit, UnitData,
    UnitDivision, UnitExponentiation, UnitMultiplication,
};
use crate::error::BuildError;
use crate::identity::{DisplayId, Namespace};
use crate::{Iri, Resource, Sbol2Class, Term};
use sbol_core::error::BuildError as LexError;

fn top_seed(
    namespace: &Namespace,
    display_id: &DisplayId,
) -> (Resource, IdentifiedData, TopLevelData) {
    let (identity, persistent) = build_top_level_identity(namespace, display_id, DEFAULT_VERSION);
    (identity, identified_seed(display_id, persistent), top_level_seed())
}

/// Builder for [`Measure`].
#[derive(Clone, Debug)]
pub struct MeasureBuilder {
    identity: Resource,
    identified: IdentifiedData,
    has_numerical_value: Option<String>,
    has_unit: Option<Resource>,
    types: Vec<Iri>,
}

impl MeasureBuilder {
    pub(crate) fn seed(parent: &Resource, display_id: DisplayId) -> Result<Self, BuildError> {
        let (identity, identified) = child_seed(parent, display_id)?;
        Ok(Self {
            identity,
            identified,
            has_numerical_value: None,
            has_unit: None,
            types: Vec::new(),
        })
    }

    identified_setters!();

    pub fn has_numerical_value(mut self, value: f64) -> Self {
        self.has_numerical_value = Some(value.to_string());
        self
    }
    pub fn has_unit(mut self, value: Resource) -> Self {
        self.has_unit = Some(value);
        self
    }
    pub fn types(mut self, values: impl IntoIterator<Item = Iri>) -> Self {
        self.types = values.into_iter().collect();
        self
    }
    pub fn add_type(mut self, value: Iri) -> Self {
        self.types.push(value);
        self
    }

    pub fn build(self) -> Result<Measure, BuildError> {
        let has_numerical_value = self.has_numerical_value.ok_or_else(|| {
            missing(&self.identity, Sbol2Class::OmMeasure, "hasNumericalValue")
        })?;
        let has_unit = self
            .has_unit
            .ok_or_else(|| missing(&self.identity, Sbol2Class::OmMeasure, "hasUnit"))?;
        Ok(Measure {
            identity: self.identity,
            identified: self.identified,
            has_numerical_value: Some(has_numerical_value),
            has_unit: Some(has_unit),
            types: self.types,
        })
    }
}

impl Measure {
    pub fn new(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
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
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<MeasureBuilder, BuildError> {
        MeasureBuilder::seed(parent, display_id.try_into()?)
    }
}

macro_rules! unit_builder {
    (
        $class:ident, $builder:ident, $sbol_class:ident,
        fields { $( $field:ident : $fty:ty = $default:expr ),* $(,)? }
        setters { $( $setter:item )* }
        require { $( ($req:ident, $prop:literal) )* }
    ) => {
        /// Builder for the corresponding OM Unit-hierarchy class.
        #[derive(Clone, Debug)]
        pub struct $builder {
            identity: Resource,
            identified: IdentifiedData,
            top_level: TopLevelData,
            unit: UnitData,
            $( $field: $fty, )*
        }

        impl $builder {
            pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
                let (identity, identified, top_level) = top_seed(&namespace, &display_id);
                Self { identity, identified, top_level, unit: unit_seed(), $( $field: $default, )* }
            }

            identified_setters!();
            top_level_setters!();
            unit_setters!();
            $( $setter )*

            pub fn build(self) -> Result<$class, BuildError> {
                if self.unit.symbol.is_none() {
                    return Err(missing(&self.identity, Sbol2Class::$sbol_class, "symbol"));
                }
                if self.unit.label.is_none() {
                    return Err(missing(&self.identity, Sbol2Class::$sbol_class, "label"));
                }
                $(
                    if self.$req.is_none() {
                        return Err(missing(&self.identity, Sbol2Class::$sbol_class, $prop));
                    }
                )*
                Ok($class {
                    identity: self.identity,
                    identified: self.identified,
                    top_level: self.top_level,
                    unit: self.unit,
                    $( $field: self.$field, )*
                })
            }
        }

        impl $class {
            pub fn builder(
                namespace: impl TryInto<Namespace, Error = LexError>,
                display_id: impl TryInto<DisplayId, Error = LexError>,
            ) -> Result<$builder, BuildError> {
                Ok($builder::seed(namespace.try_into()?, display_id.try_into()?))
            }

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
        }
    };
}

unit_builder! {
    Unit, UnitBuilder, OmUnit,
    fields {}
    setters {}
    require {}
}

unit_builder! {
    SingularUnit, SingularUnitBuilder, OmSingularUnit,
    fields { has_unit: Option<Resource> = None, has_factor: Option<String> = None }
    setters {
        pub fn has_unit(mut self, value: Resource) -> Self { self.has_unit = Some(value); self }
        pub fn has_factor(mut self, value: f64) -> Self {
            self.has_factor = Some(value.to_string());
            self
        }
    }
    require {}
}

unit_builder! {
    CompoundUnit, CompoundUnitBuilder, OmCompoundUnit,
    fields {}
    setters {}
    require {}
}

unit_builder! {
    UnitMultiplication, UnitMultiplicationBuilder, OmUnitMultiplication,
    fields { has_term1: Option<Resource> = None, has_term2: Option<Resource> = None }
    setters {
        pub fn has_term1(mut self, value: Resource) -> Self { self.has_term1 = Some(value); self }
        pub fn has_term2(mut self, value: Resource) -> Self { self.has_term2 = Some(value); self }
    }
    require { (has_term1, "hasTerm1") (has_term2, "hasTerm2") }
}

unit_builder! {
    UnitDivision, UnitDivisionBuilder, OmUnitDivision,
    fields { has_numerator: Option<Resource> = None, has_denominator: Option<Resource> = None }
    setters {
        pub fn has_numerator(mut self, value: Resource) -> Self { self.has_numerator = Some(value); self }
        pub fn has_denominator(mut self, value: Resource) -> Self { self.has_denominator = Some(value); self }
    }
    require { (has_numerator, "hasNumerator") (has_denominator, "hasDenominator") }
}

unit_builder! {
    UnitExponentiation, UnitExponentiationBuilder, OmUnitExponentiation,
    fields { has_base: Option<Resource> = None, has_exponent: Option<i64> = None }
    setters {
        pub fn has_base(mut self, value: Resource) -> Self { self.has_base = Some(value); self }
        pub fn has_exponent(mut self, value: i64) -> Self { self.has_exponent = Some(value); self }
    }
    require { (has_base, "hasBase") (has_exponent, "hasExponent") }
}

unit_builder! {
    PrefixedUnit, PrefixedUnitBuilder, OmPrefixedUnit,
    fields { has_unit: Option<Resource> = None, has_prefix: Option<Resource> = None }
    setters {
        pub fn has_unit(mut self, value: Resource) -> Self { self.has_unit = Some(value); self }
        pub fn has_prefix(mut self, value: Resource) -> Self { self.has_prefix = Some(value); self }
    }
    require { (has_unit, "hasUnit") (has_prefix, "hasPrefix") }
}

macro_rules! prefix_builder {
    ($class:ident, $builder:ident, $sbol_class:ident) => {
        /// Builder for the corresponding OM Prefix-hierarchy class.
        #[derive(Clone, Debug)]
        pub struct $builder {
            identity: Resource,
            identified: IdentifiedData,
            top_level: TopLevelData,
            prefix: PrefixData,
        }

        impl $builder {
            pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
                let (identity, identified, top_level) = top_seed(&namespace, &display_id);
                Self { identity, identified, top_level, prefix: prefix_seed() }
            }

            identified_setters!();
            top_level_setters!();
            prefix_setters!();

            pub fn build(self) -> Result<$class, BuildError> {
                if self.prefix.symbol.is_none() {
                    return Err(missing(&self.identity, Sbol2Class::$sbol_class, "symbol"));
                }
                if self.prefix.label.is_none() {
                    return Err(missing(&self.identity, Sbol2Class::$sbol_class, "label"));
                }
                if self.prefix.has_factor.is_none() {
                    return Err(missing(&self.identity, Sbol2Class::$sbol_class, "hasFactor"));
                }
                Ok($class {
                    identity: self.identity,
                    identified: self.identified,
                    top_level: self.top_level,
                    prefix: self.prefix,
                })
            }
        }

        impl $class {
            pub fn builder(
                namespace: impl TryInto<Namespace, Error = LexError>,
                display_id: impl TryInto<DisplayId, Error = LexError>,
            ) -> Result<$builder, BuildError> {
                Ok($builder::seed(namespace.try_into()?, display_id.try_into()?))
            }

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
        }
    };
}

prefix_builder!(Prefix, PrefixBuilder, OmPrefix);
prefix_builder!(SIPrefix, SIPrefixBuilder, OmSiPrefix);
prefix_builder!(BinaryPrefix, BinaryPrefixBuilder, OmBinaryPrefix);

/// Builder for [`GenericTopLevel`].
#[derive(Clone, Debug)]
pub struct GenericTopLevelBuilder {
    identity: Resource,
    identified: IdentifiedData,
    top_level: TopLevelData,
    rdf_type: Option<Iri>,
}

impl GenericTopLevelBuilder {
    pub(crate) fn seed(namespace: Namespace, display_id: DisplayId) -> Self {
        let (identity, identified, top_level) = top_seed(&namespace, &display_id);
        Self {
            identity,
            identified,
            top_level,
            rdf_type: None,
        }
    }

    identified_setters!();
    top_level_setters!();

    pub fn rdf_type(mut self, value: Iri) -> Self {
        self.rdf_type = Some(value);
        self
    }

    pub fn build(self) -> Result<GenericTopLevel, BuildError> {
        let rdf_type = self
            .rdf_type
            .ok_or_else(|| missing(&self.identity, Sbol2Class::GenericTopLevel, "rdfType"))?;
        Ok(GenericTopLevel {
            identity: self.identity,
            identified: self.identified,
            top_level: self.top_level,
            rdf_type: Some(rdf_type),
        })
    }
}

impl GenericTopLevel {
    pub fn new(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
        rdf_type: Iri,
    ) -> Result<Self, BuildError> {
        Self::builder(namespace, display_id)?.rdf_type(rdf_type).build()
    }

    pub fn builder(
        namespace: impl TryInto<Namespace, Error = LexError>,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<GenericTopLevelBuilder, BuildError> {
        Ok(GenericTopLevelBuilder::seed(namespace.try_into()?, display_id.try_into()?))
    }
}

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

impl IdentifiedExtension {
    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<IdentifiedExtensionBuilder, BuildError> {
        IdentifiedExtensionBuilder::seed(parent, display_id.try_into()?)
    }
}
