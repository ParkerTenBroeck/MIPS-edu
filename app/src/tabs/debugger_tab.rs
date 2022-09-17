use eframe::egui::RichText;
use egui_dock::Tab;


type Debugger = std::sync::Arc<std::sync::Mutex<dyn crate::emulator::debugger_thread::DebuggerConnection<crate::emulator::debug_target::MipsTargetInterface<crate::emulator::handlers::ExternalHandler>>>>;


pub struct DebuggerTab {
    debugger: Debugger,
}
impl Tab for DebuggerTab {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        if let Ok(mut debugger) = self.debugger.try_lock(){
            let debugger = &mut *debugger;
            if let Ok(Some(connection)) = debugger.try_get_connection_info(){
                ui.label(format!("{:#?}", connection));
            }
            debugger.try_target(Box::new(|target|{
                ui.label(format!("{:#?}", target.breakpoints()));
            }));
        }
    }

    fn title(&mut self) -> eframe::egui::WidgetText {
        RichText::new("Debugger").into()
    }
}
impl DebuggerTab {
    pub fn new(debugger: Debugger) -> Self {
        Self { debugger }
    }
}
