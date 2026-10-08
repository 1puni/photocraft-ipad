use super::*;
use photocraft_ui_egui::brush_panel;

impl TabletUi {
    pub(super) fn brush(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        if full_button(ui, "All brush dynamics & preview…", false).clicked() {
            self.open_sheet(Sheet::Brush);
        }
        let before = app.session.tools.brush.clone();
        let mut b = before.clone();
        ui.strong("Brush tip");
        ui.add(
            egui::Slider::new(&mut b.size, 0.5..=5000.)
                .logarithmic(true)
                .text("Size"),
        );
        for (value, label) in [
            (&mut b.hardness, "Hardness"),
            (&mut b.opacity, "Opacity"),
            (&mut b.flow, "Flow"),
            (&mut b.roundness, "Roundness"),
        ] {
            ui.add(egui::Slider::new(value, 0.0..=1.).text(label));
        }
        ui.add(egui::Slider::new(&mut b.spacing, 0.01..=2.).text("Spacing"));
        ui.add(egui::Slider::new(&mut b.angle, -180.0..=180.).text("Angle"));
        ui.separator();
        ui.strong("Apple Pencil");
        ui.checkbox(&mut b.pressure_size, "Pressure controls size");
        ui.checkbox(&mut b.pressure_opacity, "Pressure controls opacity");
        ui.checkbox(&mut b.shape_dynamics.enabled, "Shape dynamics");
        ui.add(
            egui::Slider::new(&mut b.shape_dynamics.tilt_scale, 0.0..=1.).text("Tilt influence"),
        );
        ui.add(egui::Slider::new(&mut b.smoothing.amount, 0.0..=1.).text("Smoothing"));
        ui.checkbox(&mut b.smoothing.catch_up, "Catch up to Pencil");
        ui.checkbox(&mut b.wet_edges, "Wet edges");
        ui.checkbox(&mut b.build_up, "Airbrush build-up");
        brush_panel::commit_gesture(app, ui.ctx(), &before, &b);
        ui.separator();
        ui.strong("Presets");
        let presets = app.session.tools.presets.clone();
        for preset in presets {
            if full_button(ui, &preset.name, brush_panel::is_current(&preset.brush, &b)).clicked() {
                self.invoke(
                    app,
                    ui.ctx(),
                    "tools.setBrush",
                    json!({"preset":preset.name}),
                );
            }
        }
    }
    pub(super) fn history(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.command(app, ui, "Undo", "edit.undo");
            self.command(app, ui, "Redo", "edit.redo");
        });
        let Some(st) = app.session.active() else {
            return;
        };
        let entries = st.history.entries();
        let current = entries.len().saturating_sub(1);
        let all: Vec<String> = entries
            .iter()
            .cloned()
            .chain(st.history.redo_labels().map(str::to_owned))
            .collect();
        for (i, label) in all.iter().enumerate() {
            if ui
                .add_enabled(
                    app.ui.transform.is_none(),
                    Button::new(label)
                        .selected(i == current)
                        .min_size(vec2(ui.available_width(), 48.)),
                )
                .clicked()
            {
                let command = if i < current {
                    "edit.undo"
                } else {
                    "edit.redo"
                };
                for _ in 0..i.abs_diff(current) {
                    self.invoke(app, ui.ctx(), command, json!({}));
                    if !self.message.is_empty() {
                        break;
                    }
                }
            }
        }
    }
}

impl TabletUi {
    pub(super) fn brush_studio(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let before = app.session.tools.brush.clone();
        let mut brush = before.clone();
        let tokens = Tokens::get(ui.ctx());
        let width = ui.available_width().clamp(1., 600.);
        let texture = photocraft_ui_egui::brush_preview::stroke_texture(
            ui.ctx(),
            "ipad-brush-preview",
            &brush,
            width as u32,
            80,
            tokens.text,
        );
        ui.add(egui::Image::new((texture.id(), vec2(width, 80.))));
        egui::ComboBox::from_id_salt("ipad-brush-section")
            .width(width - 32.)
            .selected_text(
                brush_panel::SECTIONS
                    .get(app.ui.brush_section)
                    .map_or("Brush Tip Shape", |s| s.0),
            )
            .show_ui(ui, |ui| {
                for (i, (label, _)) in brush_panel::SECTIONS.iter().enumerate() {
                    if full_button(ui, label, app.ui.brush_section == i).clicked() {
                        app.ui.brush_section = i;
                    }
                }
            });
        let section = app.ui.brush_section.min(brush_panel::SECTIONS.len() - 1);
        if let Some(enabled) = brush_panel::section_flag(&mut brush, section) {
            ui.checkbox(enabled, "Enable this section");
        }
        if let Some(locked) = brush_panel::section_lock(&mut brush, section) {
            ui.checkbox(locked, "Keep these settings when switching presets");
        }
        let enabled = brush_panel::section_flag(&mut brush, section).is_none_or(|enabled| *enabled);
        ui.add_enabled_ui(enabled, |ui| {
            brush_panel::section_body(ui, &mut brush, section, &app.session.tools.presets)
        });
        brush_panel::commit_gesture(app, ui.ctx(), &before, &brush);
        ui.separator();
        ui.label("Save these settings as a preset");
        ui.horizontal_wrapped(|ui| {
            ui.add_sized(
                [220., 44.],
                egui::TextEdit::singleline(&mut self.brush_name),
            );
            if ui
                .add_enabled(
                    !self.brush_name.trim().is_empty(),
                    Button::new("Save preset").min_size(vec2(100., 44.)),
                )
                .clicked()
            {
                self.invoke(
                    app,
                    ui.ctx(),
                    "brush.presets.save",
                    json!({"name":self.brush_name.trim()}),
                );
            }
        });
    }
}
