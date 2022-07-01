//#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub mod syntax_highlighter;
pub mod loggers;
pub mod tabs;
pub mod side_panel;
pub mod resource_manager;
pub mod emulator;
pub mod util;

pub use app::Application;

// ----------------------------------------------------------------------------
// When compiling for web:

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    clike_gui::loggers::init();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let app = Application::default();
    eframe::start_web(canvas_id, Box::new(app))
}
