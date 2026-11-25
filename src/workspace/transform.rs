//! Discoverable tablet entry points into the shared transform session.
use super::*;
use photocraft_ui_egui::{state::TransformMode, transform_tool};

impl TabletUi {
    pub(super) fn transform_entry(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        self.command(app, ui, "Resize / Rotate", "edit.freeTransform");
        self.command(app, ui, "Warp", "edit.transform.warp");
    }

    pub(super) fn transform_modes(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let Some(t) = app.ui.transform.as_ref() else {
            return;
        };
        let warp = t.warp.is_some();
        let mode = t.mode;
        let lone_target = t.target.is_some();
        ui.menu_button(
            if warp {
                "Mode: Warp"
            } else {
                match mode {
                    TransformMode::Free => "Mode: Resize / Rotate",
                    TransformMode::Skew => "Mode: Skew",
                    TransformMode::Distort => "Mode: Distort",
                    TransformMode::Perspective => "Mode: Perspective",
                }
            },
            |ui| {
                if warp {
                    ui.label("Apply to keep the warp before resizing again.");
                    if button(ui, "Discard warp", false).clicked() {
                        transform_tool::leave_warp(app);
                        ui.close();
                    }
                } else {
                    for (label, id, selected) in [
                        (
                            "Resize / Rotate",
                            "edit.freeTransform",
                            mode == TransformMode::Free,
                        ),
                        ("Skew", "edit.transform.skew", mode == TransformMode::Skew),
                        (
                            "Distort",
                            "edit.transform.distort",
                            mode == TransformMode::Distort,
                        ),
                        (
                            "Perspective",
                            "edit.transform.perspective",
                            mode == TransformMode::Perspective,
                        ),
                    ] {
                        if button(ui, label, selected).clicked() {
                            self.invoke(app, ui.ctx(), id, json!({}));
                            ui.close();
                        }
                    }
                    ui.add_enabled_ui(!lone_target, |ui| {
                        if button(ui, "Warp", false).clicked() {
                            self.invoke(app, ui.ctx(), "edit.transform.warp", json!({}));
                            ui.close();
                        }
                    });
                }
            },
        );
        ui.label(if warp {
            "Drag mesh points to warp"
        } else {
            match mode {
                TransformMode::Free => "Corners resize · outside rotates",
                TransformMode::Skew => "Drag an edge to skew",
                TransformMode::Distort => "Drag corners independently",
                TransformMode::Perspective => "Drag corners for perspective",
            }
        });
    }
}
