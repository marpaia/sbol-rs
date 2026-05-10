use std::collections::BTreeMap;

use crate::client::{FeatureData, IdentifiedData, LocationData, SbolObject, ToRdf, TopLevelData};
use crate::schema::{Cardinality, FieldDescriptor};
use crate::validation::class_spec;
use crate::vocab::*;
use crate::{BuildError, Iri, Literal, Resource, SbolClass, Term, Triple};

impl ToRdf for SbolObject {
    fn to_rdf_triples(&self) -> Result<Vec<Triple>, BuildError> {
        match self {
            Self::Attachment(o) => o.to_rdf_triples(),
            Self::Collection(o) => o.to_rdf_triples(),
            Self::CombinatorialDerivation(o) => o.to_rdf_triples(),
            Self::Component(o) => o.to_rdf_triples(),
            Self::ComponentReference(o) => o.to_rdf_triples(),
            Self::Constraint(o) => o.to_rdf_triples(),
            Self::Cut(o) => o.to_rdf_triples(),
            Self::EntireSequence(o) => o.to_rdf_triples(),
            Self::Experiment(o) => o.to_rdf_triples(),
            Self::ExperimentalData(o) => o.to_rdf_triples(),
            Self::ExternallyDefined(o) => o.to_rdf_triples(),
            Self::Implementation(o) => o.to_rdf_triples(),
            Self::Interaction(o) => o.to_rdf_triples(),
            Self::Interface(o) => o.to_rdf_triples(),
            Self::LocalSubComponent(o) => o.to_rdf_triples(),
            Self::Model(o) => o.to_rdf_triples(),
            Self::Participation(o) => o.to_rdf_triples(),
            Self::Range(o) => o.to_rdf_triples(),
            Self::Sequence(o) => o.to_rdf_triples(),
            Self::SequenceFeature(o) => o.to_rdf_triples(),
            Self::SubComponent(o) => o.to_rdf_triples(),
            Self::VariableFeature(o) => o.to_rdf_triples(),
            Self::Activity(o) => o.to_rdf_triples(),
            Self::Agent(o) => o.to_rdf_triples(),
            Self::Association(o) => o.to_rdf_triples(),
            Self::Plan(o) => o.to_rdf_triples(),
            Self::Usage(o) => o.to_rdf_triples(),
            Self::Measure(o) => o.to_rdf_triples(),
            Self::Unit(o) => o.to_rdf_triples(),
            Self::SingularUnit(o) => o.to_rdf_triples(),
            Self::CompoundUnit(o) => o.to_rdf_triples(),
            Self::UnitDivision(o) => o.to_rdf_triples(),
            Self::UnitExponentiation(o) => o.to_rdf_triples(),
            Self::UnitMultiplication(o) => o.to_rdf_triples(),
            Self::PrefixedUnit(o) => o.to_rdf_triples(),
            Self::Prefix(o) => o.to_rdf_triples(),
            Self::SIPrefix(o) => o.to_rdf_triples(),
            Self::BinaryPrefix(o) => o.to_rdf_triples(),
            Self::IdentifiedExtension(o) => o.to_rdf_triples(),
        }
    }
}

pub(crate) fn emit_identified(
    e: &mut Emitter<'_>,
    data: &IdentifiedData,
) -> Result<(), BuildError> {
    e.literal(SBOL_DISPLAY_ID, data.display_id.as_deref())?;
    e.literal(SBOL_NAME, data.name.as_deref())?;
    e.literal(SBOL_DESCRIPTION, data.description.as_deref())?;
    e.resources(PROV_WAS_DERIVED_FROM, &data.derived_from)?;
    e.resources(PROV_WAS_GENERATED_BY, &data.generated_by)?;
    e.resources(SBOL_HAS_MEASURE, &data.measures)?;
    for extension in &data.extensions {
        e.extension_triple(&extension.predicate, &extension.object);
    }
    Ok(())
}

pub(crate) fn emit_top_level(e: &mut Emitter<'_>, data: &TopLevelData) -> Result<(), BuildError> {
    e.iri(SBOL_HAS_NAMESPACE, data.namespace.as_ref())?;
    e.resources(SBOL_HAS_ATTACHMENT, &data.attachments)?;
    Ok(())
}

pub(crate) fn emit_feature(e: &mut Emitter<'_>, data: &FeatureData) -> Result<(), BuildError> {
    e.iris(SBOL_ROLE, &data.roles)?;
    e.iri(SBOL_ORIENTATION, data.orientation.as_ref())?;
    Ok(())
}

