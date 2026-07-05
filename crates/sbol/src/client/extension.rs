//! Generic typed wrapper for subjects whose only typed signal is
//! `sbol:Identified` (and optionally `sbol:TopLevel`).
//!
//! Before this variant existed, RDF subjects typed solely as
//! `sbol:Identified` dropped on the typed round trip: `Document::from_objects`
//! emits only the typed cache, and nothing in the cache carried those
//! subjects. The `IdentifiedExtension` variant preserves the original
//! `rdf:type` IRIs plus the shared Identified / TopLevel data so the
//! to_rdf → from_rdf chain is faithful.

use crate::client::accessors::impl_sbol_identified;
use crate::client::builder::IdentifiedExtensionBuilder;
use crate::client::{IdentifiedData, ToRdf, TopLevelData, TryFromObject};
use crate::error::BuildError;
use sbol_core::error::BuildError as LexError;
use crate::identity::DisplayId;
use crate::vocab::*;
use crate::{Iri, Literal, Object, Resource, Term, Triple};

/// An RDF subject preserved through the typed model when no concrete
/// SBOL class variant matched its `rdf:type` set. The Identified shared
/// fields (displayId, name, …, derived_from, generated_by, measures,
/// extensions) are populated as usual; when the subject also carried
/// `sbol:TopLevel` semantics, `top_level` is `Some`. The `rdf_types`
/// vector preserves every original `rdf:type` IRI so re-serialization
/// emits exactly what the parser saw.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct IdentifiedExtension {
    pub identity: Resource,
    pub identified: IdentifiedData,
    pub top_level: Option<TopLevelData>,
    pub rdf_types: Vec<Iri>,
}

impl IdentifiedExtension {
    pub fn builder(
        parent: &Resource,
        display_id: impl TryInto<DisplayId, Error = LexError>,
    ) -> Result<IdentifiedExtensionBuilder, BuildError> {
        IdentifiedExtensionBuilder::seed(parent, display_id.try_into()?)
    }
}

impl ToRdf for IdentifiedExtension {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        let mut triples = Vec::new();
        for rdf_type in &self.rdf_types {
            triples.push(Triple {
                subject: self.identity.clone(),
                predicate: Iri::from_static(RDF_TYPE),
                object: Term::Resource(Resource::Iri(rdf_type.clone())),
            });
        }
        if !self
            .rdf_types
            .iter()
            .any(|iri| iri.as_str() == SBOL_IDENTIFIED_CLASS)
        {
            triples.push(Triple {
                subject: self.identity.clone(),
                predicate: Iri::from_static(RDF_TYPE),
                object: Term::Resource(Resource::Iri(Iri::from_static(SBOL_IDENTIFIED_CLASS))),
            });
        }
        push_string(
            &mut triples,
            &self.identity,
            SBOL_DISPLAY_ID,
            self.identified.display_id.as_deref(),
        );
        push_string(
            &mut triples,
            &self.identity,
            SBOL_NAME,
            self.identified.name.as_deref(),
        );
        push_string(
            &mut triples,
            &self.identity,
            SBOL_DESCRIPTION,
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
        for value in &self.identified.measures {
            push_resource(
                &mut triples,
                &self.identity,
                SBOL_HAS_MEASURE,
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
            if let Some(namespace) = &top_level.namespace {
                push_resource(
                    &mut triples,
                    &self.identity,
                    SBOL_HAS_NAMESPACE,
                    Resource::Iri(namespace.clone()),
                );
            }
            for attachment in &top_level.attachments {
                push_resource(
                    &mut triples,
                    &self.identity,
                    SBOL_HAS_ATTACHMENT,
                    attachment.clone(),
                );
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

impl_sbol_identified!(IdentifiedExtension);
