// The domain types moved to the `ejas-core` crate so the WebAssembly UI can
// share them without pulling in Z3. Re-exported here so `crate::*` keeps
// resolving throughout the solver.
pub use ejas_core::structs;

pub mod anonymize;
pub mod enums;
