![sbol-rs: a Rust implementation of SBOL](docs/images/sbol-rs.png)

`sbol-rs` is a Rust implementation of the Synthetic Biology Open Language
(SBOL 3.1.0): a typed API for reading, building, and rewriting SBOL
documents, with offline validation against all 109 machine-checkable
SBOL 3.1.0 rules.

New to the codebase? Start with the [**crate guide**](docs/crate-guide.md).

## Installation

Add the library to your `Cargo.toml`:

```toml
[dependencies]
sbol = "0.1"
```

Or with `cargo add`:

```sh
cargo add sbol
```

The CLI ships as a separate crate. `cargo install sbol-cli` installs a
binary named `sbol`:

```sh
cargo install sbol-cli
sbol validate design.ttl
```

## Example

```rust
use sbol::constants::{EDAM_IUPAC_DNA, SBO_DNA, SO_PROMOTER};
use sbol::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let namespace = "https://example.org/lab";

    let sequence = Sequence::builder(namespace, "j23119_seq")?
        .elements("ttgacagctagctcagtcctaggtataatgctagc")
        .encoding(EDAM_IUPAC_DNA)
        .build()?;

    let component = Component::builder(namespace, "j23119")?
        .types([SBO_DNA])
        .add_component_role(SO_PROMOTER)
        .add_sequence(sequence.identity.clone())
        .name("J23119 constitutive promoter")
        .build()?;

    let document = Document::from_objects(vec![
        SbolObject::Component(component),
        SbolObject::Sequence(sequence),
    ])?;

    document.check()?;
    println!("{}", document.write_turtle()?);
    Ok(())
}
```

Reading documents, traversing references across documents, expanding
combinatorial derivations, and inspecting validation reports are covered
in [`crates/sbol/examples/`](crates/sbol/examples/). Run any of them with
`cargo run -p sbol --example <name>`.

## Validation

The SBOL 3.1.0 specification defines 109 machine-checkable validation
rules in [Appendix B](spec/SBOL3.1.0.md#b-validation-rules). `sbol`
checks all of them offline and deterministically, with per-rule overrides
and text / JSON / SARIF output. See [`docs/validation.md`](docs/validation.md)
for the full overview and [`docs/conformance.md`](docs/conformance.md)
for the per-rule status grid.

## Ontology Extensions

EDAM, SBO, SO, GO, ChEBI, and Cell Ontology ship bundled. NCIT and
lab-specific ontologies install on demand into a local cache; see
[`docs/ontology-extensions.md`](docs/ontology-extensions.md).

## Performance

Round-trip cost (`parse → serialize` in the same format) on
`toggle_switch_v2.ttl` (~30 KB), median microseconds across 100
measured iterations (20 warmup); lower is better. Every
implementation runs in its own pinned Docker image so the rows are
apples-to-apples. Rows sorted by `rdfxml` p50 ascending; fastest first:

| Impl              | turtle | rdfxml | jsonld | ntriples |
| ----------------- | -----: | -----: | -----: | -------: |
| sbol-rs           |    352 |    368 |    750 |      393 |
| libSBOLj3 1.0.5.2 |  1,976 |  2,264 |  4,418 |    2,096 |
| sboljs 3.0.2      |    n/a |  2,543 |    n/a |      n/a |
| pySBOL3 1.2       |  7,566 |  9,864 |  6,489 |    7,234 |

Apple M4 Max (16 cores), 128 GB RAM, macOS 26.3.1, Docker Desktop
29.0.1. sboljs's underlying `rdfoo` only emits RDF/XML and its parser
stack is too fragile to reach the other format rows on real SBOL 3
fixtures; the [bench README](benches/cross-impl/README.md) documents
the specific failure modes. The
[`crates/sbol-bench`](crates/sbol-bench) crate runs the comparison
end-to-end; see [`benches/cross-impl/README.md`](benches/cross-impl/README.md)
for results across smaller fixtures, full methodology, and per-row
caveats.
