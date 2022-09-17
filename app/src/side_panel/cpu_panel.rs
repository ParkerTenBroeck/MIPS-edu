use std::{
    fs::File,
    io::{BufReader, Read},
};

use eframe::egui::WidgetText;
use mips_emulator::memory::{
    page_pool::PagedMemoryInterface, single_cached_memory::SingleCachedMemory,
};

use crate::platform::sync::PlatSpecificLocking;

use super::side_tabbed_panel::SideTab;

#[derive(PartialEq)]
enum IntegerFormat {
    SignedBase10,
    UnsignedBase10,
    Base16,
    Base2,
}

#[derive(PartialEq)]
enum FloatFormat {
    Base10,
    Base16,
    Base2,
}

pub struct CPUSidePanel {
    int_format: IntegerFormat,
    float_foramt: FloatFormat,
    use_reg_names: bool,
    thing: Vec<(u128, u64)>,
}

impl Default for CPUSidePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl CPUSidePanel {
    pub fn new() -> Self {
        Self {
            int_format: IntegerFormat::SignedBase10,
            float_foramt: FloatFormat::Base10,
            use_reg_names: true,
            thing: Default::default(),
        }
    }
}

impl From<CPUSidePanel> for Box<dyn SideTab> {
    fn from(panel: CPUSidePanel) -> Self {
        Box::new(panel)
    }
}

impl CPUSidePanel {
    pub fn u32_to_str(&self, val: u32) -> String {
        match self.int_format {
            IntegerFormat::SignedBase10 => format!("{}", val as i32),
            IntegerFormat::UnsignedBase10 => format!("{}", val),
            IntegerFormat::Base16 => format!("0x{:08X}", val),
            IntegerFormat::Base2 => format!("0b{:032b}", val),
        }
    }
    pub fn u64_to_str(&self, val: u64) -> String {
        match self.int_format {
            IntegerFormat::SignedBase10 => format!("{}", val as i64),
            IntegerFormat::UnsignedBase10 => format!("{}", val),
            IntegerFormat::Base16 => format!("0x{:016X}", val),
            IntegerFormat::Base2 => format!("0b{:064b}", val),
        }
    }
    pub fn f32_to_str(&self, val: f32) -> String {
        match self.float_foramt {
            FloatFormat::Base10 => format!("{}", val),
            FloatFormat::Base16 => format!("0x{:08X}", unsafe {
                core::mem::transmute::<[u8; 4], u32>(val.to_ne_bytes())
            }),
            FloatFormat::Base2 => format!("0b{:032b}", unsafe {
                core::mem::transmute::<[u8; 4], u32>(val.to_ne_bytes())
            }),
        }
    }
    pub fn f64_to_str(&self, val: f64) -> String {
        match self.float_foramt {
            FloatFormat::Base10 => format!("{}", val),
            FloatFormat::Base16 => format!("0x{:016X}", unsafe {
                core::mem::transmute::<[u8; 8], u64>(val.to_ne_bytes())
            }),
            FloatFormat::Base2 => format!("0b{:064b}", unsafe {
                core::mem::transmute::<[u8; 8], u64>(val.to_ne_bytes())
            }),
        }
    }
    pub fn fmt_reg(&self, reg: usize) -> &'static str {
        if self.use_reg_names {
            [
                "zero", "$at", "$v0", "$v1", "$a0", "$a1", "$a2", "$a3", "$t0", "$t1", "$t2",
                "$t3", "$t4", "$t5", "$t6", "$t7", "$s0", "$s1", "$s2", "$s3", "$s4", "$s5", "$s6",
                "$s7", "$t8", "$t9", "$k0", "$k1", "$gp", "$sp", "$fp", "$ra",
            ][reg]
        } else {
            [
                "$0", "$1", "$2", "$3", "$4", "$5", "$6", "$7", "$8", "$9", "$10", "$11", "$12",
                "$13", "$14", "$15", "$17", "$18", "$19", "$20", "$21", "$22", "$23", "$24", "$25",
                "$26", "$27", "$28", "$29", "$30", "$31",
            ][reg]
        }
    }
}

