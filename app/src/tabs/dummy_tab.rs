use egui_dock::Tab;

#[derive(Debug)]
pub struct DummyTab{
    title: String,
    force_close: bool,
    on_close: bool,
}

impl DummyTab{
    pub fn new(title: impl Into<String>, force_close: bool, on_close: bool) -> Self{
        Self { title: title.into(), force_close, on_close }
    }
}

impl Tab for DummyTab{
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.label(format!("{:#?}", self));
    }

    fn title(&mut self) -> eframe::egui::WidgetText {
        self.title.clone().into()
    }

    fn force_close(&mut self) -> bool {
        self.force_close
    }
    fn on_close(&mut self) -> bool {
        self.on_close
    }
}