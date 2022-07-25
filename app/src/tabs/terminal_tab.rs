use eframe::{epaint::{text::{LayoutJob}, FontId, Color32, Rounding, Stroke, self}, egui::{TextFormat, ScrollArea, Sense, TextEdit}};

use super::tabbed_area::Tab;



pub struct TerminalTab{
    data: String,
}

impl TerminalTab{
    pub fn new() -> Self{
        Self { 
            data: "hello\nthis is a test\n01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567895\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n24".into()
        }
    }
}

impl Tab for TerminalTab{
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        let mut layout = LayoutJob::default();
        
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