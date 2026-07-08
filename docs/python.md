# Python SDK (`sbol`)

`sbol` is the Python binding for sbol-rs, living in
[`crates/sbol-py`](../crates/sbol-py/). It is the single Python package that
covers **SBOL 2, SBOL 3, and lossless conversion between them**, backed by the
Rust core — so it is far faster than pySBOL3 (8–2300× on
parse/validate/serialize; see [Benchmark](#benchmark)) and does what no existing
Python library does: both versions in one install.

The API is idiomatic Python (keyword arguments, opaque handles), not a
re-creation of pySBOL3's mutable object graph. Full API reference and a usage
tour live in the crate's [README](../crates/sbol-py/README.md); the Rust API it
wraps is documented via rustdoc (`cargo doc --open`).

## Install / use

Not yet on PyPI. Build from source into a virtualenv with
[maturin](https://www.maturin.rs/):

```bash
python3 -m venv .venv && source .venv/bin/activate
pip install maturin
maturin develop -m crates/sbol-py/Cargo.toml
```

Then:

```python
from sbol import Design, Sbol2Document, RdfFormat

# Author SBOL 3
d = Design("https://example.org/lab")
plac = d.promoter("pLac", "caatacg", description="LacI-repressible")
tetr = d.cds("tetR", "atggtg")
d.engineered_region("pLac_tu", [plac, tetr])
doc = d.finish()
doc.check()

# Convert between versions
sbol2_text = doc.to_sbol2(RdfFormat.Turtle)          # SBOL 3 -> SBOL 2
doc3 = Sbol2Document.read_path("legacy.xml").to_sbol3()  # SBOL 2 -> SBOL 3
```

See the crate README for the full surface: the raw `component()`/`sequence()`
path, `compute_sequences()`, `expand_derivations()`, GenBank/FASTA I/O,
`from_document()`, constants, and `SbolError`.

## Develop

**Layout.** The crate is a PyO3 `cdylib` named `sbol`. It is deliberately
**excluded from the Cargo workspace** (`exclude` in the root `Cargo.toml`, like
`fuzz`): a PyO3 extension module links against the Python runtime, so building
it through `cargo build --workspace` would fail to link. Keeping it out means
`cargo test --workspace` stays green and the Python module builds only through
maturin. The trade-off is that the binding is covered by the Python test suite,
not by `cargo test`.

**Build.** `maturin develop -m crates/sbol-py/Cargo.toml` compiles the crate and
installs the module into the active virtualenv. The first build downloads
`pyo3`, so it needs network access.

**Test.** The suite uses the standard-library `unittest` (no extra
dependencies):

```bash
python -m unittest discover -s crates/sbol-py/tests
```

**Benchmark.** [`benchmarks/vs_pysbol3.py`](../crates/sbol-py/benchmarks/vs_pysbol3.py)
times `sbol` against pySBOL3 on build+serialize, parse, and validate. It needs
pySBOL3 installed (`pip install sbol3`) and a release build
(`maturin develop --release -m crates/sbol-py/Cargo.toml`).

<a name="benchmark"></a>
Representative results (release, arm64 macOS, min-of-3, byte-identical output):

| Operation | N=1000 | N=5000 |
|---|---|---|
| build + serialize | 487× | 2349× |
| parse | 18× | 19× |
| validate | 23× | 8× |

The build+serialize ratio is amplified by pySBOL3's ~quadratic serializer;
parse is the cleanest apples-to-apples (~19×). Validate is the narrowest axis
(the cross-reference resolver's scaling is the standing optimization target),
still ~8× at N=5000.

**Extending the binding.** All of it is in
[`crates/sbol-py/src/lib.rs`](../crates/sbol-py/src/lib.rs). The pattern:
`#[pyclass]` wrappers (`Design`, `Document`, `Sbol2Document`) delegate to the
Rust crates; verbs collapse the Rust builder chains into single methods with
keyword arguments and return opaque handle classes (`ComponentId`,
`SequenceId`); every Rust error maps to the `SbolError` Python exception; new
classes, functions, and constants are registered in the `#[pymodule] fn sbol`.
It wraps `sbol3` (model, `Design` arena, validation), `sbol2` (SBOL 2 model),
`sbol-convert` (SBOL 2⇄3), `sbol-utilities` (verbs, `compute_sequence`,
combinatorial expansion), and `sbol-genbank` / `sbol-fasta`.

**Packaging.** Distribution as an `abi3` wheel with cross-platform CI and a PyPI
release is not yet set up; `maturin build --release` produces a local wheel in
the meantime.
