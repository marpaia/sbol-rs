//! GenBank → SBOL 3 conversion engine.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use gb_io::reader::SeqReader;
use gb_io::seq::{Feature, Location, Seq, Topology};
use sbol3::constants::{
    EDAM_IUPAC_DNA, EDAM_IUPAC_PROTEIN, ORIENTATION_INLINE, ORIENTATION_REVERSE_COMPLEMENT,
    SBO_DNA, SBO_PROTEIN, SBO_RNA, SBO_SIMPLE_CHEMICAL, SO_CIRCULAR, SO_LINEAR,
};
use sbol3::{BuildError, Component, Document, Iri, Range as SbolRange, Resource, SbolObject};
use sbol3::{Sequence as SbolSequence, SequenceFeature};

use crate::feature_map::{GENERIC_FEATURE, feature_key_to_so};

/// Imports one or more GenBank records and emits SBOL 3 [`Document`]
/// objects.
///
/// `GenbankImporter::new` takes the namespace IRI that the resulting
/// SBOL 3 top-level objects should be rooted under — typically the
/// owning lab or repository (e.g. `https://example.org/lab`). Component
/// identities are derived as `{namespace}/{accession or locus name}`.
#[derive(Clone, Debug)]
pub struct GenbankImporter {
    namespace: Iri,
}

impl GenbankImporter {
    /// Builds a new importer scoped to the supplied namespace IRI.
    pub fn new(namespace: impl AsRef<str>) -> Result<Self, ImportError> {
        let namespace = Iri::new(namespace.as_ref().to_owned())
            .map_err(|err| ImportError::Namespace(err.to_string()))?;
        Ok(Self { namespace })
    }

    /// Reads every GenBank record from the supplied reader and returns
    /// one SBOL 3 [`Document`] containing every emitted object plus an
    /// [`ImportReport`] tallying what was produced.
    pub fn read<R: Read>(&self, mut reader: R) -> Result<(Document, ImportReport), ImportError> {
        // Pull the whole stream so we can normalize a few well-known
        // dialect quirks (SynBioHub emits mixed-case month names in the
        // LOCUS line, which gb-io's strict parser rejects).
        let mut buffer = String::new();
        reader
            .read_to_string(&mut buffer)
            .map_err(|err| ImportError::Io {
                path: PathBuf::from("<reader>"),
                source: err,
            })?;
        let normalized = normalize_genbank_input(&buffer);
        self.read_str(&normalized)
    }

    /// Reads from a string slice. Convenient for fixtures and tests.
    pub fn read_str(&self, input: &str) -> Result<(Document, ImportReport), ImportError> {
        let normalized = normalize_genbank_input(input);
        let mut objects: Vec<SbolObject> = Vec::new();
        let mut report = ImportReport::default();

        for entry in SeqReader::new(normalized.as_bytes()) {
            let seq = entry.map_err(ImportError::Parse)?;
            self.append_record(&seq, &mut objects, &mut report)?;
        }

        let document = Document::from_objects(objects).map_err(ImportError::Build)?;
        Ok((document, report))
    }

