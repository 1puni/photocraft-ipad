use super::*;
impl TabletUi {
    pub(super) fn sheets(&mut self, app: &mut PhotocraftApp, ctx: &egui::Context) {
        let Some(sheet) = self.sheet else { return };
        let mut close = false;
        let title = match sheet {
            Sheet::Commands => "Studio",
            Sheet::Files => "Documents & files",
            Sheet::Brush => "Brush studio",
            Sheet::Channels => "Channels",
            Sheet::Paths => "Paths",
        };
        let width = (ctx.content_rect().width() - 40.).clamp(240., 640.);
        let modal = egui::Modal::new(egui::Id::new(("ipad-sheet", title))).show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().interact_size = vec2(44., 44.);
            ui.spacing_mut().scroll.floating = false;
            ui.spacing_mut().scroll.bar_width = 14.;
            ui.spacing_mut().scroll.bar_inner_margin = 6.;
            ui.horizontal(|ui| {
                ui.heading(title);
                if button(ui, "Close", false).clicked() { close = true; }
            });
            if sheet == Sheet::Commands { self.command_navigation(ui); }
            let body_height = (ctx.content_rect().height() - if sheet == Sheet::Commands { 280. } else { 180. }).max(120.);
            egui::ScrollArea::vertical().scroll_source(egui::scroll_area::ScrollSource::ALL).max_height(body_height).min_scrolled_height(body_height).show(ui, |ui| {
                match sheet {
                    Sheet::Brush => self.brush_studio(app, ui),
                    Sheet::Channels => self.channels(app, ui),
                    Sheet::Paths => self.paths(app, ui),
                    Sheet::Commands => { close |= self.command_browser(app, ui); }
                    Sheet::Files => {
                        ui.horizontal_wrapped(|ui| {
                            for (label, id) in [("New image", "file.new"), ("Open…", "file.open"), ("Save", "file.save"), ("Save as…", "file.saveAs"), ("Export…", "file.export.exportAs"), ("PNG", "file.export.quickExportAsPng")] {
                                if ui.add_enabled(menus::is_enabled(app, id), Button::new(label).min_size(vec2(120., 48.))).clicked() {
                                    self.invoke(app, ctx, id, json!({}));
                                    close = true;
                                }
                            }
                        });
                        ui.separator();
                        ui.strong("Open documents");
                        let active = app.session.active_index();
                        let docs: Vec<_> = app.session.documents().iter().map(|d| format!("{}{}", d.doc.name, if d.is_dirty() { " •" } else { "" })).collect();
                        for (i, name) in docs.iter().enumerate() {
                            if full_button(ui, name, active == Some(i)).clicked() { app.session.set_active(i); close = true; }
                        }
                        ui.add_space(12.);
                        ui.label("Save downloads to Files on iPad. Save before closing or reloading this page.");
                    }
                }
            });
        });
        if close || modal.should_close() {
            self.sheet = None;
        }
    }
}
