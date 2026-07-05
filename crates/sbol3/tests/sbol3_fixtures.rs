use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

mod common;

const MANIFEST: &str = include_str!("../../../tests/sbol3_fixtures_manifest.tsv");
const SBOL_SPEC_MARKDOWN: &str = include_str!("../../../spec/SBOL3.1.0.md");
const EXPECTED_TOTAL: usize = 33;
const EXPECTED_VALID: usize = 33;
const EXPECTED_INVALID: usize = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expectation {
    Valid,
    Invalid,
}

#[derive(Debug)]
struct Fixture<'a> {
    path: &'a str,
    format: &'a str,
    expectation: Expectation,
    coverage: &'a str,
}

fn fixtures() -> Vec<Fixture<'static>> {
    MANIFEST
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            if index == 0 || line.trim().is_empty() || line.starts_with('#') {
                return None;
            }

            let columns: Vec<_> = line.split('\t').collect();
            assert_eq!(columns.len(), 4, "malformed fixture manifest line {index}");

            let expectation = match columns[2] {
                "valid" => Expectation::Valid,
                "invalid" => Expectation::Invalid,
                other => panic!("unknown fixture expectation `{other}` on line {index}"),
            };

            Some(Fixture {
                path: columns[0],
                format: columns[1],
                expectation,
                coverage: columns[3],
            })
        })
        .collect()
}

fn manifest_path(relative_path: &str) -> PathBuf {
    common::fixture_root().join(relative_path)
}

fn fixture_root() -> &'static Path {
    common::fixture_root()
}

#[test]
fn fixture_manifest_is_well_formed() {
    let fixtures = fixtures();

    assert_eq!(fixtures.len(), EXPECTED_TOTAL);
    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.expectation == Expectation::Valid)
            .count(),
        EXPECTED_VALID
    );
    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.expectation == Expectation::Invalid)
            .count(),
        EXPECTED_INVALID
    );

    for fixture in fixtures {
        assert_eq!(fixture.format, "turtle");
        assert!(!fixture.coverage.is_empty());

        let path = manifest_path(fixture.path);
        assert!(path.exists(), "missing fixture {}", fixture.path);
        assert_eq!(path.extension().and_then(|ext| ext.to_str()), Some("ttl"));

        let contents = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", fixture.path));
        assert!(
            contents.contains("http://sbols.org/v3#"),
            "fixture {} does not declare the SBOL 3 namespace",
            fixture.path
        );
    }
}

#[test]
fn fixture_corpus_covers_sbol_classes() {
    let mut corpus = String::new();
    for fixture in fixtures() {
        let path = manifest_path(fixture.path);
        corpus.push_str(&fs::read_to_string(path).unwrap());
        corpus.push('\n');
    }

    let observed_classes = expected_sbol_classes()
        .iter()
        .filter(|class_name| corpus.contains(&format!("sbol:{class_name}")))
        .copied()
        .collect::<BTreeSet<_>>();

    for class_name in expected_sbol_classes() {
        assert!(
            observed_classes.contains(class_name),
            "fixture corpus does not exercise sbol:{class_name}"
        );
    }
}

#[test]
fn fixture_cache_records_pinned_commit() {
    let sentinel = fs::read_to_string(fixture_root().join(common::CACHE_SENTINEL)).unwrap();
    assert_eq!(sentinel.trim(), common::SBOLTESTSUITE_COMMIT);
}

#[test]
fn valid_fixtures_parse_and_validate() {
    for fixture in fixtures()
        .into_iter()
        .filter(|fixture| fixture.expectation == Expectation::Valid)
    {
        let contents = fs::read_to_string(manifest_path(fixture.path)).unwrap();
        let document = sbol3::Document::read_turtle(&contents)
            .unwrap_or_else(|error| panic!("{} did not parse: {error}", fixture.path));
        let report = document.validate();

        assert!(
            report.is_valid(),
            "{} did not validate: {report}",
            fixture.path
        );
    }
}

#[test]
fn invalid_fixtures_report_validation_errors() {
    for fixture in fixtures()
        .into_iter()
        .filter(|fixture| fixture.expectation == Expectation::Invalid)
    {
        let contents = fs::read_to_string(manifest_path(fixture.path)).unwrap();
        let document = sbol3::Document::read_turtle(&contents)
            .unwrap_or_else(|error| panic!("{} did not parse: {error}", fixture.path));
        let report = document.validate();

        assert!(
            report.has_errors(),
            "{} unexpectedly validated successfully",
            fixture.path
        );
    }
}

