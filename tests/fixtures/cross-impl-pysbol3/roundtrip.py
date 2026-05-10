#!/usr/bin/env python3
"""Round-trip wrapper for the pySBOL3 cross-implementation conformance harness.

Reads an SBOL 3 Turtle file via pySBOL3 and writes the parsed document back
out in the requested output format. Output goes to stdout.

Invoked from inside the Docker container built by ``Dockerfile``. pySBOL3
may emit warnings to stderr during parsing; those are filtered by the
regenerate binary, and stdout carries only the serialized output.

Usage::

    python3 roundtrip.py <input.ttl> <format>

Where ``format`` is one of ``turtle``, ``rdfxml``, ``jsonld``, ``ntriples``.
"""

from __future__ import annotations

import sys
from pathlib import Path

import sbol3


# pySBOL3 expresses output formats as RDFlib serializer names. These are
# the canonical strings accepted by ``sbol3.Document.write_string``.
FORMAT_TO_RDFLIB = {
    "turtle": "turtle",
    "rdfxml": "xml",
    "jsonld": "json-ld",
    "ntriples": "nt",
}


def main(argv: list[str]) -> int:
    if len(argv) < 3:
        print("usage: roundtrip.py <input.ttl> <format>", file=sys.stderr)
        print("formats: turtle, rdfxml, jsonld, ntriples", file=sys.stderr)
        return 2

    input_path = Path(argv[1])
    requested_format = argv[2].lower()
    rdflib_format = FORMAT_TO_RDFLIB.get(requested_format)
    if rdflib_format is None:
        print(f"unknown format: {requested_format}", file=sys.stderr)
        return 2

    doc = sbol3.Document()
    doc.read(str(input_path))
    serialized = doc.write_string(file_format=rdflib_format)
    sys.stdout.write(serialized)
    if not serialized.endswith("\n"):
        sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
