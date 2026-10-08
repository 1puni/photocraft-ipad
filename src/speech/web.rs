//! A real HTML button preserves the synchronous user gesture required by browsers.
//! Recognition is browser-native, optional, single-utterance and capped at ten seconds.
use super::*;
use js_sys::{Function, Reflect};
use std::cell::Cell;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

type Callback = Closure<dyn FnMut(JsValue)>;
struct Run {
    object: JsValue,
    callbacks: Vec<Callback>,
    timeout: Rc<Cell<i32>>,
    timer: Closure<dyn FnMut()>,
}
impl Drop for Run {
    fn drop(&mut self) {
        for name in ["onstart", "onresult", "onerror", "onend"] {
            let _ = Reflect::set(&self.object, &name.into(), &JsValue::NULL);
        }
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(self.timeout.get());
        }
        let _ = call(&self.object, "abort");
        // Callbacks stay alive until their handlers and timer have been detached.
        let _ = (&self.callbacks, &self.timer);
    }
}
struct Inner {
    run: Option<Run>,
    state: Rc<RefCell<State>>,
    ctx: egui::Context,
}
pub struct Host {
    button: web_sys::HtmlElement,
    inner: Rc<RefCell<Inner>>,
    click: Callback,
    visibility: Callback,
    document: web_sys::Document,
}
impl Host {
    pub fn new(state: Rc<RefCell<State>>, ctx: egui::Context) -> Result<Self, String> {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or("Browser document unavailable")?;
        let button = document
            .create_element("button")
            .map_err(|_| "Cannot create microphone button")?
            .dyn_into::<web_sys::HtmlElement>()
            .map_err(|_| "Cannot create microphone button")?;
        button.set_inner_html(r#"<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round"><rect x="9" y="2" width="6" height="12" rx="3"/><path d="M5 10v2a7 7 0 0 0 14 0v-2M12 19v3m-4 0h8"/></svg>"#);
        button
            .set_attribute("aria-label", "Speak a command")
            .map_err(|_| "Cannot label microphone")?;
        button
            .set_attribute("type", "button")
            .map_err(|_| "Cannot configure microphone")?;
        button
            .set_attribute("style", "display:none")
            .map_err(|_| "Cannot position microphone")?;
        document
            .body()
            .ok_or("Browser body unavailable")?
            .append_child(&button)
            .map_err(|_| "Cannot attach microphone")?;
        if let Ok(mut s) = state.try_borrow_mut() {
            s.supported = constructor().is_some();
        }
        let inner = Rc::new(RefCell::new(Inner {
            run: None,
            state,
            ctx,
        }));
        let weak = Rc::downgrade(&inner);
        let click = Callback::new(move |_| {
            let Some(inner) = weak.upgrade() else { return };
            let Ok(mut host) = inner.try_borrow_mut() else {
                return;
            };
            let active = host
                .state
                .try_borrow()
                .is_ok_and(|s| s.phase != Phase::Idle);
            if active {
                if let Ok(mut s) = host.state.try_borrow_mut() {
                    s.cancel();
                }
                if let Some(run) = host.run.as_ref() {
                    let _ = call(&run.object, "abort");
                }
            } else {
                // Old event handlers are removed before starting a new recognizer. Its generation
                // also makes late callbacks inert after cancel/retry.
                host.run = None;
                if let Ok(mut s) = host.state.try_borrow_mut() {
                    s.open_requested = true;
                }
                match start(host.state.clone(), host.ctx.clone()) {
                    Ok(run) => host.run = Some(run),
                    Err(error) => {
                        if let Ok(mut s) = host.state.try_borrow_mut() {
                            s.phase = Phase::Idle;
                            s.error = error;
                        }
                    }
                }
            }
            host.ctx.request_repaint();
        });
        button
            .add_event_listener_with_callback("click", click.as_ref().unchecked_ref())
            .map_err(|_| "Cannot attach microphone action")?;
        let weak = Rc::downgrade(&inner);
        let page = document.clone();
        let visibility = Callback::new(move |_| {
            if !page.hidden() {
                return;
            }
            if let Some(inner) = weak.upgrade()
                && let Ok(host) = inner.try_borrow()
            {
                if let Ok(mut state) = host.state.try_borrow_mut() {
                    state.cancel();
                }
                if let Some(run) = &host.run {
                    let _ = call(&run.object, "abort");
                }
                host.ctx.request_repaint();
            }
        });
        document
            .add_event_listener_with_callback(
                "visibilitychange",
                visibility.as_ref().unchecked_ref(),
            )
            .map_err(|_| "Cannot attach microphone visibility handling")?;
        Ok(Self {
            button,
            inner,
            click,
            visibility,
            document,
        })
    }
    pub fn cancel(&self) {
        if let Ok(host) = self.inner.try_borrow()
            && let Some(run) = host.run.as_ref()
        {
            let _ = call(&run.object, "abort");
        }
    }
    pub fn place(
        &self,
        rect: Option<egui::Rect>,
        zoom: f32,
        text: egui::Color32,
        accent: egui::Color32,
    ) {
        let Some(rect) = rect else {
            let _ = self.button.set_attribute("style", "display:none");
            return;
        };
        let active = self
            .inner
            .try_borrow()
            .ok()
            .and_then(|h| h.state.try_borrow().ok().map(|s| s.phase != Phase::Idle))
            .unwrap_or(false);
        let color = if active { accent } else { text };
        let style = format!(
            "position:fixed;z-index:2;display:grid;place-items:center;left:{}px;top:{}px;width:{}px;height:{}px;border:0;border-radius:4px;background:transparent;color:rgb({},{},{});padding:0;touch-action:manipulation;cursor:pointer",
            rect.left() * zoom,
            rect.top() * zoom,
            rect.width() * zoom,
            rect.height() * zoom,
            color.r(),
            color.g(),
            color.b()
        );
        let _ = self.button.set_attribute("style", &style);
        let label = if active {
            "Cancel dictation"
        } else {
            "Speak a command"
        };
        let _ = self.button.set_attribute("aria-label", label);
        let _ = self.button.set_attribute("title", label);
        let _ = self
            .button
            .set_attribute("aria-pressed", if active { "true" } else { "false" });
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        let _ = self
            .button
            .remove_event_listener_with_callback("click", self.click.as_ref().unchecked_ref());
        self.button.remove();
        let _ = self.document.remove_event_listener_with_callback(
            "visibilitychange",
            self.visibility.as_ref().unchecked_ref(),
        );
    }
}
fn constructor() -> Option<Function> {
    let window = web_sys::window()?;
    ["SpeechRecognition", "webkitSpeechRecognition"]
        .into_iter()
        .find_map(|name| {
            Reflect::get(&window, &name.into())
                .ok()?
                .dyn_into::<Function>()
                .ok()
        })
}
fn get(object: &JsValue, key: &str) -> JsValue {
    Reflect::get(object, &key.into()).unwrap_or(JsValue::UNDEFINED)
}
fn call(object: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    get(object, key)
        .dyn_into::<Function>()
        .map_err(|_| JsValue::from_str("Browser speech method unavailable"))?
        .call0(object)
}
fn start(state: Rc<RefCell<State>>, ctx: egui::Context) -> Result<Run, String> {
    let constructor = constructor()
        .ok_or("Browser speech is unavailable. Type here or use the iPad keyboard’s dictation.")?;
    let object = Reflect::construct(&constructor, &js_sys::Array::new())
        .map_err(|_| "Browser speech could not start.")?;
    for (key, value) in [
        ("lang", JsValue::from_str("en-US")),
        ("continuous", JsValue::FALSE),
        ("interimResults", JsValue::TRUE),
        ("maxAlternatives", JsValue::from_f64(1.)),
    ] {
        Reflect::set(&object, &key.into(), &value).map_err(|_| "Browser speech setup failed")?;
    }
    let generation = state
        .try_borrow_mut()
        .map_err(|_| "Speech is busy")?
        .begin();
    let timeout = Rc::new(Cell::new(0));
    let timer_state = state.clone();
    let timer_object = object.clone();
    let timer_ctx = ctx.clone();
    let timer = Closure::<dyn FnMut()>::new(move || {
        let stop = timer_state.try_borrow().is_ok_and(|s| s.live(generation));
        if stop {
            if let Ok(mut s) = timer_state.try_borrow_mut() {
                let message = if s.phase == Phase::Starting {
                    "Microphone did not start. Check browser permission, or type a command."
                } else {
                    "Listening ended after ten seconds. Try a short command, or type it."
                };
                s.failed(generation, message);
            }
            let _ = call(&timer_object, "abort");
            timer_ctx.request_repaint();
        }
    });
    let mut callbacks = Vec::new();
    for name in ["onstart", "onresult", "onerror", "onend"] {
        let state = state.clone();
        let ctx = ctx.clone();
        let recognition = object.clone();
        let timeout = timeout.clone();
        let timer_function: Function = timer.as_ref().unchecked_ref::<Function>().clone();
        let callback = Callback::new(move |event| {
            let mut stop = false;
            if let Ok(mut s) = state.try_borrow_mut() {
                match name {
                    "onstart" => {
                        // A permission prompt may resolve after cancellation or timeout.
                        // Do not leave its microphone running even though its text is ignored.
                        if s.live(generation) {
                            s.started(generation);
                            if let Some(window) = web_sys::window() {
                                window.clear_timeout_with_handle(timeout.get());
                                match window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                    &timer_function,
                                    10_000,
                                ) {
                                    Ok(handle) => timeout.set(handle),
                                    Err(_) => {
                                        s.failed(generation, "Cannot start speech timer.");
                                        stop = true;
                                    }
                                }
                            }
                        } else {
                            stop = true;
                        }
                    }
                    "onresult" => {
                        let results = get(&event, "results");
                        let index = get(&event, "resultIndex").as_f64().unwrap_or(0.);
                        if index.is_finite() && (0.0..=100.).contains(&index) {
                            let result = Reflect::get(&results, &JsValue::from_f64(index))
                                .unwrap_or(JsValue::UNDEFINED);
                            let alternative = Reflect::get(&result, &JsValue::from_f64(0.))
                                .unwrap_or(JsValue::UNDEFINED);
                            if let Some(text) = get(&alternative, "transcript").as_string() {
                                let final_result =
                                    get(&result, "isFinal").as_bool().unwrap_or(false);
                                s.result(generation, &text, final_result);
                                stop = final_result;
                            }
                        }
                    }
                    "onerror" => {
                        let code = get(&event, "error").as_string().unwrap_or_default();
                        let message = match code.as_str() {
                            "not-allowed" | "service-not-allowed" => {
                                "Speech permission was denied or the service is unavailable. On iPad, check Safari microphone permission and Siri; keyboard dictation also works."
                            }
                            "audio-capture" => {
                                "No microphone is available. Type or use keyboard dictation."
                            }
                            "network" => {
                                "The browser speech service could not connect. Type a command or try again."
                            }
                            "no-speech" => "No command heard. Try again or type it.",
                            "aborted" => "Listening stopped.",
                            _ => {
                                "Browser speech failed. Try again, type, or use keyboard dictation."
                            }
                        };
                        s.failed(generation, message);
                    }
                    _ => s.ended(generation),
                }
            }
            if stop {
                let _ = call(&recognition, "abort");
            }
            ctx.request_repaint();
        });
        Reflect::set(&object, &name.into(), callback.as_ref())
            .map_err(|_| "Cannot attach speech callbacks")?;
        callbacks.push(callback);
    }
    let startup_timeout = web_sys::window()
        .ok_or("Browser window unavailable")?
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            timer.as_ref().unchecked_ref(),
            60_000,
        )
        .map_err(|_| "Cannot start speech timer")?;
    timeout.set(startup_timeout);
    let run = Run {
        object,
        callbacks,
        timeout,
        timer,
    };
    call(&run.object, "start").map_err(
        |_| "Speech could not start. Check browser permission, or use keyboard dictation.",
    )?;
    Ok(run)
}
