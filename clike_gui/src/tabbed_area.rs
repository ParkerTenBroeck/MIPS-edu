use eframe::{egui, epaint::Color32};
use mips_emulator::memory::LooslyCachedMemory;

pub trait Tab {
    fn ui(&mut self, ui: &mut egui::Ui);
    fn get_name(&self) -> egui::WidgetText;
}

pub struct TabbedArea {
    tabs: Vec<Box<dyn Tab>>,
    selected: u32,
}

impl TabbedArea {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // ui.spacing_mut().item_spacing.x = 0.0;
        // ui.add(tab());

        // ui.spacing_mut().window_margin.top = 0.0;
        // ui.spacing_mut().window_margin.left = 0.0;
        // ui.spacing_mut().window_margin.bottom = 0.0;
        // ui.spacing_mut().window_margin.right = 0.0;
        // ui.spacing_mut().item_spacing.y = 0.0;
        // ui.spacing_mut().item_spacing.x = 0.0;

        if self.selected > 0 {
            ui.vertical(|ui| {
                let color = ui
                    .style()
                    .visuals
                    .window_fill()
                    .linear_multiply(1.4)
                    .to_opaque();
                let mut frame_no_marg = egui::Frame {
                    margin: egui::style::Margin {
                        left: 0.0,
                        right: 0.0,
                        top: 0.0,
                        bottom: 0.0,
                    },
                    rounding: eframe::epaint::Rounding::none(),
                    fill: color,
                    stroke: eframe::epaint::Stroke::default(), //egui::Stroke::new(5.0, color),
                    ..Default::default()
                };
                egui::panel::TopBottomPanel::top("idk")
                    .min_height(0.0)
                    .frame(frame_no_marg)
                    .show_inside(ui, |ui| {
                        //ui.spacing_mut().item_spacing.y = 0.0;
                        //ui.spacing_mut().item_spacing.x = 0.0;
                        ui.horizontal_wrapped(|ui| {
                            //ui.spacing_mut().item_spacing.y = 0.0;
                            let mut i = 1u32;
                            let len = self.tabs.len();

                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.add(tab());

                            self.tabs.retain(|tab| {
                                //ui.spacing_mut().item_spacing.x = 0.0;
                                if ui
                                    .selectable_label(self.selected == i, tab.get_name())
                                    .clicked()
                                {
                                    if self.selected == i {
                                        if (i as usize) >= len {
                                            self.selected -= 1;
                                        }
                                        return false;
                                    }

                                    self.selected = i;
                                }
                                i += 1;
                                true
                            });
                        });
                    });
                let color = Color32::from_rgb(0, 255, 0);
                frame_no_marg.fill = color;
                //frame_no_marg.stroke = egui::Stroke::new(5.0, color);
                egui::panel::TopBottomPanel::top("idk")
                    .min_height(0.0)
                    .frame(frame_no_marg)
                    .show_inside(ui, |ui| {
                        //ui.spacing_mut().item_spacing.y = 0.0;
                        //ui.spacing_mut().item_spacing.x = 0.0;
                        ui.horizontal_wrapped(|ui| {
                            //ui.spacing_mut().item_spacing.y = 0.0;
                            let mut i = 1u32;
                            let len = self.tabs.len();

                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.add(tab());

                            self.tabs.retain(|tab| {
                                //ui.spacing_mut().item_spacing.x = 0.0;
                                if ui
                                    .selectable_label(self.selected == i, tab.get_name())
                                    .clicked()
                                {
                                    if self.selected == i {
                                        if (i as usize) >= len {
                                            self.selected -= 1;
                                        }
                                        return false;
                                    }

                                    self.selected = i;
                                }
                                i += 1;
                                true
                            });
                        });
                    });
                if self.selected > 0 {
                    ui.separator();
                    self.tabs[self.selected as usize - 1].ui(ui);
                }
            });
        }
    }

    pub fn add_tab(&mut self, tab: Box<dyn Tab>) {
        self.tabs.push(tab);
        if self.selected == 0 {
            self.selected = 1;
        }
    }
}

impl Default for TabbedArea {
    fn default() -> Self {
        Self {
            tabs: Default::default(),
            selected: 0,
        }
    }
}

pub struct CodeEditor {
    title: String,
    code: String,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            title: "CodeEditor".into(),
            code: r#"
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
"#
            .into(),
        }
    }
}

impl CodeEditor {
    pub fn new(title: String, code: String) -> Self {
        Self { title, code }
    }
}

impl Tab for CodeEditor {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Code Editor");
        egui::warn_if_debug_build(ui);

        //ui.add(toggle(&mut false));

        let mut theme = crate::syntax_highlighter::CodeTheme::from_memory(ui.ctx());
        ui.collapsing("Theme", |ui| {
            ui.group(|ui| {
                theme.ui(ui);
                theme.clone().store_in_memory(ui.ctx());
            });
        });

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job =
                crate::syntax_highlighter::highlight(ui.ctx(), &theme, string, "rs");
            layout_job.wrap_width = wrap_width;
            ui.fonts().layout_job(layout_job)
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.code)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        //.desired_rows(10)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                )
            });
            //.on_hover_ui_at_pointer(|ui| {
            //});
        });
    }

    fn get_name(&self) -> egui::WidgetText {
        self.title.clone().into()
    }
}

