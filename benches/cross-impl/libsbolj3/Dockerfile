# libSBOLj3 round-trip benchmark image. Pinned to the same libSBOLj3
# release as the conformance harness in tests/fixtures/cross-impl/ so
# the parse/serialize numbers we publish are tied to a known library
# version.
#
# Build:
#     docker build -t libsbolj3-bench benches/cross-impl/libsbolj3/
# Used by:
#     crates/sbol/src/bin/run-cross-impl-bench.rs
#
# IMPORTANT — bump LIBSBOLJ3_VERSION when a new release ships. The
# library version (1.0.x) does NOT track the SBOL spec version (3.x);
# 1.0.5.2 implements SBOL 3.1.0 at the time of writing. After bumping,
# rerun the bench driver and re-record any baseline numbers captured
# in benches/cross-impl/README.md.

FROM eclipse-temurin:17-jdk AS runtime

ARG LIBSBOLJ3_VERSION=1.0.5.2
ARG LIBSBOLJ3_JAR_URL=https://github.com/SynBioDex/libSBOLj3/releases/download/${LIBSBOLJ3_VERSION}/libsbolj3-${LIBSBOLJ3_VERSION}-withDependencies.jar

WORKDIR /opt/libsbolj3-bench

ADD ${LIBSBOLJ3_JAR_URL} /opt/libsbolj3-bench/libSBOLj3.jar

COPY Bench.java /opt/libsbolj3-bench/Bench.java

RUN javac -cp /opt/libsbolj3-bench/libSBOLj3.jar /opt/libsbolj3-bench/Bench.java

# Override the JVM defaults so the bench is reproducible. -Xshare:auto
# lets the JVM use the class-data archive when available.
ENTRYPOINT ["java", "-cp", "/opt/libsbolj3-bench/libSBOLj3.jar:/opt/libsbolj3-bench", "Bench"]
