use eframe::egui::RichText;
use egui_dock::Tab;

pub struct DebuggerTab {}

impl Tab for DebuggerTab {
    fn ui(&mut self, _ui: &mut eframe::egui::Ui) {}

    fn title(&mut self) -> eframe::egui::WidgetText {
        RichText::new("Debugger").into()
    }
}
