use crate::emulator::debugger_thread::{self};
use crate::tabs::settings::{EguiInspectionTab, EguiMemoryTab, SettingsTab};
use crate::{emulator::handlers::CPUAccessInfo, platform::sync::PlatSpecificLocking};
use std::sync::Arc;
use std::sync::Mutex;
//use crate::platform::sync::Mutex;
use eframe::{
    egui::{self},
    epaint::{Color32, ColorImage, TextureHandle},
    App, Frame,
};
use egui_dock::{DockArea, DynamicTabViewer, DynamicTree, Tab};
//use egui_glium::{Painter, egui_winit::egui::Painter};
use mips_emulator::cpu::{CpuExternalHandler, EmulatorInterface, MipsCpu};

use crate::{
    emulator::handlers::ExternalHandler, side_panel::side_tabbed_panel::SideTabbedPanel,
    tabs::code_editor::CodeEditor, util::keyboard_util::KeyboardMemory,
};

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
    pub cpu: EmulatorInterface<ExternalHandler>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub cpu_screen: TextureHandle,
    #[cfg_attr(feature = "persistence", serde(skip))]
    cpu_screen_texture: Arc<Mutex<(u32, Option<ColorImage>)>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub cpu_virtual_keyboard: Arc<Mutex<KeyboardMemory>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub tabs: DynamicTree,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub side_panel: Arc<Mutex<SideTabbedPanel>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub access_info: CPUAccessInfo,
}

impl Application {
    pub fn new(ctx: &egui::Context) -> Self {
        let access_info: Arc<Mutex<crate::emulator::handlers::AccessInfo>> = Default::default();
        let cpu_screen_texture = Arc::new(Mutex::new((0, Option::None)));
        let cpu_screen = ctx.load_texture(
            "ImageTabImage",
            ColorImage::new([1, 1], Color32::BLACK),
            egui::TextureFilter::LinearTiled,
        );
        let cpu_virtual_keyboard = Arc::new(Mutex::new(KeyboardMemory::new()));
        let cpu = MipsCpu::new_interface(ExternalHandler::new(
            access_info.clone(),
            cpu_screen_texture.clone(),
            cpu_virtual_keyboard.clone(),
        ));

        let mut ret = Self {
            settings: ApplicationSettings::default(),
            side_panel: Default::default(),

            access_info,
            cpu,
            frame: 0,
            cpu_screen,
            cpu_screen_texture,
            cpu_virtual_keyboard,
            tabs: Default::default(),
        };

        ret.add_cpu_memory_tab();
        ret.add_cpu_screen_tab();
        ret.add_cpu_terminal_tab();
        ret.add_cpu_sound_tab();

        ret.add_tab(CodeEditor::new("Assembly".into(),
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
        ));

        ret.add_tab(CodeEditor::default());

        ret
    }
}

impl Application {
    pub fn settings(&self) -> &ApplicationSettings {
        &self.settings
    }
}

impl Application {
    pub fn frame(&self) -> usize {
        self.frame as usize
    }

    pub fn add_tab(&mut self, tab: impl Tab + 'static) {
        self.tabs.push_to_focused_leaf(Box::new(tab));
    }

    pub fn add_cpu_terminal_tab(&mut self) {
        let tab = crate::tabs::terminal_tab::TerminalTab::new();
        self.add_tab(tab);
    }

    pub fn add_debugger_tab(
        &mut self,
        debugger: Arc<
            Mutex<
                dyn debugger_thread::DebuggerConnection<
                    crate::emulator::debug_target::MipsTargetInterface<impl CpuExternalHandler>,
                >,
            >,
        >,
    ) {
        let tab = crate::tabs::debugger_tab::DebuggerTab::new(debugger);
        self.add_tab(tab);
    }

    pub fn create_debugger(&mut self) {
        let has_debugger = self.cpu.cpu_mut(|cpu| loop {
            if let Some(has_debugger) = cpu.has_debugger() {
                break has_debugger;
            }
        });

        if has_debugger {
            log::error!("Cannot attach debugger emulator already has debugger");
            return;
        }

        let emulator = self.cpu.clone();

        enum TwoStep<Y, T> {
            One(T),
            Two(Y),
            None,
        }
        let mut socket = TwoStep::One(std::net::TcpListener::bind("localhost:1234"));

        let builder = debugger_thread::mips_emulator_debugger_builder(
            emulator,
            Box::new(move || {
                let mut tmp = TwoStep::None;
                std::mem::swap(&mut tmp, &mut socket);
                if let TwoStep::One(tmp) = tmp {
                    let tmp = tmp?;
                    tmp.set_nonblocking(true)?;
                    socket = TwoStep::Two(tmp);
                } else {
                    socket = tmp;
                }

                if let TwoStep::Two(socket) = &mut socket {
                    match socket.incoming().next() {
                        Some(Ok(socket)) => Ok(Some(socket)),
                        Some(Err(e)) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
                        Some(Err(e)) => Err(e.into()),
                        None => Ok(None),
                    }
                } else {
                    panic!();
                }
            }),
        );

        match debugger_thread::start(builder) {
            Ok(ok) => {
                self.add_debugger_tab(ok);
            }
            Err(err) => {
                log::error!("Failed to start debugger: {}", err);
            }
        }
    }

