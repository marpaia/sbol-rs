use std::collections::BTreeMap;
use std::path::Path;

use crate::client::{
    Activity, Agent, Association, Attachment, BinaryPrefix, Collection, CombinatorialDerivation,
    Component, ComponentReference, CompoundUnit, Constraint, Cut, EntireSequence, Experiment,
    ExperimentalData, ExternallyDefined, IdentifiedExtension, Implementation, Interaction,
    Interface, LocalSubComponent, Measure, Model, Participation, Plan, Prefix, PrefixedUnit, Range,
    SIPrefix, SbolObject, Sequence, SequenceFeature, SingularUnit, SubComponent, TryFromObject,
    Unit, UnitDivision, UnitExponentiation, UnitMultiplication, Usage, VariableFeature,
};
use crate::error::{ReadError, WriteError};
use crate::object::collect_objects;
use crate::upgrade::{UpgradeError, UpgradeOptions, UpgradeReport, sbol2_to_sbol3};
use crate::validation::{ValidationContext, ValidationOptions, ValidationReport, Validator};
use crate::{Object, RdfFormat, RdfGraph, Resource};

/// An SBOL document parsed from RDF.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Document {
    graph: RdfGraph,
    objects: BTreeMap<Resource, Object>,
    typed: Vec<SbolObject>,
}

macro_rules! typed_doc_iter {
    ($method:ident, $variant:ident, $ty:ty) => {
        pub fn $method(&self) -> impl Iterator<Item = &$ty> {
            self.typed.iter().filter_map(|o| match o {
                SbolObject::$variant(v) => Some(v),
                _ => None,
            })
        }
    };
}

impl Document {
    /// Parses an SBOL document from an in-memory RDF serialization.
    pub fn read(input: &str, format: RdfFormat) -> Result<Self, ReadError> {
        let graph = RdfGraph::parse(input, format).map_err(ReadError::Rdf)?;
        Ok(Self::from_rdf_graph(graph))
    }

    /// Parses an SBOL document from a file. The format is inferred from the
    /// path's extension (`.ttl`, `.rdf`, `.jsonld`, `.nt`). Returns
    /// [`ReadError::UnknownFormat`] for any other extension.
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

    /// Reads a Turtle serialization into an SBOL document.
    pub fn read_turtle(input: &str) -> Result<Self, ReadError> {
        Self::read(input, RdfFormat::Turtle)
    }

    /// Upgrades an SBOL 2 RDF document to SBOL 3 and returns the resulting
    /// [`Document`] alongside an [`UpgradeReport`] of any non-fatal issues
    /// encountered during conversion.
    ///
    /// The returned [`Document`] is always produced when the input parses as
    /// valid SBOL 2 RDF — call [`Document::check`] if you want a strict
    /// pipeline that rejects content the upgrade could not coerce into
    /// fully-conformant SBOL 3.
    pub fn upgrade_from_sbol2(
        input: &str,
        format: RdfFormat,
    ) -> Result<(Self, UpgradeReport), UpgradeError> {
        Self::upgrade_from_sbol2_with(input, format, UpgradeOptions::default())
    }

    /// Like [`Document::upgrade_from_sbol2`], with explicit
    /// [`UpgradeOptions`].
    pub fn upgrade_from_sbol2_with(
        input: &str,
        format: RdfFormat,
        options: UpgradeOptions,
    ) -> Result<(Self, UpgradeReport), UpgradeError> {
        let parsed = RdfGraph::parse(input, format).map_err(UpgradeError::Parse)?;
        let (upgraded, report) = sbol2_to_sbol3(&parsed, options)?;
        Ok((Self::from_rdf_graph(upgraded), report))
    }

