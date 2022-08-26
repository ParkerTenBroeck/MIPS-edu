use eframe::{
    egui::{self, Painter, Sense, WidgetText},
    epaint::{Color32, Rect, RectShape, Rounding, Shape, Stroke, TextureHandle, Vec2},
};
use egui_dock::Tab;

use crate::{
    emulator::{
        handlers::CPUAccessInfo,
        screen::{Layer, Screen},
    },
    platform::sync::PlatSpecificLocking,
};

pub struct MipsDisplay {
    screen: Screen,
    mouse: Option<([u32; 2], bool, bool, bool)>,
    access_info: CPUAccessInfo,
}

impl MipsDisplay {
    pub fn new(bg1: TextureHandle, access_info: CPUAccessInfo) -> Self {
        Self {
            screen: Screen {
                onscreen_resolution: [0; 2],
                layers: [
                    Layer::BitMapExpand(bg1),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                ],
            },
            mouse: None,
            access_info,
        }
    }
}

impl Tab for MipsDisplay {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.screen.onscreen_resolution =
            if let Layer::BitMapExpand(texture) = &self.screen.layers[0] {
                texture.size()
            } else {
                [128, 128]
            };

        let (rect, resp) = ui.allocate_exact_size(
            ui.available_size_before_wrap(),
            Sense {
                click: true,
                drag: false,
                focusable: false,
            },
        );

        let mut calc_rect = Vec2::new(
            self.screen.onscreen_resolution[0] as f32,
            self.screen.onscreen_resolution[1] as f32,
        )
        .normalized();
        if self.screen.onscreen_resolution[0] == 0 || self.screen.onscreen_resolution[1] == 0 {
        } else {
            let ta = calc_rect.x / calc_rect.y;
            let ga = rect.aspect_ratio();
            if ta > ga {
                calc_rect.y = rect.size().x * 1.0 / ta;
                calc_rect.x = rect.size().x;
            } else if ta < ga {
                calc_rect.x = rect.size().y * ta;
                calc_rect.y = rect.size().y;
            }
        }
        let calc_rect = Rect::from_center_size(rect.center(), calc_rect);

        if let Option::Some(pos) = resp.hover_pos() {
            if calc_rect.contains(pos) {
                let mut pos = pos - calc_rect.min;
                pos.x /= calc_rect.size().x;
                pos.y /= calc_rect.size().y;
                let x = (pos.x * self.screen.onscreen_resolution[0] as f32) as u32;
                let y = (pos.y * self.screen.onscreen_resolution[1] as f32) as u32;

                let pri = ui
                    .ctx()
                    .input()
                    .pointer
                    .button_down(egui::PointerButton::Primary);
                let sec = ui
                    .ctx()
                    .input()
                    .pointer
                    .button_down(egui::PointerButton::Secondary);
                let middle = ui
                    .ctx()
                    .input()
                    .pointer
                    .button_down(egui::PointerButton::Middle);
                self.mouse = Option::Some(([x, y], pri, sec, middle));
            } else {
                self.mouse = Option::None;
            }
        } else {
            self.mouse = Option::None;
        }

        // if let Layer::Dissabled = self.screen.layers[1]{

        //     let handle = ui.ctx().load_texture("iamgfeasd", egui::ColorImage::example(), egui::TextureFilter::Nearest);
        //     self.screen.layers[1] = Layer::BitMapScroll { texture: handle.clone(), scroll: [0,0] };

        //     let data = include_bytes!("../../res/sprite_sheet.qoi");
        //     let image = qoi::decode_to_vec(data).unwrap();
        //     let mut pixels = Vec::new();
        //     let mut iter = image.1.iter();