    /// Reads from a file on disk, inferring the path's extension is
    /// `.gb`, `.gbk`, or otherwise GenBank-shaped.
    pub fn read_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(Document, ImportReport), ImportError> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|err| ImportError::Io {
            path: path.to_path_buf(),
            source: err,
        })?;
        let reader = BufReader::new(file);
        self.read(reader)
    }

    fn append_record(
        &self,
        seq: &Seq,
        objects: &mut Vec<SbolObject>,
        report: &mut ImportReport,
    ) -> Result<(), ImportError> {
        let identifier = pick_identifier(seq, report);
        let display_id = sanitize_display_id(&identifier);
        let component_identity = format!("{}/{}", self.namespace.as_str(), display_id);
        let sequence_display_id = format!("{display_id}_sequence");

        // Sequence
        let mut sequence_builder =
            SbolSequence::builder(self.namespace.as_str(), sequence_display_id.as_str())
                .map_err(ImportError::Build)?;
        if !seq.seq.is_empty() {
            let elements = std::str::from_utf8(&seq.seq)
                .map_err(|err| ImportError::NonUtf8Sequence(err.to_string()))?
                .to_ascii_lowercase();
            sequence_builder = sequence_builder.elements(elements);
        }
        let encoding = encoding_for(seq.molecule_type.as_deref());
        sequence_builder = sequence_builder.encoding(encoding);
        let sequence = sequence_builder.build().map_err(ImportError::Build)?;
        let sequence_resource = sequence.identity.clone();
        objects.push(SbolObject::Sequence(sequence));
        report.sequences += 1;

        // Features (built before Component so we can collect their IRIs).
        let component_identity_resource = Resource::iri(component_identity.clone());
        let mut feature_resources: Vec<Resource> = Vec::new();
        let mut feature_objects: Vec<SbolObject> = Vec::new();
        let mut location_objects: Vec<SbolObject> = Vec::new();
        let mut used_display_ids: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for (idx, feature) in seq.features.iter().enumerate() {
            if feature.kind.as_ref() == "source" {
                // Source is metadata about the whole molecule, not an
                // annotation. SBOL3 captures the equivalent via
                // Component.type / hasNamespace.
                continue;
            }
            match self.append_feature(
                &component_identity_resource,
                &sequence_resource,
                feature,
                idx,
                &mut used_display_ids,
                &mut feature_objects,
                &mut location_objects,
                report,
            ) {
                Ok(Some(resource)) => feature_resources.push(resource),
                Ok(None) => {}
                Err(err) => return Err(err),
            }
        }

        // Component
        let mut component_builder =
            Component::builder(self.namespace.as_str(), display_id.as_str())
                .map_err(ImportError::Build)?;
        component_builder = component_builder.types([
            component_type_for(seq.molecule_type.as_deref()),
            // SBOL 3 carries linear/circular as an additional
            // `Component.type` (SO topology term), not as a role.
            match seq.topology {
                Topology::Circular => SO_CIRCULAR,
                Topology::Linear => SO_LINEAR,
            },
        ]);

        if let Some(definition) = seq.definition.as_deref().map(str::trim)
            && !definition.is_empty()
        {
            let first_line = definition.lines().next().unwrap_or(definition);
            component_builder = component_builder.name(first_line);
            if definition.len() > first_line.len() {
                component_builder = component_builder.description(definition);
            }
        }

        component_builder = component_builder.add_sequence(sequence_resource);
        for feature_resource in &feature_resources {
            component_builder = component_builder.add_feature(feature_resource.clone());
        }
        let component = component_builder.build().map_err(ImportError::Build)?;
        let component_iri = component
            .identity
            .as_iri()
            .map(|i| i.as_str().to_owned())
            .unwrap_or_default();
        if component_iri != component_identity {
            return Err(ImportError::Internal(format!(
                "component identity mismatch: expected `{component_identity}`, got `{component_iri}`"
            )));
        }

        objects.push(SbolObject::Component(component));
        objects.extend(feature_objects);
        objects.extend(location_objects);
        report.components += 1;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn append_feature(
        &self,
        component_identity: &Resource,
        sequence_identity: &Resource,
        feature: &Feature,
        index: usize,
        used_display_ids: &mut std::collections::HashSet<String>,
        feature_objects: &mut Vec<SbolObject>,
        location_objects: &mut Vec<SbolObject>,
        report: &mut ImportReport,
    ) -> Result<Option<Resource>, ImportError> {
        let base = pick_feature_display_id(feature, index);
        let feature_display_id = dedupe_display_id(base, used_display_ids);
        let mut sf_builder =
            SequenceFeature::builder(component_identity, feature_display_id.as_str())
                .map_err(ImportError::Build)?;
        // Locations are children of the SequenceFeature per SBOL 3 IRI
        // compliance, so we need the SF's identity to seed Range
        // builders. Peek the builder's identity now; the SequenceFeature
        // itself is finalized below.
        let feature_identity = build_child_identity(component_identity, &feature_display_id);

        // Map the GenBank feature key to its canonical SO term, with a
        // fallback to the generic sequence_feature umbrella.
        let so_iri = feature_key_to_so(&feature.kind)
            .map(Iri::from_static)
            .unwrap_or_else(|| {
                report.warnings.push(ImportWarning::UnknownFeatureKey {
                    kind: feature.kind.to_string(),
                });
                Iri::from_static(GENERIC_FEATURE)
            });
        sf_builder = sf_builder.add_role(so_iri);

        // Pull human-readable metadata from common qualifier keys.
        if let Some(label) =
            qualifier_first(feature, "label").or_else(|| qualifier_first(feature, "gene"))
        {
            sf_builder = sf_builder.name(label);
        } else if let Some(product) = qualifier_first(feature, "product") {
            sf_builder = sf_builder.name(product);
        }
        if let Some(note) = qualifier_first(feature, "note") {
            sf_builder = sf_builder.description(note);
        }

        // Locations.
        let mut location_resources: Vec<Resource> = Vec::new();
        let mut location_index = 0usize;
        if !lower_locations(
            &feature_identity,
            sequence_identity,
            &feature.location,
            &feature_display_id,
            &mut location_index,
            None,
            location_objects,
            &mut location_resources,
            report,
        ) {
            // No locations could be derived; the SequenceFeature would
            // fail validation. Skip emitting it.
            return Ok(None);
        }
        sf_builder = sf_builder.locations(location_resources);

        let sequence_feature = sf_builder.build().map_err(ImportError::Build)?;
        let feature_resource = sequence_feature.identity.clone();

        feature_objects.push(SbolObject::SequenceFeature(sequence_feature));
        report.features += 1;

        Ok(Some(feature_resource))
    }
}