pub(crate) fn emit_location(e: &mut Emitter<'_>, data: &LocationData) -> Result<(), BuildError> {
    e.resource(SBOL_HAS_SEQUENCE, data.sequence.as_ref())?;
    e.iri(SBOL_ORIENTATION, data.orientation.as_ref())?;
    e.i64(SBOL_ORDER, data.order)?;
    Ok(())
}

/// Returns a fresh triple buffer seeded with the RDF type triple for `class`.
pub(crate) fn seed_triples(identity: &Resource, class: SbolClass) -> Vec<Triple> {
    vec![rdf_type_triple(identity, class)]
}

/// Generic, descriptor-driven triple emitter. Looks up the
/// [`FieldDescriptor`] for the (class, predicate) pair, enforces
/// cardinality, and pushes a triple per value.
pub(crate) struct Emitter<'t> {
    triples: &'t mut Vec<Triple>,
    identity: &'t Resource,
    class: SbolClass,
    descriptors: BTreeMap<&'static str, FieldDescriptor>,
}

impl<'t> Emitter<'t> {
    pub(crate) fn new(
        triples: &'t mut Vec<Triple>,
        identity: &'t Resource,
        class: SbolClass,
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

    fn descriptor(&self, predicate: &'static str) -> &FieldDescriptor {
        self.descriptors
            .get(predicate)
            .unwrap_or_else(|| panic!("unknown predicate `{predicate}` for class {:?}", self.class))
    }

    fn missing(&self, predicate: &'static str) -> BuildError {
        BuildError::MissingRequired {
            identity: self.identity.clone(),
            class: self.class,
            property: predicate,
        }
    }

    pub(crate) fn iris(
        &mut self,
        predicate: &'static str,
        values: &[Iri],
    ) -> Result<(), BuildError> {
        let descriptor = self.descriptor(predicate);
        if descriptor.cardinality == Cardinality::OneOrMore && values.is_empty() {
            return Err(self.missing(predicate));
        }
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
        let descriptor = self.descriptor(predicate);
        if descriptor.cardinality == Cardinality::OneOrMore && values.is_empty() {
            return Err(self.missing(predicate));
        }
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
        let descriptor = self.descriptor(predicate);
        match (descriptor.cardinality, value) {
            (Cardinality::ExactlyOne, None) => Err(self.missing(predicate)),
            (_, Some(value)) => {
                push_iri(self.triples, self.identity, predicate, value.clone());
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub(crate) fn resource(
        &mut self,
        predicate: &'static str,
        value: Option<&Resource>,
    ) -> Result<(), BuildError> {
        let descriptor = self.descriptor(predicate);
        match (descriptor.cardinality, value) {
            (Cardinality::ExactlyOne, None) => Err(self.missing(predicate)),
            (_, Some(value)) => {
                push_resource(self.triples, self.identity, predicate, value.clone());
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub(crate) fn literal(
        &mut self,
        predicate: &'static str,
        value: Option<&str>,
    ) -> Result<(), BuildError> {
        let descriptor = self.descriptor(predicate);
        match (descriptor.cardinality, value) {
            (Cardinality::ExactlyOne, None) => Err(self.missing(predicate)),
            (_, Some(value)) => {
                push_literal(self.triples, self.identity, predicate, value);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub(crate) fn literals(
        &mut self,
        predicate: &'static str,
        values: &[String],
    ) -> Result<(), BuildError> {
        let descriptor = self.descriptor(predicate);
        if descriptor.cardinality == Cardinality::OneOrMore && values.is_empty() {
            return Err(self.missing(predicate));
        }
        for value in values {
            push_literal(self.triples, self.identity, predicate, value);
        }
        Ok(())
    }

    pub(crate) fn i64(
        &mut self,
        predicate: &'static str,
        value: Option<i64>,
    ) -> Result<(), BuildError> {
        let descriptor = self.descriptor(predicate);
        match (descriptor.cardinality, value) {
            (Cardinality::ExactlyOne, None) => Err(self.missing(predicate)),
            (_, Some(value)) => {
                push_literal(self.triples, self.identity, predicate, &value.to_string());
                Ok(())
            }
            _ => Ok(()),
        }
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

/// Walks the class hierarchy and collects every field descriptor
/// (inherited plus own) for emission.
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

fn rdf_type_triple(identity: &Resource, class: SbolClass) -> Triple {
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
) {
    triples.push(Triple {
        subject: identity.clone(),
        predicate: Iri::from_static(predicate),
        object: Term::Literal(Literal::simple(value)),
    });
}
