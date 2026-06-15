//! Offline ontology facts for SBOL validation.
//!
//! `sbol-ontology` embeds a compact, SBOL-specific fact snapshot derived from
//! canonical ontology sources. It does not fetch network resources at runtime.
//! The bundled [`Ontology`] accepts common SBOL document IRIs, OBO PURLs, and
//! compact IDs, then exposes branch membership, conflict, and compatibility
//! queries used by the `sbol` validator.
//!
//! Extension snapshots (e.g. NCIT) can be loaded from a TSV that follows the
//! same column schema as the bundled file. Compose them with the bundled
//! snapshot through [`OntologyRegistry`].

#![forbid(unsafe_code)]

mod facts;
mod registry;

pub mod cache;
pub mod download;
pub mod parser;

pub use cache::{
    BranchRoot, BuildError, InstallError, InstalledOntology, KnownOntology, OntologyCache,
    OntologyDescriptor, SourceFormat, VerifyError,
};
pub use registry::OntologyRegistry;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

const FACTS: &str = include_str!("../data/sbol3_ontology_facts.tsv");
const SOURCES: &str = include_str!("../data/ontology_sources.tsv");

/// Current TSV format version. Snapshots that do not carry this version
/// in a `# format_version: N` header line are rejected at load time.
pub const TSV_FORMAT_VERSION: u32 = 1;

/// Broad family for SBOL Component type terms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComponentTypeFamily {
    NucleicAcid,
    Protein,
    SimpleChemical,
    Complex,
    Functional,
}

/// Broad family for SBOL Sequence encoding terms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SequenceEncodingFamily {
    NucleicAcid,
    Protein,
    SimpleChemical,
    OtherTextual,
}

/// Ontology namespace represented by a bundled or extension term.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OntologyNamespace {
    Edam,
    Sbo,
    So,
    Go,
    Chebi,
    Cl,
    Ncit,
}

/// SBOL-facing role assigned to a bundled ontology term.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TermRole {
    SequenceEncoding,
    ComponentType,
    ComponentTypeModifier,
    InteractionType,
    ParticipationRole,
    FeatureRole,
    Other,
}

/// Provenance for one upstream ontology source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OntologyProvenance {
    pub ontology: String,
    pub source_url: String,
    pub version: String,
    pub license: String,
    pub retrieved: String,
    pub raw_sha256: String,
    pub fact_sha256: String,
    pub notes: String,
}

/// Offline ontology query surface used by SBOL validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ontology {
    terms: BTreeMap<String, TermFact>,
    aliases: BTreeMap<String, String>,
    branches: BTreeSet<(String, String)>,
    compatibilities: BTreeSet<(String, String)>,
    conflicts: BTreeSet<(String, String)>,
    component_role_terms: BTreeSet<String>,
    component_role_compatibilities: BTreeSet<(String, String)>,
    participation_compatibilities: BTreeSet<(String, String)>,
    provenance: Vec<OntologyProvenance>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TermFact {
    iri: String,
    label: String,
    parents: Vec<String>,
    namespace: OntologyNamespace,
    role: TermRole,
    component_family: Option<ComponentTypeFamily>,
    sequence_family: Option<SequenceEncodingFamily>,
    table_1_sequence_encoding: bool,
    table_2_component_type: bool,
}

