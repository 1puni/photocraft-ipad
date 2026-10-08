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

#[test]
fn switching_sheets_keeps_close_reachable_without_layout_drift() {
    use egui_kittest::{Harness, kittest::Queryable};
    let mut h = Harness::builder()
        .with_size(vec2(507., 768.))
        .build_ui_state(
            |ui, (app, workspace): &mut (PhotocraftApp, TabletUi)| workspace.show(app, ui),
            (app(), TabletUi::default()),
        );
    PhotocraftApp::setup_context(&h.ctx, photocraft_ui_egui::theme::ThemeKind::Pro);
    photocraft_ui_egui::touch_ui::set_enabled(&h.ctx, true);
    for sheet in [
        Sheet::Paths,
        Sheet::Brush,
        Sheet::Files,
        Sheet::Commands,
        Sheet::Channels,
    ] {
        h.state_mut().1.open_sheet(sheet);
        h.run_steps(3);
        let before = h.get_by_label("Close").rect();
        assert!(
            before.min.y >= 0. && before.max.y <= 768.,
            "{sheet:?}: {before:?}"
        );
        h.get_by_label("Close").click();
        h.run_steps(3);
        assert!(
            h.state().1.sheet.is_none(),
            "{sheet:?} moved away from the tap"
        );
    }
}

#[test]
fn layer_name_follows_unlock_and_undo_without_overwriting_typing() {
    use egui_kittest::Harness;
    let mut app = app();
    app.run("file.new", json!({"width":32,"height":32}))
        .unwrap();
    let mut h = Harness::builder()
        .with_size(vec2(1194., 834.))
        .build_ui_state(
            |ui, (app, workspace): &mut (PhotocraftApp, TabletUi)| workspace.show(app, ui),
            (
                app,
                TabletUi {
                    sheet: Some(Sheet::LayerProperties),
                    ..Default::default()
                },
            ),
        );
    h.run_steps(3);
    assert_eq!(h.state().1.layer_name, "Background");
    h.state_mut()
        .0
        .run("layer.new.layerFromBackground", json!({}))
        .unwrap();
    h.run_steps(3);
    let name = h.state().0.session.active().unwrap().doc.layers[0]
        .name
        .clone();
    assert_eq!(h.state().1.layer_name, name);
    h.state_mut().1.layer_name = "Draft name".into();
    h.run_steps(3);
    assert_eq!(h.state().1.layer_name, "Draft name");
    h.state_mut().0.run("edit.undo", json!({})).unwrap();
    h.run_steps(3);
    assert_eq!(h.state().1.layer_name, "Background");
}

#[test]
fn scrollable_rail_contains_every_tool_once() {
    let tools: Vec<_> = navigation::TOOL_GROUPS
        .iter()
        .flat_map(|group| group.iter().copied())
        .collect();
    assert_eq!(tools.len(), Tool::ALL.len());
    for tool in Tool::ALL {
        assert_eq!(
            tools.iter().filter(|candidate| **candidate == tool).count(),
            1,
            "{tool:?}"
        );
        assert!(photocraft_ui_egui::icons::exists(
            photocraft_ui_egui::icons::tool_icon(tool)
        ));
    }
}

#[test]
fn studio_browses_nested_actions_without_a_flat_catalogue() {
    use egui_kittest::{Harness, kittest::Queryable};
    let mut app = app();
    app.run("file.new", json!({"width":32,"height":32}))
        .unwrap();
    let mut h = Harness::builder()
        .with_size(vec2(1194., 834.))
        .build_ui_state(
            |ui, (app, workspace): &mut (PhotocraftApp, TabletUi)| workspace.show(app, ui),
            (app, TabletUi::default()),
        );
    PhotocraftApp::setup_context(&h.ctx, photocraft_ui_egui::theme::ThemeKind::Pro);
    photocraft_ui_egui::touch_ui::set_enabled(&h.ctx, true);
    h.run_steps(3);
    h.get_by_label("Eraser Tool").click();
    h.run_steps(3);
    assert_eq!(h.state().0.ui.tool, Tool::Eraser);
    h.get_by_label("Studio").click();
    h.run_steps(3);
    assert!(h.query_by_label("Levels…").is_none());
    h.get_by_label("Image").click();
    h.run_steps(3);
    h.get_by_label("Adjustments").click();
    h.run_steps(3);
    assert_eq!(h.state().1.command_path, ["Image", "Adjustments"]);
    h.get_by_label("Levels…").click();
    h.run_steps(3);
    assert!(h.state().1.sheet.is_none());
    assert_eq!(h.state().0.ui.dialogs.len(), 1);
}

