# pySBOL3 round-trip benchmark image. Pinned to the same pySBOL3
# release as the conformance harness in tests/fixtures/cross-impl-pysbol3/
# so the parse/serialize numbers we publish are tied to a known
# library version.
#
# Build:
#     docker build -t pysbol3-bench benches/cross-impl/pysbol3/
# Used by:
#     crates/sbol/src/bin/run-cross-impl-bench.rs
#
# IMPORTANT — bump PYSBOL3_VERSION when a new release ships. After
# bumping, rerun the bench driver and re-record any baseline numbers
# captured in benches/cross-impl/README.md.

FROM python:3.12-slim AS runtime

ARG PYSBOL3_VERSION=1.2

WORKDIR /opt/pysbol3-bench

RUN pip install --no-cache-dir "sbol3==${PYSBOL3_VERSION}"

COPY bench.py /opt/pysbol3-bench/bench.py

ENTRYPOINT ["python3", "/opt/pysbol3-bench/bench.py"]
