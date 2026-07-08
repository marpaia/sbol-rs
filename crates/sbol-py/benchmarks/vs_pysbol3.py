"""Benchmark the idiomatic `sbol` bindings against pySBOL3 on the operations
that dominate real workloads: end-to-end build+serialize, parse, and validate.

Both libraries build the *same logical document* (N DNA promoter Components, each
with a Sequence) and are timed on identical inputs where a shared artifact makes
the comparison apples-to-apples (parse reads the exact same N-Triples string).

Run inside a venv that has both installed:
    pip install sbol3
    maturin develop -m crates/sbol3-py/Cargo.toml
    python crates/sbol3-py/benchmarks/vs_pysbol3.py
"""

import time

import sbol3  # pySBOL3
import sbol  # ours

NS = "https://example.org/lab"
ELEMENTS = "acgtacgtac" * 5  # 50 bp


def best(fn, reps=3):
    times = []
    for _ in range(reps):
        start = time.perf_counter()
        result = fn()
        times.append(time.perf_counter() - start)
    return min(times), result


# ---------------------------------------------------------------- pySBOL3
def build_py(n):
    sbol3.set_namespace(NS)
    doc = sbol3.Document()
    objects = []
    for i in range(n):
        c = sbol3.Component(f"part{i}", sbol3.SBO_DNA, roles=[sbol3.SO_PROMOTER])
        s = sbol3.Sequence(
            f"part{i}_seq", elements=ELEMENTS, encoding=sbol3.IUPAC_DNA_ENCODING
        )
        c.sequences = [s.identity]
        objects += [c, s]
    doc.add(objects)
    return doc


def build_and_write_py(n):
    return build_py(n).write_string(sbol3.NTRIPLES)


def read_py(data):
    doc = sbol3.Document()
    doc.read_string(data, sbol3.NTRIPLES)
    return doc


# ---------------------------------------------------------------- sbol (ours)
def build_rs(n):
    d = sbol.Design(NS)
    for i in range(n):
        d.promoter(f"part{i}", ELEMENTS)
    return d.finish()


def build_and_write_rs(n):
    return build_rs(n).to_string(sbol.RdfFormat.NTriples)


def read_rs(data):
    return sbol.Document.read_str(data, sbol.RdfFormat.NTriples)


# ---------------------------------------------------------------- harness
def row(label, t_py, t_rs):
    speedup = t_py / t_rs if t_rs else float("inf")
    print(f"{label:<28} pySBOL3 {t_py*1000:9.1f} ms   sbol {t_rs*1000:9.1f} ms   {speedup:6.1f}x")


def main():
    for n in (1000, 5000):
        print(f"\n=== N = {n} parts ({n} Components + {n} Sequences) ===")

        # Build + serialize end-to-end.
        t_py, data_py = best(lambda: build_and_write_py(n))
        t_rs, data_rs = best(lambda: build_and_write_rs(n))
        row("build + serialize (NT)", t_py, t_rs)
        print(f"    (bytes: pySBOL3 {len(data_py)}, sbol {len(data_rs)})")

        # Parse the SAME N-Triples produced by pySBOL3.
        t_py, _ = best(lambda: read_py(data_py))
        t_rs, _ = best(lambda: read_rs(data_py))
        row("parse (identical input)", t_py, t_rs)

        # Validate.
        doc_py = build_py(n)
        doc_rs = build_rs(n)
        t_py, _ = best(lambda: doc_py.validate())
        t_rs, _ = best(lambda: doc_rs.check())
        row("validate", t_py, t_rs)


if __name__ == "__main__":
    main()
