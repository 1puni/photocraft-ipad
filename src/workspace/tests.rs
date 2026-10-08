use super::*;
use photocraft_engine::Session;
use photocraft_ui_egui::Services;

fn app() -> PhotocraftApp {
    PhotocraftApp::new(Session::new(), Services::default())
}

#[test]
fn fresh_pointer_drag_scrolls_layers_without_selecting_or_editing() {
    use egui_kittest::{Harness, kittest::Queryable};
    let mut app = app();
    app.run("file.new", json!({"width":32,"height":32}))
        .unwrap();
    for i in 0..20 {
        app.run(
            "layer.new.layer",
            json!({"name":format!("Scroll layer {i}")}),
        )
        .unwrap();
    }
    let before = app.session.active().unwrap().doc.clone();
    let active = app.session.active().unwrap().active_layer;
    let mut h = Harness::builder()
        .with_size(vec2(1194., 834.))
        .build_ui_state(
            |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
            (app, TabletUi::default()),
        );
    h.run_steps(3);
    assert!(!h.ctx.input(|i| i.has_touch_screen()));
    let pos = h.get_by_label("Image: Scroll layer 19").rect().center();
    h.hover_at(pos);
    h.drag_at(pos);
    h.run_steps(1);
    h.hover_at(pos - vec2(0., 80.));
    h.run_steps(1);
    h.hover_at(pos - vec2(0., 160.));
    h.run_steps(1);
    h.drop_at(pos - vec2(0., 160.));
    h.run_steps(3);
    assert_eq!(h.state().0.session.active().unwrap().doc, before);
    assert_eq!(h.state().0.session.active().unwrap().active_layer, active);
    assert!(h.get_by_label("Image: Scroll layer 19").rect().center().y < pos.y - 50.);
}

#[test]
fn selection_menu_copies_or_cuts_selected_pixels_and_undo_restores_them() {
    use egui_kittest::{Harness, kittest::Queryable};
    for (label, cut) in [("Layer via Copy", false), ("Layer via Cut", true)] {
        let mut app = app();
        app.run(
            "file.new",
            json!({"width":32,"height":32,"background":"transparent"}),
        )
        .unwrap();
        app.run("select.rect", json!({"x":0,"y":0,"width":16,"height":32}))
            .unwrap();
        app.run("edit.fill", json!({"color":"#ff0000"})).unwrap();
        app.run("select.rect", json!({"x":0,"y":0,"width":8,"height":32}))
            .unwrap();
        app.ui.tool = Tool::RectMarquee;
        let before = app.session.active().unwrap().doc.clone();
        let source = app.session.active().unwrap().active_layer.unwrap();
        let mut h = Harness::builder()
            .with_size(vec2(1194., 834.))
            .build_ui_state(
                |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
                (app, TabletUi::default()),
            );
        h.run_steps(3);
        h.get_by_label("Selection").click();
        h.run_steps(3);
        h.get_by_label(label).click();
        h.run_steps(3);
        assert!(h.state().1.message.is_empty(), "{}", h.state().1.message);
        let doc = &h.state().0.session.active().unwrap().doc;
        let active = h.state().0.session.active().unwrap().active_layer.unwrap();
        assert_ne!(source, active);
        let pixels = doc.layer(active).unwrap().surface().unwrap();
        assert!(pixels.sample_channel(4, 16, 3) > 0.99);
        assert!(pixels.sample_channel(12, 16, 3) < 0.01);
        let original = doc.layer(source).unwrap().surface().unwrap();
        assert_eq!(original.sample_channel(4, 16, 3) > 0.99, !cut);
        assert!(original.sample_channel(12, 16, 3) > 0.99);
        h.get_by_label("Undo").click();
        h.run_steps(3);
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
    }
}

