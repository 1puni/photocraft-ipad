//! Crop choices remain beside the canvas, with an explicit pixel-retention policy.
use super::*;
use photocraft_ui_egui::{canvas, crop_ui};

pub(super) const CROP_RATIOS: &[(&str, &str)] = &[
    ("", "Free"),
    ("original", "Original"),
    ("1:1", "Square"),
    ("4:3", "4:3"),
    ("3:4", "3:4"),
    ("16:9", "16:9"),
    ("9:16", "9:16"),
];

impl TabletUi {
    pub(super) fn crop_bar(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let name = CROP_RATIOS
                .iter()
                .find(|(value, _)| *value == app.ui.tool_options.crop_ratio)
                .map_or("Custom", |(_, name)| *name);
            ui.menu_button(format!("Crop: {name}"), |ui| {
                for &(value, label) in CROP_RATIOS {
                    if full_button(ui, label, app.ui.tool_options.crop_ratio == value).clicked() {
                        crop_ui::set_ratio(app, value);
                        ui.close();
                    }
                }
            });
            ui.menu_button(
                if app.ui.tool_options.crop_delete {
                    "Outside: Delete"
                } else {
                    "Outside: Keep"
                },
                |ui| {
                    ui.set_max_width(260.);
                    ui.label("Pixels outside the crop");
                    for (delete, label) in [
                        (false, "Keep cropped pixels"),
                        (true, "Delete cropped pixels"),
                    ] {
                        if full_button(ui, label, app.ui.tool_options.crop_delete == delete)
                            .clicked()
                        {
                            app.ui.tool_options.crop_delete = delete;
                            ui.close();
                        }
                    }
                    ui.label("Keeping pixels lets a later canvas expansion reveal them again.");
                },
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let t = Tokens::get(ui.ctx());
                let pending = crop_ui::pending(app);
                if ui
                    .add_enabled(
                        pending,
                        Button::new(egui::RichText::new("Apply").color(t.primary_text))
                            .fill(t.primary_bg)
                            .min_size(vec2(64., 44.)),
                    )
                    .clicked()
                {
                    canvas::commit_crop(app);
                }
                if button(ui, "Cancel", false).clicked() {
                    crop_ui::cancel(app);
                }
            });
        });
    }
}
