//#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

pub mod syntax_highlighter;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = clike_gui::ClikeGui::default();
    

    let icon = match image::open("./docs/icon-256.png"){
        Result::Ok(val) => {
            let icon = val.to_rgba8();
            let (icon_width, icon_height) = icon.dimensions();
            Some(eframe::epi::IconData{
                rgba: icon.into_raw(),
                width: icon_width,
                height: icon_height,
            })
        }
        Result::Err(_) => {
            Option::None
        }
    };

    let native_options = eframe::NativeOptions{
        icon_data: icon,
        ..Default::default()
    };
    eframe::run_native(Box::new(app), native_options);
}
