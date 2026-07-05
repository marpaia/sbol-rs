#!/usr/bin/env bash
# Regenerate the committed BioPython record-table oracle for the FASTA
# importer. Builds the pinned BioPython image, parses each fixture, and
# writes the normalized record table to expected/{name}.json.
#
# Run from anywhere; paths resolve against the repository root. Requires
# Docker. Commit the refreshed expected/*.json alongside any fixture or
# script change.
set -euo pipefail

ORACLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$ORACLE_DIR/../../../.." && pwd)"
EXPECTED_DIR="$ORACLE_DIR/expected"
IMAGE="sbol-fasta-oracle:biopython-1.85"
SCRIPT="crates/sbol-fasta/tests/oracle/emit_records.py"

# fixture-name -> path (relative to repo root)
FIXTURES=(
  "pUC19:tests/fixtures/fasta/pUC19.fasta"
  "pBR322:tests/fixtures/fasta/pBR322.fasta"
  "GFP_protein:tests/fixtures/fasta/GFP_protein.fasta"
  "multi_protein:tests/fixtures/fasta/multi_protein.fasta"
)

echo "Building $IMAGE ..."
docker build -t "$IMAGE" "$ORACLE_DIR"

mkdir -p "$EXPECTED_DIR"
for entry in "${FIXTURES[@]}"; do
  name="${entry%%:*}"
  path="${entry#*:}"
  echo "Parsing $path -> expected/$name.json"
  docker run --rm -v "$REPO_ROOT":/work:ro -w /work "$IMAGE" \
    python "$SCRIPT" "$path" > "$EXPECTED_DIR/$name.json"
done

echo "Done. Committed oracle refreshed under $EXPECTED_DIR"
