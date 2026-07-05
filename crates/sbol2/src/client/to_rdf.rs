use std::collections::BTreeMap;

use crate::client::shared::{
    ComponentInstanceData, IdentifiedData, LocationData, MeasuredData, TopLevelData,
};
use crate::client::object::for_each_variant;
use crate::client::{Sbol2Object, ToRdf};
use crate::schema::{FieldDescriptor, ValueKind, class_spec, xsd_datatype};
use crate::vocab::*;
use crate::{BuildError, Iri, Literal, Resource, Sbol2Class, Term, Triple};

impl ToRdf for Sbol2Object {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        for_each_variant!(self, object => object.to_rdf_triples())
    }
}

pub(crate) fn emit_identified(
    e: &mut Emitter<'_>,
    data: &IdentifiedData,
) -> Result<(), BuildError> {
    e.resource(SBOL2_PERSISTENT_IDENTITY, data.persistent_identity.as_ref())?;
    e.literal(SBOL2_DISPLAY_ID, data.display_id.as_deref())?;
    e.literal(SBOL2_VERSION, data.version.as_deref())?;
    e.literal(DCTERMS_TITLE, data.name.as_deref())?;
    e.literal(DCTERMS_DESCRIPTION, data.description.as_deref())?;
    e.resources(PROV_WAS_DERIVED_FROM, &data.derived_from)?;
    e.resources(PROV_WAS_GENERATED_BY, &data.generated_by)?;
    for extension in &data.extensions {
        e.extension_triple(&extension.predicate, &extension.object);
    }
    Ok(())
}

pub(crate) fn emit_top_level(e: &mut Emitter<'_>, data: &TopLevelData) -> Result<(), BuildError> {
    e.resources(SBOL2_ATTACHMENT, &data.attachments)?;
    Ok(())
}

pub(crate) fn emit_measured(e: &mut Emitter<'_>, data: &MeasuredData) -> Result<(), BuildError> {
    e.resources(SBOL2_MEASURE, &data.measures)?;
    Ok(())
}

pub(crate) fn emit_component_instance(
    e: &mut Emitter<'_>,
    data: &ComponentInstanceData,
) -> Result<(), BuildError> {
    e.resource(SBOL2_DEFINITION, data.definition.as_ref())?;
    e.iri(SBOL2_ACCESS, data.access.as_ref())?;
    e.resources(SBOL2_MAPS_TO, &data.maps_tos)?;
    emit_measured(e, &data.measured)?;
    Ok(())
}

pub(crate) fn emit_location(e: &mut Emitter<'_>, data: &LocationData) -> Result<(), BuildError> {
    e.iri(SBOL2_ORIENTATION, data.orientation.as_ref())?;
    e.resource(SBOL2_SEQUENCE, data.sequence.as_ref())?;
    Ok(())
}

/// Returns a fresh triple buffer seeded with the RDF type triple for `class`.
pub(crate) fn seed_triples(identity: &Resource, class: Sbol2Class) -> Vec<Triple> {
    vec![rdf_type_triple(identity, class)]
}

/// Generic, descriptor-driven triple emitter. Looks up the
/// [`FieldDescriptor`] for the (class, predicate) pair, enforces cardinality,
/// and pushes a triple per value.
pub(crate) struct Emitter<'t> {
    triples: &'t mut Vec<Triple>,
    identity: &'t Resource,
    class: Sbol2Class,
    descriptors: BTreeMap<&'static str, FieldDescriptor>,
}

impl<'t> Emitter<'t> {
    pub(crate) fn new(
        triples: &'t mut Vec<Triple>,
        identity: &'t Resource,
        class: Sbol2Class,
    ) -> Self {
        let mut descriptors = BTreeMap::new();
        collect_descriptors(class.iri(), &mut descriptors);
        Self {
            triples,
            identity,
            class,
            descriptors,
        }
    }

    fn value_kind(&self, predicate: &'static str) -> ValueKind {
        self.descriptors
            .get(predicate)
            .unwrap_or_else(|| panic!("unknown predicate `{predicate}` for class {:?}", self.class))
            .value_kind
    }

