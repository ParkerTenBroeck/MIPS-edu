use eframe::egui;

use super::{cpu_panel::CPUSidePanel, project_panel::ProjectSidePanel};

pub struct SideTabbedPanel {
    selected: usize,
    tabs: Vec<Box<dyn SideTab>>,
}

pub trait SideTab {
    fn ui(&mut self, ui: &mut egui::Ui, app: &mut crate::Application);
    fn get_icon(&mut self);
}

impl SideTabbedPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui, app: &mut crate::Application) {
        let max_height = ui.max_rect().bottom();

        ui.spacing_mut().item_spacing.x = 0.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            ui.vertical(|ui| {
                let mut i = 1;
                for _tab in self.tabs.iter() {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    if ui.selectable_label(self.selected == i, "💻").clicked() {
                        if self.selected == i {
                            self.selected = 0;
                        } else {
                            self.selected = i;
                        }
                    }
                    i += 1;
                }
                ui.add_space(max_height - ui.max_rect().bottom() - 3.0);
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.set_max_height(max_height);
                    if ui.selectable_label(false, "⚙").clicked() {
                        app.add_settings_tab();
                    }
                    if ui
                        .selectable_label(ui.ctx().debug_on_hover(), "🐛")
                        .on_hover_text("debug on hover")
                        .clicked()
                    {
                        ui.ctx().set_debug_on_hover(!ui.ctx().debug_on_hover());
                    }
                });
            });

            if self.selected != 0 {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.separator();
                ui.vertical(|ui| {
                    egui::ScrollArea::both().show(ui, |ui| {
                        let index = self.selected - 1;
                        self.tabs[index].ui(ui, app);
                        ui.allocate_space(ui.available_size());
                    });
                });
            }
        });

        // ui.set_max_width(ui.max_rect().right());
        // println!("{}", ui.max_rect().right());
        // ui.spacing_mut().item_spacing.x = 0.0;
        //                 ui.horizontal(|ui| {
        //                     // ui.spacing_mut().item_spacing.x = 0.0;
        //                     //egui::SidePanel::left("left_icon_panel").show(ui.ctx(), |ui|{

        //                     // ui.vertical(|ui| {
        //                     //     ui.spacing_mut().item_spacing.x = 0.0;
        //                     //     if ui.selectable_label(self.visible && self.selected == 1, "💻").clicked() {
        //                     //         if self.selected == 1{
        //                     //             self.visible = false;
        //                     //             self.selected = 0;
        //                     //         }else{
        //                     //             self.visible = true;
        //                     //             self.selected = 1;
        //                     //         }
        //                     //     }
        //                     //     ui.spacing_mut().item_spacing.x = 0.0;
        //                     //     if ui.selectable_label(self.visible && self.selected == 2, "🖹").clicked() {
        //                     //         if self.selected == 2{
        //                     //             self.visible = false;
        //                     //             self.selected = 0;
        //                     //         }else{
        //                     //             self.visible = true;
        //                     //             self.selected = 2;
        //                     //         }
        //                     //     }
        //                     //     //println!("{}, {}, {}", max_height, min_height, ui.max_rect().bottom());
        //                     //     ui.add_space(max_height - ui.max_rect().bottom() - 3.0);
        //                     //     ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        //                     //         ui.set_max_height(max_height);
        //                     //         if ui.selectable_label(false, "⚙").clicked() {}
        //                     //         if ui
        //                     //             .selectable_label(ui.ctx().debug_on_hover(), "🐛")
        //                     //             .on_hover_text("debug on hover")
        //                     //             .clicked()
        //                     //         {
        //                     //             ui.ctx().set_debug_on_hover(!ui.ctx().debug_on_hover());
        //                     //         }
        //                     //     });
        //                     //     //    ui.horizontal(|ui| {
        //                     //     //ui.spacing_mut().item_spacing.y = 500.0;
        //                     //     //ui.add_space(500.0);
        //                     //     //ui.separator();

        //                     //     //    });
        //                     //     //});
        //                     // });

        //                     if self.selected != 0 {

        //                         ui.spacing_mut().item_spacing.x = 0.0;
        //                         ui.separator();

        //                         ui.vertical(|ui| {
        //                             egui::ScrollArea::both().show(ui, |ui| {
        //                                 if self.selected == 1 {
        //                                     ui.heading("CPU control");

        //                                     //ui.horizontal(|ui| {
        //                                     //    ui.label("Write something: ");
        //                                     //    ui.text_edit_singleline(label);
        //                                     //});

        //                                     let (pc, hi, lo, reg) = {
        //                                         (
        //                                             app.cpu.get_pc(),
        //                                             app.cpu.get_hi_register(),
        //                                             app.cpu.get_lo_register(),
        //                                             app.cpu.get_general_registers(),
        //                                         )
        //                                     };

        //                                     ui.horizontal(|ui| {
        //                                         ui.label("PC: ");
        //                                         ui.label(pc.to_string());
        //                                     });
        //                                     ui.horizontal(|ui| {
        //                                         ui.label("Hi: ");
        //                                         ui.label(hi.to_string());
        //                                     });
        //                                     ui.horizontal(|ui| {
        //                                         ui.label("Lo: ");
        //                                         ui.label(lo.to_string());
        //                                     });
        //                                     ui.label("Reg: ");
        //                                     let mut i = 0usize;
        //                                     while i < 32 {
        //                                         ui.horizontal(|ui| {
        //                                             ui.label(format!(" {}: {}", i, reg[i]));
        //                                             i += 1;
        //                                             ui.label(format!(" {}: {}", i, reg[i]));
        //                                             i += 1;
        //                                         });
        //                                     }
        //                                     //ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
        //                                     if ui.button("Start CPU").clicked() {
        //                                         unsafe {
        //                                             // static mut CPU: Option<MipsCpu> = Option::None;
        //                                             // let cpu = CPU.get_or_insert_with(||{
        //                                             //     MipsCpu::new()
        //                                             // });
        //                                             if app.cpu.is_running() {
        //                                                 log::warn!("CPU is already running");
        //                                             } else {
        //                                                 log::info!("CPU Starting");
        //                                                 let cpu: &'static mut mips_emulator::cpu::MipsCpu =
        //                                                     std::mem::transmute(app.cpu.as_mut());

        //                                                 cpu.start_new_thread();
        //                                             }
        //                                         }
        //                                     }
        //                                     if ui.button("Step CPU").clicked() {
        //                                         unsafe {
        //                                             if app.cpu.is_running() {
        //                                                 log::warn!("CPU is already running");
        //                                             } else {
        //                                                 let cpu: &'static mut mips_emulator::cpu::MipsCpu =
        //                                                     std::mem::transmute(app.cpu.as_mut());
        //                                                 cpu.step_new_thread();
        //                                             }
        //                                         }
        //                                     }

        //                                     if ui.button("Stop CPU").clicked() {
        //                                         if app.cpu.is_running() {
        //                                             app.cpu.stop();
        //                                             log::info!("Stopping CPU");
        //                                         } else {
        //                                             log::warn!("CPU is already stopped");
        //                                         }
        //                                     }
        //                                     if ui.button("Pause CPU").clicked() {
        //                                         if app.cpu.paused_or_stopped(){
        //                                             log::warn!("CPU is already paused");
        //                                         }else{
        //                                             app.cpu.pause();
        //                                             log::info!("CPU is paused");
        //                                         }
        //                                     }
        //                                     if ui.button("Resume CPU").clicked() {
        //                                         if app.cpu.paused_or_stopped(){
        //                                             app.cpu.resume();
        //                                             log::info!("CPU resumed");
        //                                         }else{
        //                                             log::warn!("CPU is already resumed");
        //                                         }
        //                                     }
        //                                     if ui.button("Reset CPU").clicked() {
        //                                         if !app.cpu.is_running() {
        //                                             //runs 2^16 * (2^15-1)*3+2 instructions (6442254338)
        //                                             //the version written in c++ seems to be around 17% faster
        //                                             //[0x64027FFFu32, 0x00000820, 0x20210001, 0x10220001, 0x0BFFFFFD, 0x68000000][(self.pc >> 2) as usize];//

        //                                             app.cpu.clear();

        //                                             let f = std::fs::File::open("/home/may/Documents/GitHub/OxidizedMips/mips/bin/tmp.bin").unwrap();

        //                                             let mut reader = std::io::BufReader::new(f);
        //                                             let mut buffer = Vec::new();

        //                                             // Read file into vector.

        //                                             let _size = std::io::Read::read_to_end(&mut reader, &mut buffer).unwrap();

        //                                             let test_prog = buffer.as_mut_slice();
        //                                             // for i in 0..(size / 4){
        //                                             //     let base = i * 4;
        //                                             //     let b1 = test_prog[base];
        //                                             //     let b2 = test_prog[base + 1];

        //                                             //     // test_prog[base] = test_prog[base + 3];
        //                                             //     // test_prog[base + 1] = test_prog[base + 2];
        //                                             //     // test_prog[base + 3] = b1;
        //                                             //     // test_prog[base + 2] = b2;
        //                                             // }

        //                                             //  let test_prog = &[
        //                                             //     0x64027FFFu32.to_be(),
        //                                             //     0x00000820u32.to_be(),
        //                                             //     0x20210001u32.to_be(),
        //                                             //     0x10220001u32.to_be(),
        //                                             //     0x0BFFFFFDu32.to_be(),
        //                                             //     0x68000000u32.to_be(),
        //                                             //  ];
        //                                             app.cpu.get_mem().copy_into_raw(0, test_prog);

        //                                             log::info!("reset CPU");
        //                                         } else {
        //                                             log::warn!("Cannot reset CPU while running");
        //                                         }
        //                                     }
        //                                 } else if self.selected == 2{
        //                                     ui.add(
        //                                         egui::Label::new(egui::RichText::new("Workspace").heading()).wrap(false)
        //                                     );
        //                                     ui.collapsing("info", |ui|{
        //                                         ui.label("current workspace files ext(just the current directory of the exe for now)");
        //                                         ui.label("note opening files will only read them and never save to them currently");
        //                                     });
        //                                     ui.separator();

        //                                     generate_tree(".".into(),app, ui);

        //                                     fn generate_tree(path: std::path::PathBuf, t: &mut crate::ClikeGui,  ui: &mut egui::Ui){
        //                                         match std::fs::read_dir(path) {
        //                                             Ok(val) => {

        //                                                 let mut test: Vec<Result<std::fs::DirEntry, std::io::Error>> = val.collect();
        //                                                 test.sort_by(|t1, t2| {
        //                                                     if let Result::Ok(t1) = t1{
        //                                                         if let Result::Ok(t2) = t2{
        //                                                             //let t1 = t1.unwrap();
        //                                                             //let t2 = t2.unwrap();
        //                                                             let t1d = t1.metadata().unwrap().is_dir();
        //                                                             let t2d = t2.metadata().unwrap().is_dir();
        //                                                             if t1d && t2d {
        //                                                                 return t1.file_name().to_ascii_lowercase().to_str().unwrap()
        //                                                                 .cmp(t2.file_name().to_ascii_lowercase().to_str().unwrap())
        //                                                             }else if t1d{
        //                                                                 return std::cmp::Ordering::Less
        //                                                             }else if t2d{
        //                                                                 return std::cmp::Ordering::Greater
        //                                                             }else{
        //                                                                 return t1.file_name().to_ascii_lowercase().to_str().unwrap()
        //                                                                 .cmp(t2.file_name().to_ascii_lowercase().to_str().unwrap())
        //                                                             }
        //                                                         }
        //                                                     }
        //                                                     std::cmp::Ordering::Equal
        //                                                 });
        //                                                 for d in test{
        //                                                     if let Result::Ok(val) = d{

        //                                                         if val.metadata().unwrap().is_dir(){
        //                                                             ui.collapsing(val.file_name().into_string().unwrap(), |ui|{
        //                                                                 generate_tree(val.path(),t, ui);
        //                                                             });
        //                                                         }else{
        //                                                             if ui.selectable_label(false, val.file_name().into_string().unwrap()).clicked(){

        //                                                                 if let Result::Ok(str) = std::fs::read_to_string(val.path()){

        //                                                                     t.tabbed_area.add_tab(Box::new(crate::tabs::code_editor::CodeEditor::new(val.file_name().into_string().unwrap(), str)));
        //                                                                 }
        //                                                                 log::info!("loaded file: {}", val.path().display());
        //                                                             }

        //                                                         }
        //                                                     }
        //                                                 }
        //                                             },
        //                                             Err(_) => {

        //                                             },
        //                                         }
        //                                     }
        //                                 }
        //                             });
        //                             ui.allocate_space(ui.available_size());
        //                         });
        //                     }
        //                 });
    }

    pub fn is_visible(&self) -> bool {
        self.selected != 0
    }
}

impl Default for SideTabbedPanel {
    fn default() -> Self {
        let mut tmp = Self {
            selected: 1,
            tabs: Default::default(),
        };
        tmp.tabs.push(CPUSidePanel::new().into());
        tmp.tabs.push(ProjectSidePanel::default().into());
        tmp
    }
}
