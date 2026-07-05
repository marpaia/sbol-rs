use crate::client::accessors::impl_sbol_identified;
use crate::client::shared::resources;
use crate::client::to_rdf::{Emitter, emit_identified, seed_triples};
use crate::client::{IdentifiedData, ToRdf, TryFromObject};
use crate::vocab::*;
use crate::{Iri, Object, Resource, Sbol2Class, Triple};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct VariableComponent {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub variable: Option<Resource>,
    pub variants: Vec<Resource>,
    pub variant_collections: Vec<Resource>,
    pub variant_derivations: Vec<Resource>,
    pub operator: Option<Iri>,
}

impl ToRdf for VariableComponent {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::VariableComponent);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::VariableComponent);
        emit_identified(&mut e, &self.identified)?;
        e.resource(SBOL2_VARIABLE, self.variable.as_ref())?;
        e.resources(SBOL2_VARIANT, &self.variants)?;
        e.resources(SBOL2_VARIANT_COLLECTION, &self.variant_collections)?;
        e.resources(SBOL2_VARIANT_DERIVATION, &self.variant_derivations)?;
        e.iri(SBOL2_OPERATOR, self.operator.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for VariableComponent {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            variable: object.first_resource(SBOL2_VARIABLE).cloned(),
            variants: resources(object, SBOL2_VARIANT),
            variant_collections: resources(object, SBOL2_VARIANT_COLLECTION),
            variant_derivations: resources(object, SBOL2_VARIANT_DERIVATION),
            operator: object.first_iri(SBOL2_OPERATOR).cloned(),
        })
    }
}

impl_sbol_identified!(VariableComponent);