impl SideTab for CPUSidePanel {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, app: &mut crate::Application) {
        ui.horizontal(|ui| {
            ui.heading("CPU/Debug Info");
            ui.add_space(5.0);

            ui.menu_button("...", |ui| {
                ui.menu_button("Integer Format", |ui| {
                    let val = &mut self.int_format;
                    let mut pressed = ui
                        .selectable_value(val, IntegerFormat::SignedBase10, "Signed")
                        .clicked();
                    pressed |= ui
                        .selectable_value(val, IntegerFormat::UnsignedBase10, "Unsigned")
                        .clicked();
                    pressed |= ui
                        .selectable_value(val, IntegerFormat::Base16, "Hex")
                        .clicked();
                    pressed |= ui
                        .selectable_value(val, IntegerFormat::Base2, "Binary")
                        .clicked();
                    if pressed {
                        ui.close_menu()
                    }
                });

                ui.menu_button("Float Format", |ui| {
                    let val = &mut self.float_foramt;
                    let mut pressed = ui
                        .selectable_value(val, FloatFormat::Base10, "Signed")
                        .clicked();
                    pressed |= ui
                        .selectable_value(val, FloatFormat::Base16, "Hex")
                        .clicked();
                    pressed |= ui
                        .selectable_value(val, FloatFormat::Base2, "Binary")
                        .clicked();
                    if pressed {
                        ui.close_menu()
                    }
                });

                ui.menu_button("Register Format", |ui| {
                    let val = &mut self.use_reg_names;
                    let mut pressed = ui.selectable_value(val, true, "Name").clicked();
                    pressed |= ui.selectable_value(val, false, "Number").clicked();
                    if pressed {
                        ui.close_menu()
                    }
                });
            });
        });

        let (pc, hi, lo, reg, running, instructions_ran) = unsafe {
            let cpu = app.cpu.raw_cpu();
            (
                (*cpu).pc(),
                (*cpu).hi(),
                (*cpu).lo(),
                *(*cpu).reg(),
                (*cpu).is_running(),
                (*cpu).instructions_ran(),
            )
        };

        macro_rules! register_lable {
            ($ui:expr, $reg:expr) => {
                $ui.label(format!(
                    "{}: {}",
                    self.fmt_reg($reg),
                    self.u32_to_str(reg[$reg])
                ));
            };
        }

        ui.label(format!("instructions ran: {}", instructions_ran));

        let ins_p_s;

        if running {
            self.thing.push((
                crate::platform::time::duration_since_epoch().as_nanos(),
                instructions_ran,
            ));
            if self.thing.len() > 60 {
                self.thing.remove(0);
            }
            let start = self.thing[0];
            let end = *self.thing.last().unwrap();
            if let Option::Some(val) =
                ((end.1 - start.1) * 1000000000).checked_div((end.0 - start.0) as u64)
            {
                ins_p_s = val;
            } else {
                ins_p_s = 0;
            }
        } else {
            self.thing.clear();
            ins_p_s = 0;
        }
        ui.label(format!("Instructions/Second: {}", ins_p_s));

