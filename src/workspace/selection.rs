//! Selection construction and its next useful action stay beside the canvas.
use super::layers::LayerTarget;
use super::*;

impl TabletUi {
    pub(super) fn selection_bar(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let modes = ["New", "Add", "Subtract", "Intersect"];
            let current = modes.get(app.ui.selection_mode as usize).unwrap_or(&"New");
            ui.menu_button(format!("Mode: {current}"), |ui| {
                for (i, label) in modes.iter().enumerate() {
                    if full_button(ui, label, app.ui.selection_mode == i as u8).clicked() {
                        app.ui.selection_mode = i as u8;
                        // Explicit mode selection supersedes virtual keyboard overrides.
                        self.shift = false;
                        self.alt = false;
                        ui.close();
                    }
                }
            });
            self.selection_actions(app, ui);
            self.context_options(app, ui);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let target = app.session.active().and_then(|st| {
                    st.active_layer
                        .and_then(|id| st.doc.layer(id).map(|l| (id, l.mask.is_some())))
                });
                if let Some((id, true)) = target {
                    if button(ui, "Edit mask", false).clicked() {
                        self.open_mask_controls(app, ui.ctx(), id, LayerTarget::Mask);
                    }
                } else {
                    let enabled = target.is_some()
                        && app.session.active().is_some_and(|st| {
                            st.doc.selection.is_some() && st.doc.quick_mask.is_none()
                        });
                    ui.add_enabled_ui(enabled, |ui| {
                        ui.menu_button("Make mask", |ui| {
                            ui.label("Mask the active layer without deleting pixels.");
                            for (label, command) in [
                                ("Keep selected area", "layer.layerMask.revealSelection"),
                                ("Hide selected area", "layer.layerMask.hideSelection"),
                            ] {
                                if full_button(ui, label, false).clicked() {
                                    if let Some((id, false)) = target {
                                        self.target_layer(app, ui.ctx(), id, LayerTarget::Image);
                                        if self.message.is_empty() {
                                            self.invoke(
                                                app,
                                                ui.ctx(),
                                                command,
                                                json!({"layer":id.0}),
                                            );
                                            if self.message.is_empty() {
                                                self.open_mask_controls(
                                                    app,
                                                    ui.ctx(),
                                                    id,
                                                    LayerTarget::Mask,
                                                );
                                            }
                                        }
                                    }
                                    ui.close();
                                }
                            }
                        });
                    });
                }
            });
        });
    }

    pub(super) fn context_options(&mut self, app: &PhotocraftApp, ui: &mut Ui) {
        ui.menu_button("Options", |ui| {
            if full_button(ui, "Tool settings", false).clicked() {
                self.inspector = Inspector::Tool;
                self.inspector_open = true;
                ui.close();
            }
            ui.separator();
            ui.checkbox(&mut self.shift, "Shift");
            if !matches!(app.ui.tool, Tool::CloneStamp | Tool::Healing) {
                ui.checkbox(&mut self.alt, "Alt");
            }
        });
    }
}
