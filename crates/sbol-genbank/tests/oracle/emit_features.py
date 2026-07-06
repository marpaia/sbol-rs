"""Emit a normalized feature table for a GenBank file using BioPython.

The output is the external oracle the Rust test compares `sbol-genbank`
against. For every non-`source` feature it records the GenBank key and
its location spans as 1-based-closed `[start, end, strand]` triples,
sorted so the comparison is independent of the order BioPython lists
compound-location parts (BioPython reverses part order for
reverse-strand compound locations; sorting cancels that out).

Strand is normalized to `1` (forward / SBOL inline) or `-1` (reverse /
SBOL reverseComplement); BioPython's `None` strand is treated as
forward, matching the importer's inline default.

Coordinates: BioPython stores 0-based half-open `[start, end)`; the
1-based-closed form is `start + 1 .. end`, the same convention the
importer emits.
"""

import json
import sys

from Bio import SeqIO


def normalize_strand(strand):
    return -1 if strand == -1 else 1


def feature_signature(feature):
    spans = []
    for part in feature.location.parts:
        start = int(part.start) + 1
        end = int(part.end)
        spans.append([start, end, normalize_strand(part.strand)])
    spans.sort()
    return {"key": feature.type, "spans": spans}


def main():
    path = sys.argv[1]
    fmt = sys.argv[2] if len(sys.argv) > 2 else "genbank"
    features = []
    for record in SeqIO.parse(path, fmt):
        for feature in record.features:
            if feature.type == "source":
                continue
            features.append(feature_signature(feature))
    json.dump({"features": features}, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
