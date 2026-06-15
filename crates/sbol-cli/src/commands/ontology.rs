use std::process::ExitCode;

use sbol_ontology::{KnownOntology, OntologyCache, OntologyDescriptor};

use crate::cli::{OntologyCommand, OntologyInstallArgs, OntologyRemoveArgs, OntologyVerifyArgs};
use crate::style::{Styles, paint};

pub(crate) fn ontology(command: OntologyCommand, styles: Styles) -> ExitCode {
    let cache = OntologyCache::from_default_path();
    match command {
        OntologyCommand::Install(args) => ontology_install(&cache, args, styles),
        OntologyCommand::List => ontology_list(&cache, styles),
        OntologyCommand::Path => {
            println!("{}", cache.path().display());
            ExitCode::SUCCESS
        }
        OntologyCommand::Remove(args) => ontology_remove(&cache, args, styles),
        OntologyCommand::Verify(args) => ontology_verify(&cache, args, styles),
    }
}

pub(crate) fn known_ontology_by_name(name: &str) -> Option<KnownOntology> {
    match name.to_ascii_lowercase().as_str() {
        "ncit" => Some(KnownOntology::Ncit),
        _ => None,
    }
}

fn ontology_install(cache: &OntologyCache, args: OntologyInstallArgs, styles: Styles) -> ExitCode {
    let Some(known) = known_ontology_by_name(&args.name) else {
        eprintln!(
            "{}: unknown ontology `{}` — try one of: ncit",
            styles.err_label(),
            args.name
        );
        return ExitCode::from(2);
    };
    let descriptor: &OntologyDescriptor = known.descriptor();
    let result = if args.force {
        cache.install(descriptor)
    } else {
        cache.ensure_installed(descriptor)
    };
    match result {
        Ok(installed) => {
            println!(
                "{} `{}` from {}\n  fact sha256: {}",
                paint(styles.stdout, "1;32", "installed"),
                installed.name,
                installed.source_url,
                installed.fact_sha256,
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}: ontology install failed: {error}", styles.err_label());
            ExitCode::from(2)
        }
    }
}

fn ontology_list(cache: &OntologyCache, styles: Styles) -> ExitCode {
    match cache.list() {
        Ok(installed) => {
            if installed.is_empty() {
                println!("(no extensions installed)");
                return ExitCode::SUCCESS;
            }
            for entry in installed {
                println!(
                    "{name}\t{url}\tsha256={hash}\tinstalled_at={installed_at}",
                    name = entry.name,
                    url = entry.source_url,
                    hash = entry.fact_sha256,
                    installed_at = entry.installed_at,
                );
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}: cache list failed: {error}", styles.err_label());
            ExitCode::from(2)
        }
    }
}

fn ontology_remove(cache: &OntologyCache, args: OntologyRemoveArgs, styles: Styles) -> ExitCode {
    match cache.remove(&args.name) {
        Ok(true) => {
            println!(
                "{} `{}`",
                paint(styles.stdout, "1;32", "removed"),
                args.name
            );
            ExitCode::SUCCESS
        }
        Ok(false) => {
            eprintln!("{}: `{}` is not installed", styles.err_label(), args.name);
            ExitCode::from(2)
        }
        Err(error) => {
            eprintln!("{}: cache remove failed: {error}", styles.err_label());
            ExitCode::from(2)
        }
    }
}

fn ontology_verify(cache: &OntologyCache, args: OntologyVerifyArgs, styles: Styles) -> ExitCode {
    let names = match args.name {
        Some(name) => vec![name],
        None => match cache.list() {
            Ok(installed) => installed.into_iter().map(|m| m.name).collect(),
            Err(error) => {
                eprintln!("{}: cache list failed: {error}", styles.err_label());
                return ExitCode::from(2);
            }
        },
    };
    if names.is_empty() {
        println!("(no extensions installed)");
        return ExitCode::SUCCESS;
    }
    let mut had_failure = false;
    for name in names {
        match cache.verify(&name) {
            Ok(_) => println!("{}\t{name}", paint(styles.stdout, "32", "ok")),
            Err(error) => {
                eprintln!("{}\t{name}: {error}", paint(styles.stderr, "1;31", "FAIL"));
                had_failure = true;
            }
        }
    }
    if had_failure {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
