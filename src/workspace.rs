//! Touch chrome. Document edits always use the shared command path.
mod channels_paths;
mod command_tray;
mod inspectors;
mod layers;
mod navigation;
mod options;
mod sheets;

use egui::{Button, Frame, Ui, vec2};
use photocraft_ui_egui::{PhotocraftApp, menus, state::Tool, theme::Tokens};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Inspector {
    Tool,
    #[default]
    Layers,
    Brush,
    Color,
    History,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sheet {
    Commands,
    Files,
    Brush,
    Channels,
    Paths,
    LayerProperties,
}

pub struct TabletUi {
    pub message: String,
    pub dictation: crate::speech::Dictation,
    command_text: String,
    command_tray_open: bool,
    last_selection: Tool,
    tool_receipt: Option<(Tool, Tool)>,
    inspector: Inspector,
    inspector_open: bool,
    sheet: Option<Sheet>,
    search: String,
    command_path: Vec<String>,
    search_open: bool,
    multi_select: bool,
    color_background: bool,
    color_hex: String,
    color_hue: f32,
    color_source: Option<[f32; 4]>,
    brush_name: String,
    path_name: String,
    rename_layer: Option<(u64, u64)>,
    layer_name: String,
    layer_name_source: String,
    edit_gesture: u64,
    edit_pass: u64,
    pub shift: bool,
    pub alt: bool,
    hardware_modifiers: egui::Modifiers,
}
impl Default for TabletUi {
    fn default() -> Self {
        Self {
            message: String::new(),
            dictation: Default::default(),
            command_text: String::new(),
            command_tray_open: false,
            last_selection: Tool::RectMarquee,
            tool_receipt: None,
            inspector: Inspector::Layers,
            inspector_open: true,
            sheet: None,
            search: String::new(),
            command_path: Vec::new(),
            search_open: false,
            multi_select: false,
            color_background: false,
            color_hex: String::new(),
            color_hue: 0.0,
            color_source: None,
            brush_name: String::from("My brush"),
            path_name: String::from("Path 1"),
            rename_layer: None,
            layer_name: String::new(),
            layer_name_source: String::new(),
            edit_gesture: 0,
            edit_pass: 0,
            shift: false,
            alt: false,
            hardware_modifiers: egui::Modifiers::NONE,
        }
    }
}

/// Keep a useful canvas in portrait and Split View; inspector moves below it.
pub fn side_inspector(size: egui::Vec2) -> bool {
    size.x >= 1000. && size.x > size.y
}

pub(super) fn button(ui: &mut Ui, label: &str, selected: bool) -> egui::Response {
    ui.add(
        Button::new(label)
            .selected(selected)
            .min_size(vec2(44., 44.)),
    )
}
pub(super) fn full_button(ui: &mut Ui, label: &str, selected: bool) -> egui::Response {
    ui.add_sized(
        [ui.available_width(), 48.],
        Button::new(label).selected(selected),
    )
}

impl TabletUi {
    fn invoke(&mut self, app: &mut PhotocraftApp, ctx: &egui::Context, id: &str, params: Value) {
        // Window commands must reveal this workspace's panels, not invisible desktop docks.
        let inspector = match id
            .trim_start_matches("window.")
            .trim_start_matches("toggle.")
        {
            "layers" => Some(Inspector::Layers),
            "history" => Some(Inspector::History),
            "color" | "swatches" => Some(Inspector::Color),
            "brushSettings" | "brushes" => Some(Inspector::Brush),
            _ => None,
        };
        if id.starts_with("window.")
            && let Some(inspector) = inspector
        {
            self.inspector = inspector;
            self.inspector_open = true;
            self.message.clear();
            return;
        }
        if matches!(id, "window.properties" | "window.toggle.properties") {
            self.open_sheet(Sheet::LayerProperties);
            return;
        }
        self.message = match menus::invoke(app, ctx, id, params) {
            Ok(_) => String::new(),
            Err(e) => e,
        };
    }
    fn command(&mut self, app: &mut PhotocraftApp, ui: &mut Ui, label: &str, id: &str) {
        let enabled = menus::is_live(id) && menus::is_enabled(app, id);
        if ui
            .add_enabled(enabled, Button::new(label).min_size(vec2(44., 44.)))
            .clicked()
        {
            self.invoke(app, ui.ctx(), id, json!({}));
        }
    }
    fn open_sheet(&mut self, sheet: Sheet) {
        self.dictation.cancel();
        self.command_tray_open = false;
        self.sheet = Some(sheet);
        self.search.clear();
        self.command_path.clear();
        self.search_open = false;
    }
    pub fn show(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let pass = ctx.cumulative_pass_nr();
        if ctx.input(|input| input.pointer.any_pressed()) && self.edit_pass != pass {
            self.edit_gesture = self.edit_gesture.wrapping_add(1);
            self.edit_pass = pass;
        }
        let size = ctx.content_rect().size();
        let t = Tokens::get(&ctx);
        ui.spacing_mut().interact_size = vec2(44., 44.);
        ui.spacing_mut().button_padding = vec2(10., 8.);
        ui.spacing_mut().item_spacing = vec2(6., 6.);
        egui::Panel::top("ipad-document-bar")
            .frame(
                Frame::NONE
                    .fill(t.chrome)
                    .inner_margin(egui::Margin::symmetric(8, 2)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if button(ui, "Files", false).clicked() {
                        self.open_sheet(Sheet::Files);
                    }
                    self.command(app, ui, "Save", "file.save");
                    for (label, icon, id) in [
                        ("Undo", "undo-2", "edit.undo"),
                        ("Redo", "redo-2", "edit.redo"),
                    ] {
                        if ui
                            .add_enabled_ui(menus::is_enabled(app, id), |ui| {
                                navigation::icon_button(ui, icon, label, false, 44.)
                            })
                            .inner
                            .clicked()
                        {
                            self.invoke(app, ui.ctx(), id, json!({}));
                        }
                    }
                    if button(ui, "Studio", self.sheet == Some(Sheet::Commands)).clicked() {
                        self.open_sheet(Sheet::Commands);
                    }
                    if navigation::icon_button(
                        ui,
                        "panel-right",
                        "Inspector",
                        self.inspector_open,
                        44.,
                    )
                    .clicked()
                    {
                        self.inspector_open = !self.inspector_open;
                    }
                    self.command(app, ui, "Fit", "view.fitOnScreen");
                });
            });
        self.command_bar(app, ui);
        let action_pending =
            app.ui.transform.is_some() || app.ui.tool == Tool::Crop || app.ui.text_edit.is_some();
        if !self.command_tray_open || action_pending {
            egui::Panel::bottom("ipad-context")
                .frame(Frame::NONE.fill(t.chrome).inner_margin(8))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.strong(app.ui.tool.label());
                        if !self.command_tray_open
                            && button(
                                ui,
                                "Tool settings",
                                self.inspector_open && self.inspector == Inspector::Tool,
                            )
                            .clicked()
                        {
                            self.inspector = Inspector::Tool;
                            self.inspector_open = true;
                        }
                        if app.ui.transform.is_some() {
                            if button(ui, "Apply transform", false).clicked() {
                                photocraft_ui_egui::transform_tool::commit(app);
                            }
                            if button(ui, "Cancel transform", false).clicked() {
                                photocraft_ui_egui::transform_tool::cancel(app);
                            }
                        } else if app.ui.tool == Tool::Crop {
                            if button(ui, "Apply crop", false).clicked() {
                                photocraft_ui_egui::canvas::commit_crop(app);
                            }
                            if button(ui, "Cancel crop", false).clicked() {
                                app.ui.crop_rect = None;
                            }
                        } else if app.ui.text_edit.is_some() {
                            if button(ui, "Apply text", false).clicked() {
                                photocraft_ui_egui::type_tool::commit(app);
                            }
                            if button(ui, "Cancel text", false).clicked() {
                                photocraft_ui_egui::type_tool::cancel(app);
                            }
                        } else if app.ui.tool.is_brushlike() {
                            let before = app.session.tools.brush.clone();
                            let mut after = before.clone();
                            ui.add(
                                egui::Slider::new(&mut after.size, 0.5..=5000.)
                                    .logarithmic(true)
                                    .text("Size"),
                            );
                            ui.add(egui::Slider::new(&mut after.opacity, 0.0..=1.).text("Opacity"));
                            photocraft_ui_egui::brush_panel::commit_gesture(
                                app, &ctx, &before, &after,
                            );
                        } else if photocraft_ui_egui::tool_feedback::is_selection_tool(app.ui.tool)
                        {
                            for (i, label) in
                                ["New", "Add", "Subtract", "Intersect"].iter().enumerate()
                            {
                                if button(ui, label, app.ui.selection_mode == i as u8).clicked() {
                                    app.ui.selection_mode = i as u8;
                                }
                            }
                            self.command(app, ui, "Deselect", "select.deselect");
                        }
                        if !self.command_tray_open && button(ui, "Shift", self.shift).clicked() {
                            self.shift = !self.shift;
                        }
                        if !self.command_tray_open && button(ui, "Alt", self.alt).clicked() {
                            self.alt = !self.alt;
                        }
                    });
                    if !self.message.is_empty() {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(&self.message);
                            if button(ui, "Dismiss", false).clicked() {
                                self.message.clear();
                            }
                        });
                    }
                });
        }
        self.command_tray(app, ui);
        if self.inspector_open && (side_inspector(size) || !self.command_tray_open) {
            if side_inspector(size) {
                egui::Panel::right("ipad-inspector-side")
                    .exact_size(304.)
                    .frame(Frame::NONE.fill(t.dock).inner_margin(10))
                    .show(ui, |ui| self.inspector(app, ui));
            } else {
                egui::Panel::bottom("ipad-inspector-bottom")
                    .exact_size((size.y * 0.32).clamp(276., 340.))
                    .frame(Frame::NONE.fill(t.dock).inner_margin(10))
                    .show(ui, |ui| self.inspector(app, ui));
            }
        }
        self.tool_rail(app, ui);
        self.sheets(app, &ctx);
    }
    fn inspector(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        ui.spacing_mut().slider_width = 96.0;
        ui.horizontal(|ui| {
            for (tab, icon, label) in [
                (Inspector::Tool, "sliders-horizontal", "Tool settings"),
                (Inspector::Layers, "layers", "Layers"),
                (Inspector::Brush, "brush", "Brush"),
                (Inspector::Color, "palette", "Colour"),
                (Inspector::History, "clock", "History"),
            ] {
                if navigation::icon_button(ui, icon, label, self.inspector == tab, 44.).clicked() {
                    self.inspector = tab;
                }
            }
        });
        if self.inspector == Inspector::Layers {
            self.layers(app, ui);
            return;
        }
        ui.strong(match self.inspector {
            Inspector::Tool => app.ui.tool.label(),
            Inspector::Layers => "Layers",
            Inspector::Brush => "Brush",
            Inspector::Color => "Colour",
            Inspector::History => "History",
        });
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("ipad-inspector-body")
            .auto_shrink([false, false])
            .show(ui, |ui| match self.inspector {
                Inspector::Tool => self.options(app, ui),
                Inspector::Layers => self.layers(app, ui),
                Inspector::Brush => self.brush(app, ui),
                Inspector::Color => self.color(app, ui),
                Inspector::History => self.history(app, ui),
            });
    }
    pub fn raw_input(&mut self, raw: &mut egui::RawInput) {
        let mut initial = self.hardware_modifiers;
        initial.shift |= self.shift;
        initial.alt |= self.alt;
        for event in &mut raw.events {
            match event {
                egui::Event::ModifiersChanged(modifiers) => {
                    self.hardware_modifiers = *modifiers;
                    modifiers.shift |= self.shift;
                    modifiers.alt |= self.alt;
                }
                egui::Event::PointerButton { modifiers, .. }
                | egui::Event::Key { modifiers, .. } => {
                    modifiers.shift |= self.shift;
                    modifiers.alt |= self.alt;
                }
                _ => {}
            }
        }
        raw.events.insert(0, egui::Event::ModifiersChanged(initial));
    }
}

#[cfg(test)]
mod tests;
