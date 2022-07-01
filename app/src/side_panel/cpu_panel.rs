use mips_emulator::memory::page_pool::MemoryDefault;

use super::side_tabbed_panel::SideTab;

#[derive(PartialEq)]
enum IntegerFormat{
    SignedBase10,
    UnsignedBase10,
    Base16,
    Base2
}

#[derive(PartialEq)]
enum FloatFormat{
    Base10,
    Base16,
    Base2,
}

pub struct CPUSidePanel {
    int_format: IntegerFormat,
    float_foramt: FloatFormat,
    use_reg_names: bool,
}

impl CPUSidePanel {
    pub fn new() -> Self {
        Self {
            int_format: IntegerFormat::SignedBase10,
            float_foramt: FloatFormat::Base10,
            use_reg_names: true,
        }
    }
}

impl From<CPUSidePanel> for Box<dyn SideTab> {
    fn from(panel: CPUSidePanel) -> Self {
        Box::new(panel)
    }
}

impl CPUSidePanel{
    pub fn u32_to_str(&self, val: u32) -> String{
        match self.int_format{
            IntegerFormat::SignedBase10 => format!("{}", val),
            IntegerFormat::UnsignedBase10 => format!("{}", val as i32),
            IntegerFormat::Base16 => format!("0x{:08X}", val),
            IntegerFormat::Base2 => format!("0b{:032b}", val),
        }
    }
    pub fn u64_to_str(&self, val: u64) -> String{
        match self.int_format{
            IntegerFormat::SignedBase10 => format!("{}", val),
            IntegerFormat::UnsignedBase10 => format!("{}", val as i64),
            IntegerFormat::Base16 => format!("0x{:016X}", val),
            IntegerFormat::Base2 => format!("0b{:064b}", val),
        }
    }
    pub fn f32_to_str(&self, val: f32) -> String{
        match self.float_foramt{
            FloatFormat::Base10 => format!("{}", val),
            FloatFormat::Base16 => format!("0x{:08X}", val as u32),
            FloatFormat::Base2 => format!("0b{:032b}", val as u32),
        }
    }
    pub fn f64_to_str(&self, val: f64) -> String{
        match self.float_foramt{
            FloatFormat::Base10 => format!("{}", val),
            FloatFormat::Base16 => format!("0x{:016X}", val as u64),
            FloatFormat::Base2 => format!("0b{:064b}", val as u64),
        }
    }
    pub fn fmt_reg(&self, reg: usize) -> &'static str{
        if self.use_reg_names{
            ["zero","$at","$v0","$v1","$a0","$a1","$a2","$a3","$t0","$t1","$t2","$t3","$t4","$t5","$t6","$t7",
             "$s0","$s1","$s2","$s3","$s4","$s5","$s6","$s7","$t8","$t9","$k0","$k1","$gp","$sp","$fp","$ra"][reg]
        }else{
            ["$0","$1","$2","$3","$4","$5","$6","$7","$8","$9","$10","$11","$12","$13","$14","$15",
             "$17","$18","$19","$20","$21","$22","$23","$24","$25","$26","$27","$28","$29","$30","$31"][reg]
        }
    }
}


impl SideTab for CPUSidePanel {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, app: &mut crate::Application) {
        ui.horizontal(|ui|{
            ui.heading("CPU/Debug Info");
            ui.add_space(5.0);

            ui.menu_button("...", |ui|{
                ui.menu_button("Integer Format", |ui|{
                    let val = &mut self.int_format;
                    let mut pressed = ui.selectable_value(val, IntegerFormat::SignedBase10, "Signed").clicked();
                    pressed |= ui.selectable_value(val, IntegerFormat::UnsignedBase10, "Unsigned").clicked();
                    pressed |= ui.selectable_value(val, IntegerFormat::Base16, "Hex").clicked();
                    pressed |=ui.selectable_value(val, IntegerFormat::Base2, "Binary").clicked();    
                    if pressed{
                        ui.close_menu()
                    }
                });

                ui.menu_button("Float Format", |ui|{
                    let val = &mut self.float_foramt;
                    let mut pressed = ui.selectable_value(val, FloatFormat::Base10, "Signed").clicked();
                    pressed |= ui.selectable_value(val, FloatFormat::Base16, "Hex").clicked();
                    pressed |=ui.selectable_value(val, FloatFormat::Base2, "Binary").clicked();    
                    if pressed{
                        ui.close_menu()
                    }
                });

                ui.menu_button("Register Format", |ui|{
                    let val = &mut self.use_reg_names;
                    let mut pressed = ui.selectable_value(val, true, "Name").clicked();
                    pressed |= ui.selectable_value(val, false, "Number").clicked();
                    if pressed{
                        ui.close_menu()
                    }
                });
            });
        });



