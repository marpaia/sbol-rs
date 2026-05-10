#!/usr/bin/env python3
"""Round-trip benchmark wrapper for pySBOL3.

Reads an SBOL 3 RDF document, runs the configured number of warmup and
timed (parse + serialize) iterations, and writes per-iteration nanosecond
timings to a JSON file. The output-file convention (rather than stdout)
keeps the timings clean even when the underlying library decides to log
to stdout for any reason.

Invoked from inside the Docker container built by the sibling Dockerfile.

Usage:
    python3 bench.py <input> <parse_fmt> <serialize_fmt> <warmup> <iters> <output_json>
"""

from __future__ import annotations

import io
import json
import sys
import time
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path

import sbol3


# pySBOL3 expresses formats as rdflib serializer names. These keys are
# the canonical strings the driver passes; they match the labels
# emitted by the other language benches.
FORMAT_TO_RDFLIB = {
    "turtle": "turtle",
    "rdfxml": "xml",
    "jsonld": "json-ld",
    "ntriples": "nt",
}


def _pysbol3_version() -> str:
    try:
        return version("sbol3")
    except PackageNotFoundError:
        return "unknown"


def _parse_once(rdf_text: str, rdflib_format: str) -> sbol3.Document:
    doc = sbol3.Document()
    doc.read_string(rdf_text, file_format=rdflib_format)
    return doc


def main(argv: list[str]) -> int:
    if len(argv) < 7:
        print(
            "usage: bench.py <input> <parse_fmt> <serialize_fmt> <warmup> <iters> <output_json>",
            file=sys.stderr,
        )
        return 2

    input_path = Path(argv[1])
    parse_fmt = argv[2]
    serialize_fmt = argv[3]
    warmup = int(argv[4])
    iters = int(argv[5])
    output_json = Path(argv[6])

    parse_rdflib = FORMAT_TO_RDFLIB.get(parse_fmt)
    serialize_rdflib = FORMAT_TO_RDFLIB.get(serialize_fmt)
    if parse_rdflib is None:
        print(f"unknown parse format: {parse_fmt}", file=sys.stderr)
        return 2
    if serialize_rdflib is None:
        print(f"unknown serialize format: {serialize_fmt}", file=sys.stderr)
        return 2

    rdf_text = input_path.read_text(encoding="utf-8")

    parse_ns: list[int] = []
    serialize_ns: list[int] = []
    last_bytes = 0

    for _ in range(warmup):
        doc = _parse_once(rdf_text, parse_rdflib)
        doc.write_string(file_format=serialize_rdflib)

    for _ in range(iters):
        t0 = time.perf_counter_ns()
        doc = _parse_once(rdf_text, parse_rdflib)
        t1 = time.perf_counter_ns()
        serialized = doc.write_string(file_format=serialize_rdflib)
        t2 = time.perf_counter_ns()
        parse_ns.append(t1 - t0)
        serialize_ns.append(t2 - t1)
        last_bytes = len(serialized.encode("utf-8") if isinstance(serialized, str) else serialized)

    output_json.write_text(
        json.dumps(
            {
                "impl": "pysbol3",
                "version": _pysbol3_version(),
                "fixture": str(input_path),
                "parse_format": parse_fmt,
                "serialize_format": serialize_fmt,
                "warmup_iters": warmup,
                "measured_iters": iters,
                "serialized_bytes": last_bytes,
                "parse_ns": parse_ns,
                "serialize_ns": serialize_ns,
            }
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
