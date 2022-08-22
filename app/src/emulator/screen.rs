
use eframe::{epaint::{TextureHandle, Rect, Shape, Mesh, Color32, pos2, Vec2, ColorImage}, emath::Rot2, egui};

pub type Coords = [usize; 2];

mycelium_bitfield::bitfield!{
    #[derive(Eq, PartialEq)]
    pub struct PosRot<u32>{
        pub const POX_X = 15;
        pub const POX_Y = 15;
        pub const ROT = 2;
    }
}

mycelium_bitfield::bitfield!{
    #[derive(Eq, PartialEq)]
    pub struct SizeRot<u8>{
        pub const SIZE_X = 3;
        pub const SIZE_Y = 3;
        pub const ROT = 2;
    }
}



#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Tile{
    pub index_rot: PosRot,
    pub tint: [u8; 4],
}

#[repr(C)]
pub struct Sprite{
    pub sp_pos: [i16; 2],
    pub screen_pos: [i16; 2],
    pub tint: [u8; 4],
    pub x_size: u16,
    pub y_size: u16,
    pub rot: i16,
}

#[derive(Default)]
pub enum Layer{
    #[default]
    Dissabled,
    BitMapExpand(TextureHandle),
    BitMapScroll{
        texture: TextureHandle,
        scroll: [i16; 2],
    },
    TileMap{
        scroll: [i16; 2],
        tiles_x_y: [i16; 2],
        tiles_text: TextureHandle,
        tiles: Vec<Tile>,
    },
    Sprite{
        sprite_text: TextureHandle,
        image: ColorImage,
        sprites: Vec<Sprite>,
    }
}

impl Layer{
    pub fn draw(&self,  display_area: Rect, resolution: [usize; 2], shapes: &mut Vec<Shape>, _ui: &egui::Ui){
        //let pixels_per_point = ui.ctx().pixels_per_point();
        //let points_per_pixels = 1.0 / pixels_per_point;
        let pixels_per_point_x = resolution[0] as f32 / display_area.size().x;
        let pixels_per_point_y = resolution[1] as f32 / display_area.size().y;
        let points_per_pixel_x = display_area.size().x / resolution[0] as f32;
        let points_per_pixel_y = display_area.size().y / resolution[1] as f32;

        match self{
            Layer::Dissabled => {
                //skip
            },
            Layer::BitMapExpand(texture) => {
                let mut mesh = Mesh::with_texture(texture.id());
                
                mesh.add_rect_with_uv(display_area, 
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                shapes.push(Shape::Mesh(mesh));
            },
            Layer::BitMapScroll { texture, scroll } => {
                let mut rect = Rect::from_min_max(
                    pos2(0.0, 0.0), 
                    pos2(
                        resolution[0] as f32 / texture.size()[0] as f32, 
                        resolution[1] as f32 / texture.size()[1] as f32
                    )
                );
                rect = rect.translate(
                    Vec2::new(
                        (scroll[0] % texture.size()[0] as i16) as f32 / texture.size()[0] as f32,
                        (scroll[1] % texture.size()[1] as i16) as f32 / texture.size()[1] as f32
                    )
                );
                
                let mut mesh = Mesh::with_texture(texture.id());
        
                mesh.add_rect_with_uv(display_area, rect, Color32::WHITE);
                shapes.push(Shape::Mesh(mesh));
            },
            Layer::TileMap { scroll, tiles_x_y, tiles_text, tiles } => {
                let tiles_x = ((resolution[0] + 7) / 8) as i16;
                let tiles_y = ((resolution[1] + 7) / 8) as i16;

                let size = tiles_text.size_vec2() / 8.0;

                let rotation: [Rot2; 4] = [Rot2::IDENTITY, Rot2::from_angle(std::f32::consts::PI / 2.0), Rot2::IDENTITY, Rot2::from_angle(std::f32::consts::PI * 3.0 / 2.0)];

                let scroll_x_offset = ((scroll[0] % 8)) as f32 * display_area.width() / (tiles_x * 8) as f32;
                let scroll_y_offset = ((scroll[1] % 8)) as f32 * display_area.height() / (tiles_y * 8) as f32;

                let mut rect = display_area;
                let x_step = (rect.max.x - rect.min.x) / tiles_x as f32;
                let y_step = (rect.max.y - rect.min.y) / tiles_y as f32;
                rect.max.x = rect.min.x + x_step - scroll_x_offset;
                rect.max.y = rect.min.y + y_step - scroll_y_offset;
                rect.min.x -= scroll_x_offset;
                rect.min.y -= scroll_y_offset;
                for x in (scroll[0] / 8)..=(tiles_x + (scroll[0] / 8)){
                    for y in (scroll[1] / 8)..=(tiles_y + (scroll[1] / 8)){
                        let index = (((x as usize).wrapping_sub(1)) % tiles_x_y[1] as usize) + ((y as usize).wrapping_sub(1) % tiles_x_y[1] as usize) * tiles_x_y[0] as usize;
                        let tile = &tiles[index];
                        let pos_x = tile.index_rot.get(PosRot::POX_X);
                        let pos_y = tile.index_rot.get(PosRot::POX_Y);
                        let rot = tile.index_rot.get(PosRot::ROT);
                        
                        let mut mesh = Mesh::with_texture(tiles_text.id());
                        
                        mesh.add_rect_with_uv(rect, 
                            Rect::from_min_max(
                                pos2((pos_x as f32)/size.x, (pos_y as f32)/ size.y as f32), 
                                pos2((pos_x as f32 + 1.0)/size.x, (pos_y as f32 + 1.0)/ size.y as f32 as f32)), 
                                Color32::from_rgba_unmultiplied(tile.tint[0], tile.tint[1], tile.tint[2], tile.tint[3])
                                );
                        if rot != 0{
                            mesh.rotate(rotation[rot as usize], rect.center())
                        }

                        shapes.push(Shape::Mesh(mesh));
                        //shapes.push(Shape::Rect(RectShape{ rect, rounding: Rounding::none(), fill: Color32::TRANSPARENT, stroke: Stroke::new(1.0, Color32::RED) }));
                        rect.min.y += y_step;
                        rect.max.y += y_step;    
                    }
                    rect.min.y = display_area.min.y - scroll_y_offset;
                    rect.max.y = rect.min.y + y_step ;

                    rect.min.x += x_step;
                    rect.max.x += x_step;
                }
            },
            Layer::Sprite { sprite_text, sprites, image } => {
                for sprite in sprites{
                    
                    let dis_size = display_area.size();
                    let pos = pos2(
                        dis_size.x * sprite.screen_pos[0] as f32 / resolution[0] as f32,
                        dis_size.y * sprite.screen_pos[1] as f32 / resolution[1] as f32
                    );
                    let sprite_width = sprite.x_size as f32;
                    let sprite_height = sprite.y_size as f32;
                    
                    let mut rect = Rect::from_min_max(pos, pos2(pos.x + dis_size.x * sprite_width as f32 / resolution[0] as f32, pos.y+ dis_size.y * sprite_height as f32 / resolution[1] as f32));
                    rect = rect.translate(display_area.min.to_vec2());
                    
                    let angle = sprite.rot as f32 * (core::f32::consts::TAU / 1024.0);
                        
                    
                    if sprite.rot % 256 == 0{

                        let mut mesh = Mesh::with_texture(sprite_text.id());

                        mesh.add_rect_with_uv(rect, 
                            Rect::from_min_max(
                                pos2(
                                    sprite.sp_pos[0] as f32 / sprite_text.size()[0] as f32,
                                    sprite.sp_pos[1] as f32 / sprite_text.size()[1] as f32
                                ), 
                                pos2(
                                    (sprite_width + sprite.sp_pos[0] as f32) / sprite_text.size()[0] as f32,
                                    (sprite_height + sprite.sp_pos[1] as f32) / sprite_text.size()[1] as f32
                                ), 
                            ), 
                            Color32::from_rgba_unmultiplied(sprite.tint[0], sprite.tint[1], sprite.tint[2], sprite.tint[3]
                            )
                        );

                        if sprite.rot != 0{
                            mesh.rotate(Rot2::from_angle(angle), rect.center());
                        }
                        shapes.push(Shape::Mesh(mesh));
                    }else{
                        let sin = angle.sin();
                        let cos = angle.cos();
                        let sin_abs = sin.abs();
                        let sin_abs_90 = (core::f32::consts::FRAC_PI_2 - angle).sin().abs();
                        
                        let bb_width = sprite_height * sin_abs_90 + sprite_width * sin_abs;
                        let bb_height = sprite_width * sin_abs_90 + sprite_height * sin_abs;
                        let new_width = bb_width.round() as usize;
                        let new_height = bb_height.round() as usize;

                        let mut center = rect.center();

                        if new_width % 2 == 1{
                            center.x += points_per_pixel_x / 2.0;
                        }
                        if new_height % 2 == 1{
                            center.y += points_per_pixel_y / 2.0;
                        }
                        let bounding = Rect::from_center_size(center, Vec2::new(new_width as f32 / pixels_per_point_x, new_height  as f32 / pixels_per_point_y));
                        
                        let x_step = points_per_pixel_x;
                        let y_step = points_per_pixel_y;
                        let mut rect = Rect::from_min_max(bounding.min, pos2(bounding.min.x + x_step, bounding.min.y + y_step));
                        
                        let tmp_1 =  0.5 - bb_width / 2.0;
                        
                        let mut mesh = Mesh::default();
                        for y in 0..new_height{

                            if {
                                let range = bounding.min.y..=bounding.max.y;
                                range.contains(&rect.min.y) || range.contains(&rect.max.y)
                            }{
                                let y = y as f32 + 0.5 - bb_height / 2.0;

                                let tmp_x = 0.0 - y * sin - 0.5 + sprite_width as f32 / 2.0;
                                let tmp_y = y * cos - 0.5 + sprite_height as f32 / 2.0;
    
                                for x in 0..new_width{
                                    
                                    let x = x as f32 + tmp_1;
                                    let x_t = x * cos + tmp_x;
                                    let y_t = x * sin + tmp_y;
                                    //so we need to reverse these?? idk either
                                    let x = y_t.round() as usize;
                                    let y = x_t.round() as usize;
                                    
                                    let index = (x as isize + sprite.sp_pos[0] as isize) as usize % sprite_text.size()[0]
                                                     + (y as isize + sprite.sp_pos[1] as isize) as usize % sprite_text.size()[1] * sprite_text.size()[0] as usize;
                                    
                                    if  x > sprite.x_size as usize - 1 || 
                                        y > sprite.y_size as usize - 1 || 
                                        x_t.round() < 0.0 || 
                                        y_t.round() < 0.0 || 
                                        {
                                            let range = bounding.min.x..=bounding.max.x;
                                            !(range.contains(&rect.min.x) || range.contains(&rect.max.x))
                                        }{
                                    }else{
                                        let color = image.pixels[index];
                                        
                                        if color.a() != 0{
                                            mesh.add_colored_rect(rect, color);
                                        }
                                    }
                                    rect.min.y = rect.max.y;
                                    rect.max.y += y_step;
                                }   
                            }
                            rect.min.y = bounding.min.y;
                            rect.max.y = y_step + bounding.min.y;

                            rect.min.x = rect.max.x;
                            rect.max.x += x_step;
                        }
                        shapes.push(Shape::Mesh(mesh));
                    }
                }
            },
        }
    }
}

#[derive(Default)]
pub struct Screen{
    pub onscreen_resolution: Coords,
    pub layers: [Layer; 4]
}

impl Screen{
    pub fn draw(&self, display_area: Rect, shapes: &mut Vec<Shape>, ui: &egui::Ui){
        
        for layer in &self.layers{
            layer.draw(display_area, self.onscreen_resolution, shapes, ui);
        }
    }
}
