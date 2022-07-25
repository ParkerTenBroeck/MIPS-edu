use eframe::{epaint::{text::{LayoutJob}, FontId, Color32, Rounding}, egui::{TextFormat, ScrollArea}, emath::Vec2};

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
        let mut size = (ui.max_rect().width() * 1.84 / 80.0) as i32;
        if size <= 0 {
            size = 1;
        };
        if size & 1 == 1{
            size += 1;
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
                for _ in 0..(80 - line.chars().count()){
                    layout.append(" ", 0.0, TextFormat::default());
                }
            }else{
                layout.append("\n", 0.0, TextFormat::default());
            }
        }
        // layout.append("this is a test", 0.0, TextFormat{
        //     font_id: FontId::monospace(size),
        //     color: Color32::from_rgb(255,0,0),
        //     background: Color32::from_rgb(255,255,0),
        //     ..Default::default()
        // });
        // layout.append("01234567890123456789012345678901234567890123456789012345678901234567890123456789", 0.0, TextFormat{
        //     font_id: FontId::monospace(size),
        //     color: Color32::from_rgb(255,0,0),
        //     background: Color32::from_rgb(0,255,0),
        //     ..Default::default()
        // });
        // layout.append("\n01234567890123456789012345678901234567890123456789012345678901234567890123456789", 0.0, TextFormat{
        //     font_id: FontId::monospace(size),
        //     color: Color32::from_rgb(255,0,255),
        //     ..Default::default()
        // });
        // layout.wrap = TextWrapping{
        //     max_rows: 24,
        //     ..Default::default()
        // };
        
        let gallery = ui.fonts().layout_job(layout);

        let y = ui.next_widget_position().to_vec2().y;
        ui.horizontal(|ui|{

            ui.add_space((ui.max_rect().width() - gallery.rect.width()) / 2.0);
            
            
            let rect = gallery.rect;
            let rect = rect.translate(Vec2::new(ui.next_widget_position().to_vec2().x, y));
            //let rect = rect.translate(ui.next_widget_position().to_vec2());
            
            let rect = rect.expand(2.0);
            //let width = gallery.rect.width();
            //log::debug!("term width {}, font size: {}", width, size);
            ScrollArea::new([true, true]).max_height(ui.max_rect().height()).show(ui, |ui|{
                ui.painter().rect(rect, Rounding::same(5.0), ui.style().visuals.extreme_bg_color, ui.style().visuals.widgets.noninteractive.bg_stroke);
                ui.label(gallery);
            });
        });


        
        //ui.label(layout);
    }

    fn get_name(&self) -> eframe::egui::WidgetText {
        "Mips Terminal".into()
    }
}