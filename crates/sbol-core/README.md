# sbol-core

Version-neutral SBOL machinery shared by the [`sbol2`] and [`sbol3`] crates.

`sbol-core` holds the parts of an SBOL implementation that are independent of a
specific data-model version:

- the field-metadata descriptor types (`FieldDescriptor`, `ClassDescriptor`,
  `Cardinality`, `ValueKind`, `ReferenceSpec`) that drive schema-directed
  serialization and validation from a single source of truth;
- the validation framework (configuration, per-rule severity, coverage
  reporting, and the shared rule-status model);
- identity and IRI utilities; and
- the object and document scaffolding a versioned model builds on.

The `sbol2` and `sbol3` crates layer their own class and rule catalogs on top of
these primitives. Most users depend on the umbrella [`sbol`] crate rather than
on `sbol-core` directly.

## License

Licensed under either of MIT or Apache-2.0 at your option.
