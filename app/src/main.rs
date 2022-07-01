//#![forbid(unsafe_code)]



#![cfg_attr(windows, windows_subsystem = "windows")]

#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    if !app::loggers::init(){
        std::process::exit(0);
    }

    let icon = match image::open("./mips-edu/docs/icon-256.png"){
        Result::Ok(val) => {
            let icon = val.to_rgba8();
            let (icon_width, icon_height) = icon.dimensions();
            Some(eframe::epi::IconData{
                rgba: icon.into_raw(),
                width: icon_width,
                height: icon_height,
            })
        }
        Result::Err(err) => {
            log::error!("failed to load app icon: {}", err);
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

    eframe::run_native("Mips Edu", native_options, Box::new(|cc|{
        
        let app = app::Application::new(&cc.egui_ctx);
        
        match app.settings().theme {
            app::app::Theme::DarkMode => {
                cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
            }
            app::app::Theme::LightMode => {
                cc.egui_ctx.set_visuals(eframe::egui::Visuals::light());
            }   
        }     
        Box::new(app)
    }));
}

#[cfg(target_os= "linux")]    
fn create_linux_app() -> Result<(), Box<dyn std::error::Error>>{
    let home = std::env::var("HOME")?;
    let path = format!("{}/.local/share/applications/mips-edu.desktop", home);
    let app = std::fs::File::create(path.clone());
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
            let mut tmp = std::env::current_exe()?;
            if !tmp.pop() || !tmp.pop() || !tmp.pop(){
                log::error!("failed to get icon path: {:?}", tmp);
                return Result::Err(
                    Box::new(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Failed to get path {:?}", tmp))))
            } 
            tmp.push("app/docs/icon-256.png");
            
            //log::info!("{}", std::fs::canonicalize(std::path::Path::new(".")).unwrap().as_os_str().to_str().unwrap());
            let ic:String = match tmp.as_os_str().to_str(){
                Some(val) => val,
                None => {
                    log::error!("failed to get icon path: {:?}", tmp);
                    return Result::Err(
                        Box::new(
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!("Failed to get path {:?}", tmp))))
                },
            }.into();

            let data = format!("[Desktop Entry]\nName=MIPS\nExec={}\nIcon={}\nTerminal=false\nType=Application",ec, ic);
             
            use std::io::Write;
            let _ = val.write(data.as_bytes())?;
            return Result::Ok(())
        },
        Err(val) => {
            log::error!("failed to create application entry {}: {}",path, val);
            Result::Err(Box::new(val))
        },
    }
}
