use eframe::egui::WidgetText;
use egui_dock::Tab;

pub struct SettingsTab {}

impl Tab for SettingsTab {
    fn ui(&mut self, _ui: &mut eframe::egui::Ui) {}

    fn title(&mut self) -> WidgetText {
        "Settings".into()
    }
}