    pub(crate) fn iris(
        &mut self,
        predicate: &'static str,
        values: &[Iri],
    ) -> Result<(), BuildError> {
        self.value_kind(predicate);
        for value in values {
            push_iri(self.triples, self.identity, predicate, value.clone());
        }
        Ok(())
    }

    pub(crate) fn resources(
        &mut self,
        predicate: &'static str,
        values: &[Resource],
    ) -> Result<(), BuildError> {
        self.value_kind(predicate);
        for value in values {
            push_resource(self.triples, self.identity, predicate, value.clone());
        }
        Ok(())
    }

    pub(crate) fn iri(
        &mut self,
        predicate: &'static str,
        value: Option<&Iri>,
    ) -> Result<(), BuildError> {
        self.value_kind(predicate);
        if let Some(value) = value {
            push_iri(self.triples, self.identity, predicate, value.clone());
        }
        Ok(())
    }

    pub(crate) fn resource(
        &mut self,
        predicate: &'static str,
        value: Option<&Resource>,
    ) -> Result<(), BuildError> {
        self.value_kind(predicate);
        if let Some(value) = value {
            push_resource(self.triples, self.identity, predicate, value.clone());
        }
        Ok(())
    }

    pub(crate) fn literal(
        &mut self,
        predicate: &'static str,
        value: Option<&str>,
    ) -> Result<(), BuildError> {
        let kind = self.value_kind(predicate);
        if let Some(value) = value {
            push_literal(self.triples, self.identity, predicate, value, kind);
        }
        Ok(())
    }

    pub(crate) fn literals(
        &mut self,
        predicate: &'static str,
        values: &[String],
    ) -> Result<(), BuildError> {
        let kind = self.value_kind(predicate);
        for value in values {
            push_literal(self.triples, self.identity, predicate, value, kind);
        }
        Ok(())
    }

    pub(crate) fn i64(
        &mut self,
        predicate: &'static str,
        value: Option<i64>,
    ) -> Result<(), BuildError> {
        let kind = self.value_kind(predicate);
        if let Some(value) = value {
            push_literal(self.triples, self.identity, predicate, &value.to_string(), kind);
        }
        Ok(())
    }

    /// Emit an extension/annotation triple. Bypasses the descriptor table
    /// because extension predicates are by definition outside the SBOL schema.
    pub(crate) fn extension_triple(&mut self, predicate: &Iri, value: &Term) {
        self.triples.push(Triple {
            subject: self.identity.clone(),
            predicate: predicate.clone(),
            object: value.clone(),
        });
    }
}

/// Walks the class hierarchy and collects every field descriptor (inherited
/// plus own) for emission.
fn collect_descriptors(class_iri: &str, out: &mut BTreeMap<&'static str, FieldDescriptor>) {
    let Some(descriptor) = class_spec(class_iri) else {
        return;
    };
    for parent in descriptor.parents {
        collect_descriptors(parent, out);
    }
    for field in descriptor.fields {
        out.insert(field.predicate, *field);
    }
}

fn rdf_type_triple(identity: &Resource, class: Sbol2Class) -> Triple {
    Triple {
        subject: identity.clone(),
        predicate: Iri::from_static(RDF_TYPE),
        object: Term::Resource(Resource::Iri(Iri::from_static(class.iri()))),
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

fn push_iri(triples: &mut Vec<Triple>, identity: &Resource, predicate: &'static str, value: Iri) {
    push_resource(triples, identity, predicate, Resource::Iri(value));
}

fn push_literal(
    triples: &mut Vec<Triple>,
    identity: &Resource,
    predicate: &'static str,
    value: &str,
    kind: ValueKind,
) {
    let literal = match xsd_datatype(kind) {
        Some(datatype) => Literal::new(value, Iri::from_static(datatype), None),
        None => Literal::simple(value),
    };
    triples.push(Triple {
        subject: identity.clone(),
        predicate: Iri::from_static(predicate),
        object: Term::Literal(literal),
    });
}
