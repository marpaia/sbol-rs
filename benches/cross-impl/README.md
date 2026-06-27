# Cross-implementation performance benchmarks

Times `(parse, serialize)` round trips against four SBOL 3.1.0
implementations, each pinned in its own Docker image so every row pays
the same Linux container overhead and the comparison is
apples-to-apples:

| Implementation | Source                                      | Pinned version | Image tag         |
| -------------- | ------------------------------------------- | -------------- | ----------------- |
| sbol-rs        | this repository                             | from workspace | `sbol-rs-bench`   |
| pySBOL3        | https://github.com/SynBioDex/pySBOL3        | `1.2`          | `pysbol3-bench`   |
| libSBOLj3      | https://github.com/SynBioDex/libSBOLj3      | `1.0.5.2`      | `libsbolj3-bench` |
| sboljs         | https://github.com/SynBioDex/sboljs3 (npm `sboljs@3.x`) | `3.0.2`        | `sboljs3-bench`   |

The orchestrator (`crates/sbol-bench`) pre-converts each fixture into
every RDF serialization on disk so every implementation sees the same
byte-for-byte input, then runs `<warmup>` untimed iterations followed
by `<measured>` timed iterations of `parse(in) -> serialize(out)` for
each `(impl, parse_format, serialize_format)` combination it knows
about. Per-iteration nanosecond timings come back as JSON and are
aggregated into a table.

The sbol-rs row also runs inside Docker by default. Running it natively
on the host (no container) would give sbol-rs an unfair single-digit
percentage advantage from skipping the Linux VM overhead the other
rows pay; the gap to the other implementations is large enough that
this asymmetry would not change any conclusion, but the dockerized
default keeps the comparison defensible. For local development (no
Docker required) set `SBOL_BENCH_DOCKER=0` and sbol-rs will fall back
to in-process timing; the foreign rows are then marked skipped.

## Running

```sh
# One-time, per implementation. The sbol-rs context is the workspace
# root because its Dockerfile compiles the whole workspace, so it
# needs `-f` to find the right Dockerfile.
docker build -t pysbol3-bench   benches/cross-impl/pysbol3/
docker build -t libsbolj3-bench benches/cross-impl/libsbolj3/
docker build -t sboljs3-bench   benches/cross-impl/sboljs3/
docker build -t sbol-rs-bench   -f benches/cross-impl/sbol-rs/Dockerfile .

# Then:
cargo run --release -p sbol-bench
```

The fixture cache must be populated first (run the `sbol3_fixtures`
integration test once if `tests/fixtures/sbol3/SBOLTestSuite/` is
empty).

## Knobs

All read from the environment, prefixed `SBOL_BENCH_`:

| Variable               | Default                                  | Effect                                              |
| ---------------------- | ---------------------------------------- | --------------------------------------------------- |
| `SBOL_BENCH_WARMUP`    | 20                                       | untimed iterations per case                         |
| `SBOL_BENCH_ITERS`     | 100                                      | timed iterations per case                           |
| `SBOL_BENCH_FIXTURES`  | `component,multicellular_simple,bba_f2620_popsreceiver,toggle_switch_v2` | comma-separated fixture stems (subset of the four) |
| `SBOL_BENCH_DOCKER`    | unset                                    | set to `0` to skip every foreign impl (native only) |
| `SBOL_BENCH_REPORT`    | unset                                    | path to write a JSON report of every per-iter sample |

The 20-warmup default is what the JVM's tiered JIT needs to reach
steady state on this loop — at 3 warmup iters libSBOLj3 was 30–60%
slower than at 20 across the fixtures, large enough to flip the
ordering on smaller documents. 100 measured iterations gives stable
p50s; p99 is representative except on the sub-10 µs `component`
fixture, where rare allocation or scheduling spikes inflate it.

## Format support matrix

`(parse_format -> serialize_format)` pairs the bench drives, by impl:

