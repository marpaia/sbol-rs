//! Generate the compact ontology fact snapshot from a local raw ontology cache.
//!
//! The generated snapshot intentionally contains only the terms and
//! compatibility facts needed by the SBOL validator. Source ontology files are
//! bootstrapped separately with `bootstrap-ontology-cache`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::Parser;
use sbol_ontology::parser::{RawTerm, parse_obo_terms, parse_rdfxml_terms};

const CACHE_ENV: &str = "SBOL_ONTOLOGY_CACHE";
const DEFAULT_CACHE_ROOT: &str = "target/ontology-cache";
const DEFAULT_OUTPUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/sbol3_ontology_facts.tsv");

const CACHE_FILES: &[&str] = &[
    "EDAM.owl",
    "sbo.owl",
    "so.owl",
    "go-basic.obo",
    "chebi.owl",
    "cl-basic.obo",
];

#[path = "generate-ontology-facts/policies.rs"]
mod policies;
use policies::*;

/// Generates the compact ontology fact snapshot from a local raw ontology cache.
#[derive(Parser)]
#[command(about = "Generate the compact SBOL ontology fact snapshot from a raw ontology cache")]
struct Options {
    /// Directory containing the bootstrapped raw ontology cache.
    #[arg(long = "cache", env = CACHE_ENV, default_value = DEFAULT_CACHE_ROOT)]
    cache_root: PathBuf,

    /// Destination path for the generated fact snapshot.
    #[arg(long, default_value = DEFAULT_OUTPUT)]
    output: PathBuf,

    /// Verify the generated snapshot matches the committed file instead of writing it.
    #[arg(long)]
    check: bool,

    /// Continue with a fallback-only run when raw cache files are missing.
    #[arg(long)]
    allow_missing_cache: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = Options::parse();

    let raw_terms = load_raw_terms(&options.cache_root, options.allow_missing_cache)?;
    let facts = generate_facts(&raw_terms);

    if options.check {
        let existing = fs::read_to_string(&options.output).map_err(|error| {
            format!(
                "failed to read `{}` for --check: {error}",
                options.output.display()
            )
        })?;
        if existing != facts {
            return Err(format!(
                "generated ontology facts differ from `{}`",
                options.output.display()
            ));
        }
        println!("{} is up to date", options.output.display());
        return Ok(());
    }

    fs::write(&options.output, facts)
        .map_err(|error| format!("failed to write `{}`: {error}", options.output.display()))?;
    println!("wrote {}", options.output.display());
    Ok(())
}

fn load_raw_terms(
    cache_root: &Path,
    allow_missing_cache: bool,
) -> Result<BTreeMap<String, RawTerm>, String> {
    let mut terms = BTreeMap::new();
    for file_name in CACHE_FILES {
        let path = cache_root.join(file_name);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if allow_missing_cache && error.kind() == io::ErrorKind::NotFound => {
                continue;
            }
            Err(error) => {
                return Err(format!(
                    "failed to read raw ontology cache `{}`: {error}. Run `cargo run -p sbol-ontology --bin bootstrap-ontology-cache -- --cache {}` first, or pass --allow-missing-cache for a fallback-only smoke run",
                    path.display(),
                    cache_root.display()
                ));
            }
        };

        if file_name.ends_with(".obo") {
            parse_obo_terms(&text, &mut terms);
        } else {
            parse_rdfxml_terms(&text, &mut terms);
        }
    }
    Ok(terms)
}

