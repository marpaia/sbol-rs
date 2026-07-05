use std::collections::BTreeMap;
use std::path::Path;

use sbol_core::document::{ObjectStore, RawDocument};

use crate::client::{
    Activity, Agent, Association, Attachment, BinaryPrefix, Collection, CombinatorialDerivation,
    Component, ComponentDefinition, CompoundUnit, Cut, Experiment, ExperimentalData,
    FunctionalComponent, GenericLocation, GenericTopLevel, IdentifiedExtension, Implementation,
    Interaction, MapsTo, Measure, Model, Module, ModuleDefinition, Participation, Plan, Prefix,
    PrefixedUnit, Range, SIPrefix, Sbol2Object, Sequence, SequenceAnnotation, SequenceConstraint,
    SingularUnit, TryFromObject, Unit, UnitDivision, UnitExponentiation, UnitMultiplication, Usage,
    VariableComponent,
};
use crate::error::{ReadError, WriteError};
use crate::object::{canonicalize_literals, collect_objects};
use crate::{Iri, Object, RdfFormat, RdfGraph, Resource};

/// An SBOL 2 document parsed from RDF.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Document {
    raw: RawDocument,
    typed: Vec<Sbol2Object>,
}

macro_rules! typed_doc_iter {
    ($method:ident, $variant:ident, $ty:ty) => {
        pub fn $method(&self) -> impl Iterator<Item = &$ty> {
            self.typed.iter().filter_map(|o| match o {
                Sbol2Object::$variant(v) => Some(v),
                _ => None,
            })
        }
    };
}

impl Document {
    /// Parses an SBOL 2 document from an in-memory RDF serialization.
    pub fn read(input: &str, format: RdfFormat) -> Result<Self, ReadError> {
        let graph = RdfGraph::parse(input, format).map_err(ReadError::Rdf)?;
        Ok(Self::from_rdf_graph(graph))
    }

