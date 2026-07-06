//! TSV snapshot loading: parsing the bundled and extension fact tables into
//! an [`Ontology`]. The query surface lives in the crate root; this module
//! owns construction and snapshot merging.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;

use crate::{
    ComponentTypeFamily, Ontology, OntologyNamespace, OntologyProvenance, SequenceEncodingFamily,
    TSV_FORMAT_VERSION, TermFact, TermRole, ordered_pair,
};

impl Ontology {
    /// Parses an ontology snapshot from a TSV string in the bundled format.
    /// Provenance is left empty; use [`Ontology::set_provenance`] if you need
    /// to attach metadata for diagnostic output.
    pub fn from_tsv_str(facts: &str) -> Result<Self, String> {
        Self::from_tsv(facts, "")
    }

    /// Parses an ontology snapshot from a TSV file on disk.
    pub fn from_tsv_path(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        let path = path.as_ref();
        let text = fs::read_to_string(path)?;
        Self::from_tsv_str(&text)
            .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))
    }

    /// Replaces the snapshot's provenance entries.
    pub fn set_provenance(&mut self, provenance: Vec<OntologyProvenance>) {
        self.provenance = provenance;
    }

    /// Merges `other` into `self`. The current snapshot wins on every duplicate
    /// term, alias, or compatibility row. Extensions can add new facts but
    /// cannot rewrite bundled ones. Provenance from `other` is appended.
    pub fn extend_with(&mut self, other: Ontology) {
        for (id, fact) in other.terms {
            self.terms.entry(id).or_insert(fact);
        }
        for (alias, canonical) in other.aliases {
            self.aliases.entry(alias).or_insert(canonical);
        }
        self.branches.extend(other.branches);
        self.compatibilities.extend(other.compatibilities);
        self.conflicts.extend(other.conflicts);
        self.component_role_terms.extend(other.component_role_terms);
        self.component_role_compatibilities
            .extend(other.component_role_compatibilities);
        self.participation_compatibilities
            .extend(other.participation_compatibilities);
        self.provenance.extend(other.provenance);
    }

    pub(crate) fn from_tsv(facts: &str, sources: &str) -> Result<Self, String> {
        let mut ontology = Self {
            terms: BTreeMap::new(),
            aliases: BTreeMap::new(),
            branches: BTreeSet::new(),
            compatibilities: BTreeSet::new(),
            conflicts: BTreeSet::new(),
            component_role_terms: BTreeSet::new(),
            component_role_compatibilities: BTreeSet::new(),
            participation_compatibilities: BTreeSet::new(),
            provenance: parse_sources(sources)?,
        };

        let mut format_version: Option<u32> = None;
        for (line_number, line) in facts.lines().enumerate() {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("# format_version:") {
                let value = rest.trim();
                let parsed = value.parse::<u32>().map_err(|_| {
                    format!(
                        "ontology snapshot has unparseable format_version `{value}` on line {}",
                        line_number + 1
                    )
                })?;
                format_version = Some(parsed);
                continue;
            }
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let columns = line.split('\t').collect::<Vec<_>>();
            match columns.first().copied() {
                Some("term") => ontology.insert_term(&columns, line_number + 1)?,
                Some("branch") => ontology.insert_branch(&columns, line_number + 1)?,
                Some("compat") => ontology.insert_compatibility(&columns, line_number + 1)?,
                Some("conflict") => ontology.insert_conflict(&columns, line_number + 1)?,
                Some("component_role") => {
                    ontology.insert_component_role_term(&columns, line_number + 1)?
                }
                Some("component_role_compat") => {
                    ontology.insert_component_role_compatibility(&columns, line_number + 1)?
                }
                Some("participation_compat") => {
                    ontology.insert_participation_compatibility(&columns, line_number + 1)?
                }
                Some(other) => {
                    return Err(format!(
                        "unknown ontology fact kind `{other}` on line {line_number}"
                    ));
                }
                None => {}
            }
        }

        match format_version {
            Some(version) if version == TSV_FORMAT_VERSION => Ok(ontology),
            Some(version) => Err(format!(
                "ontology snapshot uses format_version {version} but this build only supports {TSV_FORMAT_VERSION}",
            )),
            None => Err(format!(
                "ontology snapshot is missing the `# format_version: {TSV_FORMAT_VERSION}` header line",
            )),
        }
    }

    fn insert_term(&mut self, columns: &[&str], line_number: usize) -> Result<(), String> {
        if columns.len() != 12 {
            return Err(format!(
                "term line {line_number} has {} columns",
                columns.len()
            ));
        }
        let id = columns[1].to_owned();
        let aliases = split_list(columns[4]);
        let parents = split_list(columns[5]);
        let fact = TermFact {
            iri: columns[2].to_owned(),
            label: columns[3].to_owned(),
            parents,
            namespace: parse_namespace(columns[6])?,
            role: parse_role(columns[7])?,
            component_family: parse_component_family(columns[8])?,
            sequence_family: parse_sequence_family(columns[9])?,
            table_1_sequence_encoding: parse_bool(columns[10])?,
            table_2_component_type: parse_bool(columns[11])?,
        };

        self.aliases.insert(id.clone(), id.clone());
        self.aliases.insert(fact.iri.clone(), id.clone());
        for alias in aliases {
            self.aliases.insert(alias, id.clone());
        }
        self.terms.insert(id, fact);
        Ok(())
    }

    fn insert_compatibility(&mut self, columns: &[&str], line_number: usize) -> Result<(), String> {
        if columns.len() != 3 {
            return Err(format!(
                "compatibility line {line_number} has {} columns",
                columns.len()
            ));
        }
        self.compatibilities
            .insert((columns[1].to_owned(), columns[2].to_owned()));
        Ok(())
    }

    fn insert_branch(&mut self, columns: &[&str], line_number: usize) -> Result<(), String> {
        if columns.len() != 3 {
            return Err(format!(
                "branch line {line_number} has {} columns",
                columns.len()
            ));
        }
        self.branches
            .insert((columns[1].to_owned(), columns[2].to_owned()));
        Ok(())
    }

    fn insert_conflict(&mut self, columns: &[&str], line_number: usize) -> Result<(), String> {
        if columns.len() != 3 {
            return Err(format!(
                "conflict line {line_number} has {} columns",
                columns.len()
            ));
        }
        self.conflicts.insert(ordered_pair(columns[1], columns[2]));
        Ok(())
    }

    fn insert_component_role_term(
        &mut self,
        columns: &[&str],
        line_number: usize,
    ) -> Result<(), String> {
        if columns.len() != 2 {
            return Err(format!(
                "component role line {line_number} has {} columns",
                columns.len()
            ));
        }
        self.component_role_terms.insert(columns[1].to_owned());
        Ok(())
    }

    fn insert_component_role_compatibility(
        &mut self,
        columns: &[&str],
        line_number: usize,
    ) -> Result<(), String> {
        if columns.len() != 3 {
            return Err(format!(
                "component role compatibility line {line_number} has {} columns",
                columns.len()
            ));
        }
        self.component_role_compatibilities
            .insert((columns[1].to_owned(), columns[2].to_owned()));
        Ok(())
    }

    fn insert_participation_compatibility(
        &mut self,
        columns: &[&str],
        line_number: usize,
    ) -> Result<(), String> {
        if columns.len() != 3 {
            return Err(format!(
                "participation compatibility line {line_number} has {} columns",
                columns.len()
            ));
        }
        self.participation_compatibilities
            .insert((columns[1].to_owned(), columns[2].to_owned()));
        Ok(())
    }
}

