//! Generate the compact ontology fact snapshot from a local raw ontology cache.
//!
//! The generated snapshot intentionally contains only the terms and
//! compatibility facts needed by the SBOL validator. Source ontology files are
//! bootstrapped separately with `bootstrap-ontology-cache`.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

#[derive(Clone, Debug)]
struct Options {
    cache_root: PathBuf,
    output: PathBuf,
    check: bool,
    allow_missing_cache: bool,
}

#[derive(Clone, Debug)]
struct TermPolicy {
    id: &'static str,
    iri: &'static str,
    label: &'static str,
    aliases: &'static [&'static str],
    parents: &'static [&'static str],
    ontology: &'static str,
    role: &'static str,
    component_family: &'static str,
    sequence_family: &'static str,
    table_1: bool,
    table_2: bool,
}

#[derive(Clone, Copy, Debug)]
struct BranchPolicy {
    root_id: &'static str,
    role: &'static str,
    component_family: &'static str,
    sequence_family: &'static str,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let Some(options) = Options::parse(env::args_os().skip(1))? else {
        print_usage();
        return Ok(());
    };

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

impl Options {
    fn parse<I>(args: I) -> Result<Option<Self>, String>
    where
        I: IntoIterator<Item = OsString>,
    {
        let mut cache_root = env::var_os(CACHE_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CACHE_ROOT));
        let mut output = PathBuf::from(DEFAULT_OUTPUT);
        let mut check = false;
        let mut allow_missing_cache = false;

        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            let arg = arg
                .into_string()
                .map_err(|_| "arguments must be valid UTF-8".to_owned())?;
            match arg.as_str() {
                "-h" | "--help" => return Ok(None),
                "--check" => check = true,
                "--allow-missing-cache" => allow_missing_cache = true,
                "--cache" => {
                    let Some(value) = args.next() else {
                        return Err("missing path after --cache".to_owned());
                    };
                    cache_root = PathBuf::from(value);
                }
                "--output" => {
                    let Some(value) = args.next() else {
                        return Err("missing path after --output".to_owned());
                    };
                    output = PathBuf::from(value);
                }
                _ if arg.starts_with("--cache=") => {
                    cache_root = PathBuf::from(arg.trim_start_matches("--cache="));
                }
                _ if arg.starts_with("--output=") => {
                    output = PathBuf::from(arg.trim_start_matches("--output="));
                }
                _ => return Err(format!("unknown argument `{arg}`")),
            }
        }

        Ok(Some(Self {
            cache_root,
            output,
            check,
            allow_missing_cache,
        }))
    }
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

fn print_usage() {
    println!(
        "\
Usage: cargo run -p sbol-ontology --bin generate-ontology-facts -- [OPTIONS]

Options:
      --cache <PATH>           Raw ontology cache directory [env: {CACHE_ENV}, default: {DEFAULT_CACHE_ROOT}]
      --output <PATH>          Facts TSV to write [default: {DEFAULT_OUTPUT}]
      --check                  Compare generated output with --output instead of writing it
      --allow-missing-cache    Use fallback seed labels/parents when raw cache files are absent
  -h, --help                   Print help
"
    );
}