impl Ontology {
    /// Returns the bundled offline ontology snapshot.
    pub fn bundled() -> &'static Self {
        static ONTOLOGY: OnceLock<Ontology> = OnceLock::new();
        ONTOLOGY.get_or_init(|| {
            Ontology::from_tsv(FACTS, SOURCES)
                .expect("bundled SBOL ontology facts must parse successfully")
        })
    }

    /// Returns provenance metadata for the upstream ontology sources.
    pub fn provenance(&self) -> &[OntologyProvenance] {
        &self.provenance
    }

    /// Returns the compact canonical ID for an IRI, PURL, or compact ID.
    pub fn canonical_id(&self, term: &str) -> Option<String> {
        if let Some(canonical) = self.aliases.get(term) {
            return Some(canonical.clone());
        }
        let candidate = normalize_term_id(term)?;
        self.terms.contains_key(&candidate).then_some(candidate)
    }

    /// Returns the preferred SBOL-facing IRI for a known term.
    pub fn canonical_iri(&self, term: &str) -> Option<&str> {
        let canonical = self.canonical_id(term)?;
        self.terms.get(&canonical).map(|fact| fact.iri.as_str())
    }

    /// Returns true when the term exists in the bundled fact snapshot.
    pub fn contains_term(&self, term: &str) -> bool {
        self.canonical_id(term).is_some()
    }

    /// Returns the preferred label for a known term.
    pub fn label(&self, term: &str) -> Option<&str> {
        let canonical = self.canonical_id(term)?;
        self.terms.get(&canonical).map(|fact| fact.label.as_str())
    }

    /// Returns the source ontology for a known term.
    pub fn namespace(&self, term: &str) -> Option<OntologyNamespace> {
        let canonical = self.canonical_id(term)?;
        self.terms.get(&canonical).map(|fact| fact.namespace)
    }

    /// Returns the SBOL-facing role for a known term.
    pub fn term_role(&self, term: &str) -> Option<TermRole> {
        let canonical = self.canonical_id(term)?;
        self.terms.get(&canonical).map(|fact| fact.role)
    }

    /// Returns whether a known term is a Sequence encoding term.
    ///
    /// `None` means the term is absent from the bundled facts.
    pub fn is_sequence_encoding_term(&self, term: &str) -> Option<bool> {
        self.term_role(term)
            .map(|role| role == TermRole::SequenceEncoding)
    }

    /// Returns whether a known term is suitable for `sbol:type` on structural
    /// SBOL entities.
    ///
    /// Component type modifiers such as topology and strand terms are accepted
    /// because SBOL permits them as additional `type` values for DNA/RNA.
    pub fn is_component_type_term(&self, term: &str) -> Option<bool> {
        self.term_role(term).map(|role| {
            matches!(
                role,
                TermRole::ComponentType | TermRole::ComponentTypeModifier
            )
        })
    }

    /// Returns whether a known term is suitable for `sbol:role` on Feature
    /// objects.
    pub fn is_feature_role_term(&self, term: &str) -> Option<bool> {
        self.term_role(term)
            .map(|role| role == TermRole::FeatureRole)
    }

    /// Returns whether a known term is suitable for `sbol:role` on Component
    /// and Component-like Feature objects.
    pub fn is_component_role_term(&self, term: &str) -> Option<bool> {
        let canonical = self.canonical_id(term)?;
        if self.component_role_terms.contains(&canonical) {
            return Some(true);
        }
        self.terms
            .get(&canonical)
            .map(|fact| fact.role == TermRole::FeatureRole)
    }

    /// Returns whether a known role term is in the SO sequence feature branch.
    pub fn is_sequence_feature_role_term(&self, term: &str) -> Option<bool> {
        self.contains_term(term)
            .then(|| self.is_in_branch(term, "SO:0000110"))
    }

    /// Returns whether a known term is a Cell Ontology cell type, i.e. is
    /// equivalent to or descends from `CL:0000000`. Returns `None` for terms
    /// absent from the bundled facts.
    pub fn is_cell_type_term(&self, term: &str) -> Option<bool> {
        self.contains_term(term)
            .then(|| self.is_in_branch(term, "CL:0000000"))
    }

    /// Returns whether a known term is suitable for `sbol:type` on Interaction
    /// objects.
    pub fn is_interaction_type_term(&self, term: &str) -> Option<bool> {
        self.term_role(term)
            .map(|role| role == TermRole::InteractionType)
    }

    /// Returns whether a known term is suitable for `sbol:role` on
    /// Participation objects.
    pub fn is_participation_role_term(&self, term: &str) -> Option<bool> {
        self.term_role(term)
            .map(|role| role == TermRole::ParticipationRole)
    }

    /// Returns true for exact SBOL Table 1 Sequence encoding terms.
    pub fn is_table_1_sequence_encoding(&self, term: &str) -> bool {
        let Some(canonical) = self.canonical_id(term) else {
            return false;
        };
        self.terms
            .get(&canonical)
            .is_some_and(|fact| fact.table_1_sequence_encoding)
    }

    /// Returns true for exact SBOL Table 2 Component type terms.
    pub fn is_table_2_component_type(&self, term: &str) -> bool {
        let Some(canonical) = self.canonical_id(term) else {
            return false;
        };
        self.terms
            .get(&canonical)
            .is_some_and(|fact| fact.table_2_component_type)
    }

    /// Returns true if `term` is a strict descendant of `ancestor`.
    pub fn is_descendant(&self, term: &str, ancestor: &str) -> bool {
        let Some(term) = self.canonical_id(term) else {
            return false;
        };
        let Some(ancestor) = self.canonical_id(ancestor) else {
            return false;
        };
        if term == ancestor {
            return false;
        }
        self.has_ancestor(&term, &ancestor)
    }

    /// Returns true if `term` is equal to or descends from `ancestor`.
    pub fn is_equivalent_or_descendant(&self, term: &str, ancestor: &str) -> bool {
        let Some(term) = self.canonical_id(term) else {
            return false;
        };
        let Some(ancestor) = self.canonical_id(ancestor) else {
            return false;
        };
        term == ancestor || self.has_ancestor(&term, &ancestor)
    }

    /// Alias for [`Ontology::is_equivalent_or_descendant`].
    pub fn is_in_branch(&self, term: &str, branch_root: &str) -> bool {
        let Some(term) = self.canonical_id(term) else {
            return false;
        };
        let Some(branch_root) = self.canonical_id(branch_root) else {
            return false;
        };
        term == branch_root
            || self.branches.contains(&(term.clone(), branch_root.clone()))
            || self.has_ancestor(&term, &branch_root)
    }

    /// Returns whether two known terms conflict.
    ///
    /// `None` means one or both terms are absent from the bundled facts.
    pub fn terms_conflict(&self, left: &str, right: &str) -> Option<bool> {
        let left = self.canonical_id(left)?;
        let right = self.canonical_id(right)?;
        if left == right {
            return Some(false);
        }
        if self.conflicts.contains(&ordered_pair(&left, &right)) {
            return Some(true);
        }
        let left_fact = self.terms.get(&left)?;
        let right_fact = self.terms.get(&right)?;
        if let (Some(left_family), Some(right_family)) =
            (left_fact.component_family, right_fact.component_family)
        {
            return Some(left_family != right_family);
        }
        if let (Some(left_family), Some(right_family)) =
            (left_fact.sequence_family, right_fact.sequence_family)
        {
            return Some(left_family != right_family);
        }
        Some(false)
    }

    /// Returns whether a Participation role is cross-listed for an Interaction
    /// type in the bundled SBOL Table 11/Table 12 facts.
    pub fn participation_role_compatible_with_interaction_type(
        &self,
        role: &str,
        interaction_type: &str,
    ) -> Option<bool> {
        let role = self.canonical_id(role)?;
        let interaction_type = self.canonical_id(interaction_type)?;
        let role_fact = self.terms.get(&role)?;
        let interaction_fact = self.terms.get(&interaction_type)?;
        if role_fact.role != TermRole::ParticipationRole
            || interaction_fact.role != TermRole::InteractionType
        {
            return None;
        }
        Some(
            self.participation_compatibilities
                .contains(&(interaction_type, role)),
        )
    }

    /// Returns whether a Component role is compatible with a Component type.
    ///
    /// `None` means one or both terms are absent, or the terms do not have the
    /// roles needed to answer this compatibility question.
    pub fn component_role_compatible_with_component_type(
        &self,
        role: &str,
        component_type: &str,
    ) -> Option<bool> {
        let role = self.canonical_id(role)?;
        let component_type = self.canonical_id(component_type)?;
        let role_fact = self.terms.get(&role)?;
        let component_fact = self.terms.get(&component_type)?;
        if role_fact.role != TermRole::FeatureRole || component_fact.role != TermRole::ComponentType
        {
            return None;
        }
        if self
            .component_role_compatibilities
            .contains(&(role.clone(), component_type.clone()))
        {
            return Some(true);
        }

        let component_family = component_fact.component_family?;
        if self.is_in_branch(&role, "SO:0000110") {
            return Some(component_family == ComponentTypeFamily::NucleicAcid);
        }
        if self.is_in_branch(&role, "GO:0003674") {
            return Some(component_family == ComponentTypeFamily::Protein);
        }
        if self.is_in_branch(&role, "CHEBI:50906") {
            return Some(component_family == ComponentTypeFamily::SimpleChemical);
        }
        None
    }

    /// Returns whether a Sequence encoding is compatible with a Component type.
    ///
    /// `None` means one or both terms are absent, or the terms do not have the
    /// roles needed to answer this compatibility question.
    pub fn encoding_compatible_with_component_type(
        &self,
        encoding: &str,
        component_type: &str,
    ) -> Option<bool> {
        let encoding = self.canonical_id(encoding)?;
        let component_type = self.canonical_id(component_type)?;
        let encoding_fact = self.terms.get(&encoding)?;
        let component_fact = self.terms.get(&component_type)?;
        if encoding_fact.role != TermRole::SequenceEncoding
            || component_fact.role != TermRole::ComponentType
        {
            return None;
        }
        if self
            .compatibilities
            .contains(&(encoding.clone(), component_type.clone()))
        {
            return Some(true);
        }
        let encoding_family = encoding_fact.sequence_family?;
        let component_family = component_fact.component_family?;
        Some(matches!(
            (encoding_family, component_family),
            (
                SequenceEncodingFamily::NucleicAcid,
                ComponentTypeFamily::NucleicAcid
            ) | (
                SequenceEncodingFamily::Protein,
                ComponentTypeFamily::Protein
            ) | (
                SequenceEncodingFamily::SimpleChemical,
                ComponentTypeFamily::SimpleChemical
            )
        ))
    }

    /// Returns the first Table 1 encoding compatible with a Component type.
    pub fn recommended_sequence_encoding_for_component_type(
        &self,
        component_type: &str,
    ) -> Option<&str> {
        self.compatible_sequence_encodings_for_component_type(component_type)
            .into_iter()
            .next()
    }

    /// Returns all Table 1 encodings compatible with a Component type.
    pub fn compatible_sequence_encodings_for_component_type(
        &self,
        component_type: &str,
    ) -> Vec<&str> {
        let Some(component_type) = self.canonical_id(component_type) else {
            return Vec::new();
        };
        self.compatibilities
            .iter()
            .filter_map(|(encoding, compatible_component)| {
                (compatible_component == &component_type)
                    .then(|| self.terms.get(encoding).map(|fact| fact.iri.as_str()))
                    .flatten()
            })
            .collect()
    }

    /// Returns the broad component family for a known term.
    pub fn component_type_family(&self, component_type: &str) -> Option<ComponentTypeFamily> {
        let canonical = self.canonical_id(component_type)?;
        self.terms
            .get(&canonical)
            .and_then(|fact| fact.component_family)
    }

    /// Returns the broad sequence encoding family for a known term.
    pub fn sequence_encoding_family(&self, encoding: &str) -> Option<SequenceEncodingFamily> {
        let canonical = self.canonical_id(encoding)?;
        self.terms
            .get(&canonical)
            .and_then(|fact| fact.sequence_family)
    }

    fn has_ancestor(&self, term: &str, ancestor: &str) -> bool {
        let Some(fact) = self.terms.get(term) else {
            return false;
        };
        fact.parents
            .iter()
            .any(|parent| parent == ancestor || self.has_ancestor(parent, ancestor))
    }
}