        let (pc, hi, lo, reg) = {
            (
                app.cpu.get_pc(),
                app.cpu.get_hi_register(),
                app.cpu.get_lo_register(),
                *app.cpu.get_general_registers(),
            )
        };



        macro_rules! register_lable {
            ($ui:expr, $reg:expr) => {
                $ui.label(format!("{}: {}", self.fmt_reg($reg), self.u32_to_str(reg[$reg])));
            };
        }

        //ui.horizontal(|ui| {
            ui.collapsing("GP Registers", |ui|{
                ui.vertical(|ui|{
                    ui.horizontal(|ui| {
                        ui.label("PC: ");
                        ui.label(self.u32_to_str(pc));
                    });
                    ui.collapsing("Hi/Lo Registers", |ui|{
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
                    ui.collapsing("Arguments", |ui|{
                        register_lable!(ui, 4);
                        register_lable!(ui, 5);
                        register_lable!(ui, 6);
                        register_lable!(ui, 7);
                    });
                    ui.collapsing("Return", |ui|{
                        register_lable!(ui, 2);
                        register_lable!(ui, 3);
                    });
                    ui.collapsing("Temporary", |ui|{
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
                    ui.collapsing("Fn Variables", |ui|{
                        register_lable!(ui, 16);
                        register_lable!(ui, 17);
                        register_lable!(ui, 18);
                        register_lable!(ui, 19);
                        register_lable!(ui, 20);
                        register_lable!(ui, 21);
                        register_lable!(ui, 22);
                        register_lable!(ui, 23);
                    });
                    ui.collapsing("Kernel", |ui|{
                        register_lable!(ui, 26);
                        register_lable!(ui, 27);
                    });
                    ui.collapsing("Special", |ui|{
                        register_lable!(ui, 0);
                        register_lable!(ui, 1);
                        register_lable!(ui, 28);
                        register_lable!(ui, 29);
                        register_lable!(ui, 30);
                        register_lable!(ui, 31);
                    });
                    ui.collapsing("All", |ui|{
                        for i in 0..32{
                            ui.label(format!(" {}: {}", self.fmt_reg(i), self.u32_to_str(reg[i])));
                        }
                    });
                });
            });
        //});



        //ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
        if ui.button("Start CPU").clicked() {
            unsafe {
                // static mut CPU: Option<MipsCpu> = Option::None;
                // let cpu = CPU.get_or_insert_with(||{
                //     MipsCpu::new()
                // });
                if app.cpu.is_running() {
                    log::warn!("CPU is already running");
                } else {
                    log::info!("CPU Starting");
                    let cpu: &'static mut mips_emulator::cpu::MipsCpu =
                        std::mem::transmute(app.cpu.as_mut());

                    cpu.start_new_thread();
                }
            }
        }
        if ui.button("Step CPU").clicked() {
            unsafe {
                if app.cpu.is_running() {
                    log::warn!("CPU is already running");
                } else {
                    let cpu: &'static mut mips_emulator::cpu::MipsCpu =
                        std::mem::transmute(app.cpu.as_mut());
                    cpu.step_new_thread();
                }
            }
        }

        if ui.button("Stop CPU").clicked() {
            if app.cpu.is_running() {
                app.cpu.stop();
                log::info!("Stopping CPU");
            } else {
                log::warn!("CPU is already stopped");
            }
        }
        if ui.button("Pause CPU").clicked() {
            if app.cpu.paused_or_stopped() {
                log::warn!("CPU is already paused");
            } else {
                app.cpu.pause();
                log::info!("CPU is paused");
            }
        }
        if ui.button("Resume CPU").clicked() {
            if app.cpu.paused_or_stopped() {
                app.cpu.resume();
                log::info!("CPU resumed");
            } else {
                log::warn!("CPU is already resumed");
            }
        }
        if ui.button("Reset CPU").clicked() {
            app.cpu.stop_and_wait();
            if !app.cpu.is_running() {
                app.cpu.clear();

                let f =
                    std::fs::File::open("/home/may/Documents/GitHub/OxidizedMips/mips/bin/tmp.bin")
                        .unwrap();

                let mut reader = std::io::BufReader::new(f);
                let mut buffer = Vec::new();

                // Read file into vector.

                let _size = std::io::Read::read_to_end(&mut reader, &mut buffer).unwrap();

                let test_prog = buffer.as_mut_slice();

                app.cpu.get_mem().copy_into_raw(0, test_prog);

                log::info!("reset CPU");
            } else {
                log::warn!("Cannot reset CPU while running");
            }
        }
    }

    fn get_icon(&mut self) {
        todo!()
    }
}

