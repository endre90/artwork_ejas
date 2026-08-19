//! Non-blocking solve requests.
//!
//! `ehttp` works on both native and wasm, and its callback form keeps the
//! solve off the render thread. The original desktop app called Z3 inline in
//! `update()`, which froze the window for the whole solve.

use std::sync::{Arc, Mutex};

use ejas_core::api::{SolveRequest, SolveResponse};

/// A solve that may still be running.
#[derive(Clone, Default)]
pub struct PendingSolve {
    slot: Arc<Mutex<Option<Result<SolveResponse, String>>>>,
    in_flight: Arc<Mutex<bool>>,
}

impl PendingSolve {
    pub fn is_in_flight(&self) -> bool {
        *self.in_flight.lock().unwrap()
    }

    /// Take the finished result, if one has arrived.
    pub fn take(&self) -> Option<Result<SolveResponse, String>> {
        self.slot.lock().unwrap().take()
    }

    /// Fire a solve. Repaints the UI when the answer lands, since an
    /// immediate-mode app would otherwise sit idle until the next input.
    pub fn start(&self, ctx: &egui::Context, base_url: &str, request: &SolveRequest) {
        if self.is_in_flight() {
            return;
        }

        let body = match serde_json::to_vec(request) {
            Ok(body) => body,
            Err(e) => {
                *self.slot.lock().unwrap() = Some(Err(format!("could not encode request: {e}")));
                return;
            }
        };

        *self.in_flight.lock().unwrap() = true;

        let url = format!("{}/api/solve", base_url.trim_end_matches('/'));
        let mut http_request = ehttp::Request::post(url, body);
        http_request
            .headers
            .insert("Content-Type", "application/json");

        let slot = self.slot.clone();
        let in_flight = self.in_flight.clone();
        let ctx = ctx.clone();

        ehttp::fetch(http_request, move |result| {
            *slot.lock().unwrap() = Some(interpret(result));
            *in_flight.lock().unwrap() = false;
            ctx.request_repaint();
        });
    }
}

fn interpret(result: ehttp::Result<ehttp::Response>) -> Result<SolveResponse, String> {
    let response = result.map_err(|e| format!("could not reach the solver: {e}"))?;

    let text = response.text().unwrap_or_default();
    if !response.ok {
        // The server sends `{"error": "..."}` for anything it rejects.
        let detail = serde_json::from_str::<serde_json::Value>(text)
            .ok()
            .and_then(|v| v["error"].as_str().map(str::to_owned))
            .unwrap_or_else(|| text.to_owned());
        return Err(format!("solver returned {}: {detail}", response.status));
    }

    serde_json::from_str(text).map_err(|e| format!("could not decode the solution: {e}"))
}

/// Where the server lives.
///
/// On the web the UI is served by the same process that solves, so a relative
/// URL is both correct and portable. Natively there is no origin, so fall back
/// to the local default and let `EJAS_SERVER` override it.
pub fn default_base_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        String::new()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var("EJAS_SERVER").unwrap_or_else(|_| "http://127.0.0.1:8080".to_owned())
    }
}