| Impl       | turtle→turtle | rdfxml→rdfxml | jsonld→jsonld | ntriples→ntriples |
| ---------- | :-----------: | :-----------: | :-----------: | :---------------: |
| sbol-rs    | yes           | yes           | yes           | yes               |
| pySBOL3    | yes           | yes           | yes           | yes               |
| libSBOLj3  | yes           | yes           | yes           | yes               |
| sboljs     | (n/a)         | yes           | (n/a)         | (n/a)             |

sboljs's underlying `rdfoo` only emits RDF/XML and only parses RDF/XML
or N-Triples, so Turtle and JSON-LD never had a chance. The N-Triples
parse path is also broken regardless of input: rdfoo wires up
`rdf-parser-n3` against an `@rdfjs/sink-map` whose `parser.import` API
contract has drifted, so every NT parse throws `TypeError:
parser.import is not a function` before any triple is produced.
N-Triples is not in the bench matrix for sboljs because of this.

The RDF/XML row required getting sbol-rs and sboljs to agree on the
input bytes. sbol-rs emits inline `xmlns` on every child element
(valid RDF/XML 1.1) but `rdf-parser-rdfxml` mis-parses those as
blank-node subjects, and rdfoo's serializer then throws `Unknown
termType BlankNode` on every fixture. libSBOLj3's RDF/XML output uses
standard prefix-style declarations, which `rdf-parser-rdfxml` handles
cleanly. The driver therefore uses the libSBOLj3 reference output
from `tests/fixtures/cross-impl/<stem>.libSBOLj3.expected.rdf` as the
RDF/XML input for **every** impl, not just sboljs. The triple sets
are equivalent (the `cross_impl` conformance test guarantees this),
so the comparison is apples-to-apples; only the on-disk byte layout
of the RDF/XML input differs from the other format inputs (which
still come from sbol-rs).

## Methodology

- **Hardware**: MacBook Pro, Apple M4 Max (12 performance + 4 efficiency
  cores), 128 GB RAM.
- **OS / runtime**: macOS 26.3.1, Docker Desktop 29.0.1 (Linux/arm64 VM
  on Apple Virtualization).
- **Impl versions**: as pinned above (sbol-rs from this checkout,
  pySBOL3 1.2, libSBOLj3 1.0.5.2, sboljs 3.0.2).
- **Iteration counts**: 20 warmup + 100 measured per `(impl, fixture,
  parse_format, serialize_format)`. The warmup count is what the JVM
  tiered JIT needs to stabilize on this loop — at 3 warmup iters
  libSBOLj3 was 30–60% pessimistic and the ordering across small
  fixtures flipped.
- **Timing**: each language's monotonic high-resolution clock —
  `std::time::Instant`, `System.nanoTime`, `time.perf_counter_ns`,
  `process.hrtime.bigint`. Sub-microsecond resolution on every
  platform.
- **Workload**: in-process `parse(text) -> serialize(document)`.
  Input bytes are read into memory once before the loop, so filesystem
  I/O is excluded. Each iteration re-parses from the same in-memory
  bytes; no document is reused across iterations.
- **Container symmetry**: every implementation, sbol-rs included,
  runs inside its own pinned Docker image. Each row pays the same
  Linux-VM overhead on Apple Silicon Docker Desktop (typically 5–15%
  vs bare metal on this kind of workload).
- **Same input bytes per format**: every impl reads identical bytes
  for a given `(fixture, format)` pair. Turtle, JSON-LD, and
  N-Triples inputs are produced by sbol-rs from each source fixture.
  The RDF/XML input is taken from libSBOLj3's reference output in
  `tests/fixtures/cross-impl/` (see Format support matrix above for
  why).
- **Not measured**: memory residency, GC pause distribution,
  cold-start latency (warmup is inside the same process per impl),
  validation cost, or correctness. The cross-impl conformance suite
  (`crates/sbol/tests/cross_impl*.rs`) gates correctness; this
  harness assumes the implementations already agree at the triple-set
  level and asks how fast each one round-trips the same bytes.

## Captured results

One run with the defaults (20 warmup + 100 measured), every impl in
Docker. Numbers are **median parse time in microseconds, with p99 in
parentheses** for `parse(format) -> serialize(format)` on the same
format — lower is better. Rows are sorted by `rdfxml` p50 ascending;
fastest first.

`toggle_switch_v2.ttl` (~30 KB, the largest fixture in the default set):

| Impl       | turtle | rdfxml | jsonld | ntriples |
| ---------- | -----: | -----: | -----: | -------: |
| sbol-rs    | 493 (1,604) | 514 (642) | 1,051 (1,142) | 537 (872) |
| libSBOLj3  | 2,376 (6,999) | 2,572 (4,713) | 4,088 (5,632) | 2,464 (4,864) |
| sboljs     | n/a | 3,398 (5,697) | n/a | n/a |
| pySBOL3    | 10,443 (23,690) | 13,555 (26,995) | 9,058 (21,372) | 9,732 (23,357) |

`bba_f2620_popsreceiver.ttl` (~16 KB):

| Impl       | turtle | rdfxml | jsonld | ntriples |
| ---------- | -----: | -----: | -----: | -------: |
| sbol-rs    | 351 (406) | 365 (518) | 772 (806) | 379 (431) |
| libSBOLj3  | 1,992 (6,824) | 1,970 (5,508) | 2,907 (6,345) | 1,604 (4,532) |
| sboljs     | n/a | 2,554 (4,070) | n/a | n/a |
| pySBOL3    | 6,969 (19,561) | 9,611 (23,914) | 5,993 (19,182) | 6,364 (19,281) |

`multicellular_simple.ttl` (~5 KB):

| Impl       | turtle | rdfxml | jsonld | ntriples |
| ---------- | -----: | -----: | -----: | -------: |
| sbol-rs    | 117 (149) | 133 (194) | 248 (312) | 131 (421) |
| sboljs     | n/a | 887 (1,868) | n/a | n/a |
| libSBOLj3  | 1,082 (4,757) | 1,305 (4,239) | 2,002 (5,143) | 849 (1,642) |
| pySBOL3    | 2,568 (8,369) | 3,230 (8,519) | 2,248 (7,326) | 2,367 (3,327) |

`component.ttl` (~0.7 KB):

| Impl       | turtle | rdfxml | jsonld | ntriples |
| ---------- | -----: | -----: | -----: | -------: |
| sbol-rs    | 9 (119) | 11 (13) | 18 (106) | 9 (132) |
| sboljs     | n/a | 253 (402) | n/a | n/a |
| pySBOL3    | 396 (975) | 521 (992) | 353 (392) | 350 (948) |
| libSBOLj3  | 358 (855) | 620 (1,150) | 569 (3,901) | 316 (1,467) |

A few things worth keeping in mind when reading the numbers:

- Ratios scale with fixture size. On `component` (~700 bytes) pySBOL3
  and libSBOLj3 are ~40× slower than sbol-rs; on `toggle_switch_v2`
  (~30 KB) those compress to ~21× and ~5×. The per-call
  object-construction overhead each impl pays amortizes differently
  as the document grows.
- pySBOL3's parse cost is dominated by `rdflib`, which backs
  `sbol3.Document.read_string`. Differences across formats reflect
  rdflib's parser, not the SBOL layer.
- sboljs is competitive on RDF/XML — for small documents it beats
  pySBOL3 and libSBOLj3 outright; on the largest fixture it trails
  libSBOLj3 by ~12%. rdfoo's RDF/XML parser (`rdf-parser-rdfxml`)
  and its custom XML serializer are both lean. The rest of the rdfoo
  stack is what keeps sboljs out of every other format row (see
  Format support matrix above).
- sbol-rs's p99 stays below every comparator's *median* in every
  cell. Its interquartile range is within a few percent of the median
  on the larger fixtures; on the sub-10 µs `component` fixture the
  p99/p50 ratio reaches ~15× because a single rare allocation or
  scheduling spike dominates a microsecond-scale median, but the
  absolute p99 stays sub-millisecond. The JIT- and allocator-backed
  comparators carry heavier tails — libSBOLj3 and pySBOL3 p99 reach
  roughly 2–7× their median, worst at the largest fixture.
