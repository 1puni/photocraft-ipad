//! Tablet navigation policy, independent of the DOM so contact lifetimes can be tested.
use std::collections::BTreeMap;

use egui::{Pos2, Rect};

#[derive(Clone, Copy, Debug)]
pub struct Navigation {
    pub from: Pos2,
    pub to: Pos2,
    pub scale: f32,
}

#[derive(Default)]
pub struct Contacts {
    pub area: Option<Rect>,
    pub pen: Option<i32>,
    fingers: BTreeMap<i32, Pos2>,
    pair: Option<(Pos2, f32)>,
    palm_hold: bool,
    pub navigation: Vec<Navigation>,
    pub pen_moves: Vec<Pos2>,
    pub cancelled_pen: Option<Pos2>,
    pub ui_pointer: Option<i32>,
    pub ui_touches: Vec<egui::Event>,
}

impl Contacts {
    /// Insert real Pencil motion before eframe's release; remove duplicate compatibility moves.
    pub fn augment_input(&mut self, raw: &mut egui::RawInput) {
        raw.events.splice(0..0, self.ui_touches.drain(..));
        if !self.pen_moves.is_empty() {
            raw.events
                .retain(|e| !matches!(e, egui::Event::PointerMoved(_)));
            raw.events.splice(
                0..0,
                self.pen_moves.drain(..).map(egui::Event::PointerMoved),
            );
        }
        if let Some(pos) = self.cancelled_pen.take() {
            raw.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            });
            raw.events.push(egui::Event::PointerGone);
        }
    }
    /// A finger that starts on the drawing area stays owned through its release, even outside.
    /// One finger is inert: the first contact of a pinch (or a palm) must never leave a dot.
    pub fn down(&mut self, id: i32, p: Pos2) -> bool {
        if self.pen.is_none() && !self.area.is_some_and(|r| r.contains(p)) {
            return false;
        }
        self.palm_hold |= self.pen.is_some();
        self.fingers.insert(id, p);
        self.rebase();
        true
    }

    pub fn moved(&mut self, id: i32, p: Pos2) -> bool {
        let Some(point) = self.fingers.get_mut(&id) else {
            return false;
        };
        *point = p;
        let before = self.pair;
        self.rebase();
        if self.pen.is_none()
            && let (Some((from, a)), Some((to, b))) = (before, self.pair)
            && a > 1.0
            && b > 1.0
        {
            self.navigation.push(Navigation {
                from,
                to,
                scale: (b / a).clamp(0.25, 4.0),
            });
        }
        true
    }

    pub fn up(&mut self, id: i32) -> bool {
        let owned = self.fingers.remove(&id).is_some();
        if self.fingers.is_empty() {
            self.palm_hold = false;
        }
        self.rebase();
        owned
    }

    pub fn pen_down(&mut self, id: i32) {
        self.pen = Some(id);
        self.palm_hold |= !self.fingers.is_empty();
        self.navigation.clear();
        self.pair = None;
    }

    pub fn pen_up(&mut self, id: i32) {
        if self.pen == Some(id) {
            self.pen = None;
            // Contacts that arrived during a pen stroke remain suppressed until lifted.
            self.pair = None;
        }
    }

    fn rebase(&mut self) {
        self.pair = if self.fingers.len() == 2 && self.pen.is_none() && !self.palm_hold {
            let mut points = self.fingers.values();
            points
                .next()
                .zip(points.next())
                .map(|(a, b)| (a.lerp(*b, 0.5), a.distance(*b)))
        } else {
            None
        };
    }
}

