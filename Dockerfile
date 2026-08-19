# syntax=docker/dockerfile:1

# Two things make this build minutes rather than an hour:
#
#  1. Z3 comes from the distro's PREBUILT libz3-dev package. The z3 crate is
#     declared without the `bundled` / `static-link-z3` feature, so it links
#     dynamically against the system library instead of compiling Z3 from
#     source. Nothing here ever builds Z3.
#  2. Dependencies are compiled in their own cached layer, so edits to this
#     project's own source do not rebuild the ~400 crates underneath it.

ARG RUST_VERSION=1.96

# ---------------------------------------------------------------- frontend --
# The egui UI compiled to WebAssembly. Z3 cannot go to wasm, which is exactly
# why the solver lives behind HTTP instead of in the browser.
FROM rust:${RUST_VERSION}-trixie AS frontend
RUN rustup target add wasm32-unknown-unknown \
 && cargo install --locked trunk@0.21.14 wasm-bindgen-cli@0.2.104

WORKDIR /app
# Dependency-only layer: real manifests, stub sources.
COPY Cargo.toml Cargo.lock Trunk.toml ./
COPY crates/ejas-core/Cargo.toml crates/ejas-core/
COPY crates/ejas-ui/Cargo.toml crates/ejas-ui/
COPY crates/ejas-server/Cargo.toml crates/ejas-server/
RUN mkdir -p src crates/ejas-core/src crates/ejas-ui/src crates/ejas-server/src \
 && echo 'fn main() {}' > crates/ejas-ui/src/main.rs \
 && touch src/lib.rs crates/ejas-core/src/lib.rs crates/ejas-ui/src/lib.rs \
 && echo 'fn main() {}' > crates/ejas-server/src/main.rs \
 && cargo build --release --target wasm32-unknown-unknown -p ejas-ui 2>/dev/null || true

COPY . .
# Cargo does not rebuild on content alone; bump the mtime past the stub layer.
RUN touch crates/ejas-core/src/lib.rs crates/ejas-ui/src/lib.rs \
 && trunk build --release

# ----------------------------------------------------------------- backend --
FROM rust:${RUST_VERSION}-trixie AS backend
# libz3-dev: prebuilt Z3 library + headers (never compiled from source).
# libclang-dev: z3-sys generates its FFI bindings with bindgen, which needs it.
RUN apt-get update \
 && apt-get install -y --no-install-recommends libz3-dev libclang-dev \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ejas-core/Cargo.toml crates/ejas-core/
COPY crates/ejas-ui/Cargo.toml crates/ejas-ui/
COPY crates/ejas-server/Cargo.toml crates/ejas-server/
RUN mkdir -p src crates/ejas-core/src crates/ejas-ui/src crates/ejas-server/src \
 && touch src/lib.rs crates/ejas-core/src/lib.rs crates/ejas-ui/src/lib.rs \
 && echo 'fn main() {}' > crates/ejas-ui/src/main.rs \
 && echo 'fn main() {}' > crates/ejas-server/src/main.rs \
 && mkdir -p crates/ejas-server/dist && touch crates/ejas-server/dist/index.html \
 && cargo build --release -p ejas-server 2>/dev/null || true

COPY . .
# rust-embed bakes the bundle into the binary, so the frontend must land first.
COPY --from=frontend /app/crates/ejas-server/dist ./crates/ejas-server/dist
RUN touch src/lib.rs crates/ejas-core/src/lib.rs crates/ejas-server/src/main.rs \
 && cargo build --release --locked -p ejas-server

# ----------------------------------------------------------------- runtime --
# Only the shared library is needed at run time, not the headers.
FROM debian:trixie-slim AS runtime
RUN apt-get update \
 && apt-get install -y --no-install-recommends libz3-4 \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --create-home --uid 10001 ejas

COPY --from=backend /app/target/release/ejas-server /usr/local/bin/ejas-server

USER ejas
ENV EJAS_PORT=8080
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD ["ejas-server", "--health-check"]
CMD ["ejas-server"]