fn generate_facts(raw_terms: &BTreeMap<String, RawTerm>) -> String {
    let policy_by_id = all_term_policies()
        .map(|policy| (policy.id, policy))
        .collect::<BTreeMap<_, _>>();
    let all_parents = parent_map(raw_terms);
    let (selected_ids, branch_memberships) = select_fact_terms(&all_parents, &policy_by_id);
    let mut output = String::from(
        "# format_version: 1\n# kind\tid\tiri\tlabel\taliases\tparents\tontology\trole\tcomponent_family\tsequence_family\ttable1\ttable2\n",
    );

    for id in &selected_ids {
        let policy = policy_by_id.get(id.as_str()).copied();
        let raw = raw_terms.get(id);
        let label = raw
            .and_then(|term| term.label.as_deref())
            .or_else(|| policy.map(|policy| policy.label))
            .unwrap_or(id);
        let parents = parents_for_fact(id, raw, policy, &selected_ids);
        let metadata = term_metadata(id, policy, &all_parents);
        let iri = policy
            .map(|policy| policy.iri.to_owned())
            .unwrap_or_else(|| canonical_iri(id));
        let ontology = policy
            .map(|policy| policy.ontology)
            .unwrap_or_else(|| ontology_namespace(id));

        push_columns(
            &mut output,
            &[
                "term",
                id,
                &iri,
                &sanitize(label),
                &join_or_dash(&aliases_for(id, policy)),
                &join_or_dash(&parents),
                ontology,
                metadata.role,
                metadata.component_family,
                metadata.sequence_family,
                bool_text(policy.is_some_and(|policy| policy.table_1)),
                bool_text(policy.is_some_and(|policy| policy.table_2)),
            ],
        );
    }

    for (term, branch_root) in branch_memberships {
        push_columns(&mut output, &["branch", &term, &branch_root]);
    }

    for (encoding, component_type) in SEQUENCE_COMPATIBILITIES {
        push_columns(&mut output, &["compat", encoding, component_type]);
    }
    for (left, right) in CONFLICTS {
        push_columns(&mut output, &["conflict", left, right]);
    }
    for term in COMPONENT_ROLE_TERMS {
        push_columns(&mut output, &["component_role", term]);
    }
    for (role, component_type) in COMPONENT_ROLE_COMPATIBILITIES {
        push_columns(
            &mut output,
            &["component_role_compat", role, component_type],
        );
    }
    for (interaction_type, role) in PARTICIPATION_COMPATIBILITIES {
        push_columns(
            &mut output,
            &["participation_compat", interaction_type, role],
        );
    }

    output
}

fn push_columns(output: &mut String, columns: &[&str]) {
    output.push_str(&columns.join("\t"));
    output.push('\n');
}

fn join_or_dash<T: AsRef<str>>(values: &[T]) -> String {
    if values.is_empty() {
        "-".to_owned()
    } else {
        values
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()
            .join("|")
    }
}

