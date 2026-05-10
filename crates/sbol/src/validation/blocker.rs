/// Classification of why a `Partial` or `Deferred` validation rule is
/// not yet fully implemented.
///
/// Provides a machine-queryable answer to "which rules need X?" without
/// reading prose notes. Recorded per rule in `rules.toml` and surfaced
/// in the generated `docs/conformance.md`.
///
/// `Implemented*` rules have `blocker == None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Blocker {
    /// Requires deeper ontology coverage than the bundled vocab tables
    /// expose (e.g. recursive SO term subsumption checks).
    Ontology,
    /// Requires resolving external documents or content URIs to verify
    /// the rule (e.g. attachment hash matching the referenced bytes).
    Resolver,
    /// Local check is done at a value-kind level, but the strict XSD
    /// datatype validation per Table 23 is not yet wired up.
    StrictDatatype,
    /// Implementation requires a policy decision that has not been
    /// taken (e.g. conflict resolution strategy for derived
    /// implementations).
    Policy,
    /// Verification requires information outside the document — typically
    /// global URI uniqueness or registry semantics that are beyond the
    /// scope of a per-document validator.
    External,
}
