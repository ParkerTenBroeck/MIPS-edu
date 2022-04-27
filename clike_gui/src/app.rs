use std::{io::Read, pin::Pin, sync::{Arc, Mutex}};

use eframe::{egui::{self}, epi, epaint::{TextureHandle, ColorImage, Color32}};
use mips_emulator::{cpu::MipsCpu, memory::page_pool::MemoryDefault};

use crate::{tabs::{code_editor::CodeEditor, tabbed_area::TabbedArea}, emulator::handlers::ExternalHandler, util::keyboard_util::KeyboardMemory};


/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
#[allow(unused)]

//#[macro_export]
//#[cfg(target_arch = "wasm32")]
//macro_rules! println {
//    ( $( $t:tt )* ) => {
//        log::info!("{}", ( $( $t )* ));
//    };
//}

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
pub struct ClikeGui {
    // Example stuff:
    settings: ApplicationSettings,


    #[cfg_attr(feature = "persistence", serde(skip))]
    cpu: Pin<Box<MipsCpu>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    cpu_screen: TextureHandle,
    #[cfg_attr(feature = "persistence", serde(skip))]
    cpu_virtual_keyboard: Arc<Mutex<KeyboardMemory>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    tabbed_area: TabbedArea,
}

impl ClikeGui {
    pub fn new(ctx: &egui::Context) -> Self {

        let mut ret = Self {
            settings: ApplicationSettings::default(),
            //cpu: MipsCpu::new(),
            tabbed_area: TabbedArea::default(),
            cpu: Box::pin(MipsCpu::new()),
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
     
        ret.cpu.set_external_handlers(ExternalHandler::new(ret.cpu_screen.clone(), ret.cpu_virtual_keyboard.clone()));
        ret.tabbed_area.add_tab(Box::new(CodeEditor::default()));
        let tab = Box::new(crate::tabs::image_tab::ImageTab::new("CPU screen", ret.cpu_screen.clone()));
        ret.tabbed_area.add_tab(tab);
        ret.tabbed_area.add_tab(Box::new(crate::tabs::hex_editor::HexEditor::new(unsafe{std::mem::transmute(ret.cpu.as_mut().get_mut())})));
        ret
    }
}

impl ClikeGui{
    pub fn settings(&self) -> &ApplicationSettings{
        &self.settings
    } 
}

impl epi::App for ClikeGui {

    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut epi::Frame) {

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
                        egui::Hyperlink::from_label_and_url("HomePage", "https://github.com/ParkerTenBroeck/CLike")
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


        //TEMP
        static mut SEL: u32 = 1;
        static mut VIS: bool = true;
        let select = unsafe { &mut SEL };
        let vis = unsafe { &mut VIS };
        //TEMP

        let frame_no_marg = egui::Frame {
            //margin: egui::style::Margin::symmetric(2.0, 2.0),
            rounding: eframe::epaint::Rounding::none(),
            fill: ctx.style().visuals.window_fill(),
            stroke: ctx.style().visuals.window_stroke(),
            inner_margin: egui::style::Margin::symmetric(2.0, 2.0),
            outer_margin: egui::style::Margin::symmetric(0.0, 0.0),
            ..Default::default()
        };
        egui::SidePanel::left("side_panel")
            .min_width(0.0)
            .frame(frame_no_marg.clone())
            .resizable(*select != 0)
            .show(ctx, |ui| {
            //let min_height = ui.min_rect().top();
            let max_height = ui.max_rect().bottom();

            //ui.set_max_width(ui.max_rect().right());
            //println!("{}", ui.max_rect().right());
            //ui.spacing_mut().item_spacing.x = 0.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                //egui::SidePanel::left("left_icon_panel").show(ui.ctx(), |ui|{
                    
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    if ui.selectable_label(*vis && *select == 1, "💻").clicked() {
                        if *select == 1{
                            *vis = false;
                            *select = 0;
                        }else{
                            *vis = true;
                            *select = 1;
                        }
                    }
                    ui.spacing_mut().item_spacing.x = 0.0;
                    if ui.selectable_label(*vis && *select == 2, "🖹").clicked() {
                        if *select == 2{
                            *vis = false;
                            *select = 0;
                        }else{
                            *vis = true;
                            *select = 2;
                        }
                    }
                    //println!("{}, {}, {}", max_height, min_height, ui.max_rect().bottom());
                    ui.add_space(max_height - ui.max_rect().bottom() - 3.0);
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.set_max_height(max_height);
                        if ui.selectable_label(false, "⚙").clicked() {}
                        if ui
                            .selectable_label(ctx.debug_on_hover(), "🐛")
                            .on_hover_text("debug on hover")
                            .clicked()
                        {
                            ctx.set_debug_on_hover(!ctx.debug_on_hover());
                        }
                    });
                    //    ui.horizontal(|ui| {
                    //ui.spacing_mut().item_spacing.y = 500.0;
                    //ui.add_space(500.0);
                    //ui.separator();

                    //    });
                    //});
                });

                if *select != 0 {

                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.separator();

                    ui.vertical(|ui| {
                        egui::ScrollArea::both().show(ui, |ui| {
                            if *select == 1 {
                                ui.heading("CPU control");
    
                                //ui.horizontal(|ui| {
                                //    ui.label("Write something: ");
                                //    ui.text_edit_singleline(label);
                                //});
    
                                let (pc, hi, lo, reg) = {
                                    (
                                        self.cpu.get_pc(),
                                        self.cpu.get_hi_register(),
                                        self.cpu.get_lo_register(),
                                        self.cpu.get_general_registers(),
                                    )
                                };
                                if self.cpu.is_running() && !self.cpu.paused_or_stopped() {
                                    ui.ctx().request_repaint();
                                }
    
                                ui.horizontal(|ui| {
                                    ui.label("PC: ");
                                    ui.label(pc.to_string());
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Hi: ");
                                    ui.label(hi.to_string());
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Lo: ");
                                    ui.label(lo.to_string());
                                });
                                ui.label("Reg: ");
                                let mut i = 0usize;
                                while i < 32 {
                                    ui.horizontal(|ui| {
                                        ui.label(format!(" {}: {}", i, reg[i]));
                                        i += 1;
                                        ui.label(format!(" {}: {}", i, reg[i]));
                                        i += 1;
                                    });
                                }
    
                                //ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
                                if ui.button("Start CPU").clicked() {
                                    unsafe {
                                        if self.cpu.is_running() {
                                            log::warn!("CPU is already running");
                                        } else {
                                            log::info!("CPU Starting");
                                            let cpu: &'static mut MipsCpu =
                                                std::mem::transmute(self.cpu.as_mut());
                                                
                                            cpu.start_new_thread();
                                        }
                                    }
                                }
                                if ui.button("Step CPU").clicked() {
                                    unsafe {
                                        if self.cpu.is_running() {
                                            log::warn!("CPU is already running");
                                        } else {
                                            let cpu: &'static mut MipsCpu =
                                                std::mem::transmute(self.cpu.as_mut());
                                            cpu.step_new_thread();
                                        }
                                    }
                                }
    
                                if ui.button("Stop CPU").clicked() {
                                    if self.cpu.is_running() {
                                        self.cpu.stop();
                                        log::info!("Stopping CPU");
                                    } else {
                                        log::warn!("CPU is already stopped");
                                    }
                                }
                                if ui.button("Pause CPU").clicked() {
                                    if self.cpu.paused_or_stopped(){
                                        log::warn!("CPU is already paused");
                                    }else{
                                        self.cpu.pause();
                                        log::info!("CPU is paused");
                                    }
                                }
                                if ui.button("Resume CPU").clicked() {
                                    if self.cpu.paused_or_stopped(){
                                        self.cpu.resume();
                                        log::info!("CPU resumed");
                                    }else{
                                        log::warn!("CPU is already resumed");
                                    }
                                }
                                //let sel: &mut bool = unsafe{
                                //    static mut internal: bool = false;
                                //    &mut internal
                                //};
                                // if ui.selectable_label(*sel, "asdasd").clicked(){
                                //     *sel = !*sel;

                                //     unsafe{
                                //         if *sel{

                                //         }else{
                                            
                                //         }
                                //     }
                                // }
                                if ui.button("Reset CPU").clicked() {
                                    if !self.cpu.is_running() {
                                        //runs 2^16 * (2^15-1)*3+2 instructions (6442254338)
                                        //the version written in c++ seems to be around 17% faster
                                        //[0x64027FFFu32, 0x00000820, 0x20210001, 0x10220001, 0x0BFFFFFD, 0x68000000][(self.pc >> 2) as usize];//

                                        self.cpu.clear();

                                        let f = std::fs::File::open("clike_gui/res/snake.mxn").unwrap();
                                        let mut reader = std::io::BufReader::new(f);
                                        let mut buffer = Vec::new();
                                        
                                        // Read file into vector.
                                        
                                        let size = reader.read_to_end(&mut buffer).unwrap();

                                        let test_prog = buffer.as_mut_slice();
                                        for i in 0..(size / 4){
                                            let base = i * 4;
                                            let b1 = test_prog[base];
                                            let b2 = test_prog[base + 1];

                                            test_prog[base] = test_prog[base + 3];
                                            test_prog[base + 1] = test_prog[base + 2];
                                            test_prog[base + 3] = b1;
                                            test_prog[base + 2] = b2;
                                        }

                                        

                                         let test_prog = &[
                                            0x64027FFFu32.to_be(),
                                            0x00000820u32.to_be(),
                                            0x20210001u32.to_be(),
                                            0x10220001u32.to_be(),
                                            0x0BFFFFFDu32.to_be(),
                                            0x68000000u32.to_be(),
                                         ];
                                        self.cpu.get_mem().copy_into_raw(0, test_prog);

                                        log::info!("reset CPU");
                                    } else {
                                        log::warn!("Cannot reset CPU while running");
                                    }
                                }
                            } else if *select == 2{
                                ui.add(
                                    egui::Label::new(egui::RichText::new("Workspace").heading()).wrap(false)
                                );
                                ui.collapsing("info", |ui|{
                                    ui.label("current workspace files ext(just the current directory of the exe for now)");
                                    ui.label("note opening files will only read them and never save to them currently");    
                                });
                                ui.separator();

                                generate_tree(".".into(),self, ui);

                                fn generate_tree(path: std::path::PathBuf, t: &mut ClikeGui,  ui: &mut egui::Ui){
                                    match std::fs::read_dir(path) {
                                        Ok(val) => {
                                            
                                            let mut test: Vec<Result<std::fs::DirEntry, std::io::Error>> = val.collect();
                                            test.sort_by(|t1, t2| {
                                                if let Result::Ok(t1) = t1{
                                                    if let Result::Ok(t2) = t2{
                                                        //let t1 = t1.unwrap();
                                                        //let t2 = t2.unwrap();
                                                        let t1d = t1.metadata().unwrap().is_dir();
                                                        let t2d = t2.metadata().unwrap().is_dir();
                                                        if t1d && t2d {
                                                            return t1.file_name().to_ascii_lowercase().to_str().unwrap()
                                                            .cmp(t2.file_name().to_ascii_lowercase().to_str().unwrap())
                                                        }else if t1d{
                                                            return std::cmp::Ordering::Less
                                                        }else if t2d{
                                                            return std::cmp::Ordering::Greater
                                                        }else{
                                                            return t1.file_name().to_ascii_lowercase().to_str().unwrap()
                                                            .cmp(t2.file_name().to_ascii_lowercase().to_str().unwrap())
                                                        }
                                                    }  
                                                }
                                                std::cmp::Ordering::Equal
                                            });
                                            for d in test{
                                                if let Result::Ok(val) = d{
                                                    
                                                    if val.metadata().unwrap().is_dir(){
                                                        ui.collapsing(val.file_name().into_string().unwrap(), |ui|{
                                                            generate_tree(val.path(),t, ui);
                                                        });
                                                    }else{
                                                        if ui.selectable_label(false, val.file_name().into_string().unwrap()).clicked(){
                                                            
                                                            if let Result::Ok(str) = std::fs::read_to_string(val.path()){
                                                                
                                                                t.tabbed_area.add_tab(Box::new(CodeEditor::new(val.file_name().into_string().unwrap(), str)));
                                                            }
                                                            log::info!("loaded file: {}", val.path().display());
                                                        }
                                                       
                                                    }
                                                }
                                            }
                                        },
                                        Err(_) => {
    
                                        },
                                    }
                                }
                            }
                        });
                        ui.allocate_space(ui.available_size());
                    });
                }
            });
        });

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

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}