#[test]
fn local_invalid_fixtures_report_expected_rule_ids() {
    let cases = [
        (
            "bad displayId",
            r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX sbol: <http://sbols.org/v3#>
:component a sbol:Component;
    sbol:displayId "bad id";
    sbol:hasNamespace <https://example.org>;
    sbol:type SBO:0000251 .
"#,
            "sbol3-10201",
        ),
        (
            "missing Component type",
            r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX sbol: <http://sbols.org/v3#>
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org> .
"#,
            "sbol3-10110",
        ),
        (
            "unknown SBOL property",
            r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX sbol: <http://sbols.org/v3#>
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasNamespace <https://example.org>;
    sbol:notAProperty "x";
    sbol:type SBO:0000251 .
"#,
            "sbol3-10105",
        ),
        (
            "range end before start",
            r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX EDAM: <https://identifiers.org/edam:>
PREFIX SBO: <https://identifiers.org/SBO:>
PREFIX sbol: <http://sbols.org/v3#>
:component a sbol:Component;
    sbol:displayId "component";
    sbol:hasFeature <component/feature>;
    sbol:hasNamespace <https://example.org>;
    sbol:hasSequence :sequence;
    sbol:type SBO:0000251 .
:sequence a sbol:Sequence;
    sbol:displayId "sequence";
    sbol:elements "atgc";
    sbol:encoding EDAM:format_1207;
    sbol:hasNamespace <https://example.org> .
<component/feature> a sbol:SequenceFeature;
    sbol:displayId "feature";
    sbol:hasLocation <component/feature/range> .
<component/feature/range> a sbol:Range;
    sbol:displayId "range";
    sbol:end "2";
    sbol:hasSequence :sequence;
    sbol:start "3" .
"#,
            "sbol3-11403",
        ),
        (
            "attachment hash without algorithm",
            r#"BASE <https://example.org/>
PREFIX : <https://example.org/>
PREFIX sbol: <http://sbols.org/v3#>
:attachment a sbol:Attachment;
    sbol:displayId "attachment";
    sbol:hash "abcdef";
    sbol:hasNamespace <https://example.org>;
    sbol:source <https://example.org/file.txt> .
"#,
            "sbol3-12808",
        ),
    ];

    for (name, turtle, expected_rule) in cases {
        let document =
            sbol3::Document::read_turtle(turtle).unwrap_or_else(|error| panic!("{name}: {error}"));
        let report = document.validate();

        assert!(
            report.errors().any(|issue| issue.rule == expected_rule),
            "{name} did not report {expected_rule}; got {:?}",
            report.issues()
        );
    }
}

#[test]
fn typed_views_expose_fixture_data() {
    let contents = fs::read_to_string(manifest_path("local/variable_feature.ttl")).unwrap();
    let document = sbol3::Document::read_turtle(&contents).unwrap();

    let component_ids = document
        .components()
        .filter_map(|component| component.identified.display_id.as_deref())
        .collect::<BTreeSet<_>>();
    assert!(component_ids.contains("promoter_a"));
    assert!(component_ids.contains("template_component"));

    let derivation = document
        .combinatorial_derivations()
        .find(|derivation| {
            derivation.identified.display_id.as_deref() == Some("promoter_derivation")
        })
        .expect("missing promoter_derivation");
    assert!(derivation.template.is_some());
    assert_eq!(derivation.variable_features.len(), 1);
}

#[test]
fn representative_fixtures_round_trip_as_normalized_rdf() {
    let fixture_paths = [
        "SBOLTestSuite/SBOL3/entity/component/component.ttl",
        "SBOLTestSuite/SBOL3/entity/range/range.ttl",
        "SBOLTestSuite/SBOL3/entity/componentreference/componentreference.ttl",
        "SBOLTestSuite/SBOL3/entity/interaction/interaction.ttl",
        "SBOLTestSuite/SBOL3/entity/annotation/annotation.ttl",
        "SBOLTestSuite/SBOL3/provenance_entity/activity/activity.ttl",
        "SBOLTestSuite/SBOL3/measurement_entity/measurement/measurement.ttl",
        "SBOLTestSuite/SBOL3/entity/attachment/attachment.ttl",
        "local/variable_feature.ttl",
    ];

    for fixture_path in fixture_paths {
        let contents = fs::read_to_string(manifest_path(fixture_path)).unwrap();
        let document = sbol3::Document::read_turtle(&contents)
            .unwrap_or_else(|error| panic!("{fixture_path} did not parse: {error}"));

        for &format in sbol3::RdfFormat::ALL {
            let serialized = document.write(format).unwrap_or_else(|error| {
                panic!("{fixture_path} did not serialize as {format}: {error}")
            });
            let reparsed = sbol3::Document::read(&serialized, format).unwrap_or_else(|error| {
                panic!("{fixture_path} did not reparse as {format}: {error}")
            });

            assert_eq!(
                document.rdf_graph().normalized_triples(),
                reparsed.rdf_graph().normalized_triples(),
                "{fixture_path} changed after {format} round trip"
            );
            assert!(
                reparsed.validate().is_valid(),
                "{fixture_path} reserialized as {format} did not validate"
            );
        }
    }
}

