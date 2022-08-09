use eframe::{egui::{self, Painter}, epaint::{TextureHandle, Shape, Mesh, Rect, Color32, pos2, Stroke, Rounding}};

use super::tabbed_area::Tab;

pub struct MipsDisplay{
    bg1: eframe::epaint::TextureHandle,
    bg2: eframe::epaint::TextureHandle,
    sprite: eframe::epaint::TextureHandle,
}

impl MipsDisplay{
    pub fn new(bg1: TextureHandle) -> Self{
        Self{
            bg1,
            bg2: todo!(),
            sprite: todo!(),
        }
    }
}

impl Tab for MipsDisplay{
    fn ui(&mut self, ui: &mut egui::Ui) {
        //ui.image(&self.image, ui.available_size());
        let painter = Painter::new(ui.ctx().clone(), ui.layer_id(), ui.available_rect_before_wrap());
        

        let mut shapes = Vec::new();

        let tiles_x = 8;
        let tiles_y = 8;

        let mut rect = painter.clip_rect();
        let x_step = (rect.max.x - rect.min.x) / tiles_x as f32;
        let y_step = (rect.max.y - rect.min.y) / tiles_y as f32;
        rect.max.x = rect.min.x + x_step;
        rect.max.y = rect.min.y + y_step;
        for x in 0..tiles_x{
            for y in 0..tiles_y{
                //ui.painter().rect_stroke(rect, Rounding::default(), Stroke::new(2.0, Color32::RED));
                let mut mesh = Mesh::with_texture(self.bg1.id());
                mesh.add_rect_with_uv(rect, 
                    Rect::from_min_max(
                        pos2((x as f32)/tiles_x as f32, (y as f32)/ tiles_y as f32), 
                        pos2((x as f32+1.0)/tiles_x as f32, (y as f32+1.0)/ tiles_y as f32)), 
                        Color32::WHITE);
                shapes.push(Shape::Mesh(mesh));
        
                rect.min.y += y_step;
                rect.max.y += y_step;    
            }
            rect.min.y = painter.clip_rect().min.y;
            rect.max.y = rect.min.y + y_step;

            rect.min.x += x_step;
            rect.max.x += x_step;
        }
        

        painter.extend(shapes);
        ui.expand_to_include_rect(painter.clip_rect());
    }

    fn get_name(&self) -> egui::WidgetText {
        "Mips Display".into()
    }
}