use eframe::egui::{WidgetText, ScrollArea};
use egui_dock::Tab;

pub struct SettingsTab {}

impl Tab for SettingsTab {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        ScrollArea::both().show(ui, |ui|{
            let ctx = ui.ctx().clone();
            ctx.settings_ui(ui);    
        });
    }

    fn title(&mut self) -> WidgetText {
        "⚙ Settings".into()
    }
}

pub struct EguiMemoryTab {}

impl Tab for EguiMemoryTab {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        ScrollArea::both().show(ui, |ui|{
            let ctx = ui.ctx().clone();
            ctx.memory_ui(ui);    
        });
    }

    fn title(&mut self) -> WidgetText {
        "📝 Memory".into()
    }
}

pub struct EguiInspectionTab {}

impl Tab for EguiInspectionTab {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        ScrollArea::both().show(ui, |ui|{
            let ctx = ui.ctx().clone();
            ctx.inspection_ui(ui);    
        });
    }

    fn title(&mut self) -> WidgetText {
        "🔍 Inspection".into()
    }
}
