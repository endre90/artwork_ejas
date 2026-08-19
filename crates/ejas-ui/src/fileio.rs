//! Loading and saving JSON on both native and web.
//!
//! Browsers have no blocking file dialog, so every path here is async and
//! delivers its bytes through a shared slot the UI polls each frame.

use std::sync::{Arc, Mutex};

/// A file the user picked, once it has actually arrived.
pub struct Incoming {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// Slot for a pending "open file" interaction.
#[derive(Clone, Default)]
pub struct FilePicker {
    slot: Arc<Mutex<Option<Incoming>>>,
}

impl FilePicker {
    /// Take the picked file, if the user has finished choosing one.
    pub fn take(&self) -> Option<Incoming> {
        self.slot.lock().unwrap().take()
    }

    /// Open a JSON file picker. Returns immediately.
    pub fn open(&self, ctx: &egui::Context) {
        let slot = self.slot.clone();
        let ctx = ctx.clone();
        let task = rfd::AsyncFileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file();

        spawn(async move {
            if let Some(handle) = task.await {
                let name = handle.file_name();
                let bytes = handle.read().await;
                *slot.lock().unwrap() = Some(Incoming { name, bytes });
                ctx.request_repaint();
            }
        });
    }

    /// Accept a file dropped onto the window.
    ///
    /// Reading is asynchronous on the web (the browser only hands over a
    /// handle), so both targets go through the same slot the picker uses.
    pub fn accept_dropped(&self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        let Some(file) = dropped.into_iter().next() else {
            return;
        };

        let name = file
            .path()
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "dropped.json".to_owned());

        let slot = self.slot.clone();
        let ctx = ctx.clone();

        #[cfg(not(target_arch = "wasm32"))]
        match file.bytes() {
            Ok(bytes) => {
                *slot.lock().unwrap() = Some(Incoming { name, bytes });
                ctx.request_repaint();
            }
            Err(e) => log::error!("could not read dropped file: {e}"),
        }

        #[cfg(target_arch = "wasm32")]
        spawn(async move {
            match file.bytes_async().await {
                Ok(bytes) => {
                    *slot.lock().unwrap() = Some(Incoming { name, bytes });
                    ctx.request_repaint();
                }
                Err(e) => log::error!("could not read dropped file: {e}"),
            }
        });
    }
}

/// Offer `contents` to the user as a download / save dialog.
pub fn save_json(suggested_name: &str, contents: String) {
    let task = rfd::AsyncFileDialog::new()
        .add_filter("JSON", &["json"])
        .set_file_name(suggested_name)
        .save_file();

    spawn(async move {
        if let Some(handle) = task.await {
            // On web this triggers the browser download; natively it writes
            // the chosen path. Nothing useful to do with a failure but log it.
            if let Err(e) = handle.write(contents.as_bytes()).await {
                log::error!("could not save file: {e}");
            }
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn spawn<F: std::future::Future<Output = ()> + 'static>(future: F) {
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn<F: std::future::Future<Output = ()> + Send + 'static>(future: F) {
    // rfd's async dialogs need a running executor; a detached thread with a
    // minimal block-on is enough and avoids pulling in a full async runtime.
    std::thread::spawn(move || pollster::block_on(future));
}
