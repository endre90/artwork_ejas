//! Keeps the embedded-frontend directory present and the embed fresh.
//!
//! Two problems this solves:
//!
//!  1. `rust-embed` fails to compile if its folder does not exist, so a fresh
//!     clone could not build the server until someone had run `trunk build`.
//!     The server is useful on its own (the desktop UI talks to it), so it has
//!     to build with an empty bundle.
//!  2. Cargo does not watch that folder by itself, so rebuilding the frontend
//!     would leave the server serving a stale copy until something else forced
//!     a recompile.

use std::path::Path;

fn main() {
    let dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("dist");
    if let Err(e) = std::fs::create_dir_all(&dist) {
        panic!("could not create {}: {e}", dist.display());
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", dist.display());
}
