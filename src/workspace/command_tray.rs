use super::*;
use crate::{
    commands::{self, Action},
    speech::Phase,
};
impl TabletUi {
    pub(super) fn command_bar(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let enabled = self.sheet.is_none()
            && app.ui.dialogs.is_empty()
            && !egui::Popup::is_any_open(ui.ctx());
        if !enabled {
            self.dictation.cancel();
        }
        if photocraft_ui_egui::tool_feedback::is_selection_tool(app.ui.tool) {
            self.last_selection = app.ui.tool;
        }
        if self.dictation.take_open() {
            self.command_tray_open = true;
        }
        if let Some(text) = self.dictation.take_final() {
            self.command_text = text;
            self.submit_command(app);
            ui.ctx().request_repaint();
        }
        let t = Tokens::get(ui.ctx());
        let state = self.dictation.snapshot();
        egui::Panel::bottom("ipad-command-bar")
            .exact_size(48.)
            .frame(
                Frame::NONE
                    .fill(t.chrome)
                    .inner_margin(egui::Margin::symmetric(8, 2)),
            )
            .show(ui, |ui| {
                ui.add_enabled_ui(enabled, |ui| {
                    ui.horizontal(|ui| {
                        let mic = button(
                            ui,
                            if self.dictation.has_button() {
                                ""
                            } else if state.phase == Phase::Idle {
                                "Speak"
                            } else {
                                "Cancel"
                            },
                            state.phase != Phase::Idle,
                        );
                        #[cfg(target_arch = "wasm32")]
                        self.dictation.place_button(
                            enabled.then_some(mic.rect),
                            ui.ctx().zoom_factor(),
                            t.text,
                            t.accent_text,
                        );
                        if mic.clicked() {
                            // Native builds/tests have no browser overlay. The wasm button's
                            // own click listener starts recognition synchronously with the tap.
                            self.command_tray_open = true;
                        }
                        let extra = 100.;
                        let response = ui.add_sized(
                            [ui.available_width() - extra, 44.],
                            egui::TextEdit::singleline(&mut self.command_text)
                                .hint_text("Find or speak a command…"),
                        );
                        if response.changed() || response.gained_focus() {
                            self.command_tray_open = true;
                            self.tool_receipt = None;
                        }
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.submit_command(app);
                        }
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
                        }
                        if let Some((before, after)) = self.tool_receipt {
                            if app.ui.tool != after {
                                self.tool_receipt = None;
                            } else if navigation::icon_button(
                                ui,
                                "undo-2",
                                "Revert tool switch",
                                false,
                                44.,
                            )
                            .on_hover_text(format!(
                                "{} selected · Restore {}",
                                after.label(),
                                before.label()
                            ))
                            .clicked()
                            {
                                app.ui.tool = before;
                                self.tool_receipt = None;
                                self.command_text.clear();
                            }
                        } else {
                            ui.allocate_space(vec2(44., 44.));
                        }
                    });
                });
            });
    }
    fn submit_command(&mut self, app: &mut PhotocraftApp) {
        let choices = commands::find(app, &self.command_text, self.last_selection);
        if let Some(tool) = commands::immediate_tool(&choices) {
            let before = app.ui.tool;
            app.ui.tool = tool;
            self.tool_receipt = Some((before, tool));
            self.command_text.clear();
            self.command_tray_open = false;
        } else {
            self.command_tray_open = true;
        }
    }
    pub(super) fn command_tray(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        if !self.command_tray_open || self.sheet.is_some() || !app.ui.dialogs.is_empty() {
            return;
        }
        let state = self.dictation.snapshot();
        let t = Tokens::get(ui.ctx());
        egui::Panel::bottom("ipad-command-tray").exact_size((ui.ctx().content_rect().height()*0.24).clamp(150.,230.))
            .frame(Frame::NONE.fill(t.dock).inner_margin(8))
            .show(ui,|ui| {
                ui.horizontal(|ui| {
                    ui.strong(match state.phase {Phase::Idle=>"Commands",Phase::Starting=>"Starting microphone…",Phase::Listening=>"Listening…"});
                    if button(ui,"Close commands",false).clicked() {self.command_tray_open=false;self.dictation.cancel();}

                });
                if !state.error.is_empty() {ui.label(&state.error);}
                else if !state.interim.is_empty() {ui.label(&state.interim);}
                if state.phase != Phase::Idle {
                    ui.label("Say one short command. Tap the microphone again to cancel.");
                    return;
                }
                if self.command_text.trim().is_empty() {
                    ui.label("Try “brush tool”, “selection tool”, “levels” or a menu command.");
                    ui.label("English browser speech may use its provider’s online service. PhotoCraft does not store audio.");
                    if !state.supported {ui.label("Browser speech is unavailable here. Type, or use keyboard dictation.");}
                    return;
                }
                let choices=commands::find(app,&self.command_text,self.last_selection);
                if choices.is_empty() {ui.label("No matching command. Try a tool name or menu path.");}
                egui::ScrollArea::vertical().id_salt("ipad-command-results").auto_shrink([false,false]).show(ui,|ui| {
                    for choice in choices {
                        if ui.add_enabled(choice.enabled,Button::new(format!("{}   ·   {}",choice.label,choice.path)).frame(false).min_size(vec2(ui.available_width(),44.))).clicked() {
                            match choice.action {
                                Action::Tool(tool)=> {let before=app.ui.tool;app.ui.tool=tool;self.tool_receipt=Some((before,tool));}
                                Action::Command(id)=> {self.invoke(app,ui.ctx(),&id,json!({}));self.tool_receipt=None;}
                            }
                            self.command_tray_open=false;self.dictation.cancel();self.command_text.clear();
                        }
                    }
                });
            });
    }
}
