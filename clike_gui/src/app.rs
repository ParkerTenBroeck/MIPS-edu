use eframe::{egui, epi};
use mips_emulator::cpu::MipsCpu;

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

pub struct ApplicationSettings{

}

#[allow(dead_code)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct ClikeGui {
    // Example stuff:
    settings: ApplicationSettings,
    code: String,
    // this how you opt-out of serialization of a member
    //#[cfg_attr(feature = "persistence", serde(skip))]
    //value: f32,
    #[cfg_attr(feature = "persistence", serde(skip))]
    cpu: MipsCpu,
}

impl Default for ClikeGui {
    fn default() -> Self {

        let settings = ApplicationSettings{};

        Self {
            settings,
            // Example stuff:
            code:
r#"
/// Outer block single line documentation
/**
	/*
		ps(you can have /*!BLOCKS*/ /**inside*/ blocks)
	*/
	Outer block multiline documentation
*/
fn test(){
	println!("dont change a thing! {}", "you are amazing ;)");
	let r#fn = test;
	let number = 12 + 2.3e-2;

	//! some inner documentation
	let boolean = false;

	/*!
		Outer block multiline documentation
	*/
	for(i: i32, i < 50; i += 2){
		println!("hello for the {} time!", i);
	}

	//this is a comment(crazy right)
	/*
		block comment
		this one goes on for a while
	*/
}
"#.into(),
            cpu: MipsCpu::new(),
        }
    }
}
/*
/// Outer block single line documentation
/**
	/*
		ps(you can have /*!BLOCKS*/ /**inside*/ blocks)
	*/
	Outer block multiline documentation
*/
fn test(){
	println!("dont change a thing! {}", "you are amazing ;)");
	let r#fn = test;
	let number = 12 + 2.3e-2;

	//! some inner documentation
	let boolean = false;

	/*!
		Outer block multiline documentation
	*/
	for(i: i32, i < 50; i += 2){
		println!("hello for the {} time!", i);
	}

	//this is a comment(crazy right)
	/*
		block comment
		this one goes on for a while
	*/
}
 */

static mut MIPS_CPU: Option<MipsCpu> = Option::None;

impl epi::App for ClikeGui {
    fn name(&self) -> &str {
        "CLike"
    }


    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        unsafe{
            MIPS_CPU = Option::Some(MipsCpu::new());
        }
        
        #[cfg(target_arch = "wasm32")]
        wasm_logger::init(wasm_logger::Config::default());
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        let Self {code,.. } = self;
        //let mut val6 = 1f32;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("CPU control");


            //ui.horizontal(|ui| {
            //    ui.label("Write something: ");
            //    ui.text_edit_singleline(label);
            //});
            let cpu = unsafe {MIPS_CPU.as_mut().unwrap()};

            let (pc,hi,lo,reg) = {
                (cpu.get_pc(),cpu.get_hi_register(),cpu.get_lo_register(),cpu.get_general_registers())
            };
            if cpu.is_running() && !cpu.is_paused() {
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
            while i < 32{
                ui.horizontal(|ui| {
                    ui.label(format!(" {}: {}",i,reg[i]));
                    i += 1;
                    ui.label(format!(" {}: {}",i,reg[i]));
                    i += 1;
                });
            }

            //ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Start CPU").clicked() {

                unsafe{
                    if MIPS_CPU.as_mut().unwrap().is_running(){
                        println!("CPU is already running");
                    }else{
                        println!("CPU Starting");
                        let cpu: &'static mut MipsCpu = std::mem::transmute(MIPS_CPU.as_mut().unwrap());
                        cpu.start_new_thread();
                    }
                }
            }
            if ui.button("Step CPU").clicked() {
                unsafe{
                    if MIPS_CPU.as_mut().unwrap().is_running(){
                        println!("CPU is already running");
                    }else{
                        let cpu: &'static mut MipsCpu = std::mem::transmute(MIPS_CPU.as_mut().unwrap());
                        cpu.step_new_thread();
                    }
                }
            }

            if ui.button("Stop CPU").clicked() {
                unsafe{
                    if MIPS_CPU.as_mut().unwrap().is_running(){
                        MIPS_CPU.as_mut().unwrap().stop();
                        println!("Stopping CPU");
                    }else{
                        println!("CPU is already stopped");
                    }
                }
            }
            if ui.button("Pause CPU").clicked() {
                unsafe{
                    MIPS_CPU.as_mut().unwrap().pause();
                    println!("CPU is paused");
                }
            }
            if ui.button("Resume CPU").clicked() {
                unsafe{
                    MIPS_CPU.as_mut().unwrap().resume();
                    println!("CPU resumed");
                }
            }
            if ui.button("Reset CPU").clicked() {
                unsafe{
                    if !MIPS_CPU.as_mut().unwrap().is_running(){
                        //runs 2^16 * (2^15-1)*3+2 instructions (6442254338)
                        //the version written in c++ seems to be around 17% faster
                        //[0x64027FFFu32, 0x00000820, 0x20210001, 0x10220001, 0x0BFFFFFD, 0x68000000][(self.pc >> 2) as usize];//

                        MIPS_CPU.as_mut().unwrap().clear();

                        let test_prog = [0x64027FFFu32, 0x00000820, 0x20210001, 0x10220001, 0x0BFFFFFD, 0x68000000];//
                        MIPS_CPU.as_mut().unwrap().get_mem_mut().copy_into_raw(0, &test_prog);
                        
                        println!("reset CPU");
                    }else{
                        println!("Cannot reset CPU while running");
                    }
                }
            }
/*
            ui.horizontal(|ui| {
                ui.text_edit_multiline(code).context_menu(|ui| {
                    ui.menu_button("Plot", |ui| {
                        if ui.radio_value(value, 2f32, "2").clicked()
                            || ui
                            .radio_value(value, 3f32, "3")
                            .clicked()
                            || ui
                            .radio_value(value, 4.5f32, "4.5")
                            .clicked()
                        {
                            ui.close_menu();
                        }
                    });
                    egui::Grid::new("button_grid").show(ui, |ui| {
                        ui.add(
                            egui::DragValue::new(value)
                                .speed(1.0)
                                .prefix("Width:"),
                        );
                        ui.add(
                            egui::DragValue::new(value)
                                .speed(1.0)
                                .prefix("Height:"),
                        );
                        ui.end_row();
                    });
                });
            });

 */

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.hyperlink_to("CLike", "https://github.com/ParkerTenBroeck/CLike");
                });
            });
        });



        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("Code Editor");
            egui::warn_if_debug_build(ui);

            let mut theme = crate::syntax_highlighter::CodeTheme::from_memory(ui.ctx());
            ui.collapsing("Theme", |ui| {
                ui.group(|ui| {
                    theme.ui(ui);
                    theme.clone().store_in_memory(ui.ctx());
                });
            });

            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job = crate::syntax_highlighter::highlight(ui.ctx(), &theme, string, "rs");
                layout_job.wrap_width = wrap_width;
                ui.fonts().layout_job(layout_job)
            };

            egui::ScrollArea::vertical().show(ui, |ui| {

                ui.add(
                    egui::TextEdit::multiline(code)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        //.desired_rows(10)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                )
                //.on_hover_ui_at_pointer(|ui| {
                //});
            });
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
