use std::{cell::UnsafeCell, pin::Pin, str::Chars};

use eframe::{epaint::{text::{LayoutJob, TextWrapping}, FontId, Color32, Rounding, Stroke, self}, egui::{TextFormat, ScrollArea, Sense, TextEdit, WidgetInfo, WidgetType}};

use super::tabbed_area::Tab;


pub enum TerminalMode{
    Basic
}
pub struct TerminalTab{
    data: String,
    mode: TerminalMode,
    cursor: Option<usize>
}

impl TerminalTab{
    pub fn new() -> Self{
        Self { 
            data: "hello\nthis is a test\n01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567895\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n24".into(),
            mode: TerminalMode::Basic,
            cursor: None,
        }
    }

}

impl Tab for TerminalTab{
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        let mut layout = LayoutJob::default();
        
        //font size calculation
        let mut size = (ui.max_rect().width() * 1.8 / 80.0) as i32;
        if size <= 0 {
            size = 1;
        };
        while size > 1{
            if FontId::monospace(size as f32).size * 24.0 > ui.available_height(){
                size -= 1;
            }else{
                break;
            }
        }
        let size = size as f32;
        //font size calculation

        let mut background: Option<Color32> = Option::None;
        let mut forground: Option<Color32> = Option::None;
        let mut bold = false;
        let mut underline = false;
        let mut italics = false;
        let mut strike = false;
        


        let mut iter = self.data.chars();
        
        for i in 0..24{
            let mut line = String::new();
            while let Option::Some(char) = iter.next(){
                if char == '\n'{
                    break;
                }
                line.push(char);
                if line.chars().count() >= 80 {
                    break;
                }
            }
            if !line.is_empty(){
                layout.append(line.as_str(), 0.0, TextFormat{
                    font_id: FontId::monospace(size),
                    color: Color32::from_rgb(255,0,0),
                    background: Color32::from_rgb(255,255,0),
                    ..Default::default()
                });
            }
            if i == 23{
                let mut string = String::new();
                for _ in 0..(80 - line.chars().count()){
                    string.push(' ');
                }
                layout.append(string.as_str(), 0.0, TextFormat{
                    font_id: FontId::monospace(size),
                    ..Default::default()
                });
            }else{
                layout.append("\n", 0.0, TextFormat{
                    font_id: FontId::monospace(size),
                    ..Default::default()
                });
            }
        }
        
        let gallery = ui.fonts().layout_job(layout);
        
        let y_space = (ui.available_height() - gallery.rect.height()) / 2.0;
        if y_space > 0.0{
            ui.add_space(y_space);
        }

        ui.horizontal(|ui|{

            let x_space = (ui.available_width() - gallery.rect.width()) / 2.0;
            if x_space > 0.0{
                ui.add_space(x_space);
            }
            
            let (pos, response) = ui.allocate_exact_size(gallery.size(), Sense{ click: true, drag: true, focusable: true });
            if response.hovered() && ui.input().pointer.any_pressed(){
                ui.memory().request_focus(response.id);
            }
            if ui.memory().has_focus(response.id){
                ui.memory().lock_focus(response.id, true);

            }
            
            //black  =  0x000000
            //red    =  0xCD3131
            //yellow =  0xE5E510
            //green  =  0x0DBC79
            //cyan   =  0x11A8CD
            //blue   =  0x2472C8
            //purple =  0xBC3FBC
            //white  =  0xE5E5E5
            //let data = UnsafeCell::new(Pin::new(&12));
            
            response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, gallery.text()));
            
            if ui.is_rect_visible(response.rect) {
                
                let stroke = if response.has_focus(){
                    Stroke::new(5.0, Color32::RED)
                }else{
                    ui.style().visuals.widgets.noninteractive.bg_stroke
                };
                ui.painter().rect(pos.expand(2.0), 
                                        Rounding::same(5.0), 
                                        ui.style().visuals.extreme_bg_color, 
                                        stroke);
            
                ui.painter().add(epaint::TextShape {
                    pos: pos.left_top(),
                    galley: gallery,
                    override_text_color: None,
                    underline: Stroke::none(),
                    angle: 0.0,
                });
            }
        });
        
        //ui.label(layout);
    }

    fn get_name(&self) -> eframe::egui::WidgetText {
        "Mips Terminal".into()
    }
}


