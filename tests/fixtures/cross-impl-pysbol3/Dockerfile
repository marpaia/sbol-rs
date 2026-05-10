# pySBOL3 round-trip image. Pinned to a specific pySBOL3 release so
# regenerated reference outputs are reproducible across contributor
# machines.
#
# Build: docker build -t pysbol3-pinned tests/fixtures/cross-impl-pysbol3/
# Used by: crates/sbol/src/bin/regenerate-cross-impl-pysbol3-expectations.rs
#
# IMPORTANT — bump PYSBOL3_VERSION when a new release ships. The
# library version tracks SBOL 3.x point releases; 1.2 implements SBOL
# 3.1.0 at the time of writing. CI does not run this image; only the
# regenerate binary on contributor machines does. After bumping, run
# the regenerate binary and commit the resulting *.expected.* files.

FROM python:3.12-slim AS runtime

ARG PYSBOL3_VERSION=1.2

WORKDIR /opt/pysbol3

RUN pip install --no-cache-dir "sbol3==${PYSBOL3_VERSION}"

COPY roundtrip.py /opt/pysbol3/roundtrip.py

ENTRYPOINT ["python3", "/opt/pysbol3/roundtrip.py"]