const TERM_POLICIES: &[TermPolicy] = &[
    TermPolicy {
        id: "EDAM:format_1915",
        iri: "https://identifiers.org/edam:format_1915",
        label: "Format",
        aliases: &["http://edamontology.org/format_1915"],
        parents: &[],
        ontology: "EDAM",
        role: "other",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "EDAM:format_2330",
        iri: "https://identifiers.org/edam:format_2330",
        label: "Textual format",
        aliases: &["http://edamontology.org/format_2330"],
        parents: &["EDAM:format_1915"],
        ontology: "EDAM",
        role: "other",
        component_family: "-",
        sequence_family: "other_textual",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "EDAM:format_1207",
        iri: "https://identifiers.org/edam:format_1207",
        label: "IUPAC DNA/RNA",
        aliases: &["http://edamontology.org/format_1207"],
        parents: &["EDAM:format_2330"],
        ontology: "EDAM",
        role: "sequence_encoding",
        component_family: "-",
        sequence_family: "nucleic_acid",
        table_1: true,
        table_2: false,
    },
    TermPolicy {
        id: "EDAM:format_1208",
        iri: "https://identifiers.org/edam:format_1208",
        label: "IUPAC protein",
        aliases: &["http://edamontology.org/format_1208"],
        parents: &["EDAM:format_2330"],
        ontology: "EDAM",
        role: "sequence_encoding",
        component_family: "-",
        sequence_family: "protein",
        table_1: true,
        table_2: false,
    },
    TermPolicy {
        id: "EDAM:format_1197",
        iri: "https://identifiers.org/edam:format_1197",
        label: "InChI",
        aliases: &["http://edamontology.org/format_1197"],
        parents: &["EDAM:format_2330"],
        ontology: "EDAM",
        role: "sequence_encoding",
        component_family: "-",
        sequence_family: "simple_chemical",
        table_1: true,
        table_2: false,
    },
    TermPolicy {
        id: "EDAM:format_1196",
        iri: "https://identifiers.org/edam:format_1196",
        label: "SMILES",
        aliases: &["http://edamontology.org/format_1196"],
        parents: &["EDAM:format_2330"],
        ontology: "EDAM",
        role: "sequence_encoding",
        component_family: "-",
        sequence_family: "simple_chemical",
        table_1: true,
        table_2: false,
    },
    TermPolicy {
        id: "EDAM:format_3752",
        iri: "https://identifiers.org/edam:format_3752",
        label: "CSV",
        aliases: &["http://edamontology.org/format_3752"],
        parents: &["EDAM:format_2330"],
        ontology: "EDAM",
        role: "other",
        component_family: "-",
        sequence_family: "other_textual",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "EDAM:data_0006",
        iri: "https://identifiers.org/edam:data_0006",
        label: "Data",
        aliases: &["http://edamontology.org/data_0006"],
        parents: &[],
        ontology: "EDAM",
        role: "other",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
];

const SBO_POLICIES: &[TermPolicy] = &[
    TermPolicy {
        id: "SBO:0000236",
        iri: "https://identifiers.org/SBO:0000236",
        label: "Physical entity representation",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000236",
            "http://biomodels.net/SBO/SBO_0000236",
        ],
        parents: &[],
        ontology: "SBO",
        role: "component_type",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000240",
        iri: "https://identifiers.org/SBO:0000240",
        label: "Material entity",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000240",
            "http://biomodels.net/SBO/SBO_0000240",
        ],
        parents: &["SBO:0000236"],
        ontology: "SBO",
        role: "component_type",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000251",
        iri: "https://identifiers.org/SBO:0000251",
        label: "DNA",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000251",
            "http://biomodels.net/SBO/SBO_0000251",
        ],
        parents: &["SBO:0000240"],
        ontology: "SBO",
        role: "component_type",
        component_family: "nucleic_acid",
        sequence_family: "-",
        table_1: false,
        table_2: true,
    },
    TermPolicy {
        id: "SBO:0000250",
        iri: "https://identifiers.org/SBO:0000250",
        label: "RNA",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000250",
            "http://biomodels.net/SBO/SBO_0000250",
        ],
        parents: &["SBO:0000240"],
        ontology: "SBO",
        role: "component_type",
        component_family: "nucleic_acid",
        sequence_family: "-",
        table_1: false,
        table_2: true,
    },
    TermPolicy {
        id: "SBO:0000252",
        iri: "https://identifiers.org/SBO:0000252",
        label: "Protein",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000252",
            "http://biomodels.net/SBO/SBO_0000252",
        ],
        parents: &["SBO:0000240"],
        ontology: "SBO",
        role: "component_type",
        component_family: "protein",
        sequence_family: "-",
        table_1: false,
        table_2: true,
    },
    TermPolicy {
        id: "SBO:0000247",
        iri: "https://identifiers.org/SBO:0000247",
        label: "Simple Chemical",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000247",
            "http://biomodels.net/SBO/SBO_0000247",
        ],
        parents: &["SBO:0000240"],
        ontology: "SBO",
        role: "component_type",
        component_family: "simple_chemical",
        sequence_family: "-",
        table_1: false,
        table_2: true,
    },
    TermPolicy {
        id: "SBO:0000253",
        iri: "https://identifiers.org/SBO:0000253",
        label: "Non-covalent complex",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000253",
            "http://biomodels.net/SBO/SBO_0000253",
        ],
        parents: &["SBO:0000240"],
        ontology: "SBO",
        role: "component_type",
        component_family: "complex",
        sequence_family: "-",
        table_1: false,
        table_2: true,
    },
    TermPolicy {
        id: "SBO:0000241",
        iri: "https://identifiers.org/SBO:0000241",
        label: "Functional Entity",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000241",
            "http://biomodels.net/SBO/SBO_0000241",
        ],
        parents: &["SBO:0000240"],
        ontology: "SBO",
        role: "component_type",
        component_family: "functional",
        sequence_family: "-",
        table_1: false,
        table_2: true,
    },
    TermPolicy {
        id: "SBO:0000289",
        iri: "https://identifiers.org/SBO:0000289",
        label: "Functional compartment",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000289",
            "http://biomodels.net/SBO/SBO_0000289",
        ],
        parents: &["SBO:0000003"],
        ontology: "SBO",
        role: "participation_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000290",
        iri: "https://identifiers.org/SBO:0000290",
        label: "Physical compartment",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000290",
            "http://biomodels.net/SBO/SBO_0000290",
        ],
        parents: &["SBO:0000240"],
        ontology: "SBO",
        role: "component_type",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000231",
        iri: "https://identifiers.org/SBO:0000231",
        label: "Occurring entity representation",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000231",
            "http://biomodels.net/SBO/SBO_0000231",
        ],
        parents: &[],
        ontology: "SBO",
        role: "interaction_type",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000168",
        iri: "https://identifiers.org/SBO:0000168",
        label: "Control",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000168",
            "http://biomodels.net/SBO/SBO_0000168",
        ],
        parents: &["SBO:0000231"],
        ontology: "SBO",
        role: "interaction_type",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000169",
        iri: "https://identifiers.org/SBO:0000169",
        label: "Inhibition",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000169",
            "http://biomodels.net/SBO/SBO_0000169",
        ],
        parents: &["SBO:0000168"],
        ontology: "SBO",
        role: "interaction_type",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000170",
        iri: "https://identifiers.org/SBO:0000170",
        label: "Stimulation",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000170",
            "http://biomodels.net/SBO/SBO_0000170",
        ],
        parents: &["SBO:0000168"],
        ontology: "SBO",
        role: "interaction_type",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000176",
        iri: "https://identifiers.org/SBO:0000176",
        label: "Biochemical reaction",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000176",
            "http://biomodels.net/SBO/SBO_0000176",
        ],
        parents: &["SBO:0000231"],
        ontology: "SBO",
        role: "interaction_type",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000177",
        iri: "https://identifiers.org/SBO:0000177",
        label: "Non-covalent binding",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000177",
            "http://biomodels.net/SBO/SBO_0000177",
        ],
        parents: &["SBO:0000231"],
        ontology: "SBO",
        role: "interaction_type",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000179",
        iri: "https://identifiers.org/SBO:0000179",
        label: "Degradation",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000179",
            "http://biomodels.net/SBO/SBO_0000179",
        ],
        parents: &["SBO:0000176"],
        ontology: "SBO",
        role: "interaction_type",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000589",
        iri: "https://identifiers.org/SBO:0000589",
        label: "Genetic production",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000589",
            "http://biomodels.net/SBO/SBO_0000589",
        ],
        parents: &["SBO:0000231"],
        ontology: "SBO",
        role: "interaction_type",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
];

