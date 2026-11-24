//! Opt-in same-origin relay; all edits still go through PhotoCraft's control queue.
use photocraft_ui_egui::{ControlRequest, PhotocraftApp};
use serde_json::{Value, json};
use std::{cell::Cell, rc::Rc, sync::mpsc};
use wasm_bindgen::{JsCast as _, JsValue};
use wasm_bindgen_futures::{JsFuture, spawn_local};

pub struct BrowserControl {
    token: String,
    session: String,
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
        // Unique per page lifetime; not an authentication credential (the token is).
        let session = format!("{}-{}", js_sys::Date::now(), js_sys::Math::random());
        Some(Self {
            token: token.to_ascii_lowercase(),
            session,
            tx,
            replies: Vec::new(),
            inbox: Rc::default(),
            busy: Rc::default(),
            next_poll: 0.0,
        })
    }

    pub fn tick(&mut self, ctx: &egui::Context) {
        for msg in self.inbox.borrow_mut().drain(..) {
            let Some(method) = msg.get("method").and_then(Value::as_str) else {
                continue;
            };
            let id = msg.get("id").cloned().unwrap_or(Value::Null);
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
            spawn_local(async move {
                // Never retry a mutation: a lost response has an unknown outcome.
                let _ = exchange("/control/reply", "POST", &token, &session, Some(&body)).await;
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
            let ctx = ctx.clone();
            spawn_local(async move {
                if let Ok(value) = exchange("/control/next", "GET", &token, &session, None).await {
                    if value.is_object() {
                        inbox.borrow_mut().push(value);
                    }
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
