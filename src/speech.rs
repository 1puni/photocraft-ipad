//! One short, explicitly started utterance. Audio belongs to the browser recognizer;
//! this app receives text only and never persists it.
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Idle,
    Starting,
    Listening,
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub supported: bool,
    pub phase: Phase,
    pub interim: String,
    pub error: String,
    pub open_requested: bool,
    generation: u64,
    final_text: Option<String>,
}
impl State {
    pub fn begin(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.phase = Phase::Starting;
        self.interim.clear();
        self.error.clear();
        self.final_text = None;
        self.open_requested = true;
        self.generation
    }
    fn live(&self, generation: u64) -> bool {
        self.generation == generation && self.phase != Phase::Idle
    }
    pub fn started(&mut self, generation: u64) {
        if self.live(generation) {
            self.phase = Phase::Listening;
        }
    }
    pub fn result(&mut self, generation: u64, text: &str, final_result: bool) {
        if !self.live(generation) {
            return;
        }
        let text: String = text.trim().chars().take(500).collect();
        if final_result && !text.is_empty() {
            self.final_text = Some(text);
            self.interim.clear();
            self.phase = Phase::Idle;
        } else {
            self.interim = text;
        }
    }
    pub fn failed(&mut self, generation: u64, message: &str) {
        if !self.live(generation) {
            return;
        }
        self.error = message.to_owned();
        self.phase = Phase::Idle;
        self.interim.clear();
    }
    pub fn ended(&mut self, generation: u64) {
        if self.live(generation) {
            self.phase = Phase::Idle;
            self.error = "No command heard. Try again, type, or use keyboard dictation.".into();
            self.interim.clear();
        }
    }
    pub fn cancel(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.phase = Phase::Idle;
        self.final_text = None;
        self.open_requested = false;
        self.interim.clear();
        self.error.clear();
    }
}
#[derive(Default)]
pub struct Dictation {
    state: Rc<RefCell<State>>,
    #[cfg(target_arch = "wasm32")]
    host: Option<web::Host>,
}
impl Dictation {
    pub fn has_button(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            self.host.is_some()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            false
        }
    }
    pub fn snapshot(&self) -> State {
        self.state
            .try_borrow()
            .map(|s| s.clone())
            .unwrap_or_default()
    }
    pub fn take_final(&mut self) -> Option<String> {
        self.state.try_borrow_mut().ok()?.final_text.take()
    }
    pub fn take_open(&mut self) -> bool {
        self.state
            .try_borrow_mut()
            .map(|mut s| std::mem::take(&mut s.open_requested))
            .unwrap_or(false)
    }
    pub fn cancel(&mut self) {
        if let Ok(mut s) = self.state.try_borrow_mut() {
            s.cancel();
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(host) = &self.host {
            host.cancel();
        }
    }
    pub fn place_button(
        &self,
        rect: Option<egui::Rect>,
        zoom: f32,
        text: egui::Color32,
        accent: egui::Color32,
    ) {
        #[cfg(target_arch = "wasm32")]
        if let Some(host) = &self.host {
            host.place(rect, zoom, text, accent);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = (rect, zoom, text, accent);
    }
    #[cfg(target_arch = "wasm32")]
    pub fn attach(&mut self, ctx: &egui::Context) {
        match web::Host::new(self.state.clone(), ctx.clone()) {
            Ok(host) => self.host = Some(host),
            Err(error) => {
                if let Ok(mut s) = self.state.try_borrow_mut() {
                    s.error = error;
                }
            }
        }
    }
}
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancelled_or_old_results_cannot_execute_a_command() {
        let mut s = State::default();
        let a = s.begin();
        s.cancel();
        s.started(a);
        assert_eq!(s.phase, Phase::Idle);
        assert!(!s.open_requested);
        s.result(a, "Delete layer", true);
        assert!(s.final_text.is_none());
        let b = s.begin();
        s.result(a, "Delete layer", true);
        assert!(s.final_text.is_none());
        s.result(b, "Brush tool", true);
        s.result(b, "Delete layer", true);
        assert_eq!(s.final_text.take().as_deref(), Some("Brush tool"));
        s.ended(b);
        assert!(s.error.is_empty());
    }
    #[test]
    fn interim_is_not_a_command_and_errors_allow_retry() {
        let mut s = State::default();
        let a = s.begin();
        s.result(a, "brush", false);
        assert!(s.final_text.is_none());
        s.failed(a, "Microphone denied");
        assert_eq!(s.phase, Phase::Idle);
        let b = s.begin();
        assert!(s.error.is_empty());
        s.started(b);
        assert_eq!(s.phase, Phase::Listening);
        s.ended(b);
        assert_eq!(s.phase, Phase::Idle);
        assert!(!s.error.is_empty());
    }
}
