//! The Ergonomic Assigner UI, built for both the browser and the desktop.
//!
//! The same widgets run on both targets; solving always happens over HTTP,
//! because Z3 cannot be compiled to WebAssembly.

pub mod app;
pub mod client;
pub mod export;
pub mod fileio;
pub mod load;
pub mod screens;
pub mod state;

pub use app::EjasApp;

pub const APP_TITLE: &str = "Ergonomic Assigner";

/// Browser entry point. Trunk emits the glue that calls this.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    use eframe::wasm_bindgen::JsCast as _;

    console_log::init_with_level(log::Level::Info).ok();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        let canvas = document
            .get_element_by_id("app_canvas")
            .expect("index.html is missing #app_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#app_canvas is not a canvas");

        let result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(EjasApp::new(cc)))),
            )
            .await;

        // Replace the loading text with something actionable if boot failed.
        if let Some(element) = document.get_element_by_id("loading") {
            match result {
                Ok(_) => element.remove(),
                Err(e) => element.set_inner_html(&format!(
                    "<p>The app failed to start.</p><p><code>{e:?}</code></p>"
                )),
            }
        }
    });
}
