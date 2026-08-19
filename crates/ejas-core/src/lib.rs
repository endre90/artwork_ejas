//! Domain types and wire format shared by the solver, the server and the UI.
//!
//! This crate must stay free of native-only dependencies: it is compiled
//! into the WebAssembly frontend, where Z3 cannot follow.

pub mod api;
pub mod structs;
pub mod validate;

pub use api::*;
pub use structs::*;