#[test]
fn image_and_mask_thumbnails_route_strokes_and_undo_independently() {
    use egui_kittest::{Harness, kittest::Queryable};
    use photocraft_ui_egui::canvas::paint_target;
    let mut app = app();
    app.run(
        "file.new",
        json!({"width":32,"height":32,"background":"transparent"}),
    )
    .unwrap();
    app.run("layer.setProps", json!({"name":"Ink"})).unwrap();
    app.run("layer.layerMask.revealAll", json!({})).unwrap();
    app.run("tools.setBrush", json!({"brush":{"size":8.,"hardness":1.}}))
        .unwrap();
    let initial = app.session.active().unwrap().doc.clone();
    let mut h = Harness::builder()
        .with_size(vec2(1194., 834.))
        .build_ui_state(
            |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
            (app, TabletUi::default()),
        );
    PhotocraftApp::setup_context(&h.ctx, photocraft_ui_egui::theme::ThemeKind::Pro);
    h.run_steps(3);
    h.get_by_label("Mask: Ink").click();
    h.run_steps(3);
    assert_eq!(paint_target(&h.state().0), json!("mask"));
    let target = paint_target(&h.state().0);
    h.state_mut()
        .0
        .run(
            "paint.stroke",
            json!({"points":[[16,16,1]],"target":target}),
        )
        .unwrap();
    let masked = h.state().0.session.active().unwrap().doc.clone();
    assert_eq!(masked.layers[0].surface(), initial.layers[0].surface());
    assert_ne!(masked.layers[0].mask, initial.layers[0].mask);
    h.state_mut()
        .0
        .run("view.layerMask", json!({"mode":"gray"}))
        .unwrap();
    h.run_steps(3);
    h.get_by_label("Image: Ink").click();
    h.run_steps(3);
    assert_eq!(paint_target(&h.state().0), json!("pixels"));
    assert!(
        photocraft_engine::mask_view_cmds::current(h.state().0.session.active().unwrap()).is_none()
    );
    let target = paint_target(&h.state().0);
    h.state_mut()
        .0
        .run("paint.stroke", json!({"points":[[8,8,1]],"target":target}))
        .unwrap();
    let painted = &h.state().0.session.active().unwrap().doc;
    assert_ne!(painted.layers[0].surface(), masked.layers[0].surface());
    assert_eq!(painted.layers[0].mask, masked.layers[0].mask);
    h.state_mut().0.run("edit.undo", json!({})).unwrap();
    assert_eq!(
        h.state().0.session.active().unwrap().doc.layers,
        masked.layers
    );
    h.state_mut().0.run("edit.undo", json!({})).unwrap();
    assert_eq!(
        h.state().0.session.active().unwrap().doc.layers,
        initial.layers
    );
    h.state_mut().0.run("channel.new", json!({})).unwrap();
    h.run_steps(3);
    h.get_by_label("Image: Ink").click();
    h.run_steps(3);
    assert_eq!(paint_target(&h.state().0), json!("pixels"));
    h.state_mut()
        .0
        .run("select.editInQuickMaskMode", json!({"on":true}))
        .unwrap();
    h.run_steps(3);
    h.get_by_label("Image: Ink").click();
    h.run_steps(3);
    assert_eq!(
        paint_target(&h.state().0),
        json!("quickMask"),
        "thumbnail must not silently discard Quick Mask"
    );
    assert!(h.query_by_label("Target · Quick Mask").is_some());
}

#[test]
fn layer_stack_keeps_footer_reachable_and_visibility_does_not_select() {
    use egui_kittest::{Harness, kittest::Queryable};
    for size in [vec2(1194., 834.), vec2(834., 1194.), vec2(507., 768.)] {
        let mut app = app();
        app.run("file.new", json!({"width":32,"height":32}))
            .unwrap();
        for i in 0..24 {
            app.run("layer.new.layer", json!({"name":format!("Layer {i}")}))
                .unwrap();
        }
        let active = app.session.active().unwrap().active_layer;
        let mut h = Harness::builder().with_size(size).build_ui_state(
            |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
            (app, TabletUi::default()),
        );
        PhotocraftApp::setup_context(&h.ctx, photocraft_ui_egui::theme::ThemeKind::Pro);
        h.run_steps(3);
        let before = h.get_by_label("New layer").rect();
        let first_row = h.get_by_label("Image: Layer 23").rect();
        assert!(
            before.top() - first_row.top() >= 100.,
            "two layer rows must fit: {size:?}"
        );
        assert!(
            before.bottom() <= size.y && before.right() <= size.x,
            "{size:?}: {before:?}"
        );
        h.get_by_label("Visibility: Layer 23").click();
        h.run_steps(3);
        assert_eq!(h.state().0.session.active().unwrap().active_layer, active);
        assert!(
            !h.state()
                .0
                .session
                .active()
                .unwrap()
                .doc
                .layer(active.unwrap())
                .unwrap()
                .visible
        );
        h.get_by_label("Select layer: Layer 0").scroll_to_me();
        h.run_steps(3);
        assert_eq!(
            h.get_by_label("New layer").rect(),
            before,
            "scroll moved footer"
        );
        h.get_by_label("Layer properties and actions").click();
        h.run_steps(3);
        assert_eq!(h.state().1.sheet, Some(Sheet::LayerProperties));
    }
    for icon in [
        "check",
        "blend",
        "pen-tool",
        "plus",
        "folder-plus",
        "scan",
        "ellipsis",
    ] {
        assert!(photocraft_ui_egui::icons::exists(icon), "{icon}");
    }
}

#[test]
fn add_mask_uses_selection_and_exits_alpha_channel_target() {
    use egui_kittest::{Harness, kittest::Queryable};
    let mut app = app();
    app.run(
        "file.new",
        json!({"width":32,"height":32,"background":"transparent"}),
    )
    .unwrap();
    app.run("select.rect", json!({"x":0,"y":0,"width":16,"height":16}))
        .unwrap();
    app.run("channel.new", json!({})).unwrap();
    let mut h = Harness::builder()
        .with_size(vec2(1194., 834.))
        .build_ui_state(
            |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
            (app, TabletUi::default()),
        );
    h.run_steps(3);
    h.get_by_label("Add layer mask").click();
    h.run_steps(3);
    assert!(h.state().1.message.is_empty(), "{}", h.state().1.message);
    assert_eq!(
        photocraft_ui_egui::canvas::paint_target(&h.state().0),
        json!("mask")
    );
    let mask = h.state().0.session.active().unwrap().doc.layers[0]
        .mask
        .as_ref()
        .unwrap();
    assert!(mask.value(8, 8) > 0.99);
    assert!(mask.value(24, 24) < 0.01);
}
