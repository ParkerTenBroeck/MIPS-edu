//#![forbid(unsafe_code)]



//#![cfg_attr(windows, windows_subsystem = "windows")]

#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    clike_gui::loggers::init();
    log::info!("test: {}", 12);

    let app = clike_gui::ClikeGui::default();

    let icon = match image::open("./clike_gui/docs/icon-256.png"){
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

    #[cfg(target_os= "linux")]
    match create_linux_app(){
        Ok(_) => {},
        Err(err) => {
            log::info!("Failed to create .desktop file: {}", err)
        },
    }

    eframe::run_native(Box::new(app), native_options);
}

#[cfg(target_os= "linux")]    
fn create_linux_app() -> Result<(), Box<dyn std::error::Error>>{
    let home = std::env::var("HOME")?;
    let app = std::fs::File::create(format!("{}/.local/share/applications/clike_gui.desktop", home));
    match app {
        Ok(mut val) => {

            let ec = (std::env::current_exe()?)
                        .into_os_string().into_string();
            let ec = match ec{
                Ok(val) => val,
                Err(err) => {
                    return Result::Err(
                        Box::new(
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!("Failed to get path {:?}", err))))
                },
            };

            let full = std::fs::canonicalize(std::path::Path::new("./../clike_gui/docs/icon-256.png"))?;
            let ic:String = match full.as_os_str().to_str(){
                Some(val) => val,
                None => {
                    return Result::Err(
                        Box::new(
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!("Failed to get path {:?}", full))))
                },
            }.into();
            let data = format!("[Desktop Entry]\nName=CLike\nExec={}\nIcon={}\nTerminal=false\nType=Application",ec, ic);
             
            use std::io::Write;
            let _ = val.write(data.as_bytes())?;
            return Result::Ok(())
        },
        Err(val) => Result::Err(Box::new(val)),
    }
}