/// Computes the child IRI for a given parent and child displayId. SBOL 3
/// compliance requires `{child} = {parent}/{displayId}`. We compute it
/// independently so we can use the resulting identity as the parent for
/// further nested children before the SequenceFeature is finalized.
fn build_child_identity(parent: &Resource, display_id: &str) -> Resource {
    let parent_iri = parent.as_iri().map(|i| i.as_str()).unwrap_or("");
    Resource::iri(format!("{parent_iri}/{display_id}"))
}

/// Walks an `gb_io::seq::Location` and emits one or more `Range`
/// objects, accumulating both the location SbolObjects and their
/// identity resources. Returns `true` if at least one location was
/// emitted.
#[allow(clippy::too_many_arguments)]
fn lower_locations(
    component_identity: &Resource,
    sequence_identity: &Resource,
    location: &Location,
    feature_display_id: &str,
    index: &mut usize,
    parent_orientation: Option<Iri>,
    location_objects: &mut Vec<SbolObject>,
    location_resources: &mut Vec<Resource>,
    report: &mut ImportReport,
) -> bool {
    match location {
        Location::Range((start, _before), (end, _after)) => {
            // The Range is a direct child of the SequenceFeature
            // (parent IRI ends with the SF's displayId), so the
            // Range's own displayId stays simple — `range` for the
            // sole location, `range_N` for joined / multi-part
            // locations.
            let _ = feature_display_id;
            let display_id = if *index == 0 {
                "range".to_string()
            } else {
                format!("range_{}", *index + 1)
            };
            *index += 1;

            // gb-io stores 0-based half-open ranges; SBOL 3 uses 1-based
            // closed ranges. Convert: SBOL start = gb_io_start + 1,
            // SBOL end = gb_io_end.
            let sbol_start = (*start).saturating_add(1);
            let sbol_end = *end;
            if sbol_end < sbol_start {
                report.warnings.push(ImportWarning::LossyLocation {
                    feature: feature_display_id.to_string(),
                    reason: format!("non-positive range [{start},{end})"),
                });
                return false;
            }

            let mut range_builder =
                match SbolRange::builder(component_identity, display_id.as_str()) {
                    Ok(b) => b,
                    Err(_) => {
                        report.warnings.push(ImportWarning::LossyLocation {
                            feature: feature_display_id.to_string(),
                            reason: format!("invalid display id `{display_id}`"),
                        });
                        return false;
                    }
                };
            range_builder = range_builder
                .start(sbol_start)
                .end(sbol_end)
                .sequence(sequence_identity.clone());
            if let Some(orientation) = parent_orientation.clone() {
                range_builder = range_builder.orientation(orientation);
            } else {
                range_builder = range_builder.orientation(ORIENTATION_INLINE);
            }
            match range_builder.build() {
                Ok(range) => {
                    location_resources.push(range.identity.clone());
                    location_objects.push(SbolObject::Range(range));
                    true
                }
                Err(err) => {
                    report.warnings.push(ImportWarning::LossyLocation {
                        feature: feature_display_id.to_string(),
                        reason: format!("range build failed: {err}"),
                    });
                    false
                }
            }
        }
        Location::Complement(inner) => {
            // Recurse with the orientation flipped. Nested complements
            // unwrap to inline; this matches GenBank semantics where
            // `complement(complement(x..y))` == `x..y`.
            let next_orientation = match parent_orientation {
                Some(orient) if orient.as_str() == ORIENTATION_REVERSE_COMPLEMENT.as_str() => {
                    Some(ORIENTATION_INLINE)
                }
                _ => Some(ORIENTATION_REVERSE_COMPLEMENT),
            };
            lower_locations(
                component_identity,
                sequence_identity,
                inner,
                feature_display_id,
                index,
                next_orientation,
                location_objects,
                location_resources,
                report,
            )
        }
        Location::Join(parts) | Location::Order(parts) => {
            let mut any = false;
            for part in parts {
                if lower_locations(
                    component_identity,
                    sequence_identity,
                    part,
                    feature_display_id,
                    index,
                    parent_orientation.clone(),
                    location_objects,
                    location_resources,
                    report,
                ) {
                    any = true;
                }
            }
            any
        }
        other => {
            // Bond, External, Gap, etc. are rare in plasmid annotations
            // and don't have a direct SBOL 3 mapping. Record as lossy.
            report.warnings.push(ImportWarning::LossyLocation {
                feature: feature_display_id.to_string(),
                reason: format!("unsupported location shape: {other:?}"),
            });
            false
        }
    }
}

