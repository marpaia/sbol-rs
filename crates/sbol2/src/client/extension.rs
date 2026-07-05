//! `GenericTopLevel`, the SBOL 2 extension TopLevel, and `IdentifiedExtension`,
//! the catch-all preserving subjects typed only as `sbol2:Identified` or
//! `sbol2:TopLevel` so the typed round trip stays faithful.

use crate::client::accessors::{impl_sbol_identified, impl_sbol_top_level};
use crate::client::to_rdf::{Emitter, emit_identified, emit_top_level, seed_triples};
use crate::client::{IdentifiedData, ToRdf, TopLevelData, TryFromObject};
use crate::vocab::*;
use crate::{Iri, Literal, Object, Resource, Sbol2Class, Term, Triple};

/// A `sbol2:GenericTopLevel`: a user-extension TopLevel whose custom RDF class
/// is carried in the `sbol2:rdfType` property alongside arbitrary extension
/// triples.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct GenericTopLevel {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: TopLevelData,
    pub rdf_type: Option<Iri>,
}

impl ToRdf for GenericTopLevel {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = seed_triples(&self.identity, Sbol2Class::GenericTopLevel);
        let mut e = Emitter::new(&mut triples, &self.identity, Sbol2Class::GenericTopLevel);
        emit_identified(&mut e, &self.identified)?;
        emit_top_level(&mut e, &self.top_level)?;
        e.iri(SBOL2_RDF_TYPE, self.rdf_type.as_ref())?;
        drop(e);
        Ok(triples)
    }
}

impl TryFromObject for GenericTopLevel {
    fn try_from_object(object: &Object) -> Option<Self> {
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level: TopLevelData::from_object(object),
            rdf_type: object.first_iri(SBOL2_RDF_TYPE).cloned(),
        })
    }
}

/// An RDF subject preserved through the typed model when no concrete SBOL 2
/// class variant matched its `rdf:type` set. The `rdf_types` vector preserves
/// every original `rdf:type` IRI so re-serialization emits exactly what the
/// parser saw.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct IdentifiedExtension {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: Option<TopLevelData>,
    pub rdf_types: Vec<Iri>,
}

impl ToRdf for IdentifiedExtension {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, crate::BuildError> {
        let mut triples = Vec::new();
        for rdf_type in &self.rdf_types {
            triples.push(rdf_type_triple(&self.identity, rdf_type.clone()));
        }
        if !self
            .rdf_types
            .iter()
            .any(|iri| iri.as_str() == SBOL2_IDENTIFIED_CLASS)
        {
            triples.push(rdf_type_triple(
                &self.identity,
                Iri::from_static(SBOL2_IDENTIFIED_CLASS),
            ));
        }
        push_resource_opt(
            &mut triples,
            &self.identity,
            SBOL2_PERSISTENT_IDENTITY,
            self.identified.persistent_identity.as_ref(),
        );
        push_string(
            &mut triples,
            &self.identity,
            SBOL2_VERSION,
            self.identified.version.as_deref(),
        );
        push_string(
            &mut triples,
            &self.identity,
            SBOL2_DISPLAY_ID,
            self.identified.display_id.as_deref(),
        );
        push_string(
            &mut triples,
            &self.identity,
            DCTERMS_TITLE,
            self.identified.name.as_deref(),
        );
        push_string(
            &mut triples,
            &self.identity,
            DCTERMS_DESCRIPTION,
            self.identified.description.as_deref(),
        );
        for value in &self.identified.derived_from {
            push_resource(
                &mut triples,
                &self.identity,
                PROV_WAS_DERIVED_FROM,
                value.clone(),
            );
        }
        for value in &self.identified.generated_by {
            push_resource(
                &mut triples,
                &self.identity,
                PROV_WAS_GENERATED_BY,
                value.clone(),
            );
        }
        for extension in &self.identified.extensions {
            triples.push(Triple {
                subject: self.identity.clone(),
                predicate: extension.predicate.clone(),
                object: extension.object.clone(),
            });
        }
        if let Some(top_level) = &self.top_level {
            for attachment in &top_level.attachments {
                push_resource(&mut triples, &self.identity, SBOL2_ATTACHMENT, attachment.clone());
            }
        }
        Ok(triples)
    }
}

impl TryFromObject for IdentifiedExtension {
    fn try_from_object(object: &Object) -> Option<Self> {
        let rdf_types: Vec<Iri> = object.rdf_types().iter().cloned().collect();
        let top_level = object
            .is_top_level()
            .then(|| TopLevelData::from_object(object));
        Some(Self {
            identity: object.identity().clone(),
            identified: IdentifiedData::from_object(object),
            top_level,
            rdf_types,
        })
    }
}

fn rdf_type_triple(identity: &Resource, rdf_type: Iri) -> Triple {
    Triple {
        subject: identity.clone(),
        predicate: Iri::from_static(RDF_TYPE),
        object: Term::Resource(Resource::Iri(rdf_type)),
    }
}

fn push_resource(
    triples: &mut Vec<Triple>,
    identity: &Resource,
    predicate: &'static str,
    value: Resource,
) {
    triples.push(Triple {
        subject: identity.clone(),
        predicate: Iri::from_static(predicate),
        object: Term::Resource(value),
    });
}

fn push_resource_opt(
    triples: &mut Vec<Triple>,
    identity: &Resource,
    predicate: &'static str,
    value: Option<&Resource>,
) {
    if let Some(value) = value {
        push_resource(triples, identity, predicate, value.clone());
    }
}

fn push_string(
    triples: &mut Vec<Triple>,
    identity: &Resource,
    predicate: &'static str,
    value: Option<&str>,
) {
    if let Some(value) = value {
        triples.push(Triple {
            subject: identity.clone(),
            predicate: Iri::from_static(predicate),
            object: Term::Literal(Literal::simple(value)),
        });
    }
}

impl_sbol_identified!(GenericTopLevel, IdentifiedExtension);
impl_sbol_top_level!(GenericTopLevel);
