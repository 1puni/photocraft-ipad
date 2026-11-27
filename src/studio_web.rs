//! Save-through binding for a document opened from a local studio.
use js_sys::{Reflect, Uint8Array};
use photocraft_ui_egui::PhotocraftApp;
use serde_json::Value;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen(module = "/src/studio_store.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn studio_config() -> Result<String, JsValue>;
    #[wasm_bindgen(catch)]
    async fn studio_open() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn studio_bind(document_id: f64) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    async fn studio_save(bytes: &Uint8Array) -> Result<JsValue, JsValue>;
    fn studio_detach();
    fn studio_status(text: &str, error: bool);
}
fn error(v: JsValue) -> String {
    v.as_string()
        .or_else(|| Reflect::get(&v, &"message".into()).ok()?.as_string())
        .unwrap_or_else(|| "Studio connection failed".into())
}
type Loaded = Rc<RefCell<Option<Result<Vec<u8>, String>>>>;
type Finished = Rc<RefCell<Option<(u64, u64, Result<String, String>)>>>;
#[derive(Default)]
pub struct Studio {
    started: bool,
    loaded: Loaded,
    finished: Finished,
    busy: Rc<RefCell<bool>>,
    name: String,
}
impl Studio {
    fn bind(&mut self, app: &mut PhotocraftApp, ctx: &egui::Context, id: u64) {
        if let Err(e) = studio_bind(id as f64) {
            studio_status(&error(e), true);
            return;
        }
        // Save As makes an independent download, and must not silently rebind on reload.
        let mut picker = app.services.pick_save.take();
        let bound_name = self.name.clone();
        app.services.pick_save = Some(Box::new(move |suggested| {
            if suggested == bound_name {
                Some(suggested.replace(".psd", "-download.psd"))
            } else {
                picker.as_mut().and_then(|f| f(suggested))
            }
        }));
        let mut bound = true;
        let finished = self.finished.clone();
        let busy = self.busy.clone();
        let name = self.name.clone();
        let ctx = ctx.clone();
        app.services.deferred_write = Some(Box::new(move |doc, revision, path, bytes| {
            if doc != id || !bound {
                return None;
            }
            if path != name {
                bound = false;
                studio_detach();
                return None;
            }
            if *busy.borrow() {
                return Some(Err("Studio save already in progress".into()));
            }
            *busy.borrow_mut() = true;
            studio_status("Saving to studio…", false);
            let bytes = Uint8Array::from(bytes);
            let finished = finished.clone();
            let ctx = ctx.clone();
            spawn_local(async move {
                let result = studio_save(&bytes)
                    .await
                    .map(|v| v.as_string().unwrap_or_else(|| "Saved to studio".into()))
                    .map_err(error);
                *finished.borrow_mut() = Some((doc, revision, result));
                ctx.request_repaint();
            });
            Some(Ok(()))
        }));
    }
    pub fn tick(&mut self, app: &mut PhotocraftApp, ctx: &egui::Context) {
        if !self.started {
            self.started = true;
            let config = match studio_config() {
                Ok(s) if s.is_empty() => return,
                Ok(s) => s,
                Err(e) => {
                    studio_status(&error(e), true);
                    return;
                }
            };
            let config: Value = match serde_json::from_str(&config) {
                Ok(v) => v,
                Err(e) => {
                    studio_status(&e.to_string(), true);
                    return;
                }
            };
            self.name = format!(
                "studio-{}-{}.psd",
                config["panel"].as_str().unwrap_or(""),
                config["composition"].as_str().unwrap_or("")
            );
            if let Some(id) = config["documentId"].as_u64()
                && let Some(index) = app
                    .session
                    .documents()
                    .iter()
                    .position(|d| d.doc.id.0 == id)
            {
                app.session.set_active(index);
                if let Some(d) = app.session.active_mut() {
                    d.path = Some(self.name.clone());
                }
                self.bind(app, ctx, id);
                studio_status("Studio document recovered · Save writes to studio", false);
            } else {
                let loaded = self.loaded.clone();
                let ctx = ctx.clone();
                studio_status("Opening studio document…", false);
                spawn_local(async move {
                    let result = studio_open()
                        .await
                        .map_err(error)
                        .and_then(|v| Reflect::get(&v, &"bytes".into()).map_err(error))
                        .map(|v| Uint8Array::new(&v).to_vec());
                    *loaded.borrow_mut() = Some(result);
                    ctx.request_repaint();
                });
            }
        }
        let result = self.loaded.borrow_mut().take();
        if let Some(result) = result {
            let result = result.and_then(|bytes| {
                app.services
                    .import
                    .as_ref()
                    .ok_or_else(|| "No importer".to_string())?(&self.name, &bytes)
            });
            match result {
                Ok((doc, warnings)) => {
                    if !warnings.is_empty() {
                        studio_status(&format!("Open warnings: {}", warnings.join("; ")), true);
                    }
                    app.session.add_document(doc, Some(self.name.clone()));
                    app.sync_views();
                    if let Some(d) = app.session.active() {
                        let id = d.doc.id.0;
                        self.bind(app, ctx, id);
                    }
                    studio_status("Studio document · Save writes to studio", false);
                }
                Err(e) => studio_status(&e, true),
            }
        }
        let result = self.finished.borrow_mut().take();
        if let Some((id, revision, result)) = result {
            *self.busy.borrow_mut() = false;
            match result {
                Ok(mut message) => {
                    let active = app.session.active_index();
                    if let Some(index) = app
                        .session
                        .documents()
                        .iter()
                        .position(|d| d.doc.id.0 == id)
                    {
                        app.session.set_active(index);
                        if let Some(d) = app.session.active_mut() {
                            d.saved_revision = revision;
                            if d.revision != revision {
                                message.push_str(" · newer edits unsaved");
                            }
                        }
                        if let Some(active) = active {
                            app.session.set_active(active);
                        }
                    }
                    app.ui.status = message.clone();
                    app.ui.status_error = false;
                    studio_status(&message, false);
                }
                Err(e) => {
                    let message = format!("Not saved to studio: {e}");
                    app.ui.status = message.clone();
                    app.ui.status_error = true;
                    studio_status(&message, true);
                }
            }
        }
    }
}
