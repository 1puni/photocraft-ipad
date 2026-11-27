//! Live brush values open generously sized controls instead of wrapping the canvas bar.
use super::*;

impl TabletUi {
    pub(super) fn brush_bar(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let before = app.session.tools.brush.clone();
        let mut after = before.clone();
        ui.horizontal(|ui| {
            if button(
                ui,
                "Brush",
                self.inspector_open && self.inspector == Inspector::Brush,
            )
            .clicked()
            {
                self.inspector = Inspector::Brush;
                self.inspector_open = true;
            }
            ui.menu_button(format!("Size: {:.1} px", after.size), |ui| {
                ui.set_min_width(260.);
                let label = ui.label("Brush size");
                ui.add_sized(
                    [260., 44.],
                    egui::Slider::new(&mut after.size, 0.5..=5000.)
                        .logarithmic(true)
                        .suffix(" px"),
                )
                .labelled_by(label.id);
            });
            ui.menu_button(format!("Opacity: {:.0}%", after.opacity * 100.), |ui| {
                ui.set_min_width(260.);
                let label = ui.label("Brush opacity");
                let mut percent = after.opacity * 100.;
                if ui
                    .add_sized(
                        [260., 44.],
                        egui::Slider::new(&mut percent, 0.0..=100.).suffix("%"),
                    )
                    .labelled_by(label.id)
                    .changed()
                {
                    after.opacity = percent / 100.;
                }
            });
            if matches!(app.ui.tool, Tool::CloneStamp | Tool::Healing) {
                self.source_button(app, ui);
            }
            self.context_options(app, ui);
        });
        photocraft_ui_egui::brush_panel::commit_gesture(app, ui.ctx(), &before, &after);
    }
}