        //     while let (Option::Some(r),Option::Some(g),Option::Some(b),Option::Some(a)) = (iter.next(),iter.next(),iter.next(),iter.next()){
        //         match image.0.colorspace{
        //             qoi::ColorSpace::Srgb => {
        //                 pixels.push(Color32::from_rgba_premultiplied(*r,*g,*b,*a));
        //             },
        //             qoi::ColorSpace::Linear => {
        //                 pixels.push(Color32::from_rgba_unmultiplied(*r,*g,*b,*a));
        //             },
        //         }
        //     }
        //     assert!(image.0.channels.is_rgba());
        //     let image = egui::ColorImage{
        //         size: [image.0.width as usize, image.0.height as usize],
        //         pixels: pixels,
        //     };
        //     let texture = ui.ctx().load_texture("tile_map", image.clone(), egui::TextureFilter::NearestTiled);
        //     let handle = texture.clone();

        //     let mut tiles = Vec::new();
        //     for i in 0..(30 * 30){
        //         tiles.push(Tile{
        //             index_rot: crate::emulator::screen::PosRot::new().set(PosRot::POX_X, (i % 10) * 2).set(PosRot::POX_Y, (i / 10) * 2).to_owned(),
        //             tint: [255, 255, 255, 255]
        //         })
        //     }

        //     self.screen.layers[2] = Layer::TileMap {
        //         scroll: [0,0],
        //         tiles_x_y: [30,30],
        //         tiles_text: texture,
        //         tiles: tiles
        //     };

        //     self.screen.layers[3] = Layer::Sprite {
        //         sprites: vec![

        //         Sprite{
        //             sp_pos: [342, 273],
        //             screen_pos: [60, 60],
        //             tint: [255, 255, 255, 255],
        //             x_size: 61,
        //             y_size: 62,
        //             rot: 8,
        //         }],
        //         sprite_text: handle,
        //         image,
        //     };
        // }
        // if let Layer::Sprite { sprites, .. } = &mut self.screen.layers[3]{
        //     let mut millies = platform::time::duration_since_epoch().as_millis();
        //     for sprite in sprites{
        //         let rad = ((millies % (360 * 50)) as f32 / 50.0).to_radians();
        //         //sprite.screen_pos[0] = self.screen.onscreen_resolution[0] as i16 / 2 - 32;
        //         //sprite.screen_pos[1] = self.screen.onscreen_resolution[1] as i16 / 2 - 32;
        //         sprite.screen_pos[0] = ((rad.sin() + 1.0) * self.screen.onscreen_resolution[0] as f32 / 2.0) as i16 - 32;
        //         sprite.screen_pos[1] = ((rad.cos() + 1.0) * self.screen.onscreen_resolution[1] as f32 / 2.0) as i16 - 32;
        //         sprite.rot = (((millies / 50) % 1024)) as i16 ;
        //         millies += 300;
        //     }
        // }
        // if let Layer::TileMap { scroll , .. } = &mut self.screen.layers[2] {
        //     scroll[0] = ((platform::time::duration_since_epoch().as_millis() / 100) % (30 *8 * 4)) as i16;
        //     scroll[1] = ((platform::time::duration_since_epoch().as_millis() / 128) % (30 *8 * 4)) as i16;
        // }

        let painter = Painter::new(ui.ctx().clone(), ui.layer_id(), calc_rect);

        let mut shapes = Vec::new();
        self.screen.draw(calc_rect, &mut shapes, ui);

        if let Option::Some(mouse) = self.mouse {
            let mut pos = Vec2::new(mouse.0[0] as f32, mouse.0[1] as f32);
            pos = pos
                / Vec2::new(
                    self.screen.onscreen_resolution[0] as f32,
                    self.screen.onscreen_resolution[1] as f32,
                );
            pos = pos * calc_rect.size();
            pos += calc_rect.min.to_vec2();

            let mouse_rect = Rect::from_min_size(pos.to_pos2(), Vec2::new(10.0, 10.0));
            shapes.push(Shape::Rect(RectShape {
                rect: mouse_rect,
                rounding: Rounding::none(),
                fill: Color32::RED,
                stroke: Stroke::new(0.0, Color32::RED),
            }));
        }

        painter.extend(shapes);
    }

    fn title(&mut self) -> WidgetText {
        let mut text = eframe::egui::WidgetText::RichText("Mips Display".into());
        if self.access_info.plat_lock().unwrap().was_display_accessed() {
            text = text.underline();
            text = text.strong();
        }
        text
    }
}