/// Returns the compact ontology ID for a supported IRI, PURL, or compact ID.
pub fn normalize_term_id(value: &str) -> Option<String> {
    if let Some((prefix, local)) = value.split_once(':')
        && !value.starts_with("http://")
        && !value.starts_with("https://")
    {
        return Some(format!("{}:{local}", normalize_prefix(prefix)?));
    }

    if let Some(rest) = value
        .strip_prefix("https://identifiers.org/")
        .or_else(|| value.strip_prefix("http://identifiers.org/"))
    {
        let (prefix, local) = rest.split_once(':')?;
        return Some(format!("{}:{local}", normalize_prefix(prefix)?));
    }

    if let Some(local) = value.strip_prefix("http://edamontology.org/") {
        return Some(format!("EDAM:{local}"));
    }

    if let Some(local) = value
        .strip_prefix("http://biomodels.net/SBO/SBO_")
        .or_else(|| value.strip_prefix("https://biomodels.net/SBO/SBO_"))
    {
        return Some(format!("SBO:{local}"));
    }

    if let Some(local) = value.strip_prefix("http://purl.obolibrary.org/obo/") {
        let (prefix, suffix) = local.split_once('_')?;
        return Some(format!("{}:{suffix}", normalize_prefix(prefix)?));
    }

    None
}

