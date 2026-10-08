//! A touch workspace on PhotoCraft's shared engine, canvas and file services.
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
            contacts.area = app.ui.dialogs.is_empty().then_some(xf.rect);
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
        _frame: &mut eframe::Frame,
    ) {
        photocraft_ui_egui::touch_ui::set_enabled(ui.ctx(), true);
        app.ui_with_workspace(ui, |app, ui| self.workspace.show(app, ui));
        self.contacts.borrow_mut().area = if app.ui.dialogs.is_empty() {
            photocraft_ui_egui::canvas::ViewXform::active(app).map(|xf| xf.rect)
        } else {
            None
        };
    }
}