/// Keep the document point under the fingers fixed while translating and scaling the view.
pub fn navigate(view: &mut photocraft_ui_egui::state::View, rect: Rect, flip: bool, n: Navigation) {
    let zoom = (view.zoom * n.scale).clamp(0.01, 64.0);
    let a = (n.from - rect.center()) / view.zoom;
    let b = (n.to - rect.center()) / zoom;
    view.center[0] += (a.x - b.x) * if flip { -1.0 } else { 1.0 };
    view.center[1] += a.y - b.y;
    view.zoom = zoom;
    view.fit_pending = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_touches_enable_kinetic_scrolling_without_becoming_pen_pressure() {
        let mut contacts = Contacts::default();
        contacts.ui_touches.push(egui::Event::Touch {
            device_id: egui::TouchDeviceId(1),
            id: egui::TouchId(3),
            phase: egui::TouchPhase::Start,
            pos: egui::pos2(20., 20.),
            force: None,
        });
        let mut raw = egui::RawInput::default();
        contacts.augment_input(&mut raw);
        let mut stylus = photocraft_ui_egui::stylus::Stylus::default();
        stylus.update(&raw.events);
        assert!(stylus.sample().is_none());
        let ctx = egui::Context::default();
        ctx.run_ui(raw, |ui| {
            assert!(ui.input(|input| input.has_touch_screen()))
        })
        .textures_delta
        .clear();
        assert!(contacts.ui_touches.is_empty());
    }

    #[test]
    fn pen_motion_precedes_release_and_cancellation_releases_the_button() {
        let mut c = Contacts::default();
        let p = egui::pos2(30., 40.);
        c.pen_moves.push(p);
        let release = egui::Event::PointerButton {
            pos: p,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        };
        let mut raw = egui::RawInput {
            events: vec![egui::Event::PointerMoved(Pos2::ZERO), release.clone()],
            ..Default::default()
        };
        c.augment_input(&mut raw);
        assert_eq!(
            raw.events,
            vec![egui::Event::PointerMoved(p), release.clone()]
        );
        assert!(c.pen_moves.is_empty());
        raw.events.clear();
        c.cancelled_pen = Some(p);
        c.augment_input(&mut raw);
        assert_eq!(raw.events, vec![release, egui::Event::PointerGone]);
        assert!(c.cancelled_pen.is_none());
    }
    fn contacts() -> Contacts {
        Contacts {
            area: Some(Rect::from_min_max(Pos2::ZERO, egui::pos2(500., 500.))),
            ..Default::default()
        }
    }

    #[test]
    fn a_finger_never_draws_and_ui_taps_are_not_owned() {
        let mut c = contacts();
        assert!(!c.down(1, egui::pos2(600., 100.)));
        assert!(c.down(2, egui::pos2(100., 100.)));
        assert!(c.moved(2, egui::pos2(700., 100.)));
        assert!(c.navigation.is_empty());
        assert!(c.up(2));
        assert!(!c.up(2));
    }

    #[test]
    fn pinch_zooms_and_the_lifted_finger_cannot_leave_a_stroke() {
        let mut c = contacts();
        c.down(1, egui::pos2(100., 100.));
        c.down(2, egui::pos2(200., 100.));
        c.moved(2, egui::pos2(300., 100.));
        let n = c.navigation.pop().unwrap();
        assert_eq!(n.scale, 2.);
        assert_eq!(n.from, egui::pos2(150., 100.));
        assert_eq!(n.to, egui::pos2(200., 100.));
        c.up(2);
        c.moved(1, egui::pos2(50., 100.));
        assert!(c.navigation.is_empty());
    }

    #[test]
    fn palm_contacts_cannot_move_the_canvas_during_a_stroke() {
        let mut c = contacts();
        c.pen_down(9);
        c.down(1, egui::pos2(100., 100.));
        c.down(2, egui::pos2(200., 100.));
        c.moved(2, egui::pos2(300., 100.));
        assert!(c.navigation.is_empty());
        c.pen_up(8);
        assert_eq!(c.pen, Some(9));
        c.pen_up(9);
        c.moved(2, egui::pos2(350., 100.));
        c.moved(2, egui::pos2(400., 100.));
        assert!(c.navigation.is_empty(), "palms remain inert after pen lift");
        c.up(1);
        c.up(2);
        assert!(c.fingers.is_empty());
    }

    #[test]
    fn navigation_preserves_the_point_between_the_fingers() {
        let rect = Rect::from_min_max(Pos2::ZERO, egui::pos2(500., 500.));
        for flip in [false, true] {
            let mut v = photocraft_ui_egui::state::View {
                zoom: 1.,
                center: [250., 250.],
                ..Default::default()
            };
            let n = Navigation {
                from: egui::pos2(100., 100.),
                to: egui::pos2(150., 120.),
                scale: 2.,
            };
            let before = photocraft_ui_egui::canvas::ViewXform {
                rect,
                zoom: v.zoom,
                center: v.center,
                flip,
            }
            .to_doc(n.from);
            navigate(&mut v, rect, flip, n);
            let after = photocraft_ui_egui::canvas::ViewXform {
                rect,
                zoom: v.zoom,
                center: v.center,
                flip,
            }
            .to_doc(n.to);
            assert_eq!(before, after);
        }
    }
}
