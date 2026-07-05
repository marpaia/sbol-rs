//! The version-neutral validation framework: coverage taxonomy, severity and
//! reporting types, and configuration shared by the SBOL 2 and SBOL 3
//! validators. Each version supplies its own rule catalog and rule
//! implementations on top of these primitives.

mod blocker;

pub use blocker::Blocker;
