use eframe::egui;

pub trait Tab{
    fn ui(&mut self, ui: &mut egui::Ui);
    fn get_name(&self) -> egui::WidgetText;
}

pub struct TabbedArea{
    tabs: Vec<Box<dyn Tab>>,
    selected: u32,
}
impl TabbedArea{
    pub fn ui(&mut self, ui: &mut egui::Ui){
        ui.vertical(|ui|{
            
            ui.horizontal_wrapped(|ui|{
                let mut i = 1u32;
                let len = self.tabs.len();
                self.tabs.retain(|tab| {
                    if ui.selectable_label(self.selected == i, tab.get_name()).clicked(){
                        if self.selected == i{ 
                            if (i as usize) >= len{
                                self.selected -= 1;
                            }
                            return false    
                        }
                        
                        self.selected = i;
                    }
                    i += 1;
                    true
                });
            });
            if self.selected > 0{
                ui.separator();
                self.tabs[self.selected as usize - 1].ui(ui);
            }
        });
    }

    pub fn add_tab(&mut self, tab: Box<dyn Tab>){
        self.tabs.push(tab);
        if self.selected == 0{
            self.selected = 1;
        }
    }
}

impl Default for TabbedArea{
    fn default() -> Self {
        Self { 
            tabs: Default::default(),
            selected: 0
        }
    }
}

pub struct CodeEditor{
    title: String,
    code: String,
}

impl Default for CodeEditor{
    fn default() -> Self {
        Self { 
            title: "CodeEditor".into(),
            code: r#"
/// Outer block single line documentation
/**
    /*
        ps(you can have /*!BLOCKS*/ /**inside*/ blocks)
    */
    Outer block multiline documentation
*/
fn test(){
    println!("dont change a thing! {}", "you are amazing ;)");
    let r#fn = test;
    let number = 12 + 2.3e-2;

    //! some inner documentation
    let boolean = false;

    /*!
        Outer block multiline documentation
    */
    for(i: i32, i < 50; i += 2){
        println!("hello for the {} time!", i);
    }

    //this is a comment(crazy right)
    /*
        block comment
        this one goes on for a while
    */
}
"#
                        .into(),
        }
    }
}

impl CodeEditor{
    pub fn new(title: String, code: String) -> Self {
        Self{
            title,
            code,
        }
    }
}

impl Tab for CodeEditor{
    
    fn ui(&mut self, ui: &mut egui::Ui) {
        
        ui.heading("Code Editor");
        egui::warn_if_debug_build(ui);

        let mut theme = crate::syntax_highlighter::CodeTheme::from_memory(ui.ctx());
        ui.collapsing("Theme", |ui| {
            ui.group(|ui| {
                theme.ui(ui);
                theme.clone().store_in_memory(ui.ctx());
            });
        });

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job =
                crate::syntax_highlighter::highlight(ui.ctx(), &theme, string, "rs");
            layout_job.wrap_width = wrap_width;
            ui.fonts().layout_job(layout_job)
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.code)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        //.desired_rows(10)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                )
                
            });
            //.on_hover_ui_at_pointer(|ui| {
            //});
        });
    }

    fn get_name(&self) -> egui::WidgetText {
        self.title.clone().into()
    }
}