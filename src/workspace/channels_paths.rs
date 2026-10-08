use super::*;
use photocraft_engine::channel_cmds::{self, ChannelTarget};
use photocraft_ui_egui::vector_ui::{self, PathRow};

impl TabletUi {
    pub(super) fn channels(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let Some(st) = app.session.active() else {
            ui.label("Open an image to edit its channels.");
            return;
        };
        let doc = st.doc.clone();
        let view = st.channel_view.clone();
        let mode = doc.pixel_format().mode;
        let mut rows = Vec::new();
        if doc.mode != photocraft_doc::ColorMode::Multichannel {
            rows.push((
                channel_cmds::composite_name(mode).to_owned(),
                json!("composite"),
                view.target == ChannelTarget::Composite,
                view.visible_colors(mode.color_channels()) > 0,
            ));
            if mode.color_channels() > 1 {
                for (i, name) in channel_cmds::color_names(mode).iter().enumerate() {
                    rows.push((
                        (*name).into(),
                        json!({"color":i}),
                        view.target == ChannelTarget::Color(i),
                        view.color_visible(i),
                    ));
                }
            }
        }
        for (i, channel) in doc.channels.iter().enumerate() {
            rows.push((
                channel.name.clone(),
                json!(i),
                view.target == ChannelTarget::Alpha(i),
                view.alpha_shown(i),
            ));
        }
        for (label, reference, selected, visible) in rows {
            ui.push_id(reference.to_string(), |ui| {
                ui.horizontal(|ui| {
                    if button(ui, if visible { "Visible" } else { "Hidden" }, false).clicked() {
                        self.invoke(
                            app,
                            ui.ctx(),
                            "channel.setVisible",
                            json!({"channel":reference,"visible":!visible}),
                        );
                    }
                    if ui
                        .add_sized(
                            [ui.available_width(), 48.],
                            Button::new(label).selected(selected && !app.ui.mask_target),
                        )
                        .clicked()
                    {
                        self.invoke(
                            app,
                            ui.ctx(),
                            "channel.target",
                            json!({"channel":reference}),
                        );
                        app.ui.mask_target = false;
                        app.ui.vector_mask_target = false;
                    }
                })
            });
        }
        let reference = match view.target {
            ChannelTarget::Composite => json!("composite"),
            ChannelTarget::Color(i) => json!({"color":i}),
            ChannelTarget::Alpha(i) => json!(i),
        };
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            if button(ui, "Load as selection", false).clicked() {
                self.invoke(
                    app,
                    ui.ctx(),
                    "select.loadSelection",
                    json!({"channel":reference}),
                );
            }
            self.command(app, ui, "Save selection…", "select.saveSelection");
            self.command(app, ui, "New alpha channel", "channel.new");
            if button(ui, "Duplicate channel", false).clicked() {
                self.invoke(
                    app,
                    ui.ctx(),
                    "channel.duplicate",
                    json!({"channel": reference}),
                );
            }
            self.command(app, ui, "Delete channel", "channel.delete");
            self.command(app, ui, "Quick mask", "select.editInQuickMaskMode");
        });
        ui.label("The selected channel receives painting. Return to the composite channel to paint layers in colour.");
    }
    pub(super) fn paths(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let Some(st) = app.session.active() else {
            ui.label("Open an image to work with paths.");
            return;
        };
        let doc = st.doc.clone();
        let rows = vector_ui::path_rows(&doc, st.active_layer);
        if rows.is_empty() {
            ui.label("Draw a path with Pencil using the Pen tool, or create one from a selection.");
        }
        for row in &rows {
            let key = match row.kind {
                PathRow::Work => "work",
                PathRow::Layer => "layer",
                PathRow::Saved => row.name.as_str(),
            };
            if full_button(ui, &row.name, app.ui.selected_path.as_deref() == Some(key)).clicked() {
                app.ui.selected_path = Some(key.into());
            }
        }
        let selected = app
            .ui
            .selected_path
            .clone()
            .unwrap_or_else(|| "work".into());
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            if button(ui, "Pen tool", app.ui.tool == Tool::Pen).clicked() {
                app.ui.tool = Tool::Pen;
                self.sheet = None;
            }
            if button(ui, "Edit points", app.ui.tool == Tool::DirectSelection).clicked() {
                app.ui.tool = Tool::DirectSelection;
                self.sheet = None;
            }
            for (label, id, params) in [
                (
                    "Make selection",
                    "path.toSelection",
                    json!({"name":selected}),
                ),
                (
                    "Path from selection",
                    "select.toWorkPath",
                    json!({"tolerance":2.0}),
                ),
                ("Fill path", "path.fill", json!({"name":selected})),
                (
                    "Stroke with brush",
                    "path.stroke",
                    json!({"name":selected,"tool":"brush"}),
                ),
                (
                    "Vector mask",
                    "layer.vectorMask.fromPath",
                    json!({"name":selected}),
                ),
                ("Delete path", "path.delete", json!({"name":selected})),
            ] {
                if ui
                    .add_enabled(
                        menus::is_enabled(app, id),
                        Button::new(label).min_size(vec2(100., 44.)),
                    )
                    .clicked()
                {
                    self.invoke(app, ui.ctx(), id, params);
                }
            }
        });
        ui.separator();
        ui.label("Save or rename the selected path");
        ui.horizontal_wrapped(|ui| {
            ui.add_sized([220., 44.], egui::TextEdit::singleline(&mut self.path_name));
            if ui
                .add_enabled(
                    !self.path_name.trim().is_empty(),
                    Button::new("Save name").min_size(vec2(100., 44.)),
                )
                .clicked()
            {
                self.invoke(
                    app,
                    ui.ctx(),
                    "path.rename",
                    json!({"name":selected,"to":self.path_name.trim()}),
                );
                if self.message.is_empty() {
                    app.ui.selected_path = Some(self.path_name.trim().into());
                }
            }
        });
    }
}
