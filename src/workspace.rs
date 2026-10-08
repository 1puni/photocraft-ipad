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
    command_text: String,
    command_group: String,
    command_tray_open: bool,
    command_focus_requested: bool,
    last_selection: Tool,
    tool_receipt: Option<(Tool, Tool)>,
    inspector: Inspector,
    inspector_open: bool,
    sheet: Option<Sheet>,
    command_path: Vec<String>,
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
    pub clone_source_pick: bool,
    clone_source_contact: bool,
    hardware_modifiers: egui::Modifiers,
}
impl Default for TabletUi {
    fn default() -> Self {
        Self {
            message: String::new(),
            command_text: String::new(),
            command_group: String::new(),
            command_tray_open: false,
            command_focus_requested: false,
            last_selection: Tool::RectMarquee,
            tool_receipt: None,
            inspector: Inspector::Layers,
            inspector_open: true,
            sheet: None,
            command_path: Vec::new(),
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
            clone_source_pick: false,
            clone_source_contact: false,
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
        self.command_tray_open = false;
        self.sheet = Some(sheet);
        self.command_path.clear();
    }
    pub fn show(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        self.update_source_pick(app, &ctx);
        let pass = ctx.cumulative_pass_nr();
        if ctx.input(|input| input.pointer.any_pressed()) && self.edit_pass != pass {
            self.edit_gesture = self.edit_gesture.wrapping_add(1);
            self.edit_pass = pass;
        }
        if photocraft_ui_egui::tool_feedback::is_selection_tool(app.ui.tool) {
            self.last_selection = app.ui.tool;
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
                    if navigation::icon_button(
                        ui,
                        "search",
                        "Find command",
                        self.command_tray_open,
                        44.,
                    )
                    .clicked()
                    {
                        self.command_tray_open = true;
                        self.command_focus_requested = true;
                    }
                });
            });

        egui::Panel::bottom("ipad-context")
            .frame(Frame::NONE.fill(t.chrome).inner_margin(8))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.strong(if self.clone_source_pick {
                        "Tap source on canvas"
                    } else {
                        app.ui.tool.label()
                    });
                    if button(
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
                        if matches!(app.ui.tool, Tool::CloneStamp | Tool::Healing) {
                            self.source_button(app, ui);
                        }
                        let before = app.session.tools.brush.clone();
                        let mut after = before.clone();
                        ui.add(
                            egui::Slider::new(&mut after.size, 0.5..=5000.)
                                .logarithmic(true)
                                .text("Size"),
                        );
                        ui.add(egui::Slider::new(&mut after.opacity, 0.0..=1.).text("Opacity"));
                        photocraft_ui_egui::brush_panel::commit_gesture(app, &ctx, &before, &after);
                    } else if photocraft_ui_egui::tool_feedback::is_selection_tool(app.ui.tool) {
                        for (i, label) in ["New", "Add", "Subtract", "Intersect"].iter().enumerate()
                        {
                            if button(ui, label, app.ui.selection_mode == i as u8).clicked() {
                                app.ui.selection_mode = i as u8;
                            }
                        }
                        self.selection_actions(app, ui);
                    }
                    if button(ui, "Shift", self.shift).clicked() {
                        self.shift = !self.shift;
                    }
                    if !matches!(app.ui.tool, Tool::CloneStamp | Tool::Healing)
                        && button(ui, "Alt", self.alt).clicked()
                    {
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
                if app.ui.status_error {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(&app.ui.status);
                        if button(ui, "Dismiss error", false).clicked() {
                            app.ui.status_error = false;
                        }
                    });
                }
            });
        if self.inspector_open {
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
        self.command_tray(app, ui);
    }
    fn inspector(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        ui.spacing_mut().slider_width = 96.0;
        ui.spacing_mut().scroll.floating = false;
        ui.spacing_mut().scroll.bar_width = 14.;
        ui.spacing_mut().scroll.bar_inner_margin = 6.;
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
            .scroll_source(egui::scroll_area::ScrollSource::ALL)
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
        initial.alt |= self.alt || self.clone_source_pick;
        for event in &mut raw.events {
            match event {
                egui::Event::ModifiersChanged(modifiers) => {
                    self.hardware_modifiers = *modifiers;
                    modifiers.shift |= self.shift;
                    modifiers.alt |= self.alt || self.clone_source_pick;
                }
                egui::Event::PointerButton { modifiers, .. }
                | egui::Event::Key { modifiers, .. } => {
                    modifiers.shift |= self.shift;
                    modifiers.alt |= self.alt || self.clone_source_pick;
                }
                _ => {}
            }
        }
        raw.events.insert(0, egui::Event::ModifiersChanged(initial));
    }

    fn source_button(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let label = if self.clone_source_pick {
            "Cancel source"
        } else {
            "Set source"
        };
        if button(ui, label, self.clone_source_pick).clicked() {
            self.clone_source_pick = !self.clone_source_pick;
            self.clone_source_contact = false;
            self.alt = false;
            app.ui.status_error = false;
        }
    }

    fn update_source_pick(&mut self, app: &PhotocraftApp, ctx: &egui::Context) {
        if !matches!(app.ui.tool, Tool::CloneStamp | Tool::Healing)
            || !app.ui.dialogs.is_empty()
            || self.sheet.is_some()
            || self.command_tray_open
        {
            self.clone_source_pick = false;
            self.clone_source_contact = false;
        }
        if self.clone_source_pick {
            ctx.input(|input| {
                if input.pointer.button_pressed(egui::PointerButton::Primary)
                    && input.pointer.interact_pos().is_some_and(|pos| {
                        photocraft_ui_egui::canvas::ViewXform::active(app)
                            .is_some_and(|xf| xf.rect.contains(pos))
                    })
                {
                    self.clone_source_contact = true;
                }
                if self.clone_source_contact
                    && input.pointer.button_released(egui::PointerButton::Primary)
                {
                    // This frame's events already carry Alt. Release the latch for the NEXT
                    // contact, including when the user picks the same source point again.
                    self.clone_source_pick = false;
                    self.clone_source_contact = false;
                }
            });
        }
    }

    fn selection_actions(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let has_selection = app
            .session
            .active()
            .is_some_and(|doc| doc.doc.selection.is_some());
        let response = ui.add_enabled(
            has_selection,
            Button::new("Selection").min_size(vec2(44., 44.)),
        );
        egui::Popup::menu(&response).show(|ui| {
            ui.label("Selected pixels · active layer");
            for (label, command) in [
                ("Layer via Copy", "layer.new.layerViaCopy"),
                ("Layer via Cut", "layer.new.layerViaCut"),
                ("Invert selection", "select.inverse"),
                ("Deselect", "select.deselect"),
            ] {
                if ui
                    .add_enabled(
                        menus::is_enabled(app, command),
                        Button::new(label).min_size(vec2(180., 44.)),
                    )
                    .clicked()
                {
                    self.invoke(app, ui.ctx(), command, json!({}));
                    ui.close();
                }
            }
        });
    }
}

#[cfg(test)]
mod tests;
