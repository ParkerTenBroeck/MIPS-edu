use std::{pin::Pin, sync::{Arc}};
use std::sync::Mutex;
use crate::{platform::sync::PlatSpecificLocking, emulator::handlers::CPUAccessInfo};
//use crate::platform::sync::Mutex;

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
    frame: u32,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub cpu: Pin<Box<MipsCpu<ExternalHandler>>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub cpu_screen: TextureHandle,
    #[cfg_attr(feature = "persistence", serde(skip))]
    cpu_screen_texture: Arc<Mutex<(u32, Option<ColorImage>)>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub cpu_virtual_keyboard: Arc<Mutex<KeyboardMemory>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub tabbed_area: TabbedArea,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub side_panel: Arc<Mutex<SideTabbedPanel>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub access_info: CPUAccessInfo,
}

impl Application {
    pub fn new(ctx: &egui::Context) -> Self {


        let access_info: Arc<Mutex<crate::emulator::handlers::AccessInfo>> = Default::default();
        let cpu_screen_texture = Arc::new(Mutex::new((0, Option::None)));
        let cpu_screen = ctx.load_filtered_texture("ImageTabImage", ColorImage::new([1,1], Color32::BLACK), eframe::epaint::textures::TextureFilter::Nearest);        
        let cpu_virtual_keyboard = Arc::new(Mutex::new(KeyboardMemory::new()));
        let cpu = Box::pin(MipsCpu::new(ExternalHandler::new(access_info.clone(), cpu_screen_texture.clone(), cpu_virtual_keyboard.clone())));

        let mut ret = Self {
            settings: ApplicationSettings::default(),
            tabbed_area: TabbedArea::default(),
            side_panel: Default::default(),

            access_info, 
            cpu,
            frame: 0,
            cpu_screen,
            cpu_screen_texture,
            cpu_virtual_keyboard,
        };

        ret.add_cpu_memory_tab();
        ret.add_cpu_screen_tab(); 
        ret.add_cpu_sound_tab();    
        
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

        ret.tabbed_area.add_tab(Box::new(CodeEditor::default()));
 
        ret
    }
}

impl Application{
    pub fn settings(&self) -> &ApplicationSettings{
        &self.settings
    } 
}

impl Application{

    pub fn add_cpu_terminal_tab(&mut self){
        self.tabbed_area.add_tab(Box::new(crate::tabs::terminal_tab::TerminalTab::new()));
    }

    pub fn add_cpu_memory_tab(&mut self){
        self.tabbed_area.add_tab(Box::new(crate::tabs::hex_editor::HexEditor::new(unsafe{std::mem::transmute(self.cpu.as_mut().get_mut())})));
    }

    pub fn add_cpu_screen_tab(&mut self){
        let tab = Box::new(crate::tabs::image_tab::ImageTab::new("MIPS Display", self.cpu_screen.clone()));
        self.tabbed_area.add_tab(tab);
    }

    pub fn add_cpu_sound_tab(&mut self){
        #[cfg(not(target_arch = "wasm32"))]
        self.tabbed_area.add_tab(Box::new(crate::tabs::sound::SoundTab::new()));
    }
}


impl epi::App for Application {

    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }
    fn on_exit(&mut self, _gl: &eframe::glow::Context) {
        
    }
    

    fn update(&mut self, ctx: &egui::Context, frame: &mut epi::Frame) {

        if self.cpu.is_running() && !self.cpu.paused_or_stopped() {
            ctx.request_repaint();
        }
        
        if let Result::Ok(mut lock) = self.cpu_screen_texture.plat_lock(){
            let (frame, texture) = &mut *lock;
            *frame = self.frame;
            if let Option::Some(image) = texture.take(){
                self.cpu_screen.set(image, eframe::epaint::textures::TextureFilter::Nearest);
            }
        }

        if let Result::Ok(mut lock) = self.cpu_virtual_keyboard.plat_lock(){
            lock.update(ctx);
        }

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

        
        match self.side_panel.clone().plat_lock(){
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

        self.frame += 1;
    }
}