    pub fn add_cpu_memory_tab(&mut self) {
        let tab = crate::tabs::hex_editor::HexEditor::new(self.cpu.clone());
        self.add_tab(tab);
    }

    pub fn add_cpu_screen_tab(&mut self) {
        let tab = crate::tabs::mips_display::MipsDisplay::new(
            self.cpu_screen.clone(),
            self.access_info.clone(),
        );
        self.add_tab(tab);
    }

    pub fn add_cpu_sound_tab(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let tab = crate::tabs::sound::SoundTab::new();
            self.add_tab(tab);
        }
    }

    pub fn add_settings_tab(&mut self) {
        self.add_tab(SettingsTab {})
    }

    pub fn add_egui_memory_tab(&mut self) {
        self.add_tab(EguiMemoryTab {})
    }

    pub fn add_egui_inspection_tab(&mut self) {
        self.add_tab(EguiInspectionTab {})
    }
}

impl App for Application {
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        unsafe {
            if (*self.cpu.raw_cpu()).is_running() {
                ctx.request_repaint()
            }
        }

        if let Result::Ok(mut lock) = self.cpu_screen_texture.plat_lock() {
            let (frame, texture) = &mut *lock;
            *frame = self.frame;
            if let Option::Some(image) = texture.take() {
                self.cpu_screen
                    .set(image, egui::TextureFilter::NearestTiled);
            }
        }

        if let Result::Ok(mut lock) = self.cpu_virtual_keyboard.plat_lock() {
            lock.update(ctx);
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        #[cfg(not(target_arch = "wasm32"))]
                        frame.close();
                    }
                });

                ui.with_layout(
                    egui::Layout::right_to_left(eframe::emath::Align::Center),
                    |ui| {
                        ui.add(egui::Hyperlink::from_label_and_url(
                            "HomePage",
                            "https://github.com/ParkerTenBroeck/Mips-Edu",
                        ));
                        ui.separator();
                        ui.add(egui::Hyperlink::from_label_and_url(
                            "Website",
                            "https://parkertenbroeck.com",
                        ));
                        ui.separator();
                        ui.label(format!(
                            "CPU Time: {:.2} ms",
                            1e3 * frame.info().cpu_usage.unwrap_or_default()
                        ));
                        ui.separator();
                    },
                );
            });
        });

        let frame_no_marg = egui::Frame {
            rounding: eframe::epaint::Rounding::none(),
            fill: ctx.style().visuals.window_fill(),
            stroke: ctx.style().visuals.window_stroke(),
            inner_margin: egui::style::Margin::symmetric(0.0, 0.0),
            outer_margin: egui::style::Margin::symmetric(0.0, 0.0),
            ..Default::default()
        };

        match self.side_panel.clone().plat_lock() {
            Ok(mut side_panel) => {
                egui::SidePanel::left("side_panel")
                    .min_width(0.0)
                    .frame(frame_no_marg)
                    .resizable(side_panel.is_visible())
                    .show(ctx, |ui| {
                        side_panel.ui(ui, self);
                    });
            }
            Err(_) => panic!(),
        }

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::both()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            for record in crate::loggers::get_last_record(log::Level::Trace, 30)
                                .iter()
                                .rev()
                            {
                                match record.0 {
                                    log::Level::Error => {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(record.1.as_str()).color(
                                                    eframe::epaint::Color32::from_rgb(237, 67, 55),
                                                ),
                                            )
                                            .wrap(false),
                                        );
                                    }
                                    log::Level::Warn => {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(record.1.as_str()).color(
                                                    eframe::epaint::Color32::from_rgb(238, 210, 2),
                                                ),
                                            )
                                            .wrap(false),
                                        );
                                    }

                                    log::Level::Info | log::Level::Debug | log::Level::Trace => {
                                        ui.add(egui::Label::new(record.1.as_str()).wrap(false));
                                    }
                                }
                            }
                        });

                        ui.allocate_space(ui.available_size());
                    });
            });

        let frame = frame_no_marg;

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let style = egui_dock::Style::from_egui(&ui.ctx().style());

            DockArea::new(&mut self.tabs)
                .style(style)
                .show(ctx, &mut DynamicTabViewer {});
        });

        self.frame += 1;
    }
}
