use super::tabbed_area::Tab;

pub struct SettingsTab{
    
}

impl Tab for SettingsTab{
    fn ui(&mut self, _ui: &mut eframe::egui::Ui) {
        
    }

    fn get_name(&self) -> eframe::egui::WidgetText {
        "Settings".into()
    }
}