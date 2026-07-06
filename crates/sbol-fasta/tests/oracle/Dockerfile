# Pinned BioPython environment used as the external record-table oracle
# for the FASTA importer. Comparing id + sequence against BioPython's
# SeqIO grounds the parser in an independent implementation.
FROM python:3.12-slim

# --only-binary pins to the prebuilt wheel so the build needs no
# compiler and stays reproducible across platforms (incl. linux/arm64).
RUN pip install --no-cache-dir --only-binary=:all: biopython==1.85

WORKDIR /work
