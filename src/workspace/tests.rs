use super::*;
use photocraft_engine::Session;
use photocraft_ui_egui::Services;

fn app() -> PhotocraftApp {
    PhotocraftApp::new(Session::new(), Services::default())
}

#[test]
fn touch_modifiers_preserve_hardware_and_release_latches() {
    let mut workspace = TabletUi {
        shift: true,
        ..Default::default()
    };
    let mut raw = egui::RawInput::default();
    raw.events
        .push(egui::Event::ModifiersChanged(egui::Modifiers::CTRL));
    workspace.raw_input(&mut raw);
    assert!(matches!(raw.events.last(),Some(egui::Event::ModifiersChanged(m)) if m.ctrl&&m.shift));
    workspace.shift = false;
    raw.events.clear();
    workspace.raw_input(&mut raw);
    assert!(
        matches!(raw.events.first(),Some(egui::Event::ModifiersChanged(m)) if m.ctrl&&!m.shift)
    );
}

#[test]
fn touch_colour_and_layer_actions_use_shared_undo_history() {
    let mut app = app();
    let ctx = egui::Context::default();
    let mut workspace = TabletUi::default();
    workspace.invoke(&mut app, &ctx, "file.new", json!({"width":64,"height":64}));
    assert!(workspace.message.is_empty(), "{}", workspace.message);
    let original = app.session.active().unwrap().doc.layers.len();
    workspace.invoke(&mut app, &ctx, "layer.new.layer", json!({"name":"Pencil"}));
    assert_eq!(app.session.active().unwrap().doc.layers.len(), original + 1);
    workspace.invoke(&mut app, &ctx, "edit.undo", json!({}));
    assert_eq!(app.session.active().unwrap().doc.layers.len(), original);
    workspace.invoke(&mut app, &ctx, "edit.redo", json!({}));
    assert_eq!(app.session.active().unwrap().doc.layers.len(), original + 1);
    workspace.set_color(&mut app, &ctx, [0.2, 0.4, 0.6]);
    assert_eq!(app.session.tools.foreground, [0.2, 0.4, 0.6, 1.]);
}

#[test]
fn portrait_landscape_and_split_view_leave_a_canvas() {
    for size in [
        vec2(1194., 834.),
        vec2(834., 1194.),
        vec2(768., 1024.),
        vec2(507., 768.),
    ] {
        for inspector in [
            Inspector::Layers,
            Inspector::Brush,
            Inspector::Color,
            Inspector::History,
        ] {
            let ctx = egui::Context::default();
            let mut app = app();
            let mut workspace = TabletUi {
                inspector,
                ..Default::default()
            };
            let mut canvas = egui::Rect::NOTHING;
            for _ in 0..3 {
                let raw = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                    ..Default::default()
                };
                let mut output = ctx.run_ui(raw, |ui| {
                    workspace.show(&mut app, ui);
                    canvas = ui.available_rect_before_wrap();
                });
                output.textures_delta.clear();
            }
            assert!(canvas.width() > 200., "{size:?} {inspector:?} {canvas:?}");
            assert!(canvas.height() > 180., "{size:?} {inspector:?} {canvas:?}");
            assert!(
                canvas.max.x <= size.x && canvas.max.y <= size.y,
                "{size:?} {canvas:?}"
            );
        }
    }
}

#[test]
fn every_tool_inspector_fits_split_view() {
    let size = vec2(507., 768.);
    for tool in Tool::ALL {
        let ctx = egui::Context::default();
        PhotocraftApp::setup_context(&ctx, photocraft_ui_egui::theme::ThemeKind::Pro);
        photocraft_ui_egui::touch_ui::set_enabled(&ctx, true);
        let mut app = app();
        app.ui.tool = tool;
        let mut workspace = TabletUi {
            inspector: Inspector::Tool,
            ..Default::default()
        };
        let mut canvas = egui::Rect::NOTHING;
        for _ in 0..3 {
            let raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                ..Default::default()
            };
            ctx.run_ui(raw, |ui| {
                workspace.show(&mut app, ui);
                canvas = ui.available_rect_before_wrap();
            })
            .textures_delta
            .clear();
        }
        assert!(
            canvas.width() > 200. && canvas.height() > 180.,
            "{tool:?}: {canvas:?}"
        );
    }
}

#[test]
fn every_brush_section_opens_without_mutating_the_brush() {
    for section in 0..photocraft_ui_egui::brush_panel::SECTIONS.len() {
        let ctx = egui::Context::default();
        PhotocraftApp::setup_context(&ctx, photocraft_ui_egui::theme::ThemeKind::Pro);
        photocraft_ui_egui::touch_ui::set_enabled(&ctx, true);
        let mut app = app();
        app.ui.brush_section = section;
        let before = app.session.tools.brush.clone();
        let mut workspace = TabletUi {
            sheet: Some(Sheet::Brush),
            ..Default::default()
        };
        for _ in 0..3 {
            let raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    vec2(507., 768.),
                )),
                ..Default::default()
            };
            ctx.run_ui(raw, |ui| workspace.show(&mut app, ui))
                .textures_delta
                .clear();
        }
        assert_eq!(
            app.session.tools.brush, before,
            "section {section} mutated on display"
        );
    }
}

#[test]
fn channel_controls_target_duplicate_and_undo_through_real_widgets() {
    use egui_kittest::{Harness, kittest::Queryable};
    let mut app = app();
    app.run("file.new", json!({"width":32,"height":32}))
        .unwrap();
    let workspace = TabletUi::default();
    let mut h = Harness::builder()
        .with_size(vec2(1194., 834.))
        .build_ui_state(
            |ui, (app, workspace): &mut (PhotocraftApp, TabletUi)| workspace.show(app, ui),
            (app, workspace),
        );
    PhotocraftApp::setup_context(&h.ctx, photocraft_ui_egui::theme::ThemeKind::Pro);
    photocraft_ui_egui::touch_ui::set_enabled(&h.ctx, true);
    h.run_steps(3);
    h.get_by_label("Channels").click();
    h.run_steps(3);
    h.get_by_label("New alpha channel").click();
    h.run_steps(3);
    let name = h.state().0.session.active().unwrap().doc.channels[0]
        .name
        .clone();
    h.get_by_label(&name).click();
    h.run_steps(3);
    assert_eq!(
        h.state().0.session.active().unwrap().channel_view.target,
        photocraft_engine::channel_cmds::ChannelTarget::Alpha(0)
    );
    h.get_by_label("Duplicate channel").click();
    h.run_steps(3);
    assert!(h.state().1.message.is_empty(), "{}", h.state().1.message);
    assert_eq!(h.state().0.session.active().unwrap().doc.channels.len(), 2);
    h.get_by_label("Close").click();
    h.run_steps(3);
    h.get_by_label("Undo").click();
    h.run_steps(3);
    assert_eq!(h.state().0.session.active().unwrap().doc.channels.len(), 1);
}
