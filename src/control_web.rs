//! Opt-in same-origin relay; all edits still go through PhotoCraft's control queue.
use photocraft_ui_egui::{ControlRequest, PhotocraftApp};
use serde_json::{Value, json};
use std::{cell::Cell, rc::Rc, sync::mpsc};
use wasm_bindgen::{JsCast as _, JsValue, prelude::wasm_bindgen};

#[wasm_bindgen(module = "/src/session_identity.js")]
extern "C" {
    fn identity() -> String;
    fn metadata(name: &str) -> String;
    fn connection(message: &str);
    fn conflict() -> String;
    fn connected();
}
use wasm_bindgen_futures::{JsFuture, spawn_local};

pub struct BrowserControl {
    files: Rc<std::cell::RefCell<crate::control_files::Files>>,
    token: String,
    session: String,
    page: String,
    tx: mpsc::Sender<ControlRequest>,
    replies: Vec<(Value, mpsc::Receiver<Value>)>,
    inbox: Rc<std::cell::RefCell<Vec<Value>>>,
    busy: Rc<Cell<bool>>,
    next_poll: f64,
}

impl BrowserControl {
    pub fn attach(app: &mut PhotocraftApp) -> Option<Self> {
        let hash = web_sys::window()?.location().hash().ok()?;
        let token = hash.strip_prefix("#control=")?;
        if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let (tx, rx) = mpsc::channel();
        app.attach_control(rx);
        let files = Rc::new(std::cell::RefCell::new(
            crate::control_files::Files::default(),
        ));
        let read_files = files.clone();
        app.services.automation_read =
            Some(Box::new(move |name| read_files.borrow().read_file(name)));
        let write_files = files.clone();
        app.services.automation_write = Some(Box::new(move |name, bytes| {
            write_files.borrow_mut().write_file(name, bytes)
        }));
        // Stable tab identity and a separate page generation; neither is a credential.
        let ident: Value = serde_json::from_str(&identity()).ok()?;
        let session = ident["id"].as_str()?.to_owned();
        let page = ident["page"].as_str()?.to_owned();
        Some(Self {
            files,
            token: token.to_ascii_lowercase(),
            session,
            page,
            tx,
            replies: Vec::new(),
            inbox: Rc::default(),
            busy: Rc::default(),
            next_poll: 0.0,
        })
    }

    fn export_copy(&self, app: &PhotocraftApp, params: &Value) -> Result<Value, String> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or("missing name")?;
        let document = app.session.active().ok_or("no document")?;
        let export = app
            .services
            .export
            .as_ref()
            .ok_or("no exporter configured")?;
        let (bytes, warnings) = export(&document.doc, name, &Default::default())?;
        self.files.borrow_mut().write_file(name, &bytes)?;
        Ok(json!({"name":name,"size":bytes.len(),"warnings":warnings}))
    }

    pub fn tick(&mut self, ctx: &egui::Context, app: &mut PhotocraftApp) {
        let meta = metadata(
            app.session
                .active()
                .map(|d| d.doc.name.as_str())
                .unwrap_or(""),
        );
        if let Ok(value) = serde_json::from_str::<Value>(&meta) {
            if let Some(id) = value["id"].as_str() {
                self.session = id.to_owned();
            }
        }
        for msg in self.inbox.borrow_mut().drain(..) {
            let Some(method) = msg.get("method").and_then(Value::as_str) else {
                continue;
            };
            let id = msg.get("id").cloned().unwrap_or(Value::Null);
            if method.starts_with("transfer.") {
                let params = msg.get("params").cloned().unwrap_or(json!({}));
                let result = if method == "transfer.export" {
                    self.export_copy(app, &params)
                } else {
                    self.files.borrow_mut().call(method, &params)
                };
                let reply = match result {
                    Ok(value) => json!({"ok":true,"result":value}),
                    Err(error) => json!({"ok":false,"error":error}),
                };
                let (tx, rx) = mpsc::channel();
                let _ = tx.send(reply);
                self.replies.push((id, rx));
                continue;
            }
            let (request, rx) =
                ControlRequest::new(method, msg.get("params").cloned().unwrap_or(json!({})));
            if self.tx.send(request).is_ok() {
                self.replies.push((id, rx));
                ctx.request_repaint();
            }
        }
        self.replies.retain(|(id, rx)| {
            let reply = match rx.try_recv() {
                Ok(reply) => reply,
                Err(mpsc::TryRecvError::Empty) => return true,
                Err(mpsc::TryRecvError::Disconnected) => {
                    json!({"ok": false, "error": "editor disconnected"})
                }
            };
            let body = json!({"id": id, "reply": reply}).to_string();
            let token = self.token.clone();
            let session = self.session.clone();
            let page = self.page.clone();
            spawn_local(async move {
                // Never retry a mutation: a lost response has an unknown outcome.
                let _ = exchange(
                    "/control/reply",
                    "POST",
                    &token,
                    &session,
                    &page,
                    Some(&body),
                )
                .await;
            });
            false
        });
        let now = js_sys::Date::now();
        if now >= self.next_poll && !self.busy.get() {
            self.next_poll = now + 250.0;
            self.busy.set(true);
            let busy = self.busy.clone();
            let inbox = self.inbox.clone();
            let token = self.token.clone();
            let session = self.session.clone();
            let page = self.page.clone();
            let ctx = ctx.clone();
            spawn_local(async move {
                match exchange(
                    "/control/poll",
                    "POST",
                    &token,
                    &session,
                    &page,
                    Some(&meta),
                )
                .await
                {
                    Ok(value) if value.get("sessionConflict").is_some() => {
                        conflict();
                    }
                    Ok(value) => {
                        connected();
                        if value.get("method").is_some() {
                            inbox.borrow_mut().push(value);
                        }
                    }
                    Err(_) => connection("Disconnected · reconnecting…"),
                }
                busy.set(false);
                ctx.request_repaint();
            });
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
}

async fn exchange(
    path: &str,
    method: &str,
    token: &str,
    session: &str,
    page: &str,
    body: Option<&str>,
) -> Result<Value, JsValue> {
    let init = web_sys::RequestInit::new();
    init.set_method(method);
    if let Some(body) = body {
        init.set_body(&JsValue::from_str(body));
    }
    let request = web_sys::Request::new_with_str_and_init(path, &init)?;
    request
        .headers()
        .set("Authorization", &format!("Bearer {token}"))?;
    request.headers().set("X-PhotoCraft-Session", session)?;
    request.headers().set("X-PhotoCraft-Protocol", "3")?;
    request.headers().set("X-PhotoCraft-Page", page)?;
    if body.is_some() {
        request.headers().set("Content-Type", "application/json")?;
    }
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no browser window"))?;
    let response: web_sys::Response = JsFuture::from(window.fetch_with_request(&request))
        .await?
        .dyn_into()?;
    if !response.ok() {
        return Err(JsValue::from_str("relay rejected request"));
    }
    let text = JsFuture::from(response.text()?)
        .await?
        .as_string()
        .unwrap_or_default();
    serde_json::from_str(&text).map_err(|e| JsValue::from_str(&e.to_string()))
}
