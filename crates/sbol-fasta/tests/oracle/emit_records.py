"""Emit a normalized record table for a FASTA file using BioPython.

The output is the external oracle the Rust test compares `sbol-fasta`
against: for every record, the identifier (first whitespace-delimited
header token, which is BioPython's `record.id`) and the sequence in
uppercase. Case is normalized because the importer lowercases nucleotide
elements and uppercases protein elements, whereas BioPython preserves
the source case; uppercasing both sides compares the residues alone.
"""

import json
import sys

from Bio import SeqIO


def main():
    path = sys.argv[1]
    records = []
    for record in SeqIO.parse(path, "fasta"):
        records.append({"id": record.id, "seq": str(record.seq).upper()})
    json.dump({"records": records}, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
