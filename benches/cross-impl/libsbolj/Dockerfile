# libSBOLj (SBOL 2) round-trip image. Pinned to a specific libSBOLj
# release so regenerated SBOL 2 reference outputs are reproducible
# across contributor machines.
#
# Build: docker build -t libsbolj-pinned benches/cross-impl/libsbolj/
# Used by: crates/sbol2/src/bin/regenerate-cross-impl-sbol2-expectations.rs
#
# IMPORTANT — bump LIBSBOLJ_VERSION when a new release ships. libSBOLj
# 2.4.0 implements SBOL 2.3.0. CI does not run this image; only the
# regenerate binary on contributor machines does. After bumping, run
# the regenerate binary and commit the resulting *.expected.rdf files.

# JDK base (not JRE) so `javac` is available for the RoundTrip wrapper.
# The compiled image is only used locally by the regenerate binary, so
# the size cost over a JRE-only multi-stage build is acceptable.
FROM eclipse-temurin:17-jdk AS runtime

ARG LIBSBOLJ_VERSION=2.4.0
ARG LIBSBOLJ_JAR_URL=https://github.com/SynBioDex/libSBOLj/releases/download/v${LIBSBOLJ_VERSION}/libSBOLj-${LIBSBOLJ_VERSION}-withDependencies.jar

WORKDIR /opt/libsbolj

# Fetch the pinned fat jar from the GitHub release.
ADD ${LIBSBOLJ_JAR_URL} /opt/libsbolj/libSBOLj.jar

COPY RoundTrip.java /opt/libsbolj/RoundTrip.java

# Compile the round-trip wrapper against the pinned jar. The runtime
# image then ships both the jar and the compiled class.
RUN javac -cp /opt/libsbolj/libSBOLj.jar /opt/libsbolj/RoundTrip.java

ENTRYPOINT ["java", "-cp", "/opt/libsbolj/libSBOLj.jar:/opt/libsbolj", "RoundTrip"]