#[test]
fn stamp_source_pick_is_one_shot_then_the_next_stroke_clones_and_undoes() {
    use egui_kittest::{Harness, kittest::Queryable};
    use photocraft_ui_egui::canvas::{ToolEvent, tool_event};
    let mut app = app();
    app.run(
        "file.new",
        json!({"width":32,"height":32,"background":"transparent"}),
    )
    .unwrap();
    app.run("select.rect", json!({"x":0,"y":0,"width":16,"height":32}))
        .unwrap();
    app.run("edit.fill", json!({"color":"#ff0000"})).unwrap();
    app.run("select.deselect", json!({})).unwrap();
    app.session.tools.brush.size = 6.;
    app.ui.tool = Tool::CloneStamp;
    app.ui.views.push(Default::default());
    let before = app.session.active().unwrap().doc.clone();
    let mut h = Harness::builder()
        .with_size(vec2(1194., 834.))
        .build_ui_state(
            |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
            (app, TabletUi::default()),
        );
    h.run_steps(3);
    h.get_by_label("Set source").click();
    h.run_steps(3);
    assert!(h.state().1.clone_source_pick);
    let ctx = egui::Context::default();
    for pressed in [true, false] {
        let (app, w) = h.state_mut();
        let mut raw = egui::RawInput::default();
        raw.events.push(egui::Event::PointerButton {
            pos: egui::pos2(300., 300.),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        w.raw_input(&mut raw);
        ctx.run_ui(raw, |ui| {
            w.update_source_pick(app, ui.ctx());
            let mods = ui.input(|i| i.modifiers);
            assert!(mods.alt);
            tool_event(
                app,
                if pressed {
                    ToolEvent::Down {
                        x: 8.,
                        y: 16.,
                        pressure: 0.5,
                    }
                } else {
                    ToolEvent::Up { x: 8., y: 16. }
                },
                mods,
            );
        })
        .textures_delta
        .clear();
    }
    assert!(!h.state().1.clone_source_pick);
    assert_eq!(h.state().0.session.active().unwrap().doc, before);
    let (app, w) = h.state_mut();
    let mut raw = egui::RawInput::default();
    w.raw_input(&mut raw);
    ctx.run_ui(raw, |ui| {
        let mods = ui.input(|i| i.modifiers);
        assert!(!mods.alt);
        tool_event(
            app,
            ToolEvent::Down {
                x: 24.,
                y: 16.,
                pressure: 1.,
            },
            mods,
        );
        tool_event(app, ToolEvent::Up { x: 24., y: 16. }, mods);
    })
    .textures_delta
    .clear();
    assert!(!app.ui.status_error, "{}", app.ui.status);
    let st = app.session.active().unwrap();
    assert!(
        st.doc
            .layer(st.active_layer.unwrap())
            .unwrap()
            .surface()
            .unwrap()
            .sample_channel(24, 16, 3)
            > 0.5
    );
    w.invoke(app, &ctx, "edit.undo", json!({}));
    assert_eq!(app.session.active().unwrap().doc, before);
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

#[test]
fn command_tray_switches_tools_and_dispatches_dialogs_without_mutating_on_search() {
    use egui_kittest::{Harness, kittest::Queryable};
    let mut app = app();
    app.run("file.new", json!({"width":32,"height":32}))
        .unwrap();
    let before = app.session.active().unwrap().doc.clone();
    let mut h = Harness::builder()
        .with_size(vec2(1194., 834.))
        .build_ui_state(
            |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
            (app, TabletUi::default()),
        );
    h.state_mut().1.command_text = "switch to eraser tool".into();
    h.run_steps(3);
    h.get_by_label("Find command").click();
    h.run_steps(3);
    assert_eq!(h.state().0.ui.tool, Tool::Brush, "search must not execute");
    h.get_by_label("Eraser Tool   ·   Tools").click();
    h.run_steps(3);
    assert_eq!(h.state().0.ui.tool, Tool::Eraser);
    assert!(!h.state().1.command_tray_open);
    h.get_by_label("Find command").click();
    h.run_steps(3);
    h.get_by_label("Restore Brush Tool").click();
    h.run_steps(3);
    assert_eq!(h.state().0.ui.tool, Tool::Brush);
    h.state_mut().1.command_text = "levels".into();
    h.get_by_label("Find command").click();
    h.run_steps(3);
    assert!(h.state().1.command_tray_open);
    assert_eq!(h.state().0.session.active().unwrap().doc, before);
    let result = h.get_by_label("Levels…   ·   Image / Adjustments");
    result.click();
    h.run_steps(3);
    assert_eq!(h.state().0.ui.dialogs.len(), 1);
    assert!(!h.state().1.command_tray_open);
}

#[test]
fn command_search_is_on_demand_and_does_not_resize_the_canvas() {
    use egui_kittest::{Harness, kittest::Queryable};
    for size in [vec2(834., 1194.), vec2(507., 768.), vec2(507., 450.)] {
        let mut h = Harness::builder().with_size(size).build_ui_state(
            |ui, (app, w, canvas): &mut (PhotocraftApp, TabletUi, egui::Rect)| {
                w.show(app, ui);
                *canvas = ui.available_rect_before_wrap();
            },
            (app(), TabletUi::default(), egui::Rect::NOTHING),
        );
        h.run_steps(3);
        let canvas_before = h.state().2;
        assert!(h.query_by_label("Close commands").is_none());
        let search = h.get_by_label("Find command").rect();
        assert!(
            search.bottom() < 60. && search.right() <= size.x,
            "search belongs in the top bar: {search:?}"
        );
        h.get_by_label("Find command").click();
        h.run_steps(3);
        assert_eq!(
            h.state().2,
            canvas_before,
            "opening commands resized the canvas at {size:?}"
        );
        assert!(h.state().1.inspector_open, "preserve inspector state");
        let close = h.get_by_label("Close commands").rect();
        assert!(
            close.top() >= 0. && close.right() <= size.x && close.bottom() <= size.y,
            "{size:?}: {close:?}"
        );
        h.state_mut().1.command_text = "blur".into();
        h.run_steps(3);
        assert_eq!(
            h.get_by_label("Close commands").rect(),
            close,
            "typing must not move the search sheet"
        );
        h.get_by_label("Close commands").click();
        h.run_steps(3);
        assert!(!h.state().1.command_tray_open);
        assert_eq!(h.state().2, canvas_before);
        assert!(h.query_all_by_label("Layers").next().is_some());
    }
}

#[test]
fn window_commands_show_tablet_panels_instead_of_hidden_desktop_docks() {
    let ctx = egui::Context::default();
    let mut app = app();
    let mut w = TabletUi::default();
    for (id, panel) in [
        ("window.toggle.history", Inspector::History),
        ("window.layers", Inspector::Layers),
        ("window.toggle.color", Inspector::Color),
        ("window.brushes", Inspector::Brush),
    ] {
        w.inspector_open = false;
        w.invoke(&mut app, &ctx, id, json!({}));
        assert_eq!(w.inspector, panel);
        assert!(w.inspector_open);
    }
    w.invoke(&mut app, &ctx, "window.properties", json!({}));
    assert_eq!(w.sheet, Some(Sheet::LayerProperties));
}

#[test]
fn blur_search_opens_each_named_filter_instead_of_reusing_box_blur() {
    use egui_kittest::{Harness, kittest::Queryable};
    for (label, id) in [
        ("Gaussian Blur…", "filter.blur.gaussianBlur"),
        ("Box Blur…", "filter.blur.boxBlur"),
        ("Motion Blur…", "filter.blur.motionBlur"),
        ("Lens Blur…", "filter.blur.lensBlur"),
    ] {
        let mut app = app();
        app.run("file.new", json!({"width":32,"height":32}))
            .unwrap();
        let before = app.session.active().unwrap().doc.clone();
        let mut h = Harness::builder()
            .with_size(vec2(834., 1194.))
            .build_ui_state(
                |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
                (app, TabletUi::default()),
            );
        h.state_mut().1.command_text = "blur".into();
        h.get_by_label("Find command").click();
        h.run_steps(3);
        let result = format!("{label}   ·   Filter / Blur");
        h.get_by_label(&result).scroll_to_me();
        h.run_steps(3);
        h.get_by_label(&result).click();
        h.run_steps(3);
        assert_eq!(
            h.state().0.ui.dialogs.last().unwrap().fields["__command"],
            json!(id)
        );
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
    }
}

#[test]
fn rail_colour_chips_open_the_correct_target_without_changing_colours() {
    use egui_kittest::{Harness, kittest::Queryable};
    let mut app = app();
    app.run(
        "tools.setColors",
        json!({"foreground":"#245d85","background":"#e6b97a"}),
    )
    .unwrap();
    let colours = (app.session.tools.foreground, app.session.tools.background);
    let mut h = Harness::builder()
        .with_size(vec2(1194., 834.))
        .build_ui_state(
            |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
            (app, TabletUi::default()),
        );
    for (label, background, hex) in [
        ("Background colour", true, "#e6b97a"),
        ("Foreground colour", false, "#245d85"),
    ] {
        h.get_by_label(label).click();
        h.run_steps(3);
        assert_eq!(h.state().1.inspector, Inspector::Color);
        assert_eq!(h.state().1.color_background, background);
        assert_eq!(h.state().1.color_hex, hex);
        assert_eq!(
            (
                h.state().0.session.tools.foreground,
                h.state().0.session.tools.background
            ),
            colours
        );
    }
}
