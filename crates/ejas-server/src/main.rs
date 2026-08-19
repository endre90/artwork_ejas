//! HTTP front for the Z3 assignment solver.
//!
//! The browser cannot run Z3 (`z3-sys` is C++ FFI and has no wasm target),
//! so the WebAssembly UI posts a complete problem here and gets a solution
//! back. The server holds no state between requests: the roster and history
//! travel with every call, which keeps employee data off the server's disk
//! and makes the container trivially disposable.

mod solve;
mod statics;

use std::net::{Ipv4Addr, SocketAddr};

use axum::routing::{get, post};
use axum::Router;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

/// Rosters and history are JSON and can be sizeable, but not unbounded.
const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

const DEFAULT_PORT: u16 = 8080;

#[tokio::main]
async fn main() {
    // `--health-check` lets the container healthcheck reuse this one binary
    // instead of pulling curl into the runtime image.
    if std::env::args().any(|a| a == "--health-check") {
        std::process::exit(health_check().await);
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=warn".into()),
        )
        .init();

    let port = port();
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/solve", post(solve::solve))
        .fallback(statics::serve)
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        .layer(TraceLayer::new_for_http());

    // Bind all interfaces inside the container; compose publishes it to
    // loopback only, so it is not reachable from the network.
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("cannot bind {addr}: {e}"));

    tracing::info!("ejas-server listening on http://localhost:{port}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

fn port() -> u16 {
    match std::env::var("EJAS_PORT") {
        Ok(v) => v
            .parse()
            .unwrap_or_else(|_| panic!("EJAS_PORT is not a valid port number: {v:?}")),
        Err(_) => DEFAULT_PORT,
    }
}

/// Exit code for `--health-check`: 0 healthy, 1 not.
async fn health_check() -> i32 {
    let url = format!("http://127.0.0.1:{}/health", port());
    match tokio::net::TcpStream::connect(("127.0.0.1", port())).await {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("health check failed for {url}: {e}");
            1
        }
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
}
