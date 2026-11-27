//! Browser-local recovery, independent of the optional automation relay.
use crate::recovery::{self, Snapshot};
use js_sys::{Array, Reflect, Uint8Array};
use photocraft_ui_egui::PhotocraftApp;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen(module = "/src/session_identity.js")]
extern "C" {
    fn identity() -> String;
    fn recovery_identity(id: &str);
    fn metadata(name: &str) -> String;
}
#[wasm_bindgen(module = "/src/recovery_store.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn recovery_start(identity: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn recovery_save(metadata: &str, documents: &Array) -> Result<JsValue, JsValue>;
    fn recovery_status(message: &str, pending: bool);
    fn recovery_ready(message: &str);
    fn recovery_failed(message: &str);
}
fn field(value: &JsValue, key: &str) -> Result<JsValue, String> {
    Reflect::get(value, &JsValue::from_str(key)).map_err(error)
}
fn error(value: JsValue) -> String {
    value
        .as_string()
        .or_else(|| {
            Reflect::get(&value, &JsValue::from_str("message"))
                .ok()?
                .as_string()
        })
        .unwrap_or_else(|| "browser storage error".into())
}
fn decode(value: JsValue) -> Result<Option<Snapshot>, String> {
    let id = field(&value, "id")?
        .as_string()
        .ok_or("missing recovery identity")?;
    recovery_identity(&id);
    let value = field(&value, "snapshot")?;
    if value.is_null() {
        return Ok(None);
    }
    if field(&value, "version")?.as_f64() != Some(1.0) {
        return Err("unsupported recovery version".into());
    }
    let metadata = field(&value, "metadata")?
        .as_string()
        .ok_or("invalid recovery metadata")?;
    let docs = field(&value, "documents")?;
    if !Array::is_array(&docs) {
        return Err("invalid recovery documents".into());
    }
    let documents = Array::from(&docs)
        .iter()
        .map(|v| Uint8Array::new(&v).to_vec())
        .collect();
    Ok(Some(Snapshot {
        metadata,
        documents,
    }))
}

type Loaded = Rc<RefCell<Option<Result<Option<Snapshot>, String>>>>;
#[derive(Default)]
pub struct Recovery {
    loaded: Loaded,
    saved: Rc<RefCell<Option<Result<(String, String), String>>>>,
    initialized: bool,
    disabled: bool,
    busy: bool,
    observed: String,
    committed: String,
    changed_at: f64,
    first_change: f64,
    retry_at: f64,
}
impl Recovery {
    pub fn start(&mut self, ctx: &egui::Context) {
        let ident = identity();
        let loaded = self.loaded.clone();
        let ctx = ctx.clone();
        spawn_local(async move {
            let result = recovery_start(&ident).await.map_err(error).and_then(decode);
            *loaded.borrow_mut() = Some(result);
            ctx.request_repaint();
        });
    }
    /// Returns false until restoration finishes, so remote edits cannot race startup recovery.
    pub fn tick(&mut self, app: &mut PhotocraftApp, ctx: &egui::Context) -> bool {
        if !self.initialized {
            let result = self.loaded.borrow_mut().take();
            let Some(result) = result else {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
                return false;
            };
            let result = result.and_then(|s| match s {
                Some(s) => recovery::restore(&mut app.session, s),
                None => Ok(0),
            });
            self.initialized = true;
            match result {
                Ok(n) => {
                    app.sync_views();
                    self.committed = recovery::signature(&app.session);
                    self.observed = self.committed.clone();
                    recovery_ready(&if n > 0 {
                        format!("Recovered {n} document(s) · saved in this browser")
                    } else {
                        "Autosave ready · stored only in this browser".into()
                    });
                }
                Err(e) => {
                    self.disabled = true;
                    recovery_failed(&e);
                }
            }
        }
        metadata(
            app.session
                .active()
                .map(|d| d.doc.name.as_str())
                .unwrap_or(""),
        );
        if self.disabled {
            return true;
        }
        let now = js_sys::Date::now();
        if let Some(result) = self.saved.borrow_mut().take() {
            self.busy = false;
            match result {
                Ok((signature, time)) => {
                    self.committed = signature;
                    let pending = recovery::signature(&app.session) != self.committed;
                    recovery_status(
                        &if pending {
                            "Changes pending local autosave…".into()
                        } else {
                            format!("Autosaved locally at {time}")
                        },
                        pending,
                    );
                }
                Err(e) => {
                    self.retry_at = now + 10000.;
                    recovery_status(
                        &format!("Autosave failed: {e}. Previous recovery kept; save a download."),
                        true,
                    );
                }
            }
        }
        let signature = recovery::signature(&app.session);
        if signature != self.observed {
            if self.observed == self.committed {
                self.first_change = now;
            }
            self.observed = signature.clone();
            self.changed_at = now;
            recovery_status("Changes pending local autosave…", true);
        }
        if signature != self.committed
            && !self.busy
            && now >= self.retry_at
            && !ctx.input(|i| i.pointer.any_down())
            && (now - self.changed_at >= 2000. || now - self.first_change >= 10000.)
        {
            self.busy = true;
            recovery_status("Saving recovery copy…", true);
            match recovery::capture(&app.session) {
                Ok(snapshot) => {
                    let docs = Array::new();
                    for bytes in snapshot.documents {
                        docs.push(&Uint8Array::from(bytes.as_slice()));
                    }
                    let saved = self.saved.clone();
                    let ctx = ctx.clone();
                    spawn_local(async move {
                        let result = recovery_save(&snapshot.metadata, &docs)
                            .await
                            .map_err(error)
                            .map(|time| (signature, time.as_string().unwrap_or_default()));
                        *saved.borrow_mut() = Some(result);
                        ctx.request_repaint();
                    });
                }
                Err(e) => {
                    self.busy = false;
                    self.retry_at = now + 10000.;
                    recovery_status(&format!("Autosave failed: {e}. Save a download."), true);
                }
            }
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
        true
    }
}
