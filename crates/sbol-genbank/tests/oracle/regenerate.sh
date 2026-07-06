#!/usr/bin/env bash
# Regenerate the committed BioPython feature-table oracle for the GenBank
# importer. Builds the pinned BioPython image, parses each fixture, and
# writes the normalized feature table to expected/{name}.json.
#
# Run from anywhere; paths resolve against the repository root. Requires
# Docker. Commit the refreshed expected/*.json alongside any fixture or
# script change.
set -euo pipefail

ORACLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$ORACLE_DIR/../../../.." && pwd)"
EXPECTED_DIR="$ORACLE_DIR/expected"
IMAGE="sbol-genbank-oracle:biopython-1.85"
SCRIPT="crates/sbol-genbank/tests/oracle/emit_features.py"

# fixture-name -> path (relative to repo root) : format
FIXTURES=(
  "BBa_E0040:tests/fixtures/genbank/BBa_E0040.gb:genbank"
  "BBa_R0010:tests/fixtures/genbank/BBa_R0010.gb:genbank"
  "BBa_B0034:tests/fixtures/genbank/BBa_B0034.gb:genbank"
  "BBa_F2620:tests/fixtures/genbank/BBa_F2620.gb:genbank"
  "pUC19:tests/fixtures/genbank/pUC19.gbk:genbank"
  "oracle_join:crates/sbol-genbank/tests/fixtures/multispan/oracle_join.gb:genbank"
)

echo "Building $IMAGE ..."
docker build -t "$IMAGE" "$ORACLE_DIR"

mkdir -p "$EXPECTED_DIR"
for entry in "${FIXTURES[@]}"; do
  name="${entry%%:*}"
  rest="${entry#*:}"
  path="${rest%%:*}"
  fmt="${rest#*:}"
  echo "Parsing $path ($fmt) -> expected/$name.json"
  docker run --rm -v "$REPO_ROOT":/work:ro -w /work "$IMAGE" \
    python "$SCRIPT" "$path" "$fmt" > "$EXPECTED_DIR/$name.json"
done

echo "Done. Committed oracle refreshed under $EXPECTED_DIR"
