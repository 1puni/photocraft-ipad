use super::*;
use photocraft_ui_egui::{brush_panel, color_picker_ui};

impl TabletUi {
    pub(super) fn layers(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            self.command(app, ui, "+ Layer", "layer.new.layer");
            self.command(app, ui, "+ Group", "layer.new.group");
            if button(ui, "Select multiple", self.multi_select).clicked() {
                self.multi_select = !self.multi_select;
            }
        });
        let Some(st) = app.session.active() else {
            ui.label("Open or create an image to add layers.");
            return;
        };
        let doc = st.doc.clone();
        let selected = st.selected_layers().to_vec();
        let active = st.active_layer;
        for (depth, layer) in photocraft_ui_egui::layer_tree_ui::display_rows(&doc, false) {
            ui.push_id(layer.id.0,|ui|{
                ui.horizontal(|ui|{
                    ui.add_space((depth as f32*12.).min(48.));
                    if button(ui,if layer.visible{"On"}else{"Off"},false).clicked(){
                        self.invoke(app,ui.ctx(),"layer.setProps",json!({"layer":layer.id.0,"visible":!layer.visible}));
                    }
                    if let photocraft_doc::LayerContent::Group(group)=&layer.content
                        && button(ui,if group.expanded{"−"}else{"+"},false).clicked(){
                            self.invoke(app,ui.ctx(),"layer.setExpanded",json!({"layer":layer.id.0,"expanded":!group.expanded}));
                        }
                    if ui.add_sized([ui.available_width().max(44.),48.],Button::new(&layer.name).selected(selected.contains(&layer.id))).clicked(){
                        self.invoke(app,ui.ctx(),"layer.select",json!({"layer":layer.id.0,"mode":if self.multi_select{"toggle"}else{"replace"}}));
                        app.ui.mask_target=false;
                    }
                });
            });
        }
        if let Some(layer) = active.and_then(|id| doc.layer(id)) {
            ui.separator();
            ui.strong(&layer.name);
            let mut opacity = layer.opacity;
            if ui
                .add(egui::Slider::new(&mut opacity, 0.0..=1.).text("Opacity"))
                .changed()
            {
                self.invoke(
                    app,
                    ui.ctx(),
                    "layer.setProps",
                    json!({"layer":layer.id.0,"opacity":opacity}),
                );
            }
            egui::ComboBox::from_id_salt("ipad-layer-blend")
                .selected_text(layer.blend.label())
                .width(200.)
                .show_ui(ui, |ui| {
                    for mode in photocraft_color::BlendMode::LAYER_MODES {
                        if full_button(ui, mode.label(), layer.blend == mode).clicked() {
                            self.invoke(
                                app,
                                ui.ctx(),
                                "layer.setProps",
                                json!({"layer":layer.id.0,"blend":mode}),
                            );
                        }
                    }
                });
            ui.horizontal_wrapped(|ui| {
                if button(ui, "Lock", layer.locks.all).clicked() {
                    self.invoke(
                        app,
                        ui.ctx(),
                        "layer.setProps",
                        json!({"layer":layer.id.0,"locks":{"all":!layer.locks.all}}),
                    );
                }
                self.command(app, ui, "Duplicate", "layer.duplicate");
                self.command(app, ui, "Delete", "layer.delete");
                self.command(app, ui, "Raise", "layer.arrange.bringForward");
                self.command(app, ui, "Lower", "layer.arrange.sendBackward");
                if layer.mask.is_some() {
                    if button(ui, "Paint mask", app.ui.mask_target).clicked() {
                        app.ui.mask_target = !app.ui.mask_target;
                    }
                } else {
                    self.command(app, ui, "Add mask", "layer.layerMask.revealAll");
                }
                if button(ui, "More layer actions", false).clicked() {
                    self.open_sheet(Sheet::Commands);
                    self.category = "Layer".into();
                }
            });
        }
    }
    pub(super) fn brush(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
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
    pub(super) fn color(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if button(ui, "Foreground", !self.color_background).clicked() {
                self.color_background = false;
                self.color_hex.clear();
            }
            if button(ui, "Background", self.color_background).clicked() {
                self.color_background = true;
                self.color_hex.clear();
            }
        });
        let source = if self.color_background {
            app.session.tools.background
        } else {
            app.session.tools.foreground
        };
        let mut hsv = color_picker_ui::rgb_to_hsv([source[0], source[1], source[2]]);
        if hsv[1] <= f32::EPSILON || hsv[2] <= f32::EPSILON {
            hsv[0] = self.color_hue;
        }
        if self.color_source != Some(source) {
            self.color_hex = color_picker_ui::hex([source[0], source[1], source[2]]);
            self.color_source = Some(source);
        }
        let mut changed = false;
        let edge = ui.available_width().min(280.);
        let (rect, response) =
            ui.allocate_exact_size(vec2(edge, edge.min(210.)), egui::Sense::click_and_drag());
        // Tessellate a saturation/value field; hue is controlled by the wide slider below.
        for y in 0..24 {
            for x in 0..24 {
                let rgb = color_picker_ui::hsv_to_rgb(hsv[0], x as f32 / 23., 1. - y as f32 / 23.);
                let tile = egui::Rect::from_min_max(
                    rect.min
                        + vec2(
                            x as f32 / 24. * rect.width(),
                            y as f32 / 24. * rect.height(),
                        ),
                    rect.min
                        + vec2(
                            (x + 1) as f32 / 24. * rect.width(),
                            (y + 1) as f32 / 24. * rect.height(),
                        ),
                );
                ui.painter().rect_filled(
                    tile,
                    0.,
                    egui::Color32::from_rgb(
                        (rgb[0] * 255.) as u8,
                        (rgb[1] * 255.) as u8,
                        (rgb[2] * 255.) as u8,
                    ),
                );
            }
        }
        let marker = rect.min + vec2(hsv[1] * rect.width(), (1. - hsv[2]) * rect.height());
        ui.painter()
            .circle_stroke(marker, 6., egui::Stroke::new(2., egui::Color32::WHITE));
        ui.painter()
            .circle_stroke(marker, 8., egui::Stroke::new(1., egui::Color32::BLACK));
        if (response.dragged() || response.clicked())
            && let Some(p) = response.interact_pointer_pos()
        {
            hsv[1] = ((p.x - rect.left()) / rect.width()).clamp(0., 1.);
            hsv[2] = (1. - (p.y - rect.top()) / rect.height()).clamp(0., 1.);
            changed = true;
        }
        changed |= ui
            .add(egui::Slider::new(&mut hsv[0], 0.0..=359.9).text("Hue"))
            .changed();
        self.color_hue = hsv[0];
        let rgb = color_picker_ui::hsv_to_rgb(hsv[0], hsv[1], hsv[2]);
        if self.color_hex.is_empty() || changed {
            self.color_hex = color_picker_ui::hex(rgb);
        }
        ui.horizontal(|ui| {
            ui.add_sized([130., 44.], egui::TextEdit::singleline(&mut self.color_hex));
            if button(ui, "Set hex", false).clicked() {
                if let Some(rgb) = color_picker_ui::parse_hex(&self.color_hex) {
                    self.set_color(app, ui.ctx(), rgb);
                } else {
                    self.message = "Enter a six-digit hex colour, for example #245d85.".into();
                }
            }
        });
        if changed {
            self.set_color(app, ui.ctx(), rgb);
        }
        ui.horizontal_wrapped(|ui|{
            if button(ui,"Swap colours",false).clicked(){self.invoke(app,ui.ctx(),"tools.setColors",json!({"foreground":app.session.tools.background,"background":app.session.tools.foreground}));self.color_hex.clear();}
            if button(ui,"Eyedropper",app.ui.tool==Tool::Eyedropper).clicked(){app.ui.tool=Tool::Eyedropper;}
        });
    }
    pub(super) fn set_color(
        &mut self,
        app: &mut PhotocraftApp,
        ctx: &egui::Context,
        rgb: [f32; 3],
    ) {
        let key = if self.color_background {
            "background"
        } else {
            "foreground"
        };
        self.invoke(
            app,
            ctx,
            "tools.setColors",
            json!({key:[rgb[0],rgb[1],rgb[2],1.]}),
        );
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