        //ui.horizontal(|ui| {
        ui.collapsing("GP Registers", |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("PC: ");
                    ui.label(self.u32_to_str(pc));
                });
                ui.collapsing("Hi/Lo Registers", |ui| {
                    // ui.vertical(|ui|{
                    ui.horizontal(|ui| {
                        ui.label("Hi: ");
                        ui.label(self.u32_to_str(hi));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Lo: ");
                        ui.label(self.u32_to_str(lo));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Combined: ");
                        ui.label(self.u64_to_str(lo as u64 | ((hi as u64) << 32)));
                    });
                    // });
                });
                ui.collapsing("Arguments", |ui| {
                    register_lable!(ui, 4);
                    register_lable!(ui, 5);
                    register_lable!(ui, 6);
                    register_lable!(ui, 7);
                });
                ui.collapsing("Return", |ui| {
                    register_lable!(ui, 2);
                    register_lable!(ui, 3);
                });
                ui.collapsing("Temporary", |ui| {
                    register_lable!(ui, 8);
                    register_lable!(ui, 9);
                    register_lable!(ui, 10);
                    register_lable!(ui, 11);
                    register_lable!(ui, 12);
                    register_lable!(ui, 13);
                    register_lable!(ui, 14);
                    register_lable!(ui, 15);
                    register_lable!(ui, 24);
                    register_lable!(ui, 25);
                });
                ui.collapsing("Fn Variables", |ui| {
                    register_lable!(ui, 16);
                    register_lable!(ui, 17);
                    register_lable!(ui, 18);
                    register_lable!(ui, 19);
                    register_lable!(ui, 20);
                    register_lable!(ui, 21);
                    register_lable!(ui, 22);
                    register_lable!(ui, 23);
                });
                ui.collapsing("Kernel", |ui| {
                    register_lable!(ui, 26);
                    register_lable!(ui, 27);
                });
                ui.collapsing("Special", |ui| {
                    register_lable!(ui, 0);
                    register_lable!(ui, 1);
                    register_lable!(ui, 28);
                    register_lable!(ui, 29);
                    register_lable!(ui, 30);
                    register_lable!(ui, 31);
                });
                ui.collapsing("All", |ui| {
                    for (i, reg) in reg.iter().enumerate() {
                        ui.label(format!(" {}: {}", self.fmt_reg(i), self.u32_to_str(*reg)));
                    }
                });
            });
        });
        //});

        if ui.button("Start CPU").clicked() {
            if {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    app.cpu.start_new_thread()
                }
                #[cfg(target_arch = "wasm32")]
                {
                    app.cpu.start(|inner| {
                        let _ = crate::platform::thread::start_thread(move || {
                            inner();
                        });
                    })
                }
            }
            .is_ok()
            {
                log::info!("CPU Started");
            } else {
                log::warn!("CPU is already running");
            }
            ui.ctx().request_repaint();
        }
        if ui.button("Step CPU").clicked() {
            if {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    app.cpu.step_new_thread()
                }
                #[cfg(target_arch = "wasm32")]
                {
                    app.cpu.step(|inner| {
                        let _ = crate::platform::thread::start_thread(move || {
                            inner();
                        });
                    })
                }
            }
            .is_ok()
            {
                log::info!("CPU Started");
            } else {
                log::warn!("CPU is already running");
            }
            ui.ctx().request_repaint();
        }

        if ui.button("Stop CPU").clicked() {
            if app.cpu.stop().is_ok() {
                log::info!("Stopped CPU");
                ui.ctx().request_repaint();
            } else {
                log::warn!("CPU is already stopped");
            }
        }
        if ui.button("Reset CPU").clicked() {
            let _ = app.cpu.stop();
            if app.cpu.restart().is_ok() {
                log::info!("reset CPU");
                ui.ctx().request_repaint();
            } else {
                log::warn!("Cannot reset CPU while running");
            }
        }
        if ui.button("Load Demo 1").clicked() {
            let _ = app.cpu.stop();
            app.cpu.cpu_mut(|cpu| {
                cpu.clear();

                let mut test_prog = [
                    0x3C027FFFu32,
                    0x00000820,
                    /* 0x0AC01001C, */ 0x20210001,
                    0x10220001,
                    0x08000002,
                    0x0000000C,
                ];
                for mem in test_prog.iter_mut() {
                    *mem = mem.to_be();
                }
                unsafe {
                    cpu.get_mem::<SingleCachedMemory>()
                        .copy_into_raw(0, test_prog.as_slice());
                }
                log::info!("Loaded Demo 1 CPU");
            });
            ui.ctx().request_repaint();
        }
        if ui.button("Load Demo 2").clicked() {
            let _ = app.cpu.stop();
            app.cpu.cpu_mut(|cpu| {
                cpu.clear();

                let test_prog = include_bytes!("../../res/tmp.bin");
                unsafe {
                    cpu.get_mem::<SingleCachedMemory>()
                        .copy_into_raw(0, test_prog);
                }
                log::info!("Loaded Demo 2 CPU");
            });
            ui.ctx().request_repaint();
        }
        #[allow(clippy::collapsible_if)]
        if cfg!(debug_assertions) {
            if ui.button("Load Demo 3").clicked() {
                let _ = app.cpu.stop();
                app.cpu.cpu_mut(|cpu| {
                    cpu.clear();

                    let mut buf = Vec::new();
                    BufReader::new(
                        File::open("/home/may/Documents/GitHub/OxidizedMips/mips/bin/tmp.bin")
                            .unwrap(),
                    )
                    .read_to_end(&mut buf)
                    .unwrap();

                    let test_prog = buf.as_slice();
                    unsafe {
                        cpu.get_mem::<SingleCachedMemory>()
                            .copy_into_raw(0x10, test_prog);
                    }
                    log::info!("Loaded Demo 3 CPU");
                });
                ui.ctx().request_repaint();
            }
        }

        fn create_text(accessed: bool, text: &str) -> WidgetText {
            let mut text = eframe::egui::WidgetText::RichText(text.into());
            if accessed {
                text = text.underline();
                text = text.strong();
            }
            text
        }
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.vertical(|ui| {
                let clone = app.access_info.clone();
                let access = clone.plat_lock().unwrap();
                if ui
                    .button(create_text(access.was_terminal_accessed(), "Terminal"))
                    .clicked()
                {
                    app.add_cpu_terminal_tab();
                }
                if ui
                    .button(create_text(access.was_display_accessed(), "Display"))
                    .clicked()
                {
                    app.add_cpu_screen_tab();
                }
                if ui
                    .button(create_text(access.was_sound_accessed(), "Sound"))
                    .clicked()
                {
                    app.add_cpu_sound_tab();
                }
                if ui.button("Memory").clicked() {
                    app.add_cpu_memory_tab()
                }
            });
        });
    }

    fn get_icon(&mut self) {
        todo!()
    }
}