fn sanitize(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn bool_text(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

#[derive(Clone, Copy, Debug)]
struct TermMetadata {
    role: &'static str,
    component_family: &'static str,
    sequence_family: &'static str,
}

fn all_term_policies() -> impl Iterator<Item = &'static TermPolicy> {
    TERM_POLICIES
        .iter()
        .chain(SBO_POLICIES)
        .chain(PARTICIPATION_ROLE_POLICIES)
        .chain(SO_GO_CHEBI_POLICIES)
}

fn parent_map(raw_terms: &BTreeMap<String, RawTerm>) -> BTreeMap<String, Vec<String>> {
    let mut parents = raw_terms
        .iter()
        .map(|(id, raw)| (id.clone(), raw.parents.iter().cloned().collect::<Vec<_>>()))
        .collect::<BTreeMap<_, _>>();

    for policy in all_term_policies() {
        parents.entry(policy.id.to_owned()).or_insert_with(|| {
            policy
                .parents
                .iter()
                .map(|parent| (*parent).to_owned())
                .collect()
        });
    }

    parents
}

fn select_fact_terms(
    parents_by_id: &BTreeMap<String, Vec<String>>,
    policy_by_id: &BTreeMap<&'static str, &'static TermPolicy>,
) -> (BTreeSet<String>, BTreeSet<(String, String)>) {
    let mut selected_ids = policy_by_id
        .keys()
        .map(|id| (*id).to_owned())
        .collect::<BTreeSet<_>>();
    let mut branch_memberships = BTreeSet::new();
    let mut children_by_parent = BTreeMap::<String, Vec<String>>::new();

    for (id, parents) in parents_by_id {
        for parent in parents {
            children_by_parent
                .entry(parent.clone())
                .or_default()
                .push(id.clone());
        }
    }

    for branch in BRANCH_POLICIES {
        selected_ids.insert(branch.root_id.to_owned());
        let mut stack = vec![branch.root_id.to_owned()];
        let mut visited = BTreeSet::new();

        while let Some(id) = stack.pop() {
            if !visited.insert(id.clone()) {
                continue;
            }
            selected_ids.insert(id.clone());
            if id != branch.root_id {
                branch_memberships.insert((id.clone(), branch.root_id.to_owned()));
            }
            if let Some(children) = children_by_parent.get(&id) {
                stack.extend(children.iter().cloned());
            }
        }
    }

    (selected_ids, branch_memberships)
}

fn parents_for_fact(
    id: &str,
    raw: Option<&RawTerm>,
    policy: Option<&TermPolicy>,
    selected_ids: &BTreeSet<String>,
) -> Vec<String> {
    let raw_parents = raw
        .map(|term| {
            term.parents
                .iter()
                .filter(|parent| selected_ids.contains(parent.as_str()))
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !raw_parents.is_empty() {
        return raw_parents;
    }

    policy
        .map(|policy| {
            policy
                .parents
                .iter()
                .filter(|parent| selected_ids.contains(**parent))
                .map(|parent| (*parent).to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            BRANCH_POLICIES
                .iter()
                .find(|branch| branch.root_id == id)
                .map(|_| Vec::new())
                .unwrap_or_default()
        })
}

fn term_metadata(
    id: &str,
    policy: Option<&TermPolicy>,
    parents_by_id: &BTreeMap<String, Vec<String>>,
) -> TermMetadata {
    if let Some(policy) = policy {
        return TermMetadata {
            role: policy.role,
            component_family: policy.component_family,
            sequence_family: policy.sequence_family,
        };
    }

    for branch in BRANCH_POLICIES {
        if id == branch.root_id || reaches_branch_root(id, branch.root_id, parents_by_id) {
            return TermMetadata {
                role: branch.role,
                component_family: branch.component_family,
                sequence_family: branch.sequence_family,
            };
        }
    }

    TermMetadata {
        role: "other",
        component_family: "-",
        sequence_family: "-",
    }
}

fn reaches_branch_root(
    term: &str,
    branch_root: &str,
    parents_by_id: &BTreeMap<String, Vec<String>>,
) -> bool {
    fn visit(
        term: &str,
        branch_root: &str,
        parents_by_id: &BTreeMap<String, Vec<String>>,
        visited: &mut BTreeSet<String>,
    ) -> bool {
        if !visited.insert(term.to_owned()) {
            return false;
        }
        parents_by_id.get(term).is_some_and(|parents| {
            parents.iter().any(|parent| {
                parent == branch_root || visit(parent, branch_root, parents_by_id, visited)
            })
        })
    }

    visit(term, branch_root, parents_by_id, &mut BTreeSet::new())
}

fn canonical_iri(id: &str) -> String {
    let Some((prefix, local)) = id.split_once(':') else {
        return id.to_owned();
    };
    match prefix {
        "EDAM" => format!("https://identifiers.org/edam:{local}"),
        _ => format!("https://identifiers.org/{prefix}:{local}"),
    }
}

fn aliases_for(id: &str, policy: Option<&TermPolicy>) -> Vec<String> {
    let Some((prefix, local)) = id.split_once(':') else {
        return Vec::new();
    };
    let mut aliases = policy
        .map(|policy| {
            policy
                .aliases
                .iter()
                .map(|alias| (*alias).to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let generated = match prefix {
        "EDAM" => vec![format!("http://edamontology.org/{local}")],
        "SBO" => vec![
            format!("http://purl.obolibrary.org/obo/SBO_{local}"),
            format!("http://biomodels.net/SBO/SBO_{local}"),
        ],
        "SO" | "GO" | "CHEBI" | "CL" | "NCIT" => {
            vec![format!("http://purl.obolibrary.org/obo/{prefix}_{local}")]
        }
        _ => Vec::new(),
    };
    for alias in generated {
        if !aliases.contains(&alias) {
            aliases.push(alias);
        }
    }
    aliases
}

fn ontology_namespace(id: &str) -> &'static str {
    match id.split_once(':').map(|(prefix, _)| prefix) {
        Some("EDAM") => "EDAM",
        Some("SBO") => "SBO",
        Some("SO") => "SO",
        Some("GO") => "GO",
        Some("CHEBI") => "CHEBI",
        Some("CL") => "CL",
        Some("NCIT") => "NCIT",
        _ => "-",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_generation_matches_header_and_rule_fact_shapes() {
        let facts = generate_facts(&BTreeMap::new());

        assert!(facts.starts_with("# format_version: 1\n# kind\tid\tiri\tlabel\taliases\tparents"));
        assert!(facts.contains("\nterm\tSBO:0000176\t"));
        assert!(facts.contains("\nbranch\tSO:0000987\tSO:0000986\n"));
        assert!(facts.contains("\nconflict\tSBO:0000169\tSBO:0000170\n"));
        assert!(facts.contains("\ncomponent_role\tSBO:0000289\n"));
        assert!(facts.contains("\nparticipation_compat\tSBO:0000176\tSBO:0000010\n"));
    }
}
