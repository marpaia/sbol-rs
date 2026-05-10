# sboljs3 round-trip benchmark image. Pinned to a specific sboljs
# release so timing comparisons stay reproducible across contributor
# machines. sboljs (the npm package) v3.x is the SBOL 3 successor to
# the v2.x line; the upstream repository name is `sboljs3`.
#
# Build:
#     docker build -t sboljs3-bench benches/cross-impl/sboljs3/
# Used by:
#     crates/sbol/src/bin/run-cross-impl-bench.rs
#
# IMPORTANT — bump SBOLJS_VERSION when a new release ships. After
# bumping, rerun the bench driver and re-record any baseline numbers
# captured in benches/cross-impl/README.md.
#
# Format support: sboljs3's underlying rdfoo only parses N-Triples and
# RDF/XML, and only serializes RDF/XML. The driver therefore skips
# (sboljs3 x turtle) and (sboljs3 x jsonld) combinations rather than
# pretending they exist.

FROM node:20-bookworm-slim

ARG SBOLJS_VERSION=3.0.2

WORKDIR /opt/sboljs3

COPY package.json /opt/sboljs3/package.json
RUN sed -i "s/__SBOLJS_VERSION__/${SBOLJS_VERSION}/" package.json \
 && npm install --omit=dev --no-audit --no-fund --loglevel=error

COPY bench.js /opt/sboljs3/bench.js

ENTRYPOINT ["node", "/opt/sboljs3/bench.js"]
