use eframe::{epaint::{Color32}, egui::{self}};
use mips_emulator::memory::{single_cached_memory::SingleCachedMemory, page_pool::MemoryDefault};
use super::tabbed_area::Tab;

pub struct HexEditor {
    mem: mips_emulator::memory::page_pool::PagePoolRef<SingleCachedMemory>,
    cpu: &'static mut mips_emulator::cpu::MipsCpu,
    starting_offset: u32,
    cursor_offset: Option<(u32, bool)>,
    selection_offset: Option<u32>,
    bytes_per_line: u8,
    show_disassembly: bool,
    scroll_to_pc: bool,
    highlight_pc: bool,
    highlight_return: bool,
    highlight_frame: bool,
    highlight_stack: bool,
    highlight_global: bool,
}

impl HexEditor {
    pub fn new(cpu: &'static mut mips_emulator::cpu::MipsCpu) -> Self {

        HexEditor {
            mem: cpu.get_mem_controller().lock().unwrap().add_holder(SingleCachedMemory::new()),
            cpu,
            cursor_offset: Option::None,
            selection_offset: Option::None,
            bytes_per_line: 16,
            show_disassembly: false,
            highlight_pc: true,
            scroll_to_pc: false,
            highlight_return: true,
            highlight_frame: false,
            highlight_stack: false,
            highlight_global: false,
            starting_offset: 0,
        }
    }

    fn u8_to_display_char(input: u8) -> char{
        //let input = input as char;
        if !input.is_ascii_control(){
            return input as char;
        }
        match input{
            _ => '.'
        }
    }

    fn calculate_highlight(&self, address: u32) -> Option<Color32>{
        if self.highlight_pc{
            let val = self.cpu.get_pc();
            if val <= address && address <= (val + 3){
                return Option::Some(Color32::DARK_BLUE);
            }
        }
        if self.highlight_return{
            let val = self.cpu.get_reg(31);
            if val <= address && address <= (val + 3){
                return Option::Some(Color32::DARK_RED);
            }
        }
        if self.highlight_stack{
            let val = self.cpu.get_reg(29);
            if val <= address && address <= (val + 3){
                return Option::Some(Color32::DARK_GREEN);
            }
        }
        if self.highlight_frame{
            let val = self.cpu.get_reg(30);
            if val <= address && address <= (val + 3){
                return Option::Some(Color32::GOLD);
            }
        }
        if self.highlight_global{
            let val = self.cpu.get_reg(28);
            if val <= address && address <= (val + 3){
                return Option::Some(Color32::KHAKI);
            }
        }

        Option::None
    }
}

