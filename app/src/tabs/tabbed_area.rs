use eframe::{egui::{self}, epaint::Color32};

pub trait Tab {
    fn ui(&mut self, ui: &mut egui::Ui);
    fn get_name(&self) -> egui::WidgetText;
}

pub struct TabbedArea {
    tabs: Vec<Box<dyn Tab>>,
    selected: u32,
}

impl TabbedArea {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // ui.spacing_mut().item_spacing.x = 0.0;
        // ui.add(tab());

        // ui.spacing_mut().window_margin.top = 0.0;
        // ui.spacing_mut().window_margin.left = 0.0;
        // ui.spacing_mut().window_margin.bottom = 0.0;
        // ui.spacing_mut().window_margin.right = 0.0;
        // ui.spacing_mut().item_spacing.y = 0.0;
        // ui.spacing_mut().item_spacing.x = 0.0;

        if self.selected > 0 {
            ui.vertical(|ui| {
                let color = ui
                    .style()
                    .visuals
                    .window_fill()
                    .linear_multiply(1.4)
                    .to_opaque();
                let mut frame_no_marg = egui::Frame {
                    inner_margin: egui::style::Margin {
                        left: 0.0,
                        right: 0.0,
                        top: 0.0,
                        bottom: 0.0,
                    },
                    outer_margin: egui::style::Margin::symmetric(0.0, 0.0),
                    rounding: eframe::epaint::Rounding::none(),
                    fill: color,
                    stroke: eframe::epaint::Stroke::default(), //egui::Stroke::new(5.0, color),
                    ..Default::default()
                };
                egui::panel::TopBottomPanel::top("idk")
                    .min_height(0.0)
                    .frame(frame_no_marg)
                    .show_inside(ui, |ui| {
                        //ui.spacing_mut().item_spacing.y = 0.0;
                        //ui.spacing_mut().item_spacing.x = 0.0;
                        ui.horizontal_wrapped(|ui| {
                            //ui.spacing_mut().item_spacing.y = 0.0;
                            let mut i = 1u32;
                            let len = self.tabs.len();

                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.add(tab());

                            self.tabs.retain(|tab| {
                                //ui.spacing_mut().item_spacing.x = 0.0;
                                if ui
                                    .selectable_label(self.selected == i, tab.get_name())
                                    .clicked()
                                {
                                    if self.selected == i {
                                        if (i as usize) >= len {
                                            self.selected -= 1;
                                        }
                                        return false;
                                    }

                                    self.selected = i;
                                }
                                i += 1;
                                true
                            });
                        });
                    });
                let color = Color32::from_rgb(0, 255, 0);
                frame_no_marg.fill = color;
                //frame_no_marg.stroke = egui::Stroke::new(5.0, color);
                egui::panel::TopBottomPanel::top("idk")
                    .min_height(0.0)
                    .frame(frame_no_marg)
                    .show_inside(ui, |ui| {
                        //ui.spacing_mut().item_spacing.y = 0.0;
                        //ui.spacing_mut().item_spacing.x = 0.0;
                        ui.horizontal_wrapped(|ui| {
                            //ui.spacing_mut().item_spacing.y = 0.0;
                            let mut i = 1u32;
                            let len = self.tabs.len();

                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.add(tab());

                            self.tabs.retain(|tab| {
                                //ui.spacing_mut().item_spacing.x = 0.0;
                                if ui
                                    .selectable_label(self.selected == i, tab.get_name())
                                    .clicked()
                                {
                                    if self.selected == i {
                                        if (i as usize) >= len {
                                            self.selected -= 1;
                                        }
                                        return false;
                                    }

                                    self.selected = i;
                                }
                                i += 1;
                                true
                            });
                        });
                    });
                if self.selected > 0 {
                    ui.separator();
                    self.tabs[self.selected as usize - 1].ui(ui);
                }
            });
        }
    }

    pub fn add_tab(&mut self, tab: Box<dyn Tab>) {
        self.tabs.push(tab);
        self.selected = self.tabs.len() as u32;
    }
}

impl Default for TabbedArea {
    fn default() -> Self {
        Self {
            tabs: Default::default(),
            selected: 0,
        }
    }
}

pub fn tab_ui(ui: &mut egui::Ui) -> egui::Response {
    let desired_size = ui.spacing().interact_size;
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

    if ui.is_visible() {
        let visuals = ui.style().interact(&response);
        let rect = rect.expand(visuals.expansion);
        ui.painter()
            .rect_filled(rect, 0.0, Color32::from_rgb(255, 0, 0));
    }

    response
}

pub fn tab() -> impl egui::Widget {
    move |ui: &mut egui::Ui| tab_ui(ui)
}

pub fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    // Widget code can be broken up in four steps:
    //  1. Decide a size for the widget
    //  2. Allocate space for it
    //  3. Handle interactions with the widget (if any)
    //  4. Paint the widget

    // 1. Deciding widget size:
    // You can query the `ui` how much space is available,
    // but in this example we have a fixed size widget based on the height of a standard button:
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);

    // 2. Allocating space:
    // This is where we get a region of the screen assigned.
    // We also tell the Ui to sense clicks in the allocated region.
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    // 3. Interact: Time to check for clicks!
    if response.clicked() {
        *on = !*on;
        response.mark_changed(); // report back that the value changed
    }

    // Attach some meta-data to the response which can be used by screen readers:
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    // 4. Paint!
    // Make sure we need to paint:
    if ui.is_rect_visible(rect) {
        // Let's ask for a simple animation from egui.
        // egui keeps track of changes in the boolean associated with the id and
        // returns an animated value in the 0-1 range for how much "on" we are.
        let how_on = ui.ctx().animate_bool(response.id, *on);
        // We will follow the current style by asking
        // "how should something that is being interacted with be painted?".
        // This will, for instance, give us different colors when the widget is hovered or clicked.
        let visuals = ui.style().interact_selectable(&response, *on);
        // All coordinates are in absolute screen coordinates so we use `rect` to place the elements.
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        // Paint the circle, animating it from left to right with `how_on`:
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    // All done! Return the interaction response so the user can check what happened
    // (hovered, clicked, ...) and maybe show a tooltip:
    response
}

pub fn toggle(on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| toggle_ui(ui, on)
}