pub fn tab_ui(ui: &mut egui::Ui) -> egui::Response {
    let desired_size = ui.spacing().interact_size;
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

    if ui.is_visible() {
        let visuals = ui.style().interact(&response);
        let rect = rect.expand(visuals.expansion);
        ui.painter()
            .rect_filled(rect, 0.0, Color32::from_rgb(255, 0, 0));
    }

    response
}

pub fn tab() -> impl egui::Widget {
    move |ui: &mut egui::Ui| tab_ui(ui)
}

pub fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    // Widget code can be broken up in four steps:
    //  1. Decide a size for the widget
    //  2. Allocate space for it
    //  3. Handle interactions with the widget (if any)
    //  4. Paint the widget

    // 1. Deciding widget size:
    // You can query the `ui` how much space is available,
    // but in this example we have a fixed size widget based on the height of a standard button:
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);

    // 2. Allocating space:
    // This is where we get a region of the screen assigned.
    // We also tell the Ui to sense clicks in the allocated region.
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    // 3. Interact: Time to check for clicks!
    if response.clicked() {
        *on = !*on;
        response.mark_changed(); // report back that the value changed
    }

    // Attach some meta-data to the response which can be used by screen readers:
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    // 4. Paint!
    // Make sure we need to paint:
    if ui.is_rect_visible(rect) {
        // Let's ask for a simple animation from egui.
        // egui keeps track of changes in the boolean associated with the id and
        // returns an animated value in the 0-1 range for how much "on" we are.
        let how_on = ui.ctx().animate_bool(response.id, *on);
        // We will follow the current style by asking
        // "how should something that is being interacted with be painted?".
        // This will, for instance, give us different colors when the widget is hovered or clicked.
        let visuals = ui.style().interact_selectable(&response, *on);
        // All coordinates are in absolute screen coordinates so we use `rect` to place the elements.
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        // Paint the circle, animating it from left to right with `how_on`:
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    // All done! Return the interaction response so the user can check what happened
    // (hovered, clicked, ...) and maybe show a tooltip:
    response
}

pub fn toggle(on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| toggle_ui(ui, on)
}

pub struct HexEditor {
    mem: mips_emulator::memory::PagePoolRef<LooslyCachedMemory>,
    offset: Option<(u32, bool)>,
    selection: Option<u32>,
    bytes_per_line: u8,
    show_disassembly: bool,
}

impl HexEditor {
    pub fn new(mem: mips_emulator::memory::PagePoolRef<LooslyCachedMemory>) -> Self {
        HexEditor {
            mem,
            offset: Option::None,
            selection: Option::None,
            bytes_per_line: 16,
            show_disassembly: false,
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
}

impl Tab for HexEditor {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.add_enabled_ui(!self.show_disassembly, |ui| {
                        ui.label("Bytes Per Row");
                        ui.add(egui::Slider::new(&mut self.bytes_per_line, 1u8..=32))
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

                    //let previous_frame_time = previous_frame_time.unwrap_or_default();
                    // if let Some(latest) = self.frame_times.latest_mut() {
                    //     *latest = previous_frame_time; // rewrite history now that we know
                    // }
                    // self.frame_times.add(now, previous_frame_time);

                    // ui.label(format!("frame time: {:.2}ms", 1e3 * ui.ctx().input().time))
                });
            });

            ui.separator();

            //ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {

            //});
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                

                ui.horizontal(|ui| {
                    let text = egui::RichText::new(match self.offset{
                        Some(val) => {
                            let mut string = format!("offset: {:08X}", val.0);
                            string.insert(12, ':');
                            string
                        },
                        None => "offset: ----:----".into(),
                    }).monospace();
                    
                    ui.label(text);

                    let text = egui::RichText::new(match self.selection{
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

                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui|{
                    ui.set_clip_rect(ui.max_rect());
                    ui.vertical(|ui|{
                        ui.horizontal(|ui| {
                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                if let Option::Some((offset, middle)) = &mut self.offset {
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
                                            if let Option::Some((pos, middle)) = &mut self.offset{
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
            
                                let response = ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    for h in 0..=128 {
                                        let mut exit = false;
                                        ui.horizontal(|ui| {
                                            let address = h * self.bytes_per_line as u32;
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
                                                let label = match self.mem.get_u8_o(address + i){
                                                    Some(val) => {
                                                        egui::RichText::new(format!("{:02X}", val))
                                                    },
                                                    None => {
                                                        egui::RichText::new("--").color(Color32::DARK_RED)
                                                    },
                                                }.monospace();
            
                                                let response = ui.label(label);
                                                //println!("{:?}", response);
            
                                                let response = ui.interact(
                                                    response.rect.expand(2.0),
                                                    response.id,
                                                    egui::Sense::click_and_drag(),
                                                );
            
                                                if response.clicked() {
                                                    self.offset =
                                                        Option::Some((h * self.bytes_per_line as u32 + i, false));
                                                }
            
                                                if let Option::Some((offset, typing)) = self.offset {
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
                                                let label = match self.mem.get_u8_o(address + i){
                                                    Some(val) => {
                                                        egui::RichText::new(Self::u8_to_display_char(val))
                                                    },
                                                    None => {
                                                        egui::RichText::new(".").color(Color32::DARK_RED)
                                                    },
                                                }.monospace();
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
                                                let text = egui::RichText::new(text).monospace();
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
                                    self.offset = Option::None;
                                    self.selection = Option::None;
                                }
                            });
                        });
                
                    });
                });

            });
        });
    }

    fn get_name(&self) -> egui::WidgetText {
        "CPU memory".into()
    }
}
