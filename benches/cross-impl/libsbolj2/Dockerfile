# libSBOLj (SBOL 2) round-trip benchmark image. Pinned to the same
# libSBOLj release as the SBOL 2 conformance harness
# (benches/cross-impl/libsbolj/, tests/fixtures/cross-impl-sbol2/) so
# the parse/serialize numbers we publish are tied to a known library
# version. libSBOLj 2.4.0 implements SBOL 2.3.0.
#
# Build:
#     docker build -t libsbolj2-bench benches/cross-impl/libsbolj2/
# Used by:
#     crates/sbol-bench
#
# IMPORTANT — bump LIBSBOLJ_VERSION when a new release ships and keep it
# in step with benches/cross-impl/libsbolj/. After bumping, rerun the
# bench and re-record the SBOL 2 numbers in benches/cross-impl/README.md.

FROM eclipse-temurin:17-jdk AS runtime

ARG LIBSBOLJ_VERSION=2.4.0
ARG LIBSBOLJ_JAR_URL=https://github.com/SynBioDex/libSBOLj/releases/download/v${LIBSBOLJ_VERSION}/libSBOLj-${LIBSBOLJ_VERSION}-withDependencies.jar

WORKDIR /opt/libsbolj2-bench

ADD ${LIBSBOLJ_JAR_URL} /opt/libsbolj2-bench/libSBOLj.jar

COPY Bench.java /opt/libsbolj2-bench/Bench.java

RUN javac -cp /opt/libsbolj2-bench/libSBOLj.jar /opt/libsbolj2-bench/Bench.java

ENTRYPOINT ["java", "-cp", "/opt/libsbolj2-bench/libSBOLj.jar:/opt/libsbolj2-bench", "Bench"]
