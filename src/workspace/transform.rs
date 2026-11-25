//! Object manipulation gets a focused context bar, not the generic tool strip.
use super::*;
use photocraft_ui_egui::{state::TransformMode, transform_tool};

impl TabletUi {
    pub(super) fn transform_entry(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        self.command(app, ui, "Transform", "edit.freeTransform");
    }

    pub(super) fn sync_transform_modifiers(&mut self, app: &PhotocraftApp) {
        // Hidden virtual keyboard latches must not change direct manipulation.
        // Restore them when returning to the previous tool; real keys still work.
        if app.ui.transform.is_some() {
            if self.transform_modifiers.is_none() {
                self.transform_modifiers = Some((self.shift, self.alt));
                self.shift = false;
                self.alt = false;
                self.clone_source_pick = false;
            }
        } else if let Some((shift, alt)) = self.transform_modifiers.take() {
            self.shift = shift;
            self.alt = alt;
        }
    }

    pub(super) fn transform_bar(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let Some(t) = app.ui.transform.as_ref() else {
            return;
        };
        let warp = t.warp.is_some();
        let mode = t.mode;
        let lone_target = t.target.is_some();
        ui.horizontal(|ui| {
            for (label, id, selected, enabled) in [
                ("Resize", "edit.freeTransform", !warp && mode == TransformMode::Free, !warp),
                ("Distort", "edit.transform.distort", !warp && mode == TransformMode::Distort, !warp),
                ("Warp", "edit.transform.warp", warp, !lone_target),
            ] {
                if ui.add_enabled(enabled, Button::new(label).selected(selected).min_size(vec2(44., 44.))).clicked() && !selected {
                    self.invoke(app, ui.ctx(), id, json!({}));
                }
            }
            let more_label = match (warp, mode) {
                (false, TransformMode::Skew) => "Skew ▾",
                (false, TransformMode::Perspective) => "Perspective ▾",
                _ => "More",
            };
            ui.menu_button(more_label, |ui| {
                ui.set_max_width(280.);
                if warp {
                    ui.label("Drag the mesh points to reshape this layer. Choose Done to keep the warp before resizing again.");
                    if button(ui, "Discard warp", false).clicked() {
                        transform_tool::leave_warp(app);
                        ui.close();
                    }
                } else {
                    ui.label("Drag inside to move, corners to resize, edges to stretch, or just outside to rotate. Two fingers navigate the canvas.");
                    ui.separator();
                    for (label, id, selected) in [
                        ("Skew", "edit.transform.skew", mode == TransformMode::Skew),
                        ("Perspective", "edit.transform.perspective", mode == TransformMode::Perspective),
                    ] {
                        if full_button(ui, label, selected).clicked() {
                            self.invoke(app, ui.ctx(), id, json!({}));
                            ui.close();
                        }
                    }
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let tokens = Tokens::get(ui.ctx());
                if ui.add(Button::new(egui::RichText::new("Done").color(tokens.primary_text)).fill(tokens.primary_bg).min_size(vec2(64., 44.))).clicked() {
                    transform_tool::commit(app);
                }
                if button(ui, "Cancel", false).clicked() {
                    transform_tool::cancel(app);
                }
            });
        });
    }
}
