use egui_dock::Tab;



pub struct DebuggerTab{

}

impl Tab for DebuggerTab{
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.label("temporary");
    }

    fn title(&mut self) -> eframe::egui::WidgetText {
        "Debugger".into()
    }
}