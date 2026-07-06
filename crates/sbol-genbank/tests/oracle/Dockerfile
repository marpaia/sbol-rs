# Pinned BioPython environment used as the external feature-table oracle
# for the GenBank importer. BioPython is the reference GenBank parser in
# the bioinformatics community; comparing against it grounds the
# importer's location extraction in an independent implementation.
FROM python:3.12-slim

# --only-binary pins to the prebuilt wheel so the build needs no
# compiler and stays reproducible across platforms (incl. linux/arm64).
RUN pip install --no-cache-dir --only-binary=:all: biopython==1.85

WORKDIR /work