const PARTICIPATION_ROLE_POLICIES: &[TermPolicy] = &[
    TermPolicy {
        id: "SBO:0000003",
        iri: "https://identifiers.org/SBO:0000003",
        label: "Participant role",
        aliases: &[
            "http://purl.obolibrary.org/obo/SBO_0000003",
            "http://biomodels.net/SBO/SBO_0000003",
        ],
        parents: &[],
        ontology: "SBO",
        role: "participation_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000010",
        iri: "https://identifiers.org/SBO:0000010",
        label: "Reactant",
        aliases: &["http://purl.obolibrary.org/obo/SBO_0000010"],
        parents: &["SBO:0000003"],
        ontology: "SBO",
        role: "participation_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000011",
        iri: "https://identifiers.org/SBO:0000011",
        label: "Product",
        aliases: &["http://purl.obolibrary.org/obo/SBO_0000011"],
        parents: &["SBO:0000003"],
        ontology: "SBO",
        role: "participation_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000019",
        iri: "https://identifiers.org/SBO:0000019",
        label: "Modifier",
        aliases: &["http://purl.obolibrary.org/obo/SBO_0000019"],
        parents: &["SBO:0000003"],
        ontology: "SBO",
        role: "participation_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000020",
        iri: "https://identifiers.org/SBO:0000020",
        label: "Inhibitor",
        aliases: &["http://purl.obolibrary.org/obo/SBO_0000020"],
        parents: &["SBO:0000019"],
        ontology: "SBO",
        role: "participation_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000459",
        iri: "https://identifiers.org/SBO:0000459",
        label: "Stimulator",
        aliases: &["http://purl.obolibrary.org/obo/SBO_0000459"],
        parents: &["SBO:0000019"],
        ontology: "SBO",
        role: "participation_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000598",
        iri: "https://identifiers.org/SBO:0000598",
        label: "Promoter",
        aliases: &["http://purl.obolibrary.org/obo/SBO_0000598"],
        parents: &["SBO:0000003"],
        ontology: "SBO",
        role: "participation_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000642",
        iri: "https://identifiers.org/SBO:0000642",
        label: "Inhibited",
        aliases: &["http://purl.obolibrary.org/obo/SBO_0000642"],
        parents: &["SBO:0000644"],
        ontology: "SBO",
        role: "participation_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000643",
        iri: "https://identifiers.org/SBO:0000643",
        label: "Stimulated",
        aliases: &["http://purl.obolibrary.org/obo/SBO_0000643"],
        parents: &["SBO:0000644"],
        ontology: "SBO",
        role: "participation_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000644",
        iri: "https://identifiers.org/SBO:0000644",
        label: "Modified",
        aliases: &["http://purl.obolibrary.org/obo/SBO_0000644"],
        parents: &["SBO:0000003"],
        ontology: "SBO",
        role: "participation_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SBO:0000645",
        iri: "https://identifiers.org/SBO:0000645",
        label: "Template",
        aliases: &["http://purl.obolibrary.org/obo/SBO_0000645"],
        parents: &["SBO:0000003"],
        ontology: "SBO",
        role: "participation_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
];

const SO_GO_CHEBI_POLICIES: &[TermPolicy] = &[
    TermPolicy {
        id: "SO:0000110",
        iri: "https://identifiers.org/SO:0000110",
        label: "Sequence feature",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000110"],
        parents: &[],
        ontology: "SO",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000983",
        iri: "https://identifiers.org/SO:0000983",
        label: "Strand attribute",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000983"],
        parents: &[],
        ontology: "SO",
        role: "component_type_modifier",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000986",
        iri: "https://identifiers.org/SO:0000986",
        label: "Topology attribute",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000986"],
        parents: &[],
        ontology: "SO",
        role: "component_type_modifier",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000987",
        iri: "https://identifiers.org/SO:0000987",
        label: "Linear",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000987"],
        parents: &["SO:0000986"],
        ontology: "SO",
        role: "component_type_modifier",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000988",
        iri: "https://identifiers.org/SO:0000988",
        label: "Circular",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000988"],
        parents: &["SO:0000986"],
        ontology: "SO",
        role: "component_type_modifier",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000984",
        iri: "https://identifiers.org/SO:0000984",
        label: "Single-stranded",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000984"],
        parents: &["SO:0000983"],
        ontology: "SO",
        role: "component_type_modifier",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000985",
        iri: "https://identifiers.org/SO:0000985",
        label: "Double-stranded",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000985"],
        parents: &["SO:0000983"],
        ontology: "SO",
        role: "component_type_modifier",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0001030",
        iri: "https://identifiers.org/SO:0001030",
        label: "Forward",
        aliases: &["http://purl.obolibrary.org/obo/SO_0001030"],
        parents: &[],
        ontology: "SO",
        role: "other",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0001031",
        iri: "https://identifiers.org/SO:0001031",
        label: "Reverse complement",
        aliases: &["http://purl.obolibrary.org/obo/SO_0001031"],
        parents: &[],
        ontology: "SO",
        role: "other",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000167",
        iri: "https://identifiers.org/SO:0000167",
        label: "Promoter",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000167"],
        parents: &["SO:0000110"],
        ontology: "SO",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000139",
        iri: "https://identifiers.org/SO:0000139",
        label: "RBS",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000139"],
        parents: &["SO:0000110"],
        ontology: "SO",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000316",
        iri: "https://identifiers.org/SO:0000316",
        label: "CDS",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000316"],
        parents: &["SO:0000110"],
        ontology: "SO",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000141",
        iri: "https://identifiers.org/SO:0000141",
        label: "Terminator",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000141"],
        parents: &["SO:0000110"],
        ontology: "SO",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000704",
        iri: "https://identifiers.org/SO:0000704",
        label: "Gene",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000704"],
        parents: &["SO:0000110"],
        ontology: "SO",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000057",
        iri: "https://identifiers.org/SO:0000057",
        label: "Operator",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000057"],
        parents: &["SO:0000110"],
        ontology: "SO",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000804",
        iri: "https://identifiers.org/SO:0000804",
        label: "Engineered region",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000804"],
        parents: &["SO:0000110"],
        ontology: "SO",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "SO:0000234",
        iri: "https://identifiers.org/SO:0000234",
        label: "mRNA",
        aliases: &["http://purl.obolibrary.org/obo/SO_0000234"],
        parents: &["SO:0000110"],
        ontology: "SO",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "GO:0003674",
        iri: "https://identifiers.org/GO:0003674",
        label: "Molecular function",
        aliases: &["http://purl.obolibrary.org/obo/GO_0003674"],
        parents: &[],
        ontology: "GO",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "GO:0003700",
        iri: "https://identifiers.org/GO:0003700",
        label: "Transcription factor activity",
        aliases: &["http://purl.obolibrary.org/obo/GO_0003700"],
        parents: &["GO:0003674"],
        ontology: "GO",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "CHEBI:50906",
        iri: "https://identifiers.org/CHEBI:50906",
        label: "Role",
        aliases: &["http://purl.obolibrary.org/obo/CHEBI_50906"],
        parents: &[],
        ontology: "CHEBI",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
    TermPolicy {
        id: "CHEBI:35224",
        iri: "https://identifiers.org/CHEBI:35224",
        label: "Effector",
        aliases: &["http://purl.obolibrary.org/obo/CHEBI_35224"],
        parents: &["CHEBI:50906"],
        ontology: "CHEBI",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
        table_1: false,
        table_2: false,
    },
];

const SEQUENCE_COMPATIBILITIES: &[(&str, &str)] = &[
    ("EDAM:format_1207", "SBO:0000251"),
    ("EDAM:format_1207", "SBO:0000250"),
    ("EDAM:format_1208", "SBO:0000252"),
    ("EDAM:format_1197", "SBO:0000247"),
    ("EDAM:format_1196", "SBO:0000247"),
];

const CONFLICTS: &[(&str, &str)] = &[
    ("SBO:0000169", "SBO:0000170"),
    ("SBO:0000020", "SBO:0000459"),
    ("SBO:0000642", "SBO:0000643"),
];

const COMPONENT_ROLE_TERMS: &[&str] = &["SBO:0000289", "SBO:0000290"];

const COMPONENT_ROLE_COMPATIBILITIES: &[(&str, &str)] = &[
    ("SO:0000167", "SBO:0000251"),
    ("SO:0000139", "SBO:0000251"),
    ("SO:0000316", "SBO:0000251"),
    ("SO:0000141", "SBO:0000251"),
    ("SO:0000704", "SBO:0000251"),
    ("SO:0000057", "SBO:0000251"),
    ("SO:0000804", "SBO:0000251"),
    ("SO:0000234", "SBO:0000250"),
    ("CHEBI:35224", "SBO:0000247"),
    ("GO:0003700", "SBO:0000252"),
];

const PARTICIPATION_COMPATIBILITIES: &[(&str, &str)] = &[
    ("SBO:0000169", "SBO:0000020"),
    ("SBO:0000169", "SBO:0000642"),
    ("SBO:0000169", "SBO:0000598"),
    ("SBO:0000170", "SBO:0000459"),
    ("SBO:0000170", "SBO:0000643"),
    ("SBO:0000170", "SBO:0000598"),
    ("SBO:0000176", "SBO:0000010"),
    ("SBO:0000176", "SBO:0000011"),
    ("SBO:0000176", "SBO:0000019"),
    ("SBO:0000176", "SBO:0000644"),
    ("SBO:0000177", "SBO:0000010"),
    ("SBO:0000177", "SBO:0000011"),
    ("SBO:0000179", "SBO:0000010"),
    ("SBO:0000589", "SBO:0000011"),
    ("SBO:0000589", "SBO:0000598"),
    ("SBO:0000589", "SBO:0000645"),
    ("SBO:0000168", "SBO:0000019"),
    ("SBO:0000168", "SBO:0000644"),
];

const BRANCH_POLICIES: &[BranchPolicy] = &[
    BranchPolicy {
        root_id: "EDAM:format_2330",
        role: "other",
        component_family: "-",
        sequence_family: "other_textual",
    },
    BranchPolicy {
        root_id: "SBO:0000236",
        role: "component_type",
        component_family: "-",
        sequence_family: "-",
    },
    BranchPolicy {
        root_id: "SBO:0000231",
        role: "interaction_type",
        component_family: "-",
        sequence_family: "-",
    },
    BranchPolicy {
        root_id: "SBO:0000003",
        role: "participation_role",
        component_family: "-",
        sequence_family: "-",
    },
    BranchPolicy {
        root_id: "SBO:0000004",
        role: "other",
        component_family: "-",
        sequence_family: "-",
    },
    BranchPolicy {
        root_id: "SBO:0000545",
        role: "other",
        component_family: "-",
        sequence_family: "-",
    },
    BranchPolicy {
        root_id: "SO:0000986",
        role: "component_type_modifier",
        component_family: "-",
        sequence_family: "-",
    },
    BranchPolicy {
        root_id: "SO:0000983",
        role: "component_type_modifier",
        component_family: "-",
        sequence_family: "-",
    },
    BranchPolicy {
        root_id: "SO:0000110",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
    },
    BranchPolicy {
        root_id: "GO:0003674",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
    },
    BranchPolicy {
        root_id: "CHEBI:50906",
        role: "feature_role",
        component_family: "-",
        sequence_family: "-",
    },
    BranchPolicy {
        root_id: "CL:0000000",
        role: "component_type",
        component_family: "-",
        sequence_family: "-",
    },
];

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