impl Tab for HexEditor {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.add_enabled_ui(!self.show_disassembly, |ui| {
                        ui.label("Bytes Per Row");
                        ui.add(egui::Slider::new(&mut self.bytes_per_line, 1u8..=128))
                    });
                });
                ui.vertical(|ui| {
                    if ui
                        .checkbox(&mut self.show_disassembly, "Show Disassembly")
                        .clicked()
                    {
                        if self.show_disassembly {
                            self.bytes_per_line = 4;
                        } else {
                            self.bytes_per_line = 16;
                        }
                    }
                });
                ui.collapsing("Extra Options", |ui|{
                    ui.horizontal(|ui| {
                        ui.vertical(|ui|{
                            ui.checkbox(&mut self.highlight_pc, "highlight PC");
                            ui.checkbox(&mut self.highlight_return, "highlight Return");
                            ui.checkbox(&mut self.highlight_stack, "highlight Stack");
                            ui.checkbox(&mut self.highlight_frame, "highlight Frame");
                            ui.checkbox(&mut self.highlight_global, "highlight Global");
                        });
        
                        ui.vertical(|ui|{
                            ui.checkbox(&mut self.scroll_to_pc, "Scroll to PC");
                        });
                    });
                });
            });

            ui.separator();

            //ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {

            //});
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                

                ui.horizontal(|ui| {
                    let text = egui::RichText::new(match self.cursor_offset{
                        Some(val) => {
                            let mut string = format!("offset: {:08X}", val.0);
                            string.insert(12, ':');
                            string
                        },
                        None => "offset: ----:----".into(),
                    }).monospace();
                    
                    ui.label(text);

                    let text = egui::RichText::new(match self.selection_offset{
                        Some(val) => {
                            let mut string = format!("selection: {:08X}", val);
                            string.insert(15, ':');
                            string
                        },
                        None => "selection: ----:----".into(),
                    }).monospace();
                    
                    ui.label(text);
                });
                ui.separator();

                // -------------------------------------------------------------------------------------------------
                if let Option::Some((offset, middle)) = &mut self.cursor_offset {
                    if ui.ctx().input().key_pressed(egui::Key::ArrowDown) {
                        if let Option::Some(new) =
                            offset.checked_add(self.bytes_per_line as u32)
                        {
                            *offset = new;
                            *middle = false;
                        }
                    }
                    if ui.ctx().input().key_pressed(egui::Key::ArrowLeft) {
                        if let Option::Some(new) = offset.checked_sub(1) {
                            *offset = new;
                            *middle = false;
                        }
                    }
                    if ui.ctx().input().key_pressed(egui::Key::ArrowRight) {
                        if let Option::Some(new) = offset.checked_add(1) {
                            *offset = new;
                            *middle = false;
                        }
                    }
                    if ui.ctx().input().key_pressed(egui::Key::ArrowUp) {
                        if let Option::Some(new) =
                            offset.checked_sub(self.bytes_per_line as u32)
                        {
                            *offset = new;
                            *middle = false;
                        }
                    }
                    if ui.ctx().input().key_pressed(egui::Key::Backspace) {
                        if *middle{
                            *middle = false;
                            let val = self.mem.get_u8(*offset);
                            let val = val & 0b1111u8;
                            self.mem.set_u8(*offset, val);
                        }else{
                            if let Option::Some(new) = offset.checked_sub(1) {
                                *offset = new;
                                *middle = true;
                                let val = self.mem.get_u8(*offset);
                                let val = val & 0b11110000u8;
                                self.mem.set_u8(*offset, val);
                            }
                        }
                    }
                    use egui::Key::*;
                    let keys = [Num0,Num1,Num2,Num3,Num4,Num5,Num6,Num7,Num8,Num9,A,B,C,D,E,F];
                    for i in 0u8..16{
                        if ui.ctx().input().key_pressed(keys[i as usize]){
                            if let Option::Some((pos, middle)) = &mut self.cursor_offset{
                                if *middle{
                                    let val = self.mem.get_u8(*pos);
                                    let val = (val & 0b11110000u8) + i;
                                    self.mem.set_u8(*pos, val);
                                    *pos = pos.wrapping_add(1);
                                    *middle = false;
                                }else{
                                    let val = self.mem.get_u8(*pos);
                                    let val = (val & 0b1111u8) + (i << 4);
                                    self.mem.set_u8(*pos, val);
                                    *middle = true;
                                }
                            }
                        }
                    }
                }
                // -------------------------------------------------------------------------------------------------
                
                //egui::Area::new("test").show(ui.ctx(), |ui|{
                    //egui::SidePanel::new(egui::panel::Side::Left, "asd").show(ui.ctx(), |ui|{
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui|{
                    //egui::ScrollArea::horizontal().show(ui, |ui| {
                    
    
                    //let response = ui.allocate_exact_size(ui.max_rect().size(), egui::Sense{click: true, drag: false, focusable: true});
                    
                    //ui.painter().rect_stroke(response.0, 0.0, Stroke{
                    //    width: 2.0,
                    //    color: Color32::GREEN,
                    //});
                    //ui.label(label);
                    //-------------------------------------------------------
                    //#[cfg(asd)]
                    ui.set_clip_rect(ui.available_rect_before_wrap());
                    ui.vertical(|ui|{
                        ui.horizontal(|ui| {
                            let response =  ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                for h in 0..=128 {
                                    let mut exit = false;
                                    ui.horizontal(|ui| {
                                        let address = h * self.bytes_per_line as u32 + self.starting_offset;
                                        let mut string = format!("{:08X}", address);
                                        string.insert(4, ':');
        
                                        let res = ui.label(
                                            egui::RichText::new(string)
                                                .background_color(egui::Color32::from_gray(70))
                                                .monospace(),
                                        );
                                        if !ui.is_rect_visible(res.rect) {
                                            exit = true;
                                        }
        
                                        for i in 0u32..self.bytes_per_line as u32 {
                                            ui.spacing_mut().item_spacing.x = 3.0;
        
                                            if i % 4 == 0 && i > 0 {
                                                ui.allocate_space(egui::vec2(3.0, 0.0));
                                            }
                                            let mut label = match self.mem.get_u8_o(address + i){
                                                Some(val) => {
                                                    egui::RichText::new(format!("{:02X}", val))
                                                },
                                                None => {
                                                    egui::RichText::new("--").color(Color32::DARK_RED)
                                                },
                                            }.monospace();

                                            if let Option::Some(color) = self.calculate_highlight(address + i){
                                                label = label.background_color(color);
                                            }
        
                                            let response = ui.label(label);
                                            //println!("{:?}", response);
        
                                            let response = ui.interact(
                                                response.rect.expand(2.0),
                                                response.id,
                                                egui::Sense::click_and_drag(),
                                            );
        
                                            if response.clicked() {
                                                self.cursor_offset =
                                                    Option::Some((h * self.bytes_per_line as u32 + i, false));
                                            }
        
                                            if let Option::Some((offset, typing)) = self.cursor_offset {
                                                if offset == self.bytes_per_line as u32 * h + i {
                                                    let mut rect = response.rect;
                                                    if typing{
                                                        let test = (rect.left() + rect.right()) / 2.0;
                                                        *rect.left_mut()  = test - 0.5;
                                                        *rect.right_mut()  = test + 0.5;
                                                    }else{
                                                        let test = rect.left();
                                                        *rect.right_mut() = test + 2.0;
                                                        *rect.left_mut() += 1.0;
                                                    }
                                                    ui.painter().rect_filled(
                                                        rect,
                                                        egui::Rounding::none(),
                                                        egui::Color32::from_rgb(255, 0, 0),
                                                    );
                                                }
                                            }
                                        }
        
                                        ui.separator();
        
                                        for i in 0u32..self.bytes_per_line as u32 {
                                            ui.spacing_mut().item_spacing.x = 0.0;
                                            let mut label = match self.mem.get_u8_o(address + i){
                                                Some(val) => {
                                                    egui::RichText::new(Self::u8_to_display_char(val))
                                                },
                                                None => {
                                                    egui::RichText::new(".").color(Color32::DARK_RED)
                                                },
                                            }.monospace();
                                            if let Option::Some(color) = self.calculate_highlight(address + i){
                                                label = label.background_color(color);
                                            }
                                            ui.label(label);
                                        }
        
                                        if self.show_disassembly {
                                            ui.separator();
                                            let text = match self.mem.get_u32_alligned_o(address){
                                                Some(val) => {
                                                    assembler::disassembler::simple::disassemble(val)
                                                },
                                                None => {
                                                    "".into()
                                                },
                                            };
                                            let mut text = egui::RichText::new(text).monospace();
                                            if let Option::Some(color) = self.calculate_highlight(address){
                                                text = text.background_color(color);
                                            }
                                            ui.label(text);
                                        }
                                    });
                                    //ui.horizontal(add_contents)
                                    if exit {
                                        break;
                                    }
                                }
                            });
                            let response = ui.interact(
                                response.response.rect,
                                response.response.id,
                                egui::Sense {
                                    click: true,
                                    drag: false,
                                    focusable: true,
                                },
                            );
        
                            if response.clicked() {
                                response.request_focus();
                            }
                            if response.lost_focus() {
                                println!("{:?}", response);
                                self.cursor_offset = Option::None;
                                self.selection_offset = Option::None;
                            }
                        });
                    });
                });
                    //});
                //});

            });
        });
    }

    fn get_name(&self) -> egui::WidgetText {
        "CPU memory".into()
    }
}