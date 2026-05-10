# libSBOLj3 round-trip image. Pinned to a specific libSBOLj3 release so
# regenerated reference outputs are reproducible across contributor
# machines.
#
# Build: docker build -t libsbolj3-pinned tests/fixtures/cross-impl/
# Used by: crates/sbol/src/bin/regenerate-cross-impl-expectations.rs
#
# IMPORTANT — bump LIBSBOLJ3_VERSION when a new release ships. The
# library version (1.0.x) does NOT track the SBOL spec version (3.x);
# 1.0.5.2 implements SBOL 3.1.0 at the time of writing. CI does not
# run this image; only the regenerate binary on contributor machines
# does. After bumping, run the regenerate binary and commit the
# resulting *.expected.ttl files.

# JDK base (not JRE) so `javac` is available for the RoundTrip
# wrapper. The compiled image is only used locally by the regenerate
# binary, so the size cost over a JRE-only multi-stage build is
# acceptable.
FROM eclipse-temurin:17-jdk AS runtime

ARG LIBSBOLJ3_VERSION=1.0.5.2
ARG LIBSBOLJ3_JAR_URL=https://github.com/SynBioDex/libSBOLj3/releases/download/${LIBSBOLJ3_VERSION}/libsbolj3-${LIBSBOLJ3_VERSION}-withDependencies.jar

WORKDIR /opt/libsbolj3

# Fetch the pinned fat jar from the GitHub release. libSBOLj3 is not
# published to Maven Central as of 1.0.5.2; update this URL if a
# future release lands on Maven instead.
ADD ${LIBSBOLJ3_JAR_URL} /opt/libsbolj3/libSBOLj3.jar

COPY RoundTrip.java /opt/libsbolj3/RoundTrip.java

# Compile the round-trip wrapper against the pinned jar. The
# runtime image then ships both the jar and the compiled class.
RUN javac -cp /opt/libsbolj3/libSBOLj3.jar /opt/libsbolj3/RoundTrip.java

ENTRYPOINT ["java", "-cp", "/opt/libsbolj3/libSBOLj3.jar:/opt/libsbolj3", "RoundTrip"]