fn pick_identifier(seq: &Seq, report: &mut ImportReport) -> String {
    if let Some(accession) = seq.accession.as_deref().filter(|s| !s.is_empty()) {
        return accession.to_owned();
    }
    if let Some(name) = seq.name.as_deref().filter(|s| !s.is_empty()) {
        return name.to_owned();
    }
    report.warnings.push(ImportWarning::SynthesizedIdentifier);
    "imported_record".to_owned()
}

fn dedupe_display_id(base: String, used: &mut std::collections::HashSet<String>) -> String {
    if used.insert(base.clone()) {
        return base;
    }
    for suffix in 2.. {
        let candidate = format!("{base}_{suffix}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!("display ID space exhausted");
}

fn pick_feature_display_id(feature: &Feature, index: usize) -> String {
    if let Some(label) = qualifier_first(feature, "locus_tag")
        .or_else(|| qualifier_first(feature, "label"))
        .or_else(|| qualifier_first(feature, "gene"))
        && let Some(sanitized) = sanitize_display_id_opt(label)
    {
        return sanitized;
    }
    format!("{}_{}", sanitize_display_id(feature.kind.as_ref()), index)
}

fn qualifier_first<'a>(feature: &'a Feature, key: &'a str) -> Option<&'a str> {
    feature.qualifier_values(key).next()
}

fn component_type_for(molecule_type: Option<&str>) -> Iri {
    match molecule_type {
        Some(mt) if mt.contains("RNA") => SBO_RNA,
        Some(mt) if mt.contains("protein") || mt.eq_ignore_ascii_case("AA") => SBO_PROTEIN,
        Some(mt) if mt.contains("chem") => SBO_SIMPLE_CHEMICAL,
        _ => SBO_DNA,
    }
}

fn encoding_for(molecule_type: Option<&str>) -> Iri {
    if matches!(molecule_type, Some(mt) if mt.contains("protein") || mt.eq_ignore_ascii_case("AA"))
    {
        EDAM_IUPAC_PROTEIN
    } else {
        EDAM_IUPAC_DNA
    }
}

/// Replaces every non-`[A-Za-z0-9_]` character with `_`. SBOL 3 display
/// IDs must be valid Turtle local names and start with a non-digit /
/// non-underscore in the canonical case.
fn sanitize_display_id(raw: &str) -> String {
    sanitize_display_id_opt(raw).unwrap_or_else(|| "record".to_owned())
}

/// Normalizes a few real-world GenBank dialect quirks that gb-io's strict
/// nom grammar rejects:
///
/// - **LOCUS-line dates with mixed-case month abbreviations** (e.g.
///   `20-May-2026`) emitted by SynBioHub. The GenBank spec requires
///   uppercase (`20-MAY-2026`); we uppercase the three-letter month
///   token so the parser accepts the file. No other characters on the
///   line are modified.
///
/// Lines that aren't LOCUS lines pass through unchanged.
fn normalize_genbank_input(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for (idx, line) in text.split_inclusive('\n').enumerate() {
        if let Some(rest) = line.strip_prefix("LOCUS ")
            && let Some(fixed) = uppercase_month_token(rest)
        {
            out.push_str("LOCUS ");
            out.push_str(&fixed);
            continue;
        }
        // Heuristic: also retry the first line even if it doesn't
        // exactly start with "LOCUS " (some uploaders include trailing
        // whitespace on previous lines).
        if idx == 0
            && line.trim_start().starts_with("LOCUS")
            && let Some(stripped) = line.split_once("LOCUS").map(|(_, rest)| rest)
            && let Some(fixed) = uppercase_month_token(stripped)
        {
            out.push_str("LOCUS");
            out.push_str(&fixed);
            continue;
        }
        out.push_str(line);
    }
    out
}

fn uppercase_month_token(line: &str) -> Option<String> {
    const MONTHS: [&str; 12] = [
        "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
    ];
    let bytes = line.as_bytes();
    // Walk the line looking for `-XXX-` where XXX is a 3-letter month
    // in any case. There's typically one such token per LOCUS line.
    let mut idx = 0;
    while idx + 5 <= bytes.len() {
        if bytes[idx] == b'-' && bytes[idx + 4] == b'-' {
            let month = &line[idx + 1..idx + 4];
            let upper = month.to_ascii_uppercase();
            if MONTHS.contains(&upper.as_str()) && month != upper {
                let mut buf = String::with_capacity(line.len());
                buf.push_str(&line[..idx + 1]);
                buf.push_str(&upper);
                buf.push_str(&line[idx + 4..]);
                return Some(buf);
            }
        }
        idx += 1;
    }
    None
}

fn sanitize_display_id_opt(raw: &str) -> Option<String> {
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        return None;
    }
    if out
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        out.insert(0, '_');
    }
    Some(out)
}

