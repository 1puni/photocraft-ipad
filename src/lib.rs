//! A touch workspace on PhotoCraft's shared engine, canvas and file services.
pub mod commands;
pub mod input;
pub mod workspace;

#[cfg(target_arch = "wasm32")]
mod input_web;

#[cfg(target_arch = "wasm32")]
pub fn start() {
    photocraft_web::web::start_with_workspace(Some(Box::new(BrowserWorkspace::default())));
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct BrowserWorkspace {
    contacts: input_web::Shared,
    workspace: workspace::TabletUi,
}

#[cfg(target_arch = "wasm32")]
impl photocraft_web::web::BrowserWorkspace for BrowserWorkspace {
    fn attach(
        &mut self,
        canvas: &web_sys::HtmlCanvasElement,
        ctx: &egui::Context,
        app: &mut photocraft_ui_egui::PhotocraftApp,
    ) {
        input_web::install(canvas, ctx, self.contacts.clone(), app.stylus.feed.clone());
    }

    fn raw_input(&mut self, raw: &mut egui::RawInput) {
        self.contacts.borrow_mut().augment_input(raw);
        self.workspace.raw_input(raw);
    }

    fn logic(&mut self, app: &mut photocraft_ui_egui::PhotocraftApp, _ctx: &egui::Context) {
        let mut contacts = self.contacts.borrow_mut();
        if let Some(xf) = photocraft_ui_egui::canvas::ViewXform::active(app) {
            contacts.area =
                (app.ui.dialogs.is_empty() && !self.workspace.clone_source_pick).then_some(xf.rect);
            if let Some(view) = app
                .session
                .active_index()
                .and_then(|i| app.ui.views.get_mut(i))
            {
                for n in contacts.navigation.drain(..) {
                    input::navigate(view, xf.rect, xf.flip, n);
                }
            }
        } else {
            contacts.area = None;
            contacts.navigation.clear();
        }
    }

    fn ui(
        &mut self,
        app: &mut photocraft_ui_egui::PhotocraftApp,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
    ) {
        const SWATCHES: &str = "photocraft.ipad.swatches.v1";
        if !self.workspace.swatches_loaded() {
            let saved = frame
                .storage()
                .and_then(|storage| storage.get_string(SWATCHES));
            self.workspace.restore_swatches(saved.as_deref());
        }
        photocraft_ui_egui::touch_ui::set_enabled(ui.ctx(), true);
        app.ui_with_workspace(ui, |app, ui| self.workspace.show(app, ui));
        if let Some(saved) = self.workspace.swatches_to_store() {
            let success = frame.storage_mut().is_some_and(|storage| {
                storage.set_string(SWATCHES, saved.clone());
                storage.get_string(SWATCHES).as_deref() == Some(saved.as_str())
            });
            self.workspace.swatches_stored(success);
            ui.ctx().request_repaint();
        }
        self.contacts.borrow_mut().area =
            if app.ui.dialogs.is_empty() && !self.workspace.clone_source_pick {
                photocraft_ui_egui::canvas::ViewXform::active(app).map(|xf| xf.rect)
            } else {
                None
            };
    }
}