fn ordered_pair(left: &str, right: &str) -> (String, String) {
    if left <= right {
        (left.to_owned(), right.to_owned())
    } else {
        (right.to_owned(), left.to_owned())
    }
}

fn normalize_prefix(prefix: &str) -> Option<&'static str> {
    match prefix.to_ascii_uppercase().as_str() {
        "EDAM" => Some("EDAM"),
        "SBO" => Some("SBO"),
        "SO" => Some("SO"),
        "GO" => Some("GO"),
        "CHEBI" => Some("CHEBI"),
        "CL" => Some("CL"),
        "NCIT" => Some("NCIT"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_ontology_loads_core_terms() {
        let ontology = Ontology::bundled();

        assert!(ontology.contains_term("https://identifiers.org/edam:format_1207"));
        assert!(ontology.contains_term("https://identifiers.org/SBO:0000251"));
        assert!(ontology.contains_term("https://identifiers.org/SO:0000987"));
        assert!(ontology.contains_term("https://identifiers.org/GO:0003700"));
        assert!(ontology.contains_term("https://identifiers.org/CHEBI:35224"));
        assert!(ontology.contains_term("https://identifiers.org/CL:0000540"));
        assert!(!ontology.provenance().is_empty());
    }

    fn synthetic_extension_tsv() -> &'static str {
        "# format_version: 1\n# kind\tid\tiri\tlabel\taliases\tparents\tontology\trole\tcomponent_family\tsequence_family\ttable1\ttable2\n\
         term\tCL:9999999\thttps://identifiers.org/CL:9999999\tlab-only synthetic cell\t-\tCL:0000540\tCL\tcomponent_type\t-\t-\tfalse\tfalse\n\
         branch\tCL:9999999\tCL:0000000\n"
    }

    #[test]
    fn from_tsv_str_rejects_missing_format_version() {
        let result = Ontology::from_tsv_str(
            "term\tFOO:1\thttps://example.org/foo\tfoo\t-\t-\tEDAM\tother\t-\t-\tfalse\tfalse\n",
        );
        assert!(
            result.is_err(),
            "expected missing-header error, got {result:?}"
        );
    }

    #[test]
    fn from_tsv_str_rejects_unknown_format_version() {
        let bumped = "# format_version: 9999\n# kind\tid\tiri\tlabel\taliases\tparents\tontology\trole\tcomponent_family\tsequence_family\ttable1\ttable2\n\
             term\tEDAM:format_1915\thttps://identifiers.org/edam:format_1915\tFormat\t-\t-\tEDAM\tother\t-\t-\tfalse\tfalse\n";
        let err = Ontology::from_tsv_str(bumped).unwrap_err();
        assert!(
            err.contains("format_version 9999"),
            "unexpected error `{err}`"
        );
    }

    #[test]
    fn ontology_registry_layers_extensions_over_bundled() {
        let extension = Ontology::from_tsv_str(synthetic_extension_tsv()).unwrap();
        let registry = OntologyRegistry::bundled_with([extension]);
        let ontology = registry.ontology();

        assert!(ontology.contains_term("CL:9999999"));
        assert_eq!(ontology.is_cell_type_term("CL:9999999"), Some(true));
        // Bundled facts still resolve normally.
        assert_eq!(ontology.is_cell_type_term("CL:0000540"), Some(true));
    }

    #[test]
    fn ontology_registry_bundled_only_borrows_static() {
        // Two registries built without extensions point at the same bundled
        // snapshot, validating the zero-allocation default path.
        let a = OntologyRegistry::bundled_only();
        let b = OntologyRegistry::bundled_only();
        assert!(std::ptr::eq(a.ontology(), b.ontology()));
    }

    #[test]
    fn cell_ontology_terms_resolve_via_branch_root() {
        let ontology = Ontology::bundled();

        assert_eq!(ontology.is_cell_type_term("CL:0000540"), Some(true));
        assert_eq!(ontology.is_cell_type_term("CL:0000084"), Some(true));
        assert_eq!(ontology.is_cell_type_term("CL:0000000"), Some(true));
        assert_eq!(
            ontology.is_cell_type_term("http://purl.obolibrary.org/obo/CL_0000540"),
            Some(true)
        );
        assert_eq!(
            ontology.namespace("CL:0000540"),
            Some(OntologyNamespace::Cl)
        );
        assert_eq!(ontology.is_cell_type_term("SO:0000316"), Some(false));
        assert_eq!(
            ontology.is_cell_type_term("https://example.org/custom"),
            None
        );
    }

    #[test]
    fn normalizes_identifiers_org_obo_purls_and_native_edam_iris() {
        let ontology = Ontology::bundled();

        assert_eq!(
            ontology.canonical_id("http://edamontology.org/format_1207"),
            Some("EDAM:format_1207".to_owned())
        );
        assert_eq!(
            ontology.canonical_id("http://purl.obolibrary.org/obo/SBO_0000251"),
            Some("SBO:0000251".to_owned())
        );
        assert_eq!(
            ontology.canonical_id("https://identifiers.org/SO:0000987"),
            Some("SO:0000987".to_owned())
        );
    }

    #[test]
    fn branch_queries_follow_parent_links() {
        let ontology = Ontology::bundled();

        assert!(ontology.is_descendant("EDAM:format_1207", "EDAM:format_2330"));
        assert!(ontology.is_in_branch("EDAM:format_3752", "EDAM:format_2330"));
        assert!(ontology.is_in_branch("SBO:0000243", "SBO:0000236"));
        assert!(ontology.is_in_branch("SBO:0000176", "SBO:0000231"));
        assert!(ontology.is_in_branch("SBO:0000010", "SBO:0000003"));
        assert!(ontology.is_in_branch("EDAM:format_1207", "EDAM:format_1915"));
        assert!(ontology.is_in_branch("SO:0000987", "SO:0000986"));
        assert!(ontology.is_in_branch("SO:0000984", "SO:0000983"));
        assert!(ontology.is_in_branch("SO:0000167", "SO:0000110"));
        assert!(ontology.is_in_branch("GO:0001216", "GO:0003674"));
        assert!(ontology.is_in_branch("GO:0003700", "GO:0003674"));
        assert!(ontology.is_in_branch("CHEBI:35224", "CHEBI:50906"));
        assert!(!ontology.is_in_branch("SO:0000987", "SO:0000983"));
        assert!(ontology.is_equivalent_or_descendant("EDAM:format_2330", "EDAM:format_2330"));
        assert!(!ontology.is_descendant("EDAM:format_2330", "EDAM:format_2330"));
    }

    #[test]
    fn compatibility_and_conflict_queries_distinguish_unknowns() {
        let ontology = Ontology::bundled();

        assert_eq!(
            ontology.encoding_compatible_with_component_type("EDAM:format_1207", "SBO:0000251"),
            Some(true)
        );
        assert_eq!(
            ontology.encoding_compatible_with_component_type("EDAM:format_1208", "SBO:0000251"),
            Some(false)
        );
        assert_eq!(
            ontology.encoding_compatible_with_component_type(
                "https://example.org/custom",
                "SBO:0000251"
            ),
            None
        );
        assert_eq!(
            ontology.terms_conflict("SBO:0000251", "SBO:0000252"),
            Some(true)
        );
        assert_eq!(
            ontology.terms_conflict("SBO:0000169", "SBO:0000170"),
            Some(true)
        );
        assert_eq!(
            ontology.terms_conflict("SBO:0000251", "SO:0000987"),
            Some(false)
        );
        assert_eq!(
            ontology.terms_conflict("SBO:0000251", "https://example.org/custom"),
            None
        );
        assert_eq!(
            ontology.is_component_type_term("https://example.org/custom"),
            None
        );
        assert_eq!(
            ontology.is_component_role_term("https://example.org/custom"),
            None
        );
        assert_eq!(
            ontology.component_role_compatible_with_component_type(
                "https://example.org/custom",
                "SBO:0000251"
            ),
            None
        );
    }

    #[test]
    fn role_queries_cover_sbol_feature_interaction_and_participation_terms() {
        let ontology = Ontology::bundled();

        assert_eq!(ontology.is_feature_role_term("SO:0000167"), Some(true));
        assert_eq!(ontology.is_component_role_term("SO:0000167"), Some(true));
        assert_eq!(ontology.is_component_role_term("SBO:0000289"), Some(true));
        assert_eq!(ontology.is_component_role_term("SBO:0000290"), Some(true));
        assert_eq!(
            ontology.is_sequence_feature_role_term("SO:0000167"),
            Some(true)
        );
        assert_eq!(ontology.is_feature_role_term("SBO:0000176"), Some(false));
        assert_eq!(ontology.is_component_type_term("SBO:0000243"), Some(true));
        assert_eq!(ontology.is_component_type_term("SBO:0000290"), Some(true));
        assert_eq!(ontology.is_interaction_type_term("SBO:0000176"), Some(true));
        assert_eq!(
            ontology.is_participation_role_term("SBO:0000010"),
            Some(true)
        );
        assert_eq!(
            ontology
                .participation_role_compatible_with_interaction_type("SBO:0000010", "SBO:0000176"),
            Some(true)
        );
        assert_eq!(
            ontology
                .participation_role_compatible_with_interaction_type("SBO:0000459", "SBO:0000169"),
            Some(false)
        );
        assert_eq!(
            ontology.component_role_compatible_with_component_type("SO:0000167", "SBO:0000251"),
            Some(true)
        );
        assert_eq!(
            ontology.component_role_compatible_with_component_type("GO:0003700", "SBO:0000252"),
            Some(true)
        );
        assert_eq!(
            ontology.component_role_compatible_with_component_type("CHEBI:35224", "SBO:0000247"),
            Some(true)
        );
        assert_eq!(
            ontology.component_role_compatible_with_component_type("SO:0000167", "SBO:0000252"),
            Some(false)
        );
    }
}
