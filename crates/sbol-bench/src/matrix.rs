//! The benchmark matrix: which implementations, fixtures, and parse/serialize
//! format pairs make up a run, plus the default iteration counts.

use sbol::RdfFormat;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Implementation {
    SbolRs,
    Pysbol3,
    Libsbolj3,
    Sboljs,
}

impl Implementation {
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::SbolRs => "sbol-rs",
            Self::Pysbol3 => "pysbol3",
            Self::Libsbolj3 => "libsbolj3",
            Self::Sboljs => "sboljs",
        }
    }

    pub(crate) fn docker_image(self) -> &'static str {
        match self {
            Self::SbolRs => "sbol-rs-bench",
            Self::Pysbol3 => "pysbol3-bench",
            Self::Libsbolj3 => "libsbolj3-bench",
            Self::Sboljs => "sboljs3-bench",
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
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Fixture {
    pub(crate) stem: &'static str,
    pub(crate) source: &'static str,
}

pub(crate) const FIXTURES: &[Fixture] = &[
    Fixture {
        stem: "component",
        source: "SBOLTestSuite/SBOL3/entity/component/component.ttl",
    },
    Fixture {
        stem: "multicellular_simple",
        source: "SBOLTestSuite/SBOL3/multicellular_simple/multicellular_simple.ttl",
    },
    Fixture {
        stem: "bba_f2620_popsreceiver",
        source: "SBOLTestSuite/SBOL3/BBa_F2620_PoPSReceiver/BBa_F2620_PoPSReceiver.ttl",
    },
    Fixture {
        stem: "toggle_switch_v2",
        source: "SBOLTestSuite/SBOL3/toggle_switch_v2/toggle_switch_v2.ttl",
    },
];

#[derive(Clone, Copy, Debug)]
pub(crate) struct BenchCase {
    pub(crate) implementation: Implementation,
    pub(crate) parse_format: RdfFormat,
    pub(crate) serialize_format: RdfFormat,
}

pub(crate) const BENCH_CASES: &[BenchCase] = &[
    BenchCase {
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::Turtle,
    },
    BenchCase {
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
    },
    BenchCase {
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::JsonLd,
        serialize_format: RdfFormat::JsonLd,
    },
    BenchCase {
        implementation: Implementation::SbolRs,
        parse_format: RdfFormat::NTriples,
        serialize_format: RdfFormat::NTriples,
    },
    BenchCase {
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::Turtle,
    },
    BenchCase {
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
    },
    BenchCase {
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::JsonLd,
        serialize_format: RdfFormat::JsonLd,
    },
    BenchCase {
        implementation: Implementation::Pysbol3,
        parse_format: RdfFormat::NTriples,
        serialize_format: RdfFormat::NTriples,
    },
    BenchCase {
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::Turtle,
        serialize_format: RdfFormat::Turtle,
    },
    BenchCase {
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
    },
    BenchCase {
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::JsonLd,
        serialize_format: RdfFormat::JsonLd,
    },
    BenchCase {
        implementation: Implementation::Libsbolj3,
        parse_format: RdfFormat::NTriples,
        serialize_format: RdfFormat::NTriples,
    },
    // sboljs only: rdfoo's N-Triples parser stack is broken (an
    // `@rdfjs/sink-map` version skew that throws "parser.import is not
    // a function" before any triple is even produced), so sboljs's
    // only working path is RDF/XML in, RDF/XML out. The RDF/XML row
    // uses libSBOLj3's committed reference output as input (see
    // prepare_fixture_in_every_format) so every impl parses the same
    // bytes.
    BenchCase {
        implementation: Implementation::Sboljs,
        parse_format: RdfFormat::RdfXml,
        serialize_format: RdfFormat::RdfXml,
    },
];

// 20 warmup iterations is enough for the JVM's tiered JIT to reach
// steady state on a loop this small; 100 measured iterations gives
// stable p50s and acceptable p99s for every implementation in the
// matrix. The numbers in benches/cross-impl/README.md are captured at
// these defaults.
pub(crate) const DEFAULT_WARMUP: usize = 20;
pub(crate) const DEFAULT_ITERS: usize = 100;
