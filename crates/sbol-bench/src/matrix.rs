//! The benchmark matrix: which SBOL versions, implementations, fixtures,
//! and parse/serialize format pairs make up a run, plus the default
//! iteration counts.

use sbol::v3::RdfFormat;

/// The SBOL data model version a fixture and bench case belong to.
/// sbol-rs implements both natively; the foreign tools are
/// version-specific (libSBOLj3 and pySBOL3 are SBOL 3, libSBOLj is
/// SBOL 2), so the report groups and labels rows by version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Version {
    Sbol2,
    Sbol3,
}

impl Version {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Sbol2 => "SBOL 2",
            Self::Sbol3 => "SBOL 3",
        }
    }

    /// Short identifier used in the machine-readable report and as the
    /// version argument passed to the sbol-rs runner.
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Sbol2 => "sbol2",
            Self::Sbol3 => "sbol3",
        }
    }

    /// Directory under `tests/fixtures` that holds this version's source
    /// fixtures.
    pub(crate) fn fixture_dir(self) -> &'static str {
        match self {
            Self::Sbol2 => "tests/fixtures/sbol2",
            Self::Sbol3 => "tests/fixtures/sbol3",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Implementation {
    SbolRs,
    Pysbol3,
    Libsbolj3,
    Sboljs,
    Libsbolj,
}

impl Implementation {
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::SbolRs => "sbol-rs",
            Self::Pysbol3 => "pysbol3",
            Self::Libsbolj3 => "libsbolj3",
            Self::Sboljs => "sboljs",
            Self::Libsbolj => "libsbolj",
        }
    }

    pub(crate) fn docker_image(self) -> &'static str {
        match self {
            Self::SbolRs => "sbol-rs-bench",
            Self::Pysbol3 => "pysbol3-bench",
            Self::Libsbolj3 => "libsbolj3-bench",
            Self::Sboljs => "sboljs3-bench",
            Self::Libsbolj => "libsbolj2-bench",
        }
    }

    // Build command hint surfaced when an image is missing. The sbol-rs
    // image is built from the workspace root with `-f` because its
    // Dockerfile needs to see every workspace member, unlike the
    // foreign images whose context is their own directory.
    pub(crate) fn docker_build_command(self) -> &'static str {
        match self {
            Self::SbolRs => {
                "docker build -t sbol-rs-bench -f benches/cross-impl/sbol-rs/Dockerfile ."
            }
            Self::Pysbol3 => "docker build -t pysbol3-bench benches/cross-impl/pysbol3/",
            Self::Libsbolj3 => "docker build -t libsbolj3-bench benches/cross-impl/libsbolj3/",
            Self::Sboljs => "docker build -t sboljs3-bench benches/cross-impl/sboljs3/",
            Self::Libsbolj => "docker build -t libsbolj2-bench benches/cross-impl/libsbolj2/",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Fixture {
    pub(crate) stem: &'static str,
    /// Path to the source fixture relative to the version's
    /// [`Version::fixture_dir`].
    pub(crate) source: &'static str,
    pub(crate) version: Version,
}

pub(crate) const FIXTURES: &[Fixture] = &[
    // SBOL 2 fixtures, exchanged natively as RDF/XML, spanning ~1.7 KB
    // to ~79 KB.
    Fixture {
        stem: "sbol2_cd_sa_range",
        source: "real/CD_SA_Range_Example.xml",
        version: Version::Sbol2,
    },
    Fixture {
        stem: "sbol2_component_output",
        source: "real/ComponentDefinitionOutput.xml",
        version: Version::Sbol2,
    },
    Fixture {
        stem: "sbol2_bba_k093005",
        source: "real/synbiohub/BBa_K093005.xml",
        version: Version::Sbol2,
    },
    Fixture {
        stem: "sbol2_bba_f2620",
        source: "real/synbiohub/BBa_F2620.xml",
        version: Version::Sbol2,
    },
    // SBOL 3 fixtures from the SBOL test suite, exchanged as Turtle.
    Fixture {
        stem: "component",
        source: "SBOLTestSuite/SBOL3/entity/component/component.ttl",
        version: Version::Sbol3,
    },
    Fixture {
        stem: "multicellular_simple",
        source: "SBOLTestSuite/SBOL3/multicellular_simple/multicellular_simple.ttl",
        version: Version::Sbol3,
    },
    Fixture {
        stem: "bba_f2620_popsreceiver",
        source: "SBOLTestSuite/SBOL3/BBa_F2620_PoPSReceiver/BBa_F2620_PoPSReceiver.ttl",
        version: Version::Sbol3,
    },
    Fixture {
        stem: "toggle_switch_v2",
        source: "SBOLTestSuite/SBOL3/toggle_switch_v2/toggle_switch_v2.ttl",
        version: Version::Sbol3,
    },
];

#[derive(Clone, Copy, Debug)]
pub(crate) struct BenchCase {
    pub(crate) version: Version,
    pub(crate) implementation: Implementation,
    pub(crate) parse_format: RdfFormat,
    pub(crate) serialize_format: RdfFormat,
    /// Whether this case also times a `validate` phase. Set on the
    /// canonical same-format round-trip row for every implementation
    /// that ships a validator (sbol-rs for both versions, pySBOL3 and
    /// libSBOLj3 for SBOL 3). Validation runs on the parsed in-memory
    /// model, which is format-independent, so one row per implementation
    /// is enough. sboljs has no validator, and the SBOL 2 libSBOLj
    /// driver does not run one, so their rows never set this.
    pub(crate) validate: bool,
}

impl BenchCase {
    /// A case whose parse and serialize formats differ measures true
    /// format-conversion cost rather than a same-format round trip.
    pub(crate) fn is_conversion(self) -> bool {
        self.parse_format != self.serialize_format
    }
}

pub(crate) const BENCH_CASES: &[BenchCase] = &[
    // ---- SBOL 2 ----
    // sbol-rs reads and writes every RDF serialization for SBOL 2, so it
    // covers the same round-trip and conversion grid as it does for
    // SBOL 3. The Turtle round-trip row also carries validation.
    BenchCase {
        version: Version::Sbol2,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::Turtle,
        validate: true,
    },
    BenchCase {
        version: Version::Sbol2,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol2,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::JsonLd,
        serialize_format: RdfFormat::JsonLd,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol2,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::NTriples,
        serialize_format: RdfFormat::NTriples,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol2,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::Turtle,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol2,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::RdfXml,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol2,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::JsonLd,
        validate: false,
    },
    // libSBOLj (SBOL 2 Java) reads and writes RDF/XML and Turtle; it has
    // no N-Triples or JSON-LD serializer, so those cells stay dashed.
    // The RDF/XML input is the source test-suite file, which libSBOLj
    // parses natively.
    BenchCase {
        version: Version::Sbol2,
        implementation: Implementation::Libsbolj,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol2,
        implementation: Implementation::Libsbolj,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::Turtle,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol2,
        implementation: Implementation::Libsbolj,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::Turtle,
        validate: false,
    },
    // ---- SBOL 3 ----
    // Same-format round trips. The Turtle row for each implementation
    // with a validator also carries the validation phase.
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::Turtle,
        validate: true,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::JsonLd,
        serialize_format: RdfFormat::JsonLd,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::NTriples,
        serialize_format: RdfFormat::NTriples,
        validate: false,
    },
    // Cross-format conversions exercise the parse-one-format,
    // serialize-another path so the serialize phase captures real
    // conversion cost. Every RDF format is supported by sbol-rs,
    // pySBOL3, and libSBOLj3.
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::RdfXml,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::Turtle,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::JsonLd,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::Turtle,
        validate: true,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::JsonLd,
        serialize_format: RdfFormat::JsonLd,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::NTriples,
        serialize_format: RdfFormat::NTriples,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::RdfXml,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::Turtle,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::JsonLd,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::Turtle,
        validate: true,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::JsonLd,
        serialize_format: RdfFormat::JsonLd,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::NTriples,
        serialize_format: RdfFormat::NTriples,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::RdfXml,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::Turtle,
        validate: false,
    },
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::JsonLd,
        validate: false,
    },
    // sboljs only: rdfoo's N-Triples parser stack is broken (an
    // `@rdfjs/sink-map` version skew that throws "parser.import is not
    // a function" before any triple is even produced), so sboljs's
    // only working path is RDF/XML in, RDF/XML out. It ships no
    // validator, so it never runs the validation phase. The RDF/XML
    // row uses libSBOLj3's committed reference output as input (see
    // prepare_fixture_in_every_format) so every impl parses the same
    // bytes.
    BenchCase {
        version: Version::Sbol3,
        implementation: Implementation::Sboljs,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
        validate: false,
    },
];

// 20 warmup iterations is enough for the JVM's tiered JIT to reach
// steady state on a loop this small; 100 measured iterations gives
// stable p50s and acceptable p99s for every implementation in the
// matrix. The numbers in benches/cross-impl/README.md are captured at
// these defaults.
pub(crate) const DEFAULT_WARMUP: usize = 20;
pub(crate) const DEFAULT_ITERS: usize = 100;