/// Tally of what an [`GenbankImporter`] run produced. Useful for
/// summarizing CLI output and writing structural tests.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ImportReport {
    pub components: usize,
    pub sequences: usize,
    pub features: usize,
    pub warnings: Vec<ImportWarning>,
}

impl ImportReport {
    pub fn is_clean(&self) -> bool {
        self.warnings.is_empty()
    }
}

/// Non-fatal issues encountered while importing GenBank.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImportWarning {
    /// A GenBank feature key wasn't in the curated SO mapping; the
    /// importer fell back to `SO:0000110` (sequence_feature) for the
    /// resulting SequenceFeature.
    UnknownFeatureKey { kind: String },
    /// A feature's location shape couldn't be lowered to SBOL 3
    /// (rare — affects Bond, External, Gap, malformed ranges). The
    /// SequenceFeature was skipped.
    LossyLocation { feature: String, reason: String },
    /// The record had neither an ACCESSION nor a LOCUS name; the
    /// importer synthesized one (`imported_record`).
    SynthesizedIdentifier,
}

/// Fatal errors from [`GenbankImporter`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ImportError {
    /// The namespace IRI was invalid.
    Namespace(String),
    /// gb-io failed to parse a record.
    Parse(gb_io::reader::GbParserError),
    /// Filesystem read failure (for [`GenbankImporter::read_path`]).
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// SBOL 3 object construction failed (typically an invalid
    /// displayId or namespace).
    Build(BuildError),
    /// The GenBank sequence bytes were not valid UTF-8.
    NonUtf8Sequence(String),
    /// Internal invariant violation.
    Internal(String),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Namespace(msg) => write!(f, "invalid namespace: {msg}"),
            Self::Parse(err) => write!(f, "GenBank parse failed: {err}"),
            Self::Io { path, source } => {
                write!(f, "failed to read {}: {source}", path.display())
            }
            Self::Build(err) => write!(f, "SBOL object construction failed: {err}"),
            Self::NonUtf8Sequence(msg) => write!(f, "sequence bytes were not UTF-8: {msg}"),
            Self::Internal(msg) => write!(f, "internal: {msg}"),
        }
    }
}

impl std::error::Error for ImportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(err) => Some(err),
            Self::Io { source, .. } => Some(source),
            Self::Build(err) => Some(err),
            _ => None,
        }
    }
}
