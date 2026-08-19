//! Serves the WebAssembly frontend.
//!
//! The bundle is embedded in the binary rather than copied into the image,
//! so the runtime container is one executable plus libz3.

use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};

#[derive(rust_embed::Embed)]
#[folder = "dist/"]
struct Frontend;

pub async fn serve(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Frontend::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], file.data).into_response()
        }
        // Anything unknown falls back to the app shell, so the frontend can
        // own its own routing.
        None => match Frontend::get("index.html") {
            Some(index) => (
                [(header::CONTENT_TYPE, "text/html")],
                index.data,
            )
                .into_response(),
            None => (
                StatusCode::NOT_FOUND,
                "frontend bundle missing: build it with `trunk build` into crates/ejas-server/dist",
            )
                .into_response(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::Frontend;

    /// An empty `dist/` is legitimate — you can develop the server without
    /// ever running trunk. But if a bundle *was* built, it has to include the
    /// entry point, or every route silently falls through to a 404.
    #[test]
    fn a_built_bundle_contains_its_entry_point() {
        let files: Vec<String> = Frontend::iter().map(|f| f.to_string()).collect();
        if files.is_empty() {
            eprintln!("no frontend bundle embedded; run `trunk build --release`");
            return;
        }
        assert!(
            files.iter().any(|f| f == "index.html"),
            "bundle has no index.html, only {files:?}"
        );
    }
}