    /// Reads an SBOL 2 RDF file from disk and upgrades it to SBOL 3. The
    /// format is inferred from the path's extension (`.ttl`, `.rdf`, `.xml`,
    /// `.jsonld`, `.nt`).
    pub fn upgrade_from_sbol2_path(
        path: impl AsRef<Path>,
    ) -> Result<(Self, UpgradeReport), UpgradeFromPathError> {
        let path = path.as_ref();
        let format =
            infer_sbol2_rdf_format(path).ok_or_else(|| UpgradeFromPathError::UnknownFormat {
                path: path.to_path_buf(),
                extension: path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(str::to_owned),
            })?;
        let input = std::fs::read_to_string(path).map_err(|source| UpgradeFromPathError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::upgrade_from_sbol2(&input, format).map_err(UpgradeFromPathError::Upgrade)
    }

    pub(crate) fn from_rdf_graph(graph: RdfGraph) -> Self {
        let objects = collect_objects(&graph);
        let typed = objects
            .values()
            .filter_map(SbolObject::try_from_object)
            .collect();
        Self {
            graph,
            objects,
            typed,
        }
    }

    pub(crate) fn from_parts(
        graph: RdfGraph,
        objects: BTreeMap<Resource, Object>,
        typed: Vec<SbolObject>,
    ) -> Self {
        Self {
            graph,
            objects,
            typed,
        }
    }

    /// Serializes the document in the given RDF format.
    pub fn write(&self, format: RdfFormat) -> Result<String, WriteError> {
        self.graph.write(format).map_err(WriteError::Rdf)
    }

    /// Writes the document to a file in the given RDF format. The caller
    /// chooses the format explicitly; no inference from the path's
    /// extension is performed.
    pub fn write_path(&self, path: impl AsRef<Path>, format: RdfFormat) -> Result<(), WriteError> {
        let path = path.as_ref();
        let serialized = self.write(format)?;
        std::fs::write(path, serialized).map_err(|source| WriteError::Io {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Serializes the underlying RDF graph as Turtle.
    pub fn write_turtle(&self) -> Result<String, WriteError> {
        self.write(RdfFormat::Turtle)
    }

    /// Returns the underlying RDF graph.
    pub fn rdf_graph(&self) -> &RdfGraph {
        &self.graph
    }

    /// Returns RDF-backed objects indexed by identity.
    ///
    /// These are property-bag values preserving every triple under each
    /// subject — including PROV/OM and extension classes that do not yet
    /// have an owned typed representation. For SBOL classes with an owned
    /// surface, prefer [`Document::components`] and friends.
    pub fn objects(&self) -> &BTreeMap<Resource, Object> {
        &self.objects
    }

    /// Returns the RDF-backed object at `identity`, if any.
    pub fn get(&self, identity: &Resource) -> Option<&Object> {
        self.objects.get(identity)
    }

    /// Returns the owned typed SBOL objects in the document, in identity order.
    pub fn typed_objects(&self) -> &[SbolObject] {
        &self.typed
    }

    /// Returns the owned typed object whose identity matches `identity`.
    pub fn resolve(&self, identity: &Resource) -> Option<&SbolObject> {
        self.typed.iter().find(|o| o.identity() == identity)
    }

    /// Returns the owned typed object whose compliant identity matches
    /// `{namespace}/[local/]{display_id}`. The `local` path is optional
    /// per SBOL 3.1.0 §5.1, so this scans every typed object whose
    /// identity has the right namespace prefix and ends in the given
    /// display_id rather than constructing a fixed IRI.
    pub fn find_by_display_id(&self, namespace: &str, display_id: &str) -> Option<&SbolObject> {
        let prefix = if namespace.ends_with('/') {
            namespace.to_owned()
        } else {
            format!("{namespace}/")
        };
        let suffix = format!("/{display_id}");
        let exact = format!("{prefix}{display_id}");
        self.typed.iter().find(|object| {
            let identity = object.identity();
            let iri = match identity.as_iri() {
                Some(iri) => iri.as_str(),
                None => return false,
            };
            if iri == exact {
                return true;
            }
            iri.starts_with(&prefix) && iri.ends_with(&suffix)
        })
    }

    typed_doc_iter!(attachments, Attachment, Attachment);
    typed_doc_iter!(collections, Collection, Collection);
    typed_doc_iter!(
        combinatorial_derivations,
        CombinatorialDerivation,
        CombinatorialDerivation
    );
    typed_doc_iter!(components, Component, Component);
    typed_doc_iter!(component_references, ComponentReference, ComponentReference);
    typed_doc_iter!(constraints, Constraint, Constraint);
    typed_doc_iter!(cuts, Cut, Cut);
    typed_doc_iter!(entire_sequences, EntireSequence, EntireSequence);
    typed_doc_iter!(experiments, Experiment, Experiment);
    typed_doc_iter!(experimental_data, ExperimentalData, ExperimentalData);
    typed_doc_iter!(externally_defined, ExternallyDefined, ExternallyDefined);
    typed_doc_iter!(implementations, Implementation, Implementation);
    typed_doc_iter!(interactions, Interaction, Interaction);
    typed_doc_iter!(interfaces, Interface, Interface);
    typed_doc_iter!(local_sub_components, LocalSubComponent, LocalSubComponent);
    typed_doc_iter!(models, Model, Model);
    typed_doc_iter!(participations, Participation, Participation);
    typed_doc_iter!(ranges, Range, Range);
    typed_doc_iter!(sequences, Sequence, Sequence);
    typed_doc_iter!(sequence_features, SequenceFeature, SequenceFeature);
    typed_doc_iter!(sub_components, SubComponent, SubComponent);
    typed_doc_iter!(variable_features, VariableFeature, VariableFeature);
    typed_doc_iter!(activities, Activity, Activity);
    typed_doc_iter!(agents, Agent, Agent);
    typed_doc_iter!(associations, Association, Association);
    typed_doc_iter!(plans, Plan, Plan);
    typed_doc_iter!(usages, Usage, Usage);
    typed_doc_iter!(measures, Measure, Measure);
    typed_doc_iter!(units, Unit, Unit);
    typed_doc_iter!(singular_units, SingularUnit, SingularUnit);
    typed_doc_iter!(compound_units, CompoundUnit, CompoundUnit);
    typed_doc_iter!(unit_divisions, UnitDivision, UnitDivision);
    typed_doc_iter!(unit_exponentiations, UnitExponentiation, UnitExponentiation);
    typed_doc_iter!(unit_multiplications, UnitMultiplication, UnitMultiplication);
    typed_doc_iter!(prefixed_units, PrefixedUnit, PrefixedUnit);
    typed_doc_iter!(prefixes, Prefix, Prefix);
    typed_doc_iter!(si_prefixes, SIPrefix, SIPrefix);
    typed_doc_iter!(binary_prefixes, BinaryPrefix, BinaryPrefix);
    typed_doc_iter!(
        identified_extensions,
        IdentifiedExtension,
        IdentifiedExtension
    );

    /// Iterates over the TopLevel typed objects in the document.
    pub fn top_levels(&self) -> impl Iterator<Item = &SbolObject> {
        self.typed
            .iter()
            .filter(|o| o.top_level_namespace().is_some())
    }

    /// Iterates over the distinct namespaces declared by TopLevel objects in
    /// the document.
    pub fn namespaces(&self) -> impl Iterator<Item = &crate::Iri> + '_ {
        let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        self.typed.iter().filter_map(move |object| {
            let ns = object.top_level_namespace()?;
            if seen.insert(ns.as_str()) {
                Some(ns)
            } else {
                None
            }
        })
    }

    /// Builds a structured validation report.
    pub fn validate(&self) -> ValidationReport {
        self.validate_with(ValidationOptions::default())
    }

    /// Builds a structured validation report with explicit validation options.
    pub fn validate_with(&self, options: ValidationOptions) -> ValidationReport {
        let mut validator = Validator::new(self, options);
        validator.validate();
        validator.finish()
    }

    /// Builds a structured validation report with resolver-aware validation context.
    pub fn validate_with_context(&self, context: ValidationContext<'_>) -> ValidationReport {
        let mut validator = Validator::new_with_context(self, context);
        validator.validate();
        validator.finish()
    }

    /// Runs validation and returns the report wrapped as `Ok` when no
    /// fully-evaluated rule reported an error, or `Err` carrying the
    /// same report when any rule did. Coverage gaps from `Partial` rules
    /// do not on their own cause `Err`; use [`check_complete`] for that.
    ///
    /// [`check_complete`]: Document::check_complete
    pub fn check(&self) -> Result<ValidationReport, ValidationReport> {
        check_outcome(self.validate(), false)
    }

    /// `check` with explicit validation options.
    pub fn check_with(
        &self,
        options: ValidationOptions,
    ) -> Result<ValidationReport, ValidationReport> {
        check_outcome(self.validate_with(options), false)
    }

    /// `check` with explicit resolver-aware validation context.
    pub fn check_with_context(
        &self,
        context: ValidationContext<'_>,
    ) -> Result<ValidationReport, ValidationReport> {
        check_outcome(self.validate_with_context(context), false)
    }

    /// Like [`check`], but also returns `Err` when any rule's coverage
    /// is partial — i.e. the validator was unable to fully evaluate it
    /// for this run. Use for CI gates against documents the team
    /// controls end-to-end.
    ///
    /// [`check`]: Document::check
    pub fn check_complete(&self) -> Result<ValidationReport, ValidationReport> {
        check_outcome(self.validate(), true)
    }

    /// `check_complete` with explicit validation options.
    pub fn check_complete_with(
        &self,
        options: ValidationOptions,
    ) -> Result<ValidationReport, ValidationReport> {
        check_outcome(self.validate_with(options), true)
    }

    /// `check_complete` with explicit resolver-aware validation context.
    pub fn check_complete_with_context(
        &self,
        context: ValidationContext<'_>,
    ) -> Result<ValidationReport, ValidationReport> {
        check_outcome(self.validate_with_context(context), true)
    }
}

fn check_outcome(
    report: ValidationReport,
    require_complete: bool,
) -> Result<ValidationReport, ValidationReport> {
    if report.has_errors() {
        return Err(report);
    }
    if require_complete && !report.coverage().partially_applied.is_empty() {
        return Err(report);
    }
    Ok(report)
}

fn infer_sbol2_rdf_format(path: &Path) -> Option<RdfFormat> {
    if let Some(format) = RdfFormat::from_path(path) {
        return Some(format);
    }
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    (extension == "xml").then_some(RdfFormat::RdfXml)
}

/// Errors returned by [`Document::upgrade_from_sbol2_path`].
#[derive(Debug)]
#[non_exhaustive]
pub enum UpgradeFromPathError {
    /// Failed to read the file at the given path.
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    /// The path's extension did not match any supported RDF serialization.
    UnknownFormat {
        path: std::path::PathBuf,
        extension: Option<String>,
    },
    /// The file was loaded but the upgrade itself failed.
    Upgrade(UpgradeError),
}

impl std::fmt::Display for UpgradeFromPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "failed to read {}: {source}", path.display()),
            Self::UnknownFormat { path, extension } => {
                let ext = extension.as_deref().unwrap_or("<none>");
                write!(
                    f,
                    "unsupported extension `{ext}` for {} — supported: .ttl, .rdf, .jsonld, .nt",
                    path.display()
                )
            }
            Self::Upgrade(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for UpgradeFromPathError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::UnknownFormat { .. } => None,
            Self::Upgrade(err) => Some(err),
        }
    }
}