fn parse_sources(sources: &str) -> Result<Vec<OntologyProvenance>, String> {
    let mut provenance = Vec::new();
    for (line_number, line) in sources.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 8 {
            return Err(format!(
                "ontology source line {line_number} has {} columns",
                columns.len()
            ));
        }
        provenance.push(OntologyProvenance {
            ontology: columns[0].to_owned(),
            source_url: columns[1].to_owned(),
            version: columns[2].to_owned(),
            license: columns[3].to_owned(),
            retrieved: columns[4].to_owned(),
            raw_sha256: columns[5].to_owned(),
            fact_sha256: columns[6].to_owned(),
            notes: columns[7].to_owned(),
        });
    }
    Ok(provenance)
}

fn split_list(value: &str) -> Vec<String> {
    if value == "-" {
        return Vec::new();
    }
    value.split('|').map(ToOwned::to_owned).collect()
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("invalid boolean `{value}`")),
    }
}

fn parse_namespace(value: &str) -> Result<OntologyNamespace, String> {
    match value {
        "EDAM" => Ok(OntologyNamespace::Edam),
        "SBO" => Ok(OntologyNamespace::Sbo),
        "SO" => Ok(OntologyNamespace::So),
        "GO" => Ok(OntologyNamespace::Go),
        "CHEBI" => Ok(OntologyNamespace::Chebi),
        "CL" => Ok(OntologyNamespace::Cl),
        "NCIT" => Ok(OntologyNamespace::Ncit),
        _ => Err(format!("unknown ontology namespace `{value}`")),
    }
}

fn parse_role(value: &str) -> Result<TermRole, String> {
    match value {
        "sequence_encoding" => Ok(TermRole::SequenceEncoding),
        "component_type" => Ok(TermRole::ComponentType),
        "component_type_modifier" => Ok(TermRole::ComponentTypeModifier),
        "interaction_type" => Ok(TermRole::InteractionType),
        "participation_role" => Ok(TermRole::ParticipationRole),
        "feature_role" => Ok(TermRole::FeatureRole),
        "other" => Ok(TermRole::Other),
        _ => Err(format!("unknown term role `{value}`")),
    }
}

fn parse_component_family(value: &str) -> Result<Option<ComponentTypeFamily>, String> {
    match value {
        "-" => Ok(None),
        "nucleic_acid" => Ok(Some(ComponentTypeFamily::NucleicAcid)),
        "protein" => Ok(Some(ComponentTypeFamily::Protein)),
        "simple_chemical" => Ok(Some(ComponentTypeFamily::SimpleChemical)),
        "complex" => Ok(Some(ComponentTypeFamily::Complex)),
        "functional" => Ok(Some(ComponentTypeFamily::Functional)),
        _ => Err(format!("unknown Component type family `{value}`")),
    }
}

fn parse_sequence_family(value: &str) -> Result<Option<SequenceEncodingFamily>, String> {
    match value {
        "-" => Ok(None),
        "nucleic_acid" => Ok(Some(SequenceEncodingFamily::NucleicAcid)),
        "protein" => Ok(Some(SequenceEncodingFamily::Protein)),
        "simple_chemical" => Ok(Some(SequenceEncodingFamily::SimpleChemical)),
        "other_textual" => Ok(Some(SequenceEncodingFamily::OtherTextual)),
        _ => Err(format!("unknown Sequence encoding family `{value}`")),
    }
}
