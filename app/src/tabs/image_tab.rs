use eframe::{egui::{self}, epaint::{TextureHandle}};

use super::tabbed_area::Tab;

pub struct ImageTab{
    title: String,
    image: eframe::epaint::TextureHandle,
}

impl ImageTab{
    pub fn new(title: impl Into<String>, texture: TextureHandle) -> Self{
        Self{
            title: title.into(),
            image: texture,
        }
    }
}

impl Tab for ImageTab{
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.image(&self.image, ui.available_size());
    }

    fn get_name(&self) -> egui::WidgetText {
        self.title.clone().into()
    }
}