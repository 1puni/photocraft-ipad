//! On-demand command search. Text entry, including dictation, belongs to the native keyboard.
use super::*;
use crate::commands::{self, Action};

impl TabletUi {
    fn submit_command(&mut self, app: &mut PhotocraftApp) {
        let choices = commands::find(app, &self.command_text, self.last_selection);
        if let Some(tool) = commands::immediate_tool(&choices) {
            let before = app.ui.tool;
            app.ui.tool = tool;
            self.tool_receipt = Some((before, tool));
            self.command_text.clear();
            self.command_tray_open = false;
        }
    }

    pub(super) fn command_tray(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        if !self.command_tray_open || self.sheet.is_some() || !app.ui.dialogs.is_empty() {
            return;
        }
        let ctx = ui.ctx().clone();
        let screen = ctx.content_rect();
        let width = (screen.width() - 40.).clamp(240., 560.);
        let body_height = (screen.height() - 270.).clamp(60., 360.);
        let mut close = false;
        let modal = egui::Modal::new(egui::Id::new("ipad-command-search")).show(&ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().interact_size = vec2(44., 44.);
            ui.horizontal(|ui| {
                ui.heading("Find command");
                if navigation::icon_button(ui, "x", "Close commands", false, 44.).clicked() {
                    close = true;
                }
            });
            let input = ui.add_sized(
                [width, 44.],
                egui::TextEdit::singleline(&mut self.command_text).hint_text("Tool or command…"),
            );
            if std::mem::take(&mut self.command_focus_requested) {
                input.request_focus();
            }
            if input.changed() {
                self.command_group.clear();
            }
            if input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.submit_command(app);
            }
            if let Some((before, after)) = self.tool_receipt {
                if app.ui.tool != after {
                    self.tool_receipt = None;
                } else if button(ui, &format!("Restore {}", before.label()), false).clicked() {
                    app.ui.tool = before;
                    self.tool_receipt = None;
                    close = true;
                }
            }
            let choices = commands::find(app, &self.command_text, self.last_selection);
            let mut groups = std::collections::BTreeMap::<String, usize>::new();
            for choice in &choices {
                *groups
                    .entry(choice.path.split(" / ").next().unwrap_or("Commands").into())
                    .or_default() += 1;
            }
            egui::ScrollArea::horizontal()
                .scroll_source(egui::scroll_area::ScrollSource::ALL)
                .id_salt("ipad-command-groups")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.set_min_height(44.);
                        if choices.is_empty() {
                            ui.label("Tools and menu commands");
                            return;
                        }
                        if button(
                            ui,
                            &format!("All · {}", choices.len()),
                            self.command_group.is_empty(),
                        )
                        .clicked()
                        {
                            self.command_group.clear();
                        }
                        for (group, count) in &groups {
                            if button(
                                ui,
                                &format!("{group} · {count}"),
                                self.command_group == *group,
                            )
                            .clicked()
                            {
                                self.command_group = group.clone();
                            }
                        }
                    });
                });
            egui::ScrollArea::vertical()
                .scroll_source(egui::scroll_area::ScrollSource::ALL)
                .auto_shrink([false, false])
                .id_salt(egui::Id::new((
                    "ipad-command-results",
                    &self.command_text,
                    &self.command_group,
                )))
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .max_height(body_height)
                .min_scrolled_height(body_height)
                .show(ui, |ui| {
                    if self.command_text.trim().is_empty() {
                        ui.label("Try “brush tool”, “selection tool” or “levels”.");
                        ui.label("Use the keyboard’s microphone to dictate.");
                        return;
                    }
                    if choices.is_empty() {
                        ui.label("No matching command. Try a tool name or menu path.");
                    }
                    for choice in choices {
                        if !self.command_group.is_empty()
                            && choice.path.split(" / ").next() != Some(self.command_group.as_str())
                        {
                            continue;
                        }
                        if command_result(ui, &choice).clicked() {
                            match choice.action {
                                Action::Tool(tool) => {
                                    let before = app.ui.tool;
                                    app.ui.tool = tool;
                                    self.tool_receipt = Some((before, tool));
                                }
                                Action::Command(id) => {
                                    self.invoke(app, &ctx, &id, json!({}));
                                    self.tool_receipt = None;
                                }
                            }
                            self.command_text.clear();
                            close = true;
                        }
                    }
                });
        });
        if close || modal.should_close() {
            self.command_tray_open = false;
            self.command_focus_requested = false;
        }
    }
}

fn command_result(ui: &mut Ui, choice: &commands::Choice) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let mut text = egui::text::LayoutJob::default();
    text.append(
        &choice.label,
        0.,
        egui::TextFormat {
            font_id: egui::FontId::proportional(14.),
            color: t.text,
            ..Default::default()
        },
    );
    let suffix = if choice.enabled {
        ""
    } else {
        " · unavailable for this document/layer"
    };
    text.append(
        &format!("\n{}{suffix}", choice.path),
        0.,
        egui::TextFormat {
            font_id: egui::FontId::proportional(11.),
            color: t.text_dim,
            ..Default::default()
        },
    );
    let response = ui
        .push_id((&choice.path, &choice.label), |ui| {
            ui.add_enabled(
                choice.enabled,
                Button::new(text)
                    .frame(false)
                    .min_size(vec2(ui.available_width(), 48.)),
            )
        })
        .inner;
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            choice.enabled,
            format!("{}   ·   {}", choice.label, choice.path),
        )
    });
    response
}
