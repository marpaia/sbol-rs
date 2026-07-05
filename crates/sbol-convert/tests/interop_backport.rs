//! Interoperability with the sbol-utilities / sbolgraph backport
//! annotations. sbol-rs must round-trip the SBOL 3 object namespace via
//! `backport:sbol3namespace` and FunctionalComponent access via
//! `backport:sbol2_access`, byte-for-byte matching the predicate IRIs the
//! community converter emits.

mod common;

use common::downgrade::*;

use sbol3::{Document, RdfFormat};

const BACKPORT_SBOL3_NAMESPACE: &str = "http://sboltools.org/backport#sbol3namespace";
const BACKPORT_SBOL2_ACCESS: &str = "http://sboltools.org/backport#sbol2_access";

/// Downgrade stashes each SBOL 3 top-level's `hasNamespace` on the
/// corresponding SBOL 2 object as `backport:sbol3namespace`, so
/// sbol-utilities / sbolgraph can reconstruct the SBOL 3 namespace.
#[test]
fn downgrade_emits_sbol3namespace() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://example.org/lab/gfp> a sbol3:Component ;
    sbol3:hasNamespace <https://example.org/lab> ;
    sbol3:displayId "gfp" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .
"#;
    let document = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (graph, _report) = sbol_convert::downgrade(&document).expect("downgrade");

    assert!(
        has_triple(
            &graph,
            "https://example.org/lab/gfp",
            BACKPORT_SBOL3_NAMESPACE,
            "https://example.org/lab",
        ),
        "downgrade should stash the SBOL 3 namespace under \
         backport:sbol3namespace for sbol-utilities interop"
    );
}

/// Upgrade honors an sbol-utilities-produced `backport:sbol3namespace`
/// annotation, using it as the SBOL 3 `hasNamespace` in preference to the
/// value it would otherwise derive from the identity IRI, and consumes the
/// annotation (it must not leak into the SBOL 3 output).
#[test]
fn upgrade_reads_sbol3namespace() {
    // The identity IRI derivation would yield `https://example.org/lab`,
    // but the explicit annotation names a different namespace. The
    // annotation must win.
    let ttl = r#"
@prefix sbol2: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .
@prefix backport: <http://sboltools.org/backport#> .

<https://example.org/lab/gfp/1> a sbol2:ComponentDefinition ;
    sbol2:persistentIdentity <https://example.org/lab/gfp> ;
    sbol2:displayId "gfp" ;
    sbol2:version "1" ;
    sbol2:type biopax:Dna ;
    backport:sbol3namespace <https://other.example.net/space> .
"#;
    let (upgraded, _report) =
        sbol_convert::upgrade_from_sbol2(ttl, RdfFormat::Turtle).expect("upgrade");
    let graph = upgraded.rdf_graph();

    assert!(
        has_triple(
            graph,
            "https://example.org/lab/gfp",
            "http://sbols.org/v3#hasNamespace",
            "https://other.example.net/space",
        ),
        "upgrade should honor backport:sbol3namespace over IRI derivation"
    );
    assert!(
        !graph
            .triples()
            .iter()
            .any(|t| t.predicate.as_str() == BACKPORT_SBOL3_NAMESPACE),
        "backport:sbol3namespace must be consumed, not passed through to SBOL 3"
    );
}

/// A full sbol-rs round trip preserves the SBOL 3 namespace through the
/// `backport:sbol3namespace` channel: downgrade emits it, re-upgrade reads
/// it, and the SBOL 3 `hasNamespace` survives unchanged.
#[test]
fn sbol3namespace_round_trips_through_sbol2() {
    let ttl = r#"
@prefix sbol3: <http://sbols.org/v3#> .

<https://example.org/lab/gfp> a sbol3:Component ;
    sbol3:hasNamespace <https://example.org/lab> ;
    sbol3:displayId "gfp" ;
    sbol3:type <https://identifiers.org/SBO:0000251> .
"#;
    let original = Document::read(ttl, RdfFormat::Turtle).expect("parse");
    let (downgraded, _) = sbol_convert::downgrade(&original).expect("downgrade");
    let sbol2 = downgraded.write(RdfFormat::Turtle).expect("write turtle");
    let (reupgraded, _) =
        sbol_convert::upgrade_from_sbol2(&sbol2, RdfFormat::Turtle).expect("re-upgrade");

    assert!(
        has_triple(
            reupgraded.rdf_graph(),
            "https://example.org/lab/gfp",
            "http://sbols.org/v3#hasNamespace",
            "https://example.org/lab",
        ),
        "hasNamespace should survive SBOL3 → SBOL2 → SBOL3"
    );
}

/// FunctionalComponent `access` round-trips through the backport channel.
/// A non-default `private` access is preserved verbatim, and the SBOL 3
/// carrier is `backport:sbol2_access` — byte interoperable with
/// sbol-utilities.
#[test]
fn functional_component_access_round_trips() {
    let ttl = r#"
@prefix sbol2: <http://sbols.org/v2#> .
@prefix biopax: <http://www.biopax.org/release/biopax-level3.owl#> .

<https://example.org/lab/md/1> a sbol2:ModuleDefinition ;
    sbol2:persistentIdentity <https://example.org/lab/md> ;
    sbol2:displayId "md" ;
    sbol2:version "1" ;
    sbol2:functionalComponent <https://example.org/lab/md/fc/1> .

<https://example.org/lab/md/fc/1> a sbol2:FunctionalComponent ;
    sbol2:persistentIdentity <https://example.org/lab/md/fc> ;
    sbol2:displayId "fc" ;
    sbol2:version "1" ;
    sbol2:access sbol2:private ;
    sbol2:direction sbol2:in ;
    sbol2:definition <https://example.org/lab/cd/1> .

<https://example.org/lab/cd/1> a sbol2:ComponentDefinition ;
    sbol2:persistentIdentity <https://example.org/lab/cd> ;
    sbol2:displayId "cd" ;
    sbol2:version "1" ;
    sbol2:type biopax:Dna .
"#;
    let (upgraded, _) = sbol_convert::upgrade_from_sbol2(ttl, RdfFormat::Turtle).expect("upgrade");

    // On the SBOL 3 side the access is carried on the SubComponent under
    // the sbol-utilities-compatible backport predicate.
    assert!(
        has_triple(
            upgraded.rdf_graph(),
            "https://example.org/lab/md/fc",
            BACKPORT_SBOL2_ACCESS,
            "http://sbols.org/v2#private",
        ),
        "FunctionalComponent access should be preserved as backport:sbol2_access"
    );

    // Downgrade restores the original sbol2:access.
    let (downgraded, _) = sbol_convert::downgrade(&upgraded).expect("downgrade");
    assert!(
        has_triple(
            &downgraded,
            "https://example.org/lab/md/fc/1",
            "http://sbols.org/v2#access",
            "http://sbols.org/v2#private",
        ),
        "downgrade should restore sbol2:access private"
    );
    // The private access must not be silently overwritten with the
    // synthesized `public`/`private` default.
    assert_eq!(
        count_triples(
            &downgraded,
            "https://example.org/lab/md/fc/1",
            "http://sbols.org/v2#access",
            "http://sbols.org/v2#public",
        ),
        0,
        "restored private access should not gain a competing public default"
    );
}