struct TerminalParser<'a>{
    iter: Chars<'a>,
    string: String,
    row_count: usize,
    background: Option<Color32>,
    forground: Option<Color32>,
    bold: bool,
    underline: bool,
    italics: bool,
    strike: bool,
    state: usize, 
    font_size: f32,
}

impl<'a> TerminalParser<'a>{
    pub fn new(str: &'a str, font_size: f32) -> Self{
        Self{
            iter: str.chars(),
            background: Option::None,
            forground: Option::None,
            bold: false,
            underline: false,
            italics: false,
            strike: false,
            row_count: 0,
            state: 0,
            string: String::new(),
            font_size,
        }
    }

    fn generate_text_format(&self) -> TextFormat{
        TextFormat { 
            font_id: FontId::monospace(self.font_size), 
            color: self.forground.unwrap_or(Color32::GRAY), 
            background: self.background.unwrap_or(Color32::TRANSPARENT), 
            italics: self.italics, 
            underline: if self.underline {Stroke::default()} else{Stroke::none()}, 
            strikethrough:  if self.underline {Stroke::default()} else{Stroke::none()}, 
            ..Default::default()
        }
    }
}

impl<'a> Iterator for TerminalParser<'a>{
    type Item = (String, TextFormat);

    fn next(&mut self) -> Option<Self::Item> {
        let mut send: bool = false;
        loop{
            match self.iter.next(){
                Some(char) => {
                    match self.state{
                        0 => {
                            match char{
                                '\x1b' => {
                                    self.state = 1;
                                    send = true;
                                }
                                char =>{
                                    if !char.is_control(){
                                        if self.row_count >= 80{
                                            self.string.push('\n');
                                            self.row_count = 0;
                                        }
                                        self.row_count += 1;
                                    }
                                    self.string.push(char);
                                }
                            }
                        }
                        1 => {
                            match char{
                                '[' => {
                                    let mut code = String::new();
                                    while let Option::Some(char) = self.iter.next(){
                                        if char == 'm'{
                                            break;
                                        }
                                        code.push(char);
                                    }
                                    let codes = code.split(';');
                                    let mut num_codes:Vec<u8> = Vec::new();
                                    for code in codes{
                                        let res = code.parse();
                                        match res {
                                            Ok(val) => num_codes.push(val),
                                            Err(_) => {
                                                self.state = 0;
                                                continue;
                                            },
                                        }
                                    }
                                    
                                    while !num_codes.is_empty(){
                                        match num_codes.remove(0){
                                            0 => {
                                                self.forground = None;
                                                self.background = None;
                                                self.bold = false;
                                                self.italics = false;
                                                self.strike = false;
                                                self.underline = false;
                                            }
                                            1 => self.bold = true,
                                            2 => self.bold = false,
                                            3 => self.italics = true,
                                            4 => self.underline = true,
                                            9 => self.strike = false,
                                            21 => self.bold = false,
                                            22 => self.italics = false,
                                            24 => self.underline = false,
                                            29 => self.strike = false,
                                            color @ 30..=37 => {
                                                
                                            }
                                            _ => {
    
                                            }
                                        }
                                    }

                                }
                                _ => {
                                    //this is an error but we just ignore it
                                    self.state = 0;
                                    continue;
                                }
                            }
                        }
                        _ => {panic!()}
                    }
                },
                None => {
                    if self.string.is_empty(){
                        return None
                    }else{
                        send = true;
                    }
                },
            }
            if send{
                if !self.string.is_empty(){
                    let mut string = String::new();
                    std::mem::swap(&mut string, &mut self.string);
                    return Option::Some((string, self.generate_text_format()))
                }
            }
        }
    }
}