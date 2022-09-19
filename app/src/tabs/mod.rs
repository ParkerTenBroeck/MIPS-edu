pub mod code_editor;
pub mod debugger_tab;
pub mod dummy_tab;
pub mod hex_editor;
pub mod image_tab;
pub mod mips_display;
pub mod settings;
#[cfg(not(target_arch = "wasm32"))]
pub mod sound;
pub mod terminal_tab;
