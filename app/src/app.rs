use std::{pin::Pin, sync::{Arc, Mutex}};

use eframe::{egui::{self}, epi, epaint::{TextureHandle, ColorImage, Color32}};
use mips_emulator::{cpu::MipsCpu};

use crate::{tabs::{code_editor::CodeEditor, tabbed_area::TabbedArea}, emulator::handlers::ExternalHandler, util::keyboard_util::KeyboardMemory, side_panel::{side_tabbed_panel::SideTabbedPanel}};


/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
#[allow(unused)]

pub enum Theme {
    DarkMode,
    LightMode,
}
#[allow(unused)]
pub struct ApplicationSettings {
    pub theme: Theme,
}

impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            theme: Theme::DarkMode,
        }
    }
}


#[allow(dead_code)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct Application {
    // Example stuff:
    settings: ApplicationSettings,


    #[cfg_attr(feature = "persistence", serde(skip))]
    pub cpu: Pin<Box<MipsCpu>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub cpu_screen: TextureHandle,
    #[cfg_attr(feature = "persistence", serde(skip))]
    cpu_screen_texture: Arc<Mutex<Option<ColorImage>>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub cpu_virtual_keyboard: Arc<Mutex<KeyboardMemory>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub tabbed_area: TabbedArea,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub side_panel: Arc<Mutex<SideTabbedPanel>>,
}

impl Application {
    pub fn new(ctx: &egui::Context) -> Self {

        let mut ret = Self {
            settings: ApplicationSettings::default(),
            tabbed_area: TabbedArea::default(),
            side_panel: Default::default(),

            cpu: Box::pin(MipsCpu::new()),


            cpu_screen_texture: Arc::new(Mutex::new(Option::None)),
            cpu_screen:  ctx.load_filtered_texture("ImageTabImage", ColorImage::new([1,1], Color32::BLACK), eframe::epaint::textures::TextureFilter::Nearest),
            
            cpu_virtual_keyboard: Arc::new(Mutex::new(KeyboardMemory::new())), 
        };


        ret.tabbed_area.add_tab(Box::new(CodeEditor::new("Assembly".into(),
r#"//runs 2^16 * (2^15-1)*3+2 instructions (6442254338)
//0x64027FFFu32, 0x00000820, 0x20210001, 0x10220001, 0x0BFFFFFD, 0x68000000
//to run this you must reset processor then start it or program will not be loaded
//NOTE this assembly is not actually being compiled it is just to show what is being run in the demo :)
//also node that the highlighting is FAR from being done(using the highlighting from a clike language for now)
//this version usally takes around ~16.9 seconds while the java version takes ~228.7 seconds (on my machine)
//thats a cool 1250% speed increase

lhi $2, 32767
add $1, $0, $0
loop:
addi $1, $1, 1
beq $2, $1, end
j loop
end:
trap 0
"#.into()
     )));
     
        ret.cpu.set_external_handlers(ExternalHandler::new(ret.cpu_screen_texture.clone(), ret.cpu_virtual_keyboard.clone()));
        ret.tabbed_area.add_tab(Box::new(CodeEditor::default()));
        let tab = Box::new(crate::tabs::image_tab::ImageTab::new("CPU screen", ret.cpu_screen.clone()));
        ret.tabbed_area.add_tab(tab);
        ret.tabbed_area.add_tab(Box::new(crate::tabs::hex_editor::HexEditor::new(unsafe{std::mem::transmute(ret.cpu.as_mut().get_mut())})));
        

        #[cfg(not(target_arch = "wasm32"))]
        ret.tabbed_area.add_tab(Box::new(crate::tabs::sound::SoundTab::new()));
        ret
    }
}

impl Application{
    pub fn settings(&self) -> &ApplicationSettings{
        &self.settings
    } 
}

impl epi::App for Application {

    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut epi::Frame) {

        if self.cpu.is_running() && !self.cpu.paused_or_stopped() {
            ctx.request_repaint();
        }
        
        if let Option::Some(image) = self.cpu_screen_texture.lock().unwrap().to_owned(){
            self.cpu_screen.set(image, eframe::epaint::textures::TextureFilter::Nearest);
        }
        self.cpu_virtual_keyboard.lock().unwrap().update(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                    
                });
                
                ui.with_layout(egui::Layout::right_to_left(), |ui|{
                    ui.add(
                        egui::Hyperlink::from_label_and_url("HomePage", "https://github.com/ParkerTenBroeck/Mips-Edu")
                    );
                    ui.separator();
                    ui.add(
                        egui::Hyperlink::from_label_and_url("Website", "https://parkertenbroeck.com")
                    );
                    ui.separator();
                    ui.label(format!("CPU Time: {:.2} ms", 1e3 * frame.info().cpu_usage.unwrap_or_default()));
                    ui.separator();
                });
            });
        });

        let frame_no_marg = egui::Frame {
            rounding: eframe::epaint::Rounding::none(),
            fill: ctx.style().visuals.window_fill(),
            stroke: ctx.style().visuals.window_stroke(),
            inner_margin: egui::style::Margin::symmetric(2.0, 2.0),
            outer_margin: egui::style::Margin::symmetric(0.0, 0.0),
            ..Default::default()
        };

        
        match self.side_panel.clone().lock(){
            Ok(mut side_panel) => {
                egui::SidePanel::left("side_panel")
                .min_width(0.0)
                .frame(frame_no_marg.clone())
                .resizable(side_panel.is_visible())
                .show(ctx, |ui| {
                    side_panel.ui(ui, self);
            });
            },
            Err(_) => panic!(),
        }
        //let mut asd = self.side_panel.lock();
        //let mut side_panel = asd.unwrap();


        egui::TopBottomPanel::bottom("bottom_panel").resizable(true).show(ctx, |ui| {
            
            egui::ScrollArea::both().stick_to_bottom().show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    for record in crate::loggers::get_last_record(log::Level::Trace, 30).iter().rev(){

                        match record.0{
                            log::Level::Error => {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(record.1.as_str())
                                        .color(eframe::epaint::Color32::from_rgb(237, 67, 55)))
                                        .wrap(false)
                                );
                            }
                            log::Level::Warn => {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(record.1.as_str())
                                        .color(eframe::epaint::Color32::from_rgb(238, 210, 2)))
                                        .wrap(false)
                                );
                            }

                            log::Level::Info | log::Level::Debug | log::Level::Trace
                             => {
                                ui.add(
                                    egui::Label::new(record.1.as_str()).wrap(false)
                                );
                             },
                        }
                    }
                    //ui.painter().rect_filled(ui.min_rect(), rounding, ui.ctx().visuals);
                });
                
                //let (response, painter) = ui.allocate_painter(ui.max_rect().size(), egui::Sense::hover());
                //let rect = response.rect;
                //let rounding = ui.style().interact(&response).rounding;
                //painter.rect_filled(rect, rounding, ui.visuals().extreme_bg_color);
                
                ui.allocate_space(ui.available_size());

            });

        });

        let frame = frame_no_marg.clone();
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            self.tabbed_area.ui(ui);
        });

        // if false {
        //     egui::Window::new("Window").show(ctx, |ui| {
        //         ui.label("Windows can be moved by dragging them.");
        //         ui.label("They are automatically sized based on contents.");
        //         ui.label("You can turn on resizing and scrolling if you like.");
        //         ui.label("You would normally chose either panels OR windows.");
        //     });
        // }
    }
}