#[test]
fn typed_round_trip_preserves_extension_triples_on_typed_objects() {
    // The annotation fixture attaches custom `parts.igem.org` predicates to
    // its Component. They round-trip through the typed model via
    // `IdentifiedData.extensions` and are re-emitted by `to_rdf_triples`.
    //
    // Subjects whose only `rdf:type` is `sbol:Identified` (e.g. the
    // fixture's `/usage1`, `/information1` resources) still drop because
    // `try_from_object` has no concrete typed variant for them. Preserving
    // those needs owned typed models for the bare-Identified case.
    let fixture_path = "SBOLTestSuite/SBOL3/entity/annotation/annotation.ttl";
    let contents = fs::read_to_string(manifest_path(fixture_path)).unwrap();
    let document = sbol3::Document::read_turtle(&contents).expect("fixture must parse");

    let component = document
        .components()
        .find(|c| c.identified.display_id.as_deref() == Some("BBa_J23119"))
        .expect("fixture should contain BBa_J23119 Component");

    let igem_predicates: Vec<&str> = component
        .identified
        .extensions
        .iter()
        .map(|ext| ext.predicate.as_str())
        .filter(|p| p.starts_with("http://parts.igem.org/"))
        .collect();
    assert!(
        !igem_predicates.is_empty(),
        "Component should preserve iGEM extension triples on its IdentifiedData; got {igem_predicates:?}"
    );

    // Round-trip the typed Component back to RDF and verify the same
    // extension predicates are emitted.
    let rebuilt =
        sbol3::Document::from_objects(vec![sbol3::SbolObject::Component(component.clone())])
            .expect("typed objects must rebuild");
    let rebuilt_predicates: BTreeSet<String> = rebuilt
        .rdf_graph()
        .triples()
        .iter()
        .filter(|t| t.predicate.as_str().starts_with("http://parts.igem.org/"))
        .map(|t| t.predicate.as_str().to_string())
        .collect();
    for expected in &igem_predicates {
        assert!(
            rebuilt_predicates.contains(*expected),
            "rebuilt graph lost extension predicate {expected}"
        );
    }
}

#[test]
fn validation_rule_statuses_are_unique_and_cover_metadata_rules() {
    let statuses = sbol3::validation_rule_statuses();
    let mut seen = BTreeSet::new();
    for status in statuses {
        assert!(
            seen.insert(status.rule),
            "duplicate status for {}",
            status.rule
        );
        assert!(!status.note.is_empty());
    }

    for required_rule in [
        "sbol3-10105",
        "sbol3-10109",
        "sbol3-10110",
        "sbol3-10111",
        "sbol3-10112",
        "sbol3-10113",
        "sbol3-10201",
        "sbol3-10301",
        "sbol3-11403",
        "sbol3-12808",
    ] {
        assert!(
            statuses.iter().any(|status| status.rule == required_rule),
            "missing validation status for {required_rule}"
        );
    }

    let status_rules = statuses
        .iter()
        .map(|status| status.rule)
        .collect::<BTreeSet<_>>();
    let spec_rules = spec_rule_ids();
    assert_eq!(spec_rules.len(), 149);
    for rule in spec_rule_ids() {
        assert!(
            status_rules.contains(rule.as_str()),
            "missing validation status for {rule}"
        );
    }
}

#[test]
fn validation_rule_catalog_is_pinned_to_sbol_3_1_0() {
    assert_eq!(sbol3::VALIDATION_RULE_SPEC_VERSION, sbol3::SPEC_VERSION);
    assert_eq!(
        sbol3::VALIDATION_RULE_SPEC_CANONICAL_URL,
        sbol3::SPECIFICATION_URL
    );
    assert_eq!(sbol3::VALIDATION_RULE_SPEC_PATH, "spec/SBOL3.1.0.md");
    assert_eq!(
        sbol3::VALIDATION_RULE_SPEC_PDF_SHA256,
        "7c1ef88f83b8fff98acd07c742b377bbb8618508684b7dab17032396667f0b2c"
    );
    assert!(
        SBOL_SPEC_MARKDOWN.starts_with("# Synthetic Biology Open Language (SBOL) Version 3.1.0")
    );
}

fn spec_rule_ids() -> BTreeSet<String> {
    SBOL_SPEC_MARKDOWN
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
        .filter(|word| {
            word.len() == "sbol3-00000".len()
                && word.starts_with("sbol3-")
                && word["sbol3-".len()..]
                    .chars()
                    .all(|character| character.is_ascii_digit())
        })
        .map(ToOwned::to_owned)
        .collect()
}

fn expected_sbol_classes() -> &'static [&'static str] {
    &[
        "Attachment",
        "Collection",
        "CombinatorialDerivation",
        "Component",
        "ComponentReference",
        "Constraint",
        "Cut",
        "EntireSequence",
        "Experiment",
        "ExperimentalData",
        "ExternallyDefined",
        "Identified",
        "Implementation",
        "Interaction",
        "Interface",
        "LocalSubComponent",
        "Model",
        "Participation",
        "Range",
        "Sequence",
        "SequenceFeature",
        "SubComponent",
        "TopLevel",
        "VariableFeature",
    ]
}
