//! Desktop entry point.
//!
//! Identical UI to the web build; it just talks to a server you start
//! yourself instead of the one that served the page.

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 900.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title(ejas_ui::APP_TITLE),
        ..Default::default()
    };

    eframe::run_native(
        ejas_ui::APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(ejas_ui::EjasApp::new(cc)))),
    )
}

/// The web build enters through `ejas_ui::start`, not through a binary.
#[cfg(target_arch = "wasm32")]
fn main() {}
