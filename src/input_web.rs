//! Capture finger navigation before eframe's mouse emulation; Pencil remains a real pointer.
use crate::input::Contacts;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast as _, closure::Closure};

pub type Shared = Rc<RefCell<Contacts>>;

pub fn install(canvas: &web_sys::HtmlCanvasElement, ctx: &egui::Context, contacts: Shared) {
    for kind in [
        "pointerdown",
        "pointermove",
        "pointerup",
        "pointercancel",
        "lostpointercapture",
    ] {
        let canvas_ref = canvas.clone();
        let ctx = ctx.clone();
        let contacts = contacts.clone();
        let cb =
            Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |e: web_sys::PointerEvent| {
                let rect = canvas_ref.get_bounding_client_rect();
                let p = egui::pos2(
                    (e.client_x() as f32 - rect.left() as f32) / ctx.zoom_factor(),
                    (e.client_y() as f32 - rect.top() as f32) / ctx.zoom_factor(),
                );
                let id = e.pointer_id();
                let kind = e.type_();
                let mut c = contacts.borrow_mut();
                if e.pointer_type() == "pen" {
                    // eframe 0.36 listens to mousemove rather than pointermove. Safari need not
                    // generate mouse compatibility moves for Pencil, especially after touch rejection.
                    if kind == "pointerdown" || kind == "pointermove" || kind == "pointerup" {
                        c.pen_moves.push(p);
                        ctx.request_repaint();
                    }
                    if kind == "pointerdown" {
                        c.pen_down(id);
                        let _ = canvas_ref.set_pointer_capture(id);
                    } else if kind == "pointerup"
                        || kind == "pointercancel"
                        || kind == "lostpointercapture"
                    {
                        if kind != "pointerup" && c.pen == Some(id) {
                            c.cancelled_pen = Some(p);
                            ctx.request_repaint();
                        }
                        c.pen_up(id);
                    }
                } else if e.pointer_type() == "touch" {
                    let consumed = match kind.as_str() {
                        "pointerdown"
                            if c.pen.is_none()
                                && (egui::Popup::is_any_open(&ctx)
                                    || ctx.layer_id_at(p).is_some_and(|layer| {
                                        layer.order != egui::Order::Background
                                    })) =>
                        {
                            false
                        }
                        "pointerdown" => c.down(id, p),
                        "pointermove" => c.moved(id, p),
                        _ => c.up(id),
                    };
                    if consumed {
                        if kind == "pointerdown" {
                            let _ = canvas_ref.set_pointer_capture(id);
                        }
                        e.prevent_default();
                        e.stop_immediate_propagation();
                        ctx.request_repaint();
                    } else if kind == "pointerdown" || kind == "pointermove" || kind == "pointerup"
                    {
                        // UI sliders and menu taps still need motion after legacy TouchEvents were
                        // suppressed. These are outside the document gesture owned above.
                        c.pen_moves.push(p);
                        ctx.request_repaint();
                    }
                }
            });
        if canvas
            .add_event_listener_with_callback_and_bool(kind, cb.as_ref().unchecked_ref(), true)
            .is_ok()
        {
            cb.forget();
        }
    }
    // Pointer Events already carry UI taps and pen strokes. Safari also emits legacy TouchEvents;
    // forwarding both would start a second stroke and turn rejected palms back into mouse input.
    for kind in ["touchstart", "touchmove", "touchend", "touchcancel"] {
        let cb = Closure::<dyn FnMut(web_sys::Event)>::new(|e: web_sys::Event| {
            e.prevent_default();
            e.stop_immediate_propagation();
        });
        if canvas
            .add_event_listener_with_callback_and_bool(kind, cb.as_ref().unchecked_ref(), true)
            .is_ok()
        {
            cb.forget();
        }
    }
}
