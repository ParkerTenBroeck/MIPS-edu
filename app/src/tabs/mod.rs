pub mod tabbed_area;
pub mod hex_editor;
pub mod image_tab;
pub mod code_editor;
pub mod settings;
#[cfg(not(target_arch = "wasm32"))]
pub mod sound;
pub mod terminal;