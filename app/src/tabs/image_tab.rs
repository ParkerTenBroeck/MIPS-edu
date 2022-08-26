use eframe::{
    egui::{self, WidgetText},
    epaint::TextureHandle,
};
use egui_dock::Tab;

pub struct ImageTab {
    title: String,
    image: eframe::epaint::TextureHandle,
}

impl ImageTab {
    pub fn new(title: impl Into<String>, texture: TextureHandle) -> Self {
        Self {
            title: title.into(),
            image: texture,
        }
    }
}

impl Tab for ImageTab {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.image(&self.image, ui.available_size());
    }

    fn title(&mut self) -> WidgetText {
        self.title.clone().into()
    }
}
