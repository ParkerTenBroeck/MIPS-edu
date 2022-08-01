#![feature(backtrace)]
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
pub mod platform;

pub use app::Application;

// ----------------------------------------------------------------------------
// When compiling for web:

use eframe::{CreationContext, App};
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

    if !app_init(){
        panic!("Failed to init logging");
    }

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();
    let canvas_id = canvas_id.to_string();

    eframe::start_web(canvas_id.as_str(), Box::new(|cc|{
        create_app(cc)
    }))
}

fn panic_hook(info: &std::panic::PanicInfo<'_>) {
    log::error!("{}\n{}", info, std::backtrace::Backtrace::force_capture().to_string());
}

pub fn app_init() -> bool{
    if !crate::loggers::init(){
        return false;
    }
    std::panic::set_hook(Box::new(panic_hook));
    log::debug!("panic_hook set");
    
    true
}

pub fn create_app(cc: &CreationContext<'_>) -> Box<dyn App>{
    let app = app::Application::new(&cc.egui_ctx);

    // let mut fonts = eframe::egui::FontDefinitions::default(); 
    // fonts.font_data.insert( "DroidSansMono".to_owned(), eframe::egui::FontData::from_static(include_bytes!("../res/ttf/DroidSansMono.ttf")) );
    // fonts.families.insert(eframe::egui::FontFamily::Name("DroidSansMono".into()), vec!["DroidSansMono".to_owned()]);
    // cc.egui_ctx.set_fonts(fonts);

        
    match app.settings().theme {
        crate::app::Theme::DarkMode => {
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
        }
        crate::app::Theme::LightMode => {
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::light());
        }   
    }     
    Box::new(app)
}