    /// Parses an SBOL 2 document from a file, inferring the format from the
    /// path extension (`.ttl`, `.rdf`, `.jsonld`, `.nt`).
    pub fn read_path(path: impl AsRef<Path>) -> Result<Self, ReadError> {
        let path = path.as_ref();
        let format = RdfFormat::from_path(path).ok_or_else(|| ReadError::UnknownFormat {
            path: path.to_path_buf(),
            extension: path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(str::to_owned),
        })?;
        let input = std::fs::read_to_string(path).map_err(|source| ReadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::read(&input, format)
    }

    /// Reads a Turtle serialization into an SBOL 2 document.
    pub fn read_turtle(input: &str) -> Result<Self, ReadError> {
        Self::read(input, RdfFormat::Turtle)
    }

    /// Builds a document from an already-parsed SBOL 2 RDF graph, indexing its
    /// objects by identity and deriving the owned typed surface.
    pub fn from_rdf_graph(graph: RdfGraph) -> Self {
        let graph = canonicalize_literals(&graph);
        let objects = collect_objects(&graph);
        let typed = objects
            .values()
            .filter_map(Sbol2Object::try_from_object)
            .collect();
        Self {
            raw: RawDocument::from_parts(graph, objects),
            typed,
        }
    }

    pub(crate) fn from_parts(
        graph: RdfGraph,
        objects: BTreeMap<Resource, Object>,
        typed: Vec<Sbol2Object>,
    ) -> Self {
        Self {
            raw: RawDocument::from_parts(graph, objects),
            typed,
        }
    }

    /// Consumes the document, returning its version-neutral [`RawDocument`].
    pub fn into_raw(self) -> RawDocument {
        self.raw
    }

    /// Serializes the document in the given RDF format.
    pub fn write(&self, format: RdfFormat) -> Result<String, WriteError> {
        self.raw.write(format)
    }

    /// Writes the document to a file in the given RDF format.
    pub fn write_path(&self, path: impl AsRef<Path>, format: RdfFormat) -> Result<(), WriteError> {
        self.raw.write_path(path, format)
    }

    /// Serializes the underlying RDF graph as Turtle.
    pub fn write_turtle(&self) -> Result<String, WriteError> {
        self.raw.write_turtle()
    }

    /// Returns the underlying RDF graph.
    pub fn rdf_graph(&self) -> &RdfGraph {
        self.raw.rdf_graph()
    }

    /// Returns RDF-backed objects indexed by identity.
    pub fn objects(&self) -> &BTreeMap<Resource, Object> {
        self.raw.objects()
    }

    /// Returns the RDF-backed object at `identity`, if any.
    pub fn get(&self, identity: &Resource) -> Option<&Object> {
        self.raw.get(identity)
    }

    /// Returns the owned typed SBOL objects in the document, in identity order.
    pub fn typed_objects(&self) -> &[Sbol2Object] {
        &self.typed
    }

    /// Returns the owned typed object whose identity matches `identity`.
    pub fn resolve(&self, identity: &Resource) -> Option<&Sbol2Object> {
        self.typed.iter().find(|o| o.identity() == identity)
    }

    /// Returns the owned typed object whose compliant SBOL 2 identity carries
    /// the given `display_id` under `namespace`. SBOL 2 identities append a
    /// `/version` segment, so this matches both `{namespace}/{display_id}` and
    /// `{namespace}/.../{display_id}/{version}`.
    pub fn find_by_display_id(&self, namespace: &str, display_id: &str) -> Option<&Sbol2Object> {
        let prefix = if namespace.ends_with('/') {
            namespace.to_owned()
        } else {
            format!("{namespace}/")
        };
        let exact = format!("{prefix}{display_id}");
        let versioned = format!("/{display_id}/");
        let tail = format!("/{display_id}");
        self.typed.iter().find(|object| {
            let iri = match object.identity().as_iri() {
                Some(iri) => iri.as_str(),
                None => return false,
            };
            if iri == exact {
                return true;
            }
            iri.starts_with(&prefix) && (iri.contains(&versioned) || iri.ends_with(&tail))
        })
    }

    typed_doc_iter!(sequences, Sequence, Sequence);
    typed_doc_iter!(component_definitions, ComponentDefinition, ComponentDefinition);
    typed_doc_iter!(module_definitions, ModuleDefinition, ModuleDefinition);
    typed_doc_iter!(models, Model, Model);
    typed_doc_iter!(collections, Collection, Collection);
    typed_doc_iter!(
        combinatorial_derivations,
        CombinatorialDerivation,
        CombinatorialDerivation
    );
    typed_doc_iter!(implementations, Implementation, Implementation);
    typed_doc_iter!(attachments, Attachment, Attachment);
    typed_doc_iter!(experimental_data, ExperimentalData, ExperimentalData);
    typed_doc_iter!(experiments, Experiment, Experiment);
    typed_doc_iter!(generic_top_levels, GenericTopLevel, GenericTopLevel);
    typed_doc_iter!(components, Component, Component);
    typed_doc_iter!(functional_components, FunctionalComponent, FunctionalComponent);
    typed_doc_iter!(modules, Module, Module);
    typed_doc_iter!(maps_tos, MapsTo, MapsTo);
    typed_doc_iter!(sequence_annotations, SequenceAnnotation, SequenceAnnotation);
    typed_doc_iter!(sequence_constraints, SequenceConstraint, SequenceConstraint);
    typed_doc_iter!(variable_components, VariableComponent, VariableComponent);
    typed_doc_iter!(interactions, Interaction, Interaction);
    typed_doc_iter!(participations, Participation, Participation);
    typed_doc_iter!(ranges, Range, Range);
    typed_doc_iter!(cuts, Cut, Cut);
    typed_doc_iter!(generic_locations, GenericLocation, GenericLocation);
    typed_doc_iter!(activities, Activity, Activity);
    typed_doc_iter!(agents, Agent, Agent);
    typed_doc_iter!(plans, Plan, Plan);
    typed_doc_iter!(associations, Association, Association);
    typed_doc_iter!(usages, Usage, Usage);
    typed_doc_iter!(measures, Measure, Measure);
    typed_doc_iter!(units, Unit, Unit);
    typed_doc_iter!(singular_units, SingularUnit, SingularUnit);
    typed_doc_iter!(compound_units, CompoundUnit, CompoundUnit);
    typed_doc_iter!(unit_multiplications, UnitMultiplication, UnitMultiplication);
    typed_doc_iter!(unit_divisions, UnitDivision, UnitDivision);
    typed_doc_iter!(unit_exponentiations, UnitExponentiation, UnitExponentiation);
    typed_doc_iter!(prefixed_units, PrefixedUnit, PrefixedUnit);
    typed_doc_iter!(prefixes, Prefix, Prefix);
    typed_doc_iter!(si_prefixes, SIPrefix, SIPrefix);
    typed_doc_iter!(binary_prefixes, BinaryPrefix, BinaryPrefix);
    typed_doc_iter!(identified_extensions, IdentifiedExtension, IdentifiedExtension);

    /// Iterates over the TopLevel typed objects in the document.
    pub fn top_levels(&self) -> impl Iterator<Item = &Sbol2Object> {
        self.typed.iter().filter(|o| o.is_top_level_object())
    }

    /// Iterates over the distinct namespaces declared by TopLevel objects.
    pub fn namespaces(&self) -> Vec<Iri> {
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        let mut out = Vec::new();
        for object in &self.typed {
            if let Some(ns) = object.top_level_namespace() {
                if seen.insert(ns.as_str().to_owned()) {
                    out.push(ns);
                }
            }
        }
        out
    }
}

impl ObjectStore for Document {
    fn objects(&self) -> &BTreeMap<Resource, Object> {
        self.raw.objects()
    }

    fn get(&self, identity: &Resource) -> Option<&Object> {
        self.raw.get(identity)
    }
}
