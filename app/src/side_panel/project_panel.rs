use eframe::egui;

use super::side_tabbed_panel::SideTab;

#[derive(Default)]
pub struct ProjectSidePanel {}

impl From<ProjectSidePanel> for Box<dyn SideTab> {
    fn from(panel: ProjectSidePanel) -> Self {
        Box::new(panel)
    }
}

impl SideTab for ProjectSidePanel {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, app: &mut crate::Application) {
        ui.add(egui::Label::new(egui::RichText::new("Workspace").heading()).wrap(false));
        ui.collapsing("info", |ui| {
            ui.label("current workspace files ext(just the current directory of the exe for now)");
            ui.label("note opening files will only read them and never save to them currently");
        });
        ui.separator();

        generate_tree(".".into(), app, ui);

        fn generate_tree(path: std::path::PathBuf, t: &mut crate::Application, ui: &mut egui::Ui) {
            if let Ok(val) = std::fs::read_dir(path) {
                let mut test: Vec<Result<std::fs::DirEntry, std::io::Error>> = val.collect();
                test.sort_by(|t1, t2| {
                    if let Result::Ok(t1) = t1 {
                        if let Result::Ok(t2) = t2 {
                            //let t1 = t1.unwrap();
                            //let t2 = t2.unwrap();
                            let t1d = t1.metadata().unwrap().is_dir();
                            let t2d = t2.metadata().unwrap().is_dir();
                            if t1d && t2d {
                                return t1
                                    .file_name()
                                    .to_ascii_lowercase()
                                    .to_str()
                                    .unwrap()
                                    .cmp(t2.file_name().to_ascii_lowercase().to_str().unwrap());
                            } else if t1d {
                                return std::cmp::Ordering::Less;
                            } else if t2d {
                                return std::cmp::Ordering::Greater;
                            } else {
                                return t1
                                    .file_name()
                                    .to_ascii_lowercase()
                                    .to_str()
                                    .unwrap()
                                    .cmp(t2.file_name().to_ascii_lowercase().to_str().unwrap());
                            }
                        }
                    }
                    std::cmp::Ordering::Equal
                });
                for val in test.iter().flatten() {
                    if val.metadata().unwrap().is_dir() {
                        ui.collapsing(val.file_name().into_string().unwrap(), |ui| {
                            generate_tree(val.path(), t, ui);
                        });
                    } else if ui
                        .selectable_label(false, val.file_name().into_string().unwrap())
                        .clicked()
                    {
                        if let Result::Ok(str) = std::fs::read_to_string(val.path()) {
                            t.add_tab(crate::tabs::code_editor::CodeEditor::new(
                                val.file_name().into_string().unwrap(),
                                str,
                            ));
                        }
                        log::info!("loaded file: {}", val.path().display());
                    }
                }
            }
        }
    }

    fn get_icon(&mut self) {
        todo!()
    }
}
