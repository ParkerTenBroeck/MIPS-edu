use eframe::{epaint::{TextureHandle, Rect, Shape, Mesh, Color32, pos2, Vec2, RectShape, Rounding, Stroke}, emath::Rot2};

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
pub struct Tile{
    pub index_rot: PosRot,
    pub tint: [u8; 4],
}

#[repr(C)]
pub struct Sprite{
    pub sp_pos: [u16; 2],
    pub screen_pos: [i16; 2],
    pub tint: [u8; 4],
    pub size_rot: SizeRot,
}

#[derive(Default)]
pub enum Layer{
    #[default]
    Dissabled,
    BitMapExpand(TextureHandle),
    BitMapScroll{
        texture: TextureHandle,
        visible: Coords,
        scroll: Coords,
    },
    TileMap{
        scroll: Coords,
        visible_tiles_x_y: Coords,
        tiles_x_y: Coords,
        tiles_text: TextureHandle,
        tiles: Vec<Tile>,
    },
    Sprite{
        sprite_text: TextureHandle,
        resolution: Coords,
        sprites: Vec<Sprite>,
    }
}

impl Layer{
    pub fn draw(&self,  display_area: Rect, shapes: &mut Vec<Shape>){
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
            Layer::BitMapScroll { texture, visible, scroll } => {
                let mut rect = Rect::from_min_max(
                    pos2(0.0, 0.0), 
                    pos2(
                        visible[0] as f32 / texture.size()[0] as f32, 
                        visible[1] as f32 / texture.size()[1] as f32
                    )
                );
                rect = rect.translate(
                    Vec2::new(
                        (scroll[0] % texture.size()[0]) as f32 / texture.size()[0] as f32,
                        (scroll[1] % texture.size()[1]) as f32 / texture.size()[1] as f32
                    )
                );
                
                let mut mesh = Mesh::with_texture(texture.id());
        
                mesh.add_rect_with_uv(display_area, rect, Color32::WHITE);
                shapes.push(Shape::Mesh(mesh));
            },
            Layer::TileMap { scroll, visible_tiles_x_y, tiles_x_y, tiles_text, tiles } => {
                let tiles_x = visible_tiles_x_y[0];
                let tiles_y = visible_tiles_x_y[0];

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
                        let tile = &tiles[((x.wrapping_sub(1)) % tiles_x_y[1]) + (y.wrapping_sub(1) % tiles_x_y[1]) * tiles_x_y[0]];
                        let pos_x = tile.index_rot.get(PosRot::POX_X);
                        let pos_y = tile.index_rot.get(PosRot::POX_Y);
                        let rot = tile.index_rot.get(PosRot::ROT);
                        //ui.painter().rect_stroke(rect, Rounding::default(), Stroke::new(2.0, Color32::RED));
                        let mut mesh = Mesh::with_texture(tiles_text.id());
                        
                        mesh.add_rect_with_uv(rect, 
                            Rect::from_min_max(
                                pos2((pos_x as f32)/size.x, (pos_y as f32)/ size.y as f32), 
                                pos2((pos_x as f32 + 1.0)/size.x, (pos_y as f32 + 1.0)/ size.y as f32 as f32)), 
                                Color32::WHITE);
                        if rot != 0{
                            mesh.rotate(rotation[rot as usize], rect.center())
                        }

                        shapes.push(Shape::Mesh(mesh));
                        shapes.push(Shape::Rect(RectShape{ rect, rounding: Rounding::none(), fill: Color32::TRANSPARENT, stroke: Stroke::new(1.0, Color32::RED) }));
                        rect.min.y += y_step;
                        rect.max.y += y_step;    
                    }
                    rect.min.y = display_area.min.y - scroll_y_offset;
                    rect.max.y = rect.min.y + y_step ;

                    rect.min.x += x_step;
                    rect.max.x += x_step;
                }
            },
            Layer::Sprite { sprite_text, resolution, sprites } => {
                for sprite in sprites{
                    
                    let mut mesh = Mesh::with_texture(sprite_text.id());
                    let dis_size = display_area.size();
                    let pos = pos2(
                        dis_size.x * sprite.screen_pos[0] as f32 / resolution[0] as f32,
                        dis_size.y * sprite.screen_pos[1] as f32 / resolution[1] as f32
                    );
                    let sprite_width = 8;
                    let sprite_height = 8;
                    
                    let mut rect = Rect::from_min_max(pos, pos2(pos.x + dis_size.x * sprite_width as f32 / resolution[0] as f32, pos.y+ dis_size.y * sprite_height as f32 / resolution[1] as f32));
                    rect = rect.translate(display_area.min.to_vec2());
                    mesh.add_rect_with_uv(rect, 
                        Rect::from_min_max(
                            pos2(
                                sprite.sp_pos[0] as f32 / sprite_text.size()[0] as f32,
                                sprite.sp_pos[1] as f32 / sprite_text.size()[1] as f32
                            ), 
                            pos2(
                                (sprite_width + sprite.sp_pos[0]) as f32 / sprite_text.size()[0] as f32,
                                (sprite_height + sprite.sp_pos[1]) as f32 / sprite_text.size()[1] as f32
                            ), 
                        ), 
                        Color32::from_rgba_premultiplied(sprite.tint[0], sprite.tint[1], sprite.tint[2], sprite.tint[3]
                        )
                    );
                    shapes.push(Shape::Mesh(mesh));
                }
            },
        }
    }
}

#[derive(Default)]
pub struct Screen{
    pub aspect_ratio: Coords,
    pub layers: [Layer; 4]
}

impl Screen{
    pub fn draw(&self, display_area: Rect, shapes: &mut Vec<Shape>){
        
        for layer in &self.layers{
            layer.draw(display_area, shapes);
        }
    }
}