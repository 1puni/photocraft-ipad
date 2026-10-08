//! A bounded layer stack. Painting targets are explicit, controls never scroll away.
use super::*;
use photocraft_doc::{Document, Layer, LayerContent, LayerId};
use photocraft_ui_egui::{canvas, icons, layer_menu_ui, layer_reveal, mask_thumbs_ui, widgets};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum LayerTarget {
    Image,
    Mask,
    Vector,
}

impl TabletUi {
    pub(super) fn open_layer_properties(&mut self) {
        self.sheet = None;
        self.command_tray_open = false;
        self.layer_properties = true;
        self.mask_controls = None;
        self.cancel_layer_arrange_drag();
        self.inspector = Inspector::Layers;
        self.inspector_open = true;
    }

    pub(super) fn cancel_layer_arrange_drag(&mut self) {
        self.layer_arrange_drag = None;
        self.layer_autoscroll_at = None;
    }

    pub(super) fn target_layer(
        &mut self,
        app: &mut PhotocraftApp,
        ctx: &egui::Context,
        id: LayerId,
        target: LayerTarget,
    ) {
        // Explicit layer targeting finishes Quick Mask's selection edit first.
        // Otherwise its higher-priority paint target silently captures the next stroke.
        if app
            .session
            .active()
            .is_some_and(|st| st.doc.quick_mask.is_some())
        {
            self.invoke(app, ctx, "select.editInQuickMaskMode", json!({"on":false}));
            if !self.message.is_empty() {
                return;
            }
        }
        // Leave mask view when changing targets; keep it while editing the same mask.
        if let Some(view) = app
            .session
            .active()
            .and_then(photocraft_engine::mask_view_cmds::current)
            .filter(|view| target != LayerTarget::Mask || view.layer != id)
        {
            self.invoke(
                app,
                ctx,
                "view.layerMask",
                json!({"layer":view.layer.0,"mode":"off"}),
            );
            if !self.message.is_empty() {
                return;
            }
        }
        if app
            .session
            .active()
            .is_none_or(|st| st.active_layer != Some(id))
        {
            self.invoke(
                app,
                ctx,
                "layer.select",
                json!({"layer":id.0,"mode":"replace"}),
            );
            if !self.message.is_empty() {
                return;
            }
        }
        self.invoke(app, ctx, "channel.target", json!({"channel":"composite"}));
        if !self.message.is_empty() {
            return;
        }
        app.ui.mask_target = target == LayerTarget::Mask;
        app.ui.vector_mask_target = target == LayerTarget::Vector;
    }

    fn tap_layer_target(
        &mut self,
        app: &mut PhotocraftApp,
        ctx: &egui::Context,
        id: LayerId,
        target: LayerTarget,
    ) {
        if self.multi_select {
            self.invoke(
                app,
                ctx,
                "layer.select",
                json!({"layer":id.0,"mode":"toggle"}),
            );
        } else {
            self.target_layer(app, ctx, id, target);
        }
    }

    pub(super) fn layers(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        if self.mask_controls.is_some() {
            self.cancel_layer_arrange_drag();
            if self.mask_controls_panel(app, ui) {
                return;
            }
        }
        if self.layer_properties {
            self.cancel_layer_arrange_drag();
            if self.layer_properties_panel(app, ui) {
                return;
            }
        }
        let reveal = layer_reveal::track(app, ui.ctx());
        let selected_count = app
            .session
            .active()
            .map_or(0, |st| st.selected_layers().len());
        let quick_mask = app
            .session
            .active()
            .is_some_and(|st| st.doc.quick_mask.is_some());
        if quick_mask && self.arrange_layers {
            self.arrange_layers = false;
            self.cancel_layer_arrange_drag();
        }
        ui.horizontal(|ui| {
            ui.strong("Layers");
            let arrange_label = if quick_mask {
                "Arrange layers · exit Quick Mask first".to_string()
            } else if selected_count > 1 {
                format!("Arrange layers · move {selected_count} selected layers")
            } else {
                "Arrange layers".to_string()
            };
            let arrange = ui
                .add_enabled_ui(!quick_mask, |ui| {
                    navigation::icon_button(ui, "move", &arrange_label, self.arrange_layers, 44.)
                })
                .inner
                .on_hover_text(&arrange_label);
            if arrange.clicked() {
                self.arrange_layers = !self.arrange_layers;
                self.cancel_layer_arrange_drag();
            }
            if navigation::icon_button(
                ui,
                "check",
                "Select multiple layers",
                self.multi_select,
                44.,
            )
            .clicked()
            {
                self.multi_select = !self.multi_select;
            }
            if navigation::icon_button(ui, "blend", "Channels", false, 44.).clicked() {
                self.open_sheet(Sheet::Channels);
            }
            if navigation::icon_button(ui, "pen-tool", "Paths", false, 44.).clicked() {
                self.open_sheet(Sheet::Paths);
            }
        });
        let Some(st) = app.session.active() else {
            self.cancel_layer_arrange_drag();
            ui.label("Open an image to work with layers.");
            return;
        };
        let doc = st.doc.clone();
        if self
            .layer_arrange_drag
            .as_ref()
            .is_some_and(|drag| drag.doc_id != doc.id.0)
        {
            self.cancel_layer_arrange_drag();
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.cancel_layer_arrange_drag();
            self.arrange_layers = false;
        }
        let selected = st.selected_layers().to_vec();
        let active = st.active_layer;
        let target = canvas::paint_target(app);
        let caption = if target == json!("quickMask") {
            "Quick Mask"
        } else if target.get("channel").is_some() {
            "Alpha channel"
        } else if app.ui.vector_mask_target {
            "Vector path"
        } else if target == json!("mask") {
            "Layer mask"
        } else {
            "Layer image"
        };
        let spacious = ui.available_height() >= 310.;
        if spacious {
            if let Some(layer) = active.and_then(|id| doc.layer(id)) {
                self.layer_blending(app, ui, &doc, layer);
            }
            ui.label(egui::RichText::new(format!("Target · {caption}")).small());
        }
        // Reserve footer space before sizing the scroll area. The stack owns its scroll.
        let stack_height = (ui.available_height() - 50.).max(50.);
        let mut drop_target = None;
        let scroll = egui::ScrollArea::vertical()
            .scroll_source(egui::scroll_area::ScrollSource::ALL)
            .id_salt("ipad-layer-stack")
            .auto_shrink([false, false])
            .max_height(stack_height)
            .min_scrolled_height(stack_height)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 2.;
                for (depth, layer) in photocraft_ui_egui::layer_tree_ui::display_rows(&doc, false) {
                    let top = ui.cursor().top();
                    if let Some(drop) =
                        self.layer_row(app, ui, &doc, layer, depth, &selected, active)
                    {
                        drop_target = Some(drop);
                    }
                    if reveal == Some(layer.id) {
                        layer_reveal::scroll_to_row(ui, top);
                    }
                }
            });
        let touch_cancelled = ui.input(|i| {
            i.raw.events.iter().any(|event| {
                matches!(
                    event,
                    egui::Event::Touch {
                        phase: egui::TouchPhase::Cancel,
                        ..
                    }
                )
            })
        });
        let pointer = ui.input(|i| i.pointer.interact_pos());
        let dragging = self.arrange_layers
            && self.layer_arrange_drag.is_some()
            && !touch_cancelled
            && ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
        let visible_stack = scroll.inner_rect.intersect(ui.clip_rect());
        let edge_band = (visible_stack.height() / 3.).min(36.);
        let edge = pointer.and_then(|pointer| {
            if !dragging || !visible_stack.contains(pointer) || edge_band <= 0. {
                return None;
            }
            let from_top = pointer.y - visible_stack.top();
            if from_top < edge_band {
                Some((-1., (1. - from_top / edge_band).clamp(0., 1.)))
            } else {
                let from_bottom = visible_stack.bottom() - pointer.y;
                (from_bottom < edge_band)
                    .then_some((1., (1. - from_bottom / edge_band).clamp(0., 1.)))
            }
        });
        if let Some((direction, proximity)) = edge {
            let now = ui.input(|i| i.time);
            let dt = self
                .layer_autoscroll_at
                .map_or(1. / 60., |previous| (now - previous).clamp(0., 0.05));
            let speed = 80. + 520. * proximity;
            let max_offset = (scroll.content_size.y - scroll.inner_rect.height()).max(0.);
            let mut state = scroll.state;
            let previous_offset = state.offset.y;
            state.offset.y =
                (previous_offset + direction * speed * dt as f32).clamp(0., max_offset);
            if state.offset.y != previous_offset {
                state.store(ui.ctx(), scroll.id);
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(16));
                self.layer_autoscroll_at = Some(now);
            } else {
                self.layer_autoscroll_at = None;
            }
        } else {
            self.layer_autoscroll_at = None;
        }
        if touch_cancelled {
            self.cancel_layer_arrange_drag();
        } else if ui.input(|i| i.pointer.any_released()) {
            if let (Some(drag), Some((target, position))) =
                (self.layer_arrange_drag.clone(), drop_target)
                && drag.doc_id == doc.id.0
                && drag.source != target.0
            {
                if drag.layers.len() > 1 {
                    self.invoke(
                        app,
                        ui.ctx(),
                        "layer.moveTo",
                        json!({"layers":drag.layers,"target":target,"position":position}),
                    );
                } else {
                    let dragged = drag.source;
                    let preserve_target = active == Some(LayerId(dragged));
                    let revision = app.session.active().map(|st| st.revision);
                    self.invoke(
                        app,
                        ui.ctx(),
                        "layer.moveTo",
                        json!({"layer":dragged,"target":target,"position":position}),
                    );
                    let changed = app.session.active().map(|st| st.revision) != revision;
                    if self.message.is_empty() && changed && !preserve_target {
                        self.target_layer(app, ui.ctx(), LayerId(dragged), LayerTarget::Image);
                    }
                }
            }
            self.cancel_layer_arrange_drag();
        }
        ui.horizontal(|ui| {
            for (icon, label, cmd) in [
                ("plus", "New layer", "layer.new.layer"),
                ("folder-plus", "New group", "layer.new.group"),
            ] {
                if navigation::icon_button(ui, icon, label, false, 44.).clicked() {
                    self.invoke(app, ui.ctx(), cmd, json!({}));
                }
            }
            let active_layer = active.and_then(|id| doc.layer(id));
            let has_mask =
                active_layer.is_some_and(|l| l.mask.is_some() || l.vector_mask.is_some());
            if ui
                .add_enabled_ui(active.is_some(), |ui| {
                    navigation::icon_button(
                        ui,
                        "scan",
                        if has_mask {
                            "Mask controls"
                        } else {
                            "Add layer mask"
                        },
                        false,
                        44.,
                    )
                })
                .inner
                .clicked()
            {
                if has_mask {
                    let target = if active_layer.is_some_and(|l| l.vector_mask.is_some())
                        && (app.ui.vector_mask_target
                            || active_layer.is_some_and(|l| l.mask.is_none()))
                    {
                        LayerTarget::Vector
                    } else {
                        LayerTarget::Mask
                    };
                    if let Some(id) = active {
                        self.open_mask_controls(app, ui.ctx(), id, target);
                    }
                } else if let Some(id) = active {
                    self.target_layer(app, ui.ctx(), id, LayerTarget::Image);
                    if self.message.is_empty() {
                        // Finishing Quick Mask just restored its edited selection; the
                        // document snapshot from the start of this frame is now stale.
                        let selected = app
                            .session
                            .active()
                            .is_some_and(|st| st.doc.selection.is_some());
                        self.invoke(
                            app,
                            ui.ctx(),
                            layer_menu_ui::add_mask_command(selected, false),
                            json!({"layer":id.0}),
                        );
                        if self.message.is_empty() {
                            self.target_layer(app, ui.ctx(), id, LayerTarget::Mask);
                        }
                    }
                }
            }
            if navigation::icon_button(ui, "ellipsis", "Layer properties and actions", false, 44.)
                .clicked()
            {
                self.open_layer_properties();
            }
            if !spacious {
                ui.label(egui::RichText::new(caption).small());
            }
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn layer_row(
        &mut self,
        app: &mut PhotocraftApp,
        ui: &mut Ui,
        doc: &Document,
        layer: &Layer,
        depth: usize,
        selected: &[LayerId],
        active: Option<LayerId>,
    ) -> Option<(LayerId, &'static str)> {
        let t = Tokens::get(ui.ctx());
        let mut drop_target = None;
        let group = matches!(layer.content, LayerContent::Group(_));
        ui.push_id(layer.id.0, |ui| {
            let row = Frame::NONE
                .fill(if selected.contains(&layer.id) {
                    t.row_selected
                } else {
                    t.dock
                })
                .inner_margin(egui::Margin::symmetric(2, 1))
                .show(ui, |ui| {
                    ui.set_min_height(50.);
                    ui.spacing_mut().item_spacing.x = 2.;
                    ui.spacing_mut().item_spacing.y = 0.;
                    let indent = if self.arrange_layers {
                        (depth as f32 * 4.).min(8.)
                    } else {
                        (depth as f32 * 10.).min(20.)
                    };
                    let targets = 1
                        + usize::from(layer.mask.is_some())
                        + usize::from(layer.vector_mask.is_some());
                    // Keep the name readable before adding more fixed-width targets.
                    // Width, not selection or name length, determines the row geometry.
                    let grip_width = if self.arrange_layers { 44. } else { 0. };
                    let fixed =
                        48. + grip_width + indent + 46. * (targets + usize::from(group)) as f32;
                    let stacked = ui.available_width() - fixed < 88.;
                    ui.horizontal(|ui| {
                        if self.arrange_layers {
                            let (grip_rect, _) =
                                ui.allocate_exact_size(vec2(44., 48.), egui::Sense::hover());
                            let grip = ui.interact(
                                grip_rect,
                                egui::Id::new(("ipad-layer-arrange-grip", doc.id.0, layer.id.0)),
                                egui::Sense::drag(),
                            );
                            icons::paint(ui, grip_rect.shrink(11.), "move", 22., t.text);
                            grip.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::WidgetType::Button,
                                    true,
                                    format!("Reorder: {}", layer.name),
                                )
                            });
                            let started = grip.drag_started();
                            let selected_drag = selected.contains(&layer.id);
                            let tooltip = if selected_drag && selected.len() > 1 {
                                format!("Move {} selected layers", selected.len())
                            } else {
                                format!("Reorder: {}", layer.name)
                            };
                            let _ = grip.on_hover_text(tooltip);
                            if started {
                                self.layer_arrange_drag = Some(LayerArrangeDrag {
                                    doc_id: doc.id.0,
                                    source: layer.id.0,
                                    layers: if selected_drag {
                                        selected.iter().map(|id| id.0).collect()
                                    } else {
                                        vec![layer.id.0]
                                    },
                                });
                            }
                        }
                        if navigation::icon_button(
                            ui,
                            if layer.visible { "eye" } else { "eye-off" },
                            &format!("Visibility: {}", layer.name),
                            false,
                            44.,
                        )
                        .clicked()
                        {
                            self.invoke(
                                app,
                                ui.ctx(),
                                "layer.setProps",
                                json!({"layer":layer.id.0,"visible":!layer.visible}),
                            );
                        }
                        ui.add_space(indent);
                        if let LayerContent::Group(group) = &layer.content
                            && navigation::icon_button(
                                ui,
                                if group.expanded {
                                    "chevron-down"
                                } else {
                                    "chevron-right"
                                },
                                &format!("Expand group: {}", layer.name),
                                false,
                                44.,
                            )
                            .clicked()
                        {
                            self.invoke(
                                app,
                                ui.ctx(),
                                "layer.setExpanded",
                                json!({"layer":layer.id.0,"expanded":!group.expanded}),
                            );
                        }
                        if !stacked {
                            self.layer_thumbnails(app, ui, doc, layer, active);
                        }
                        let suffix = if layer.locks.all {
                            " · locked"
                        } else if layer.clipped {
                            "  ↳"
                        } else {
                            ""
                        };
                        let (name_rect, _) = ui.allocate_exact_size(
                            vec2(ui.available_width().max(20.), 48.),
                            egui::Sense::hover(),
                        );
                        let response = ui.interact(
                            name_rect,
                            egui::Id::new(("ipad-layer-name", doc.id.0, layer.id.0)),
                            egui::Sense::click(),
                        );
                        if ui.is_rect_visible(name_rect) {
                            let galley = egui::WidgetText::from(format!("{}{suffix}", layer.name))
                                .into_galley(
                                    ui,
                                    Some(egui::TextWrapMode::Truncate),
                                    name_rect.width() - 4.,
                                    egui::TextStyle::Button,
                                );
                            let pos = egui::pos2(
                                name_rect.left() + 2.,
                                name_rect.center().y - galley.size().y / 2.,
                            );
                            ui.painter()
                                .with_clip_rect(ui.clip_rect().intersect(name_rect))
                                .galley(pos, galley, t.text);
                            if response.has_focus() {
                                ui.painter().rect_stroke(
                                    name_rect.shrink(1.),
                                    2.,
                                    ui.visuals().selection.stroke,
                                    egui::StrokeKind::Inside,
                                );
                            }
                        }
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Button,
                                true,
                                format!("Select layer: {}", layer.name),
                            )
                        });
                        if response
                            .on_hover_text(format!(
                                "{} · {}",
                                layer.name,
                                layer.content.kind_name()
                            ))
                            .clicked()
                        {
                            if self.multi_select {
                                self.invoke(
                                    app,
                                    ui.ctx(),
                                    "layer.select",
                                    json!({"layer":layer.id.0,"mode":"toggle"}),
                                );
                            } else {
                                self.target_layer(app, ui.ctx(), layer.id, LayerTarget::Image);
                            }
                        }
                    });
                    if stacked {
                        ui.horizontal(|ui| {
                            ui.add_space(46. + indent);
                            self.layer_thumbnails(app, ui, doc, layer, active);
                        });
                    }
                });
            let rect = row.response.rect;
            if self.arrange_layers
                && let Some(drag) = self.layer_arrange_drag.as_ref()
                && drag.doc_id == doc.id.0
                && drag.source != layer.id.0
                && !drag.layers.contains(&layer.id.0)
                && let Some(pointer) = ui.input(|i| i.pointer.interact_pos())
                && rect.intersect(ui.clip_rect()).contains(pointer)
            {
                let fraction = (pointer.y - rect.top()) / rect.height().max(1.);
                let position = if group && (0.3..0.7).contains(&fraction) {
                    "into"
                } else if fraction < 0.5 {
                    "above"
                } else {
                    "below"
                };
                let painter = ui.painter();
                match position {
                    "into" => painter.rect_stroke(
                        rect.shrink(1.),
                        3.,
                        egui::Stroke::new(2., t.accent),
                        egui::StrokeKind::Inside,
                    ),
                    "above" => painter.line_segment(
                        [rect.left_top(), rect.right_top()],
                        egui::Stroke::new(2., t.accent),
                    ),
                    _ => painter.line_segment(
                        [rect.left_bottom(), rect.right_bottom()],
                        egui::Stroke::new(2., t.accent),
                    ),
                };
                drop_target = Some((layer.id, position));
            }
        });
        drop_target
    }

    fn layer_thumbnails(
        &mut self,
        app: &mut PhotocraftApp,
        ui: &mut Ui,
        doc: &Document,
        layer: &Layer,
        active: Option<LayerId>,
    ) {
        let is_active = active == Some(layer.id);
        let paint = canvas::paint_target(app);
        for (target, present, selected) in [
            (
                LayerTarget::Image,
                true,
                is_active && paint == json!("pixels") && !app.ui.vector_mask_target,
            ),
            (
                LayerTarget::Mask,
                layer.mask.is_some(),
                is_active && paint == json!("mask"),
            ),
            (
                LayerTarget::Vector,
                layer.vector_mask.is_some(),
                is_active && app.ui.vector_mask_target,
            ),
        ] {
            if present && thumbnail(app, ui, doc, layer, target, selected).clicked() {
                self.tap_layer_target(app, ui.ctx(), layer.id, target);
            }
        }
    }

    fn layer_blending(
        &mut self,
        app: &mut PhotocraftApp,
        ui: &mut Ui,
        doc: &Document,
        layer: &Layer,
    ) {
        ui.add_enabled_ui(!photocraft_ui_egui::doc_props_ui::is_background(doc, layer), |ui| {
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("ipad-layer-blend").selected_text(layer.blend.label()).width(110.).show_ui(ui, |ui| {
                    for mode in photocraft_color::BlendMode::LAYER_MODES {
                        if full_button(ui, mode.label(), layer.blend == mode).clicked() { self.invoke(app,ui.ctx(),"layer.setProps",json!({"layer":layer.id.0,"blend":mode})); }
                    }
                });
                let mut opacity = layer.opacity * 100.;
                ui.spacing_mut().slider_width = 55.;
                if ui.add(egui::Slider::new(&mut opacity, 0.0..=100.).suffix("%").show_value(true)).on_hover_text("Layer opacity").changed() {
                    self.invoke(app,ui.ctx(),"layer.setProps",json!({"layer":layer.id.0,"opacity":opacity/100.,"coalesce":format!("ipad-opacity:{}:{}:{}",doc.id.0,layer.id.0,self.edit_gesture)}));
                    if !ui.input(|i| i.pointer.any_down()) { self.edit_gesture = self.edit_gesture.wrapping_add(1); }
                }
            });
        });
    }

    pub(super) fn layer_properties_panel(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) -> bool {
        let Some(st) = app.session.active() else {
            self.layer_properties = false;
            return false;
        };
        let doc = st.doc.clone();
        let Some(layer) = st.active_layer.and_then(|id| doc.layer(id)) else {
            self.layer_properties = false;
            return false;
        };
        let selected_count = st.selected_layers().len();
        let paint_target = canvas::paint_target(app);
        let paint_caption = if paint_target == json!("quickMask") {
            "Quick Mask".to_string()
        } else if paint_target.get("channel").is_some() {
            "Alpha channel".to_string()
        } else if app.ui.vector_mask_target {
            "Vector path · brushes paint image".to_string()
        } else if paint_target == json!("mask") {
            "Paint: pixel mask".to_string()
        } else {
            "Paint: layer image".to_string()
        };
        let mut scope = format!("{} · {paint_caption}", layer.content.kind_name());
        if selected_count > 1 {
            scope.push_str(&format!(
                " · editing active layer · {selected_count} selected"
            ));
        }
        ui.horizontal(|ui| {
            let back = button(ui, "Back", false);
            back.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Back to layers")
            });
            if back.clicked() {
                self.layer_properties = false;
            }
            let header = ui
                .add(egui::Label::new(&layer.name).truncate())
                .on_hover_text(&layer.name);
            header.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Label,
                    true,
                    format!("{} · {}", layer.name, layer.content.kind_name()),
                )
            });
        });
        ui.label(egui::RichText::new(scope).small());
        ui.separator();
        let body_height = ui.available_height().max(1.);
        egui::ScrollArea::vertical()
            .id_salt(("ipad-layer-properties", doc.id.0, layer.id.0))
            .scroll_source(egui::scroll_area::ScrollSource::ALL)
            .max_height(body_height)
            .min_scrolled_height(body_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.layer_properties_body(app, ui, &doc, layer, selected_count)
            });
        true
    }

    fn layer_properties_body(
        &mut self,
        app: &mut PhotocraftApp,
        ui: &mut Ui,
        doc: &Document,
        layer: &Layer,
        selected_count: usize,
    ) {
        let key = (doc.id.0, layer.id.0);
        if self.rename_layer != Some(key) || self.layer_name_source != layer.name {
            self.rename_layer = Some(key);
            self.layer_name = layer.name.clone();
            self.layer_name_source = layer.name.clone();
        }
        let background = photocraft_ui_egui::doc_props_ui::is_background(doc, layer);
        if background {
            self.command(
                app,
                ui,
                "Unlock background",
                "layer.new.layerFromBackground",
            );
        }
        ui.strong("Active layer properties");
        ui.add_enabled_ui(!background, |ui| {
            ui.horizontal(|ui| {
                let label = ui.label("Layer name");
                let field = ui.add_sized(
                    [ui.available_width() - 84., 44.],
                    egui::TextEdit::singleline(&mut self.layer_name).hint_text("Layer name"),
                );
                field.labelled_by(label.id);
                if button(ui, "Rename", false).clicked() {
                    self.invoke(
                        app,
                        ui.ctx(),
                        "layer.setProps",
                        json!({"layer":layer.id.0,"name":self.layer_name.trim()}),
                    );
                }
            });
        });
        self.layer_properties_blending(app, ui, doc, layer, background);
        if ui
            .add_sized(
                [ui.available_width(), 44.],
                Button::new(if layer.visible {
                    "Visibility · Visible"
                } else {
                    "Visibility · Hidden"
                })
                .selected(layer.visible),
            )
            .on_hover_text("Toggle active layer visibility")
            .clicked()
        {
            self.invoke(
                app,
                ui.ctx(),
                "layer.setProps",
                json!({"layer":layer.id.0,"visible":!layer.visible}),
            );
        }
        ui.label("Lock");
        let width = (ui.available_width() - 6.) / 2.;
        for row in [
            [
                ("All", "Lock layer", "all", layer.locks.all),
                ("Pixels", "Lock pixels", "pixels", layer.locks.pixels),
            ],
            [
                (
                    "Position",
                    "Lock position",
                    "position",
                    layer.locks.position,
                ),
                (
                    "Transparency",
                    "Lock transparency",
                    "transparency",
                    layer.locks.transparency,
                ),
            ],
        ] {
            ui.horizontal(|ui| {
                for (short, accessible, key, value) in row {
                    let response = ui.add_enabled(
                        !background,
                        Button::new(short)
                            .selected(value)
                            .min_size(vec2(width, 44.)),
                    );
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::WidgetType::Button,
                            !background,
                            value,
                            accessible,
                        )
                    });
                    if response.clicked() {
                        self.invoke(
                            app,
                            ui.ctx(),
                            "layer.setProps",
                            json!({"layer":layer.id.0,"locks":{key:!value}}),
                        );
                    }
                }
            });
        }
        ui.separator();
        ui.strong("Pixel selection · active layer");
        self.selection_actions(app, ui);
        if doc.quick_mask.is_some() {
            self.command(app, ui, "Exit Quick Mask", "select.editInQuickMaskMode");
        }
        ui.separator();
        ui.strong("Active layer actions");
        ui.horizontal_wrapped(|ui| {
            for (label, id) in [
                ("Effects…", "layer.layerStyle.blendingOptions"),
                ("Move up", "layer.arrange.bringForward"),
                ("Move down", "layer.arrange.sendBackward"),
            ] {
                if ui
                    .add_enabled(
                        menus::is_enabled(app, id),
                        Button::new(label).min_size(vec2(100., 44.)),
                    )
                    .clicked()
                {
                    self.invoke(app, ui.ctx(), id, json!({}));
                }
            }
            if matches!(
                layer.content,
                LayerContent::Adjustment(_) | LayerContent::Fill(_)
            ) {
                let id = "layer.layerContentOptions";
                if ui
                    .add_enabled(
                        menus::is_enabled(app, id),
                        Button::new("Edit contents…").min_size(vec2(100., 44.)),
                    )
                    .clicked()
                {
                    self.invoke(app, ui.ctx(), id, json!({}));
                }
            }
        });
        ui.separator();
        let selected_actions = if selected_count > 1 {
            format!("Selected layers · {selected_count}")
        } else {
            "Selected layer".to_string()
        };
        ui.strong(selected_actions);
        ui.horizontal_wrapped(|ui| {
            let duplicate = if selected_count > 1 {
                format!("Duplicate {selected_count} layers")
            } else {
                "Duplicate".to_string()
            };
            if ui
                .add_enabled(
                    menus::is_enabled(app, "layer.duplicate"),
                    Button::new(duplicate).min_size(vec2(100., 44.)),
                )
                .clicked()
            {
                self.invoke(app, ui.ctx(), "layer.duplicate", json!({}));
            }
            let delete = if selected_count > 1 {
                format!("Delete {selected_count} layers")
            } else {
                "Delete".to_string()
            };
            if ui
                .add_enabled(
                    menus::is_enabled(app, "layer.delete"),
                    Button::new(delete).min_size(vec2(100., 44.)),
                )
                .clicked()
            {
                self.invoke(app, ui.ctx(), "layer.delete", json!({}));
            }
        });
        if layer.mask.is_some() || layer.vector_mask.is_some() {
            ui.separator();
            if full_button(ui, "Mask controls…", false).clicked() {
                let target = if app.ui.vector_mask_target && layer.vector_mask.is_some()
                    || layer.mask.is_none()
                {
                    LayerTarget::Vector
                } else {
                    LayerTarget::Mask
                };
                self.open_mask_controls(app, ui.ctx(), layer.id, target);
            }
        }
        if full_button(ui, "All layer commands…", false).clicked() {
            self.open_sheet(Sheet::Commands);
            self.command_path = vec!["Layer".into()];
        }
    }

    fn layer_properties_blending(
        &mut self,
        app: &mut PhotocraftApp,
        ui: &mut Ui,
        doc: &Document,
        layer: &Layer,
        background: bool,
    ) {
        ui.add_enabled_ui(!background, |ui| {
            egui::ComboBox::from_id_salt(("ipad-layer-properties-blend", doc.id.0, layer.id.0))
                .selected_text(format!("Blend · {}", layer.blend.label()))
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    for mode in photocraft_color::BlendMode::LAYER_MODES {
                        if full_button(ui, mode.label(), layer.blend == mode).clicked() {
                            self.invoke(
                                app,
                                ui.ctx(),
                                "layer.setProps",
                                json!({"layer":layer.id.0,"blend":mode}),
                            );
                        }
                    }
            });
            let mut opacity = layer.opacity * 100.;
            ui.horizontal(|ui| {
                let label = ui.label("Layer opacity");
                ui.spacing_mut().slider_width = (ui.available_width() - 54.).max(44.);
                if ui
                    .add(egui::Slider::new(&mut opacity, 0.0..=100.).suffix("%"))
                    .labelled_by(label.id)
                    .on_hover_text("Layer opacity")
                    .changed()
                {
                    self.invoke(
                        app,
                        ui.ctx(),
                        "layer.setProps",
                        json!({"layer":layer.id.0,"opacity":opacity/100.,"coalesce":format!("ipad-opacity:{}:{}:{}",doc.id.0,layer.id.0,self.edit_gesture)}),
                    );
                    if !ui.input(|i| i.pointer.any_down()) {
                        self.edit_gesture = self.edit_gesture.wrapping_add(1);
                    }
                }
            });
        });
    }
}

fn thumbnail(
    app: &mut PhotocraftApp,
    ui: &mut Ui,
    doc: &Document,
    layer: &Layer,
    target: LayerTarget,
    selected: bool,
) -> egui::Response {
    let (hit, _) = ui.allocate_exact_size(vec2(44., 48.), egui::Sense::hover());
    let rect = egui::Rect::from_center_size(hit.center(), vec2(34., 34.));
    let t = Tokens::get(ui.ctx());
    let label = match target {
        LayerTarget::Image => "Image",
        LayerTarget::Mask => "Mask",
        LayerTarget::Vector => "Vector mask",
    };
    // A width change can move thumbnails to another line while a pointer is down.
    // Their identities must continue to describe targets, never layout positions.
    let response = ui.interact(
        hit,
        egui::Id::new(("ipad-layer-target", doc.id.0, layer.id.0, label)),
        egui::Sense::click(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            true,
            format!("{label}: {}", layer.name),
        )
    });
    if ui.is_rect_visible(hit) {
        widgets::checker(ui.painter(), rect, 5.);
        let tex = match target {
            LayerTarget::Mask => layer
                .mask
                .as_ref()
                .map(|mask| app.mask_thumb(ui.ctx(), doc, layer.id, mask)),
            LayerTarget::Vector => layer
                .vector_mask
                .as_ref()
                .map(|mask| app.vector_mask_thumb(ui.ctx(), doc, layer.id, mask)),
            LayerTarget::Image => match &layer.content {
                LayerContent::Group(_) | LayerContent::Adjustment(_) | LayerContent::Text(_) => {
                    ui.painter().rect_filled(rect, 3., t.field);
                    icons::paint(
                        ui,
                        rect,
                        match layer.content {
                            LayerContent::Group(_) => "folder",
                            LayerContent::Text(_) => "type",
                            _ => "contrast",
                        },
                        22.,
                        t.text,
                    );
                    None
                }
                LayerContent::Fill(f) => {
                    if !photocraft_ui_egui::gradient_ui::paint_thumbnail(ui, layer.id, f, rect)
                        && let photocraft_doc::Fill::Solid(c) = f
                    {
                        let c = c.to_rgba8();
                        ui.painter().rect_filled(
                            rect,
                            3.,
                            egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]),
                        );
                    }
                    None
                }
                _ => Some(app.layer_thumb(ui.ctx(), doc, layer)),
            },
        };
        if let Some(tex) = tex {
            ui.painter().image(
                tex,
                rect,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
                egui::Color32::WHITE,
            );
        }
        ui.painter().rect_stroke(
            rect,
            3.,
            egui::Stroke::new(1., t.field_border),
            egui::StrokeKind::Outside,
        );
        if selected {
            mask_thumbs_ui::paint_brackets(ui.painter(), rect.expand(2.), t.text);
        }
        let disabled = match target {
            LayerTarget::Mask => layer.mask.as_ref().is_some_and(|m| !m.enabled),
            LayerTarget::Vector => layer.vector_mask.as_ref().is_some_and(|m| !m.enabled),
            LayerTarget::Image => false,
        };
        if disabled {
            for points in [
                [rect.left_top(), rect.right_bottom()],
                [rect.right_top(), rect.left_bottom()],
            ] {
                ui.painter()
                    .line_segment(points, egui::Stroke::new(2., t.danger));
            }
        }
    }
    response.on_hover_text(format!("{label}: {}", layer.name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };

    const NAME: &str = "Foreground restoration with a very long descriptive name";

    fn name_is_painted(shape: &egui::Shape, bounds: egui::Rect) -> bool {
        match shape {
            egui::Shape::Text(text) if text.galley.job.text.contains(NAME) => {
                let overlap = text.visual_bounding_rect().intersect(bounds);
                overlap.width() > 20. && overlap.height() > 5.
            }
            egui::Shape::Vec(shapes) => shapes.iter().any(|shape| name_is_painted(shape, bounds)),
            _ => false,
        }
    }

    fn fixture(group: bool, masks: usize) -> PhotocraftApp {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        app.run(
            "file.new",
            json!({"width":32,"height":32,"background":"transparent"}),
        )
        .unwrap();
        if group {
            app.run("layer.new.group", json!({"name":NAME})).unwrap();
        } else {
            app.run("layer.setProps", json!({"name":NAME})).unwrap();
        }
        if masks >= 1 {
            app.run("layer.layerMask.revealAll", json!({})).unwrap();
        }
        if masks >= 2 {
            app.run("layer.vectorMask.revealAll", json!({})).unwrap();
        }
        app
    }

    fn arrange_harness(app: PhotocraftApp) -> Harness<'static, (PhotocraftApp, TabletUi)> {
        let selected_count = app
            .session
            .active()
            .map_or(0, |st| st.selected_layers().len());
        let arrange_label = if selected_count > 1 {
            format!("Arrange layers · move {selected_count} selected layers")
        } else {
            "Arrange layers".to_string()
        };
        let mut h = Harness::builder()
            .with_size(vec2(1194., 834.))
            .build_ui_state(
                |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
                (app, TabletUi::default()),
            );
        h.run_steps(3);
        h.get_by_label(&arrange_label).click();
        h.run_steps(3);
        h
    }

    fn properties_harness(
        app: PhotocraftApp,
        size: egui::Vec2,
    ) -> Harness<'static, (PhotocraftApp, TabletUi)> {
        let mut h = Harness::builder().with_size(size).build_ui_state(
            |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
            (
                app,
                TabletUi {
                    layer_properties: true,
                    ..Default::default()
                },
            ),
        );
        h.run_steps(3);
        h
    }

    #[test]
    fn docked_properties_open_and_back_preserve_document_selection_and_paint_target() {
        for (mask, vector, expected_caption) in [
            (false, false, "Pixel · Paint: layer image"),
            (true, false, "Pixel · Paint: pixel mask"),
            (false, true, "Pixel · Vector path · brushes paint image"),
        ] {
            let mut app = fixture(false, 2);
            app.ui.mask_target = mask;
            app.ui.vector_mask_target = vector;
            let st = app.session.active().unwrap();
            let before_doc = st.doc.clone();
            let before_active = st.active_layer;
            let before_selected = st.selected_layers();
            let before_history = st.history.past_len();
            let mut h = properties_harness(app, vec2(507., 768.));

            assert!(h.get_by_label("Back to layers").rect().height() >= 44.);
            assert!(h.query_by_label(expected_caption).is_some());
            assert_eq!(
                h.state().0.session.active().unwrap().doc,
                before_doc,
                "opening properties edited the document"
            );
            h.get_by_label("Back to layers").click();
            h.run_steps(3);

            let st = h.state().0.session.active().unwrap();
            assert!(!h.state().1.layer_properties);
            assert_eq!(st.doc, before_doc);
            assert_eq!(st.active_layer, before_active);
            assert_eq!(st.selected_layers(), before_selected);
            assert_eq!(st.history.past_len(), before_history);
            assert_eq!(h.state().0.ui.mask_target, mask);
            assert_eq!(h.state().0.ui.vector_mask_target, vector);
        }
    }

    #[test]
    fn docked_properties_keep_alpha_and_quick_mask_targets_unchanged() {
        for (command, params, caption) in [
            ("channel.new", json!({}), "Pixel · Alpha channel"),
            (
                "select.editInQuickMaskMode",
                json!({"on":true}),
                "Pixel · Quick Mask",
            ),
        ] {
            let mut app = fixture(false, 0);
            app.run(command, params).unwrap();
            let st = app.session.active().unwrap();
            let before_doc = st.doc.clone();
            let before_history = st.history.past_len();
            let before_active = st.active_layer;
            let before_selected = st.selected_layers();
            let before_target = canvas::paint_target(&app);
            let before_mask_target = app.ui.mask_target;
            let before_vector_target = app.ui.vector_mask_target;
            let mut h = properties_harness(app, vec2(507., 768.));

            assert!(h.query_by_label(caption).is_some());
            assert_eq!(canvas::paint_target(&h.state().0), before_target);
            h.get_by_label("Back to layers").click();
            h.run_steps(3);

            let st = h.state().0.session.active().unwrap();
            assert_eq!(st.doc, before_doc);
            assert_eq!(st.history.past_len(), before_history);
            assert_eq!(st.active_layer, before_active);
            assert_eq!(st.selected_layers(), before_selected);
            assert_eq!(canvas::paint_target(&h.state().0), before_target);
            assert_eq!(h.state().0.ui.mask_target, before_mask_target);
            assert_eq!(h.state().0.ui.vector_mask_target, before_vector_target);
        }
    }

    #[test]
    fn docked_properties_rename_is_one_undoable_active_layer_edit() {
        let app = fixture(false, 0);
        let layer = app.session.active().unwrap().active_layer.unwrap();
        let previous = app
            .session
            .active()
            .unwrap()
            .doc
            .layer(layer)
            .unwrap()
            .name
            .clone();
        let before_steps = app.session.active().unwrap().history.past_len();
        let mut h = properties_harness(app, vec2(1194., 834.));
        h.state_mut().1.layer_name = "Renamed from properties".into();
        h.run_steps(1);
        h.get_by_label("Rename").click();
        h.run_steps(3);

        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.doc.layer(layer).unwrap().name, "Renamed from properties");
        assert_eq!(st.history.past_len(), before_steps + 1);
        h.state_mut().0.run("edit.undo", json!({})).unwrap();
        assert_eq!(
            h.state()
                .0
                .session
                .active()
                .unwrap()
                .doc
                .layer(layer)
                .unwrap()
                .name,
            previous
        );
    }

    #[test]
    fn docked_opacity_drag_coalesces_into_one_undo_step() {
        let app = fixture(false, 0);
        let layer = app.session.active().unwrap().active_layer.unwrap();
        let initial_opacity = app
            .session
            .active()
            .unwrap()
            .doc
            .layer(layer)
            .unwrap()
            .opacity;
        let before_steps = app.session.active().unwrap().history.past_len();
        let mut h = properties_harness(app, vec2(1194., 834.));
        {
            let opacity = h.get_by_role_and_label(egui::accesskit::Role::Slider, "Layer opacity");
            opacity.scroll_to_me();
        }
        h.run_steps(3);
        let slider = h
            .get_by_role_and_label(egui::accesskit::Role::Slider, "Layer opacity")
            .rect();
        let start = egui::pos2(slider.right() - 8., slider.center().y);
        let end = egui::pos2(slider.left() + slider.width() * 0.45, slider.center().y);
        h.hover_at(start);
        h.run_steps(1);
        h.drag_at(start);
        h.run_steps(1);
        h.hover_at(end);
        h.run_steps(2);
        h.drop_at(end);
        h.run_steps(3);

        let st = h.state().0.session.active().unwrap();
        let changed_opacity = st.doc.layer(layer).unwrap().opacity;
        assert_ne!(
            changed_opacity, initial_opacity,
            "slider drag did not change opacity"
        );
        assert_eq!(st.history.past_len(), before_steps + 1);
        h.state_mut().0.run("edit.undo", json!({})).unwrap();
        assert_eq!(
            h.state()
                .0
                .session
                .active()
                .unwrap()
                .doc
                .layer(layer)
                .unwrap()
                .opacity,
            initial_opacity
        );
    }

    #[test]
    fn portrait_properties_keep_navigation_pinned_and_common_controls_full_width() {
        let app = many_layer_fixture(8);
        let mut h = properties_harness(app, vec2(507., 768.));
        let back_before = h.get_by_label("Back to layers").rect();
        let visibility = h.get_by_label("Visibility · Visible").rect();
        let lock = h.get_by_label("Lock layer").rect();
        assert!(back_before.height() >= 44.);
        assert!(visibility.height() >= 44.);
        assert!(visibility.width() > lock.width() * 1.8);
        assert!(visibility.left() >= 0. && visibility.right() <= 507.);

        h.get_by_label("All layer commands…").scroll_to_me();
        h.run_steps(3);
        assert_eq!(h.get_by_label("Back to layers").rect(), back_before);
    }

    #[test]
    fn background_properties_keep_visibility_available_while_other_properties_stay_locked() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        app.run("file.new", json!({"width":32,"height":32}))
            .unwrap();
        let background = app.session.active().unwrap().active_layer.unwrap();
        let before_steps = app.session.active().unwrap().history.past_len();
        let mut h = properties_harness(app, vec2(1194., 834.));

        h.get_by_label("Rename").click();
        h.get_by_label("Lock layer").click();
        h.run_steps(2);
        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.doc.layer(background).unwrap().name, "Background");
        assert_eq!(st.history.past_len(), before_steps);

        h.get_by_label("Visibility · Visible").click();
        h.run_steps(3);
        let st = h.state().0.session.active().unwrap();
        assert!(!st.doc.layer(background).unwrap().visible);
        assert_eq!(st.history.past_len(), before_steps + 1);
    }

    fn drag_to(h: &mut Harness<'static, (PhotocraftApp, TabletUi)>, source: &str, target: &str) {
        let from = h
            .get_by_label(&format!("Reorder: {source}"))
            .rect()
            .center();
        let to = h.get_by_label(&format!("Select layer: {target}")).rect();
        h.hover_at(from);
        h.drag_at(from);
        h.run_steps(1);
        let drop = to.center();
        h.hover_at(drop);
        h.run_steps(1);
        h.drop_at(drop);
        h.run_steps(4);
    }

    fn many_layer_fixture(count: usize) -> PhotocraftApp {
        let mut app = fixture(false, 0);
        for i in 0..count {
            app.run(
                "layer.new.layer",
                json!({"name":format!("Scroll layer {i:02}")}),
            )
            .unwrap();
        }
        app
    }

    #[test]
    fn arrange_into_group_uses_one_move_command_and_undo_restores_hierarchy() {
        let mut app = fixture(false, 1);
        let session = &mut app.session;
        let background = session.active().unwrap().active_layer.unwrap();
        app.run("layer.setProps", json!({"name":"A"})).unwrap();
        let a = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.new.layer", json!({"name":"B"})).unwrap();
        let b = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.new.group", json!({"name":"Group"})).unwrap();
        let group = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.select", json!({"layer":b.0,"mode":"replace"}))
            .unwrap();
        let selected = app.session.active().unwrap().selected_layers();
        assert_eq!(selected, vec![b]);
        let before_steps = app.session.active().unwrap().history.past_len();
        let mut h = arrange_harness(app);

        drag_to(&mut h, "B", "Group");

        let st = h.state().0.session.active().unwrap();
        assert!(st.doc.layers.iter().all(|layer| layer.id != b));
        assert_eq!(st.doc.layer(group).unwrap().children().unwrap()[0].id, b);
        assert_eq!(st.active_layer, Some(b));
        assert_eq!(st.selected_layers(), vec![b]);
        assert_eq!(st.doc.layer(a).unwrap().name, "A");
        assert_eq!(st.doc.layers[0].id, background);
        assert_eq!(st.history.past_len(), before_steps + 1);

        h.state_mut().0.run("edit.undo", json!({})).unwrap();
        let st = h.state().0.session.active().unwrap();
        assert!(st.doc.layer(b).is_some());
        assert!(st.doc.layer(group).unwrap().children().unwrap().is_empty());
    }

    #[test]
    fn held_edge_drag_scrolls_to_an_offscreen_layer_then_drop_and_undo_work() {
        let app = many_layer_fixture(30);
        let before = app.session.active().unwrap().doc.clone();
        let source = app.session.active().unwrap().active_layer.unwrap();
        let mut h = arrange_harness(app);
        let footer = h.get_by_label("New layer").rect();
        let stack_top = h.get_by_label("Reorder: Scroll layer 29").rect().top();
        let hidden_before = h.get_by_label("Reorder: Scroll layer 00").rect();
        assert!(hidden_before.top() >= footer.bottom());

        let grip = h.get_by_label("Reorder: Scroll layer 29").rect().center();
        h.hover_at(grip);
        h.drag_at(grip);
        h.run_steps(1);
        let edge = egui::pos2(footer.center().x, footer.top() - 20.);
        h.hover_at(edge);
        let mut target_rect = hidden_before;
        let mut found = false;
        for _ in 0..250 {
            h.run_steps(1);
            target_rect = h.get_by_label("Select layer: Scroll layer 00").rect();
            let footer = h.get_by_label("New layer").rect();
            if target_rect.top() >= stack_top && target_rect.bottom() <= footer.top() - 36. {
                found = true;
                break;
            }
        }
        assert!(found, "edge scrolling reveals the named hidden target");
        assert!(target_rect.top() < hidden_before.top());
        let drop = egui::pos2(target_rect.center().x, target_rect.bottom() - 4.);
        h.hover_at(drop);
        h.run_steps(1);
        h.drop_at(drop);
        h.run_steps(3);

        let st = h.state().0.session.active().unwrap();
        let target = st
            .doc
            .layers
            .iter()
            .position(|layer| layer.name == "Scroll layer 00")
            .unwrap();
        let moved = st
            .doc
            .layers
            .iter()
            .position(|layer| layer.id == source)
            .unwrap();
        assert!(
            moved < target,
            "the dragged layer lands below the target: {moved} vs {target}"
        );
        h.state_mut().0.run("edit.undo", json!({})).unwrap();
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
    }

    #[test]
    fn ending_edge_autoscroll_stops_motion_and_short_portrait_stack_does_not_scroll() {
        let app = many_layer_fixture(24);
        let before = app.session.active().unwrap().doc.clone();
        let mut h = arrange_harness(app);
        let footer = h.get_by_label("New layer").rect();
        let source = h.get_by_label("Reorder: Scroll layer 23").rect().center();
        let stack_top = h.get_by_label("Reorder: Scroll layer 23").rect().top();
        let target_before = h.get_by_label("Reorder: Scroll layer 00").rect();
        h.hover_at(source);
        h.drag_at(source);
        h.run_steps(1);
        let edge = egui::pos2(footer.center().x, footer.top() - 20.);
        h.hover_at(edge);
        h.run_steps(24);
        let target_during = h.get_by_label("Reorder: Scroll layer 00").rect();
        assert!(target_during.top() < target_before.top());
        let top_edge = egui::pos2(edge.x, stack_top + 1.);
        h.hover_at(top_edge);
        h.run_steps(4);
        let target_reversed = h.get_by_label("Reorder: Scroll layer 00").rect();
        assert!(target_reversed.top() > target_during.top());
        assert!(h.state().1.layer_autoscroll_at.is_some());

        h.hover_at(footer.center());
        h.run_steps(1);
        let target_outside = h.get_by_label("Reorder: Scroll layer 00").rect();
        assert!(h.state().1.layer_autoscroll_at.is_none());
        h.run_steps(20);
        assert_eq!(
            h.get_by_label("Reorder: Scroll layer 00").rect(),
            target_outside
        );
        h.drop_at(footer.center());
        h.run_steps(3);
        let target_stopped = h.get_by_label("Reorder: Scroll layer 00").rect();
        h.run_steps(20);
        let target_later = h.get_by_label("Reorder: Scroll layer 00").rect();
        assert_eq!(target_later, target_stopped);
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
        assert!(h.state().1.layer_autoscroll_at.is_none());

        let app = many_layer_fixture(2);
        let mut short = arrange_harness(app);
        short.set_size(vec2(834., 1194.));
        short.run_steps(3);
        let top_before = short.get_by_label("Reorder: Scroll layer 01").rect();
        let footer = short.get_by_label("New layer").rect();
        let grip = top_before.center();
        short.hover_at(grip);
        short.drag_at(grip);
        short.run_steps(1);
        let edge = egui::pos2(footer.center().x, footer.top() - 20.);
        short.hover_at(edge);
        short.run_steps(24);
        let top_after = short.get_by_label("Reorder: Scroll layer 01").rect();
        assert_eq!(top_after, top_before);
        assert!(short.state().1.layer_autoscroll_at.is_none());
        short.drop_at(footer.center());
        short.run_steps(2);
    }

    #[test]
    fn touch_cancel_clears_active_edge_autoscroll_without_editing_document() {
        let app = many_layer_fixture(24);
        let before = app.session.active().unwrap().doc.clone();
        let mut h = arrange_harness(app);
        let source = h.get_by_label("Reorder: Scroll layer 23").rect().center();
        let footer = h.get_by_label("New layer").rect();
        h.hover_at(source);
        h.drag_at(source);
        h.run_steps(1);
        let edge = egui::pos2(footer.center().x, footer.top() - 20.);
        h.hover_at(edge);
        h.run_steps(5);
        assert!(h.state().1.layer_autoscroll_at.is_some());
        assert!(h.state().1.layer_arrange_drag.is_some());

        h.event(egui::Event::Touch {
            device_id: egui::TouchDeviceId(2),
            id: egui::TouchId(18),
            phase: egui::TouchPhase::Cancel,
            pos: edge,
            force: None,
        });
        h.run_steps(1);
        assert!(h.state().1.layer_autoscroll_at.is_none());
        assert!(h.state().1.layer_arrange_drag.is_none());
        let stopped = h.get_by_label("Reorder: Scroll layer 00").rect();
        h.run_steps(20);
        assert_eq!(h.get_by_label("Reorder: Scroll layer 00").rect(), stopped);
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
    }

    #[test]
    fn arrange_is_available_for_multiselection_without_changing_selection() {
        let mut app = fixture(false, 0);
        app.run("layer.setProps", json!({"name":"A"})).unwrap();
        let a = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.new.layer", json!({"name":"B"})).unwrap();
        let b = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.select", json!({"layer":a.0,"mode":"replace"}))
            .unwrap();
        app.run("layer.select", json!({"layer":b.0,"mode":"add"}))
            .unwrap();
        let before = app.session.active().unwrap().doc.clone();
        let selected = app.session.active().unwrap().selected_layers();
        let mut h = Harness::builder()
            .with_size(vec2(1194., 834.))
            .build_ui_state(
                |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
                (app, TabletUi::default()),
            );
        h.run_steps(3);
        h.get_by_label("Arrange layers · move 2 selected layers")
            .click();
        h.run_steps(3);
        assert!(h.state().1.arrange_layers);
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
        assert_eq!(
            h.state().0.session.active().unwrap().selected_layers(),
            selected
        );
        assert!(h.query_by_label("Reorder: A").is_some());
    }

    #[test]
    fn arrange_drop_on_descendant_reports_engine_error_without_mutating_document() {
        let mut app = fixture(false, 0);
        app.run("layer.new.group", json!({"name":"Group"})).unwrap();
        let group = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.new.layer", json!({"name":"Child"})).unwrap();
        let child = app.session.active().unwrap().active_layer.unwrap();
        app.run(
            "layer.moveTo",
            json!({"layer":child.0,"target":group.0,"position":"into"}),
        )
        .unwrap();
        app.run(
            "layer.setExpanded",
            json!({"layer":group.0,"expanded":true}),
        )
        .unwrap();
        app.run("layer.select", json!({"layer":group.0,"mode":"replace"}))
            .unwrap();
        let before = app.session.active().unwrap().doc.clone();
        let steps = app.session.active().unwrap().history.past_len();
        let mut h = arrange_harness(app);

        drag_to(&mut h, "Group", "Group");
        assert!(h.state().1.message.is_empty());
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
        assert_eq!(
            h.state().0.session.active().unwrap().history.past_len(),
            steps
        );

        drag_to(&mut h, "Group", "Child");

        assert!(!h.state().1.message.is_empty());
        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.doc, before);
        assert_eq!(st.history.past_len(), steps);
        assert_eq!(st.selected_layers(), vec![group]);
    }

    #[test]
    fn arrange_cancel_outside_rows_and_escape_leave_document_untouched() {
        let mut app = fixture(false, 0);
        app.run("layer.new.layer", json!({"name":"B"})).unwrap();
        let before = app.session.active().unwrap().doc.clone();
        let selected = app.session.active().unwrap().selected_layers();
        let steps = app.session.active().unwrap().history.past_len();
        let mut h = arrange_harness(app);
        let grip = h.get_by_label("Reorder: B").rect().center();
        h.hover_at(grip);
        h.drag_at(grip);
        h.run_steps(1);
        let outside = egui::pos2(980., 420.);
        h.hover_at(outside);
        h.run_steps(1);
        h.drop_at(outside);
        h.run_steps(3);
        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.doc, before);
        assert_eq!(st.history.past_len(), steps);
        assert_eq!(st.selected_layers(), selected);

        let grip = h.get_by_label("Reorder: B").rect().center();
        h.hover_at(grip);
        h.drag_at(grip);
        h.run_steps(1);
        h.key_press(egui::Key::Escape);
        h.run_steps(3);
        assert!(!h.state().1.arrange_layers);
        assert!(h.state().1.layer_arrange_drag.is_none());
        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.doc, before);
        assert_eq!(st.history.past_len(), steps);
        assert_eq!(st.selected_layers(), selected);
    }

    #[test]
    fn arrange_touch_cancel_wins_over_pointer_release_over_a_valid_row() {
        let mut app = fixture(false, 0);
        app.run("layer.new.layer", json!({"name":"B"})).unwrap();
        let before = app.session.active().unwrap().doc.clone();
        let selected = app.session.active().unwrap().selected_layers();
        let steps = app.session.active().unwrap().history.past_len();
        let mut h = arrange_harness(app);
        let grip = h.get_by_label("Reorder: B").rect().center();
        h.hover_at(grip);
        h.drag_at(grip);
        h.run_steps(1);
        let target = h
            .get_by_label("Select layer: Foreground restoration with a very long descriptive name")
            .rect()
            .center();
        h.event(egui::Event::Touch {
            device_id: egui::TouchDeviceId(2),
            id: egui::TouchId(18),
            phase: egui::TouchPhase::Cancel,
            pos: target,
            force: None,
        });
        h.event(egui::Event::PointerButton {
            pos: target,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        });
        h.event(egui::Event::PointerGone);
        h.run_steps(3);
        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.doc, before);
        assert_eq!(st.history.past_len(), steps);
        assert_eq!(st.selected_layers(), selected);
        assert!(h.state().1.layer_arrange_drag.is_none());
    }

    #[test]
    fn arrange_release_over_footer_cannot_drop_on_a_clipped_off_row() {
        let mut app = fixture(false, 0);
        for i in 0..24 {
            app.run(
                "layer.new.layer",
                json!({"name":format!("Scroll layer {i}")}),
            )
            .unwrap();
        }
        let before = app.session.active().unwrap().doc.clone();
        let selected = app.session.active().unwrap().selected_layers();
        let steps = app.session.active().unwrap().history.past_len();
        let mut h = arrange_harness(app);
        let footer = h.get_by_label("New layer").rect();
        let (hidden, drop) = (0..24)
            .map(|i| h.get_by_label(&format!("Reorder: Scroll layer {i}")).rect())
            .find_map(|row| {
                row.intersects(footer)
                    .then_some((row, row.intersect(footer).center()))
            })
            .expect("a clipped row extends behind the footer");
        assert!(hidden.contains(drop) && footer.contains(drop));
        let grip = h.get_by_label("Reorder: Scroll layer 23").rect().center();
        h.hover_at(grip);
        h.drag_at(grip);
        h.run_steps(1);
        h.hover_at(drop);
        h.run_steps(1);
        h.drop_at(drop);
        h.run_steps(3);
        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.doc, before);
        assert_eq!(st.history.past_len(), steps);
        assert_eq!(st.selected_layers(), selected);
        assert!(h.state().1.message.is_empty());
    }

    #[test]
    fn arrange_is_disabled_in_quick_mask_without_finishing_or_adding_history() {
        let mut app = fixture(false, 0);
        app.run("layer.new.layer", json!({"name":"B"})).unwrap();
        app.run("select.editInQuickMaskMode", json!({"on":true}))
            .unwrap();
        let before = app.session.active().unwrap().doc.clone();
        let selected = app.session.active().unwrap().selected_layers();
        let steps = app.session.active().unwrap().history.past_len();
        let mut h = Harness::builder()
            .with_size(vec2(1194., 834.))
            .build_ui_state(
                |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
                (app, TabletUi::default()),
            );
        h.run_steps(3);
        h.get_by_label("Arrange layers · exit Quick Mask first")
            .click();
        h.run_steps(3);
        let st = h.state().0.session.active().unwrap();
        assert!(!h.state().1.arrange_layers);
        assert_eq!(st.doc, before);
        assert_eq!(st.history.past_len(), steps);
        assert_eq!(st.selected_layers(), selected);
        assert!(st.doc.quick_mask.is_some());
        assert!(h.query_by_label("Reorder: B").is_none());
    }

    #[test]
    fn arrange_drag_state_clears_when_layers_are_hidden_or_mask_controls_take_over() {
        let app = fixture(false, 1);
        let doc = app.session.active().unwrap().doc.id.0;
        let layer = app.session.active().unwrap().active_layer.unwrap().0;
        let mut h = arrange_harness(app);

        h.state_mut().1.layer_arrange_drag = Some(LayerArrangeDrag {
            doc_id: doc,
            source: layer,
            layers: vec![layer],
        });
        h.state_mut().1.inspector_open = false;
        h.run_steps(1);
        assert!(h.state().1.layer_arrange_drag.is_none());

        h.state_mut().1.inspector_open = true;
        h.state_mut().1.layer_arrange_drag = Some(LayerArrangeDrag {
            doc_id: doc,
            source: layer,
            layers: vec![layer],
        });
        h.state_mut().1.mask_controls = Some(LayerTarget::Mask);
        h.run_steps(1);
        assert!(h.state().1.layer_arrange_drag.is_none());
    }

    #[test]
    fn arranging_an_unselected_row_selects_its_new_active_layer() {
        let mut app = fixture(false, 0);
        app.run("layer.setProps", json!({"name":"A"})).unwrap();
        let a = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.new.layer", json!({"name":"B"})).unwrap();
        let b = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.new.group", json!({"name":"Group"})).unwrap();
        app.run("layer.select", json!({"layer":a.0,"mode":"replace"}))
            .unwrap();
        let mut h = arrange_harness(app);

        drag_to(&mut h, "B", "Group");

        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.active_layer, Some(b));
        assert_eq!(st.selected_layers(), vec![b]);
        assert!(!h.state().0.ui.mask_target);
        assert!(!h.state().0.ui.vector_mask_target);
    }

    #[test]
    fn arranging_selected_roots_moves_together_and_undo_preserves_active_mask_target() {
        let mut app = fixture(false, 0);
        app.run("layer.setProps", json!({"name":"A"})).unwrap();
        let a = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.new.layer", json!({"name":"B"})).unwrap();
        app.run("layer.new.layer", json!({"name":"C"})).unwrap();
        let c = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.layerMask.revealAll", json!({})).unwrap();
        app.run("layer.select", json!({"layer":a.0,"mode":"replace"}))
            .unwrap();
        app.run("layer.select", json!({"layer":c.0,"mode":"add"}))
            .unwrap();
        app.ui.mask_target = true;
        let before = app.session.active().unwrap().doc.clone();
        let selected = app.session.active().unwrap().selected_layers();
        let steps = app.session.active().unwrap().history.past_len();
        let mut h = arrange_harness(app);

        let source = h.get_by_label("Reorder: A").rect().center();
        let target = h.get_by_label("Select layer: B").rect();
        h.hover_at(source);
        h.drag_at(source);
        h.run_steps(1);
        let drop = egui::pos2(target.center().x, target.bottom() - 4.);
        h.hover_at(drop);
        h.run_steps(1);
        let drag = h.state().1.layer_arrange_drag.as_ref().unwrap();
        assert_eq!(drag.layers, vec![a.0, c.0]);
        h.drop_at(drop);
        h.run_steps(3);

        let st = h.state().0.session.active().unwrap();
        let order = st
            .doc
            .layers
            .iter()
            .map(|layer| layer.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(order, ["A", "C", "B"]);
        assert_eq!(st.active_layer, Some(c));
        assert_eq!(st.selected_layers(), selected);
        assert_eq!(st.history.past_len(), steps + 1);
        assert!(h.state().0.ui.mask_target);
        assert!(!h.state().0.ui.vector_mask_target);

        h.state_mut().0.run("edit.undo", json!({})).unwrap();
        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.doc, before);
        assert_eq!(st.active_layer, Some(c));
        assert_eq!(st.selected_layers(), selected);
        assert!(h.state().0.ui.mask_target);
    }

    #[test]
    fn cancelling_a_selected_set_drag_preserves_selection_and_history() {
        let mut app = fixture(false, 0);
        app.run("layer.setProps", json!({"name":"A"})).unwrap();
        let a = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.new.layer", json!({"name":"B"})).unwrap();
        let b = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.new.layer", json!({"name":"C"})).unwrap();
        app.run("layer.select", json!({"layer":a.0,"mode":"replace"}))
            .unwrap();
        app.run("layer.select", json!({"layer":b.0,"mode":"add"}))
            .unwrap();
        let before = app.session.active().unwrap().doc.clone();
        let selected = app.session.active().unwrap().selected_layers();
        let steps = app.session.active().unwrap().history.past_len();
        let mut h = arrange_harness(app);

        let grip = h.get_by_label("Reorder: A").rect().center();
        h.hover_at(grip);
        h.drag_at(grip);
        h.run_steps(1);
        assert_eq!(
            h.state().1.layer_arrange_drag.as_ref().unwrap().layers,
            vec![a.0, b.0]
        );
        let outside = egui::pos2(980., 420.);
        h.hover_at(outside);
        h.run_steps(1);
        h.drop_at(outside);
        h.run_steps(3);

        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.doc, before);
        assert_eq!(st.history.past_len(), steps);
        assert_eq!(st.active_layer, Some(b));
        assert_eq!(st.selected_layers(), selected);
        assert!(h.state().1.message.is_empty());
    }

    #[test]
    fn unselected_grip_moves_one_layer_and_noop_does_not_retarget_selection() {
        let mut app = fixture(false, 0);
        app.run("layer.setProps", json!({"name":"A"})).unwrap();
        let a = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.new.layer", json!({"name":"B"})).unwrap();
        app.run("layer.new.layer", json!({"name":"C"})).unwrap();
        let c = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.new.layer", json!({"name":"D"})).unwrap();
        let d = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.select", json!({"layer":a.0,"mode":"replace"}))
            .unwrap();
        app.run("layer.select", json!({"layer":d.0,"mode":"add"}))
            .unwrap();
        let before = app.session.active().unwrap().doc.clone();
        let selected = app.session.active().unwrap().selected_layers();
        let steps = app.session.active().unwrap().history.past_len();
        let mut h = arrange_harness(app);

        let grip = h.get_by_label("Reorder: C").rect().center();
        let target = h.get_by_label("Select layer: B").rect();
        h.hover_at(grip);
        h.drag_at(grip);
        h.run_steps(1);
        assert_eq!(
            h.state().1.layer_arrange_drag.as_ref().unwrap().layers,
            vec![c.0]
        );
        let already_in_place = egui::pos2(target.center().x, target.top() + 3.);
        h.hover_at(already_in_place);
        h.run_steps(1);
        h.drop_at(already_in_place);
        h.run_steps(3);
        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.doc, before);
        assert_eq!(st.history.past_len(), steps);
        assert_eq!(st.active_layer, Some(d));
        assert_eq!(st.selected_layers(), selected);

        let grip = h.get_by_label("Reorder: C").rect().center();
        let target = h.get_by_label("Select layer: A").rect();
        h.hover_at(grip);
        h.drag_at(grip);
        h.run_steps(1);
        let move_to = egui::pos2(target.center().x, target.top() + 3.);
        h.hover_at(move_to);
        h.run_steps(1);
        h.drop_at(move_to);
        h.run_steps(3);
        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.active_layer, Some(c));
        assert_eq!(st.selected_layers(), vec![c]);
        assert_eq!(st.history.past_len(), steps + 1);
        let order = st
            .doc
            .layers
            .iter()
            .map(|layer| layer.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(order, ["A", "C", "B", "D"]);
    }

    #[test]
    fn selected_group_drag_to_its_child_errors_without_changing_selection() {
        let mut app = fixture(false, 0);
        app.run("layer.new.group", json!({"name":"Group"})).unwrap();
        let group = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.new.layer", json!({"name":"Child"})).unwrap();
        let child = app.session.active().unwrap().active_layer.unwrap();
        app.run(
            "layer.moveTo",
            json!({"layer":child.0,"target":group.0,"position":"into"}),
        )
        .unwrap();
        app.run(
            "layer.setExpanded",
            json!({"layer":group.0,"expanded":true}),
        )
        .unwrap();
        app.run("layer.new.layer", json!({"name":"Other"})).unwrap();
        let other = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.select", json!({"layer":group.0,"mode":"replace"}))
            .unwrap();
        app.run("layer.select", json!({"layer":other.0,"mode":"add"}))
            .unwrap();
        let before = app.session.active().unwrap().doc.clone();
        let selected = app.session.active().unwrap().selected_layers();
        let steps = app.session.active().unwrap().history.past_len();
        let mut h = arrange_harness(app);

        drag_to(&mut h, "Group", "Child");

        let st = h.state().0.session.active().unwrap();
        assert!(!h.state().1.message.is_empty());
        assert_eq!(st.doc, before);
        assert_eq!(st.history.past_len(), steps);
        assert_eq!(st.active_layer, Some(other));
        assert_eq!(st.selected_layers(), selected);
    }

    #[test]
    fn arranging_the_active_masked_layer_preserves_its_mask_target() {
        let mut app = fixture(false, 0);
        app.run("layer.new.layer", json!({"name":"Masked"}))
            .unwrap();
        let layer = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.layerMask.revealAll", json!({})).unwrap();
        app.run("layer.new.group", json!({"name":"Group"})).unwrap();
        app.run("layer.select", json!({"layer":layer.0,"mode":"replace"}))
            .unwrap();
        app.ui.mask_target = true;
        let mut h = arrange_harness(app);

        drag_to(&mut h, "Masked", "Group");

        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.active_layer, Some(layer));
        assert_eq!(st.selected_layers(), vec![layer]);
        assert!(st.doc.layer(layer).unwrap().mask.is_some());
        assert!(h.state().0.ui.mask_target);
        assert!(!h.state().0.ui.vector_mask_target);
    }

    #[test]
    fn crowded_rows_keep_readable_names_and_disjoint_full_size_targets() {
        for arrange in [false, true] {
            for width in [260., 280., 460., 740.] {
                for group in [false, true] {
                    for masks in 0..=2 {
                        for depth in [0, 2, 8] {
                            let app = fixture(group, masks);
                            let mut h = Harness::builder()
                                .with_size(vec2(width + 16., 180.))
                                .build_ui_state(
                                    move |ui,
                                          (app, w, bounds): &mut (
                                        PhotocraftApp,
                                        TabletUi,
                                        egui::Rect,
                                    )| {
                                        w.arrange_layers = arrange;
                                        let st = app.session.active().unwrap();
                                        let doc = st.doc.clone();
                                        let active = st.active_layer;
                                        let selected = st.selected_layers().to_vec();
                                        let layer = doc.layer(active.unwrap()).unwrap();
                                        // Force a scrollbar: the row receives the actual content width.
                                        ui.spacing_mut().scroll.floating = false;
                                        ui.spacing_mut().scroll.bar_width = 14.;
                                        ui.spacing_mut().scroll.bar_inner_margin = 6.;
                                        *bounds = egui::ScrollArea::vertical()
                                            .max_height(150.)
                                            .show(ui, |ui| {
                                                w.layer_row(
                                                    app, ui, &doc, layer, depth, &selected, active,
                                                );
                                                ui.add_space(300.);
                                            })
                                            .inner_rect;
                                    },
                                    (app, TabletUi::default(), egui::Rect::NOTHING),
                                );
                            h.run_steps(3);
                            let mut labels = vec![
                                format!("Select layer: {NAME}"),
                                format!("Visibility: {NAME}"),
                                format!("Image: {NAME}"),
                            ];
                            if group {
                                labels.push(format!("Expand group: {NAME}"));
                            }
                            if masks >= 1 {
                                labels.push(format!("Mask: {NAME}"));
                            }
                            if masks >= 2 {
                                labels.push(format!("Vector mask: {NAME}"));
                            }
                            if arrange {
                                labels.push(format!("Reorder: {NAME}"));
                            }
                            let rects: Vec<_> = labels
                                .iter()
                                .map(|label| h.get_by_label(label).rect())
                                .collect();
                            let context = format!(
                                "arrange={arrange} width={width} group={group} masks={masks} depth={depth}"
                            );
                            assert!(rects[0].width() >= 88., "name: {context} {:?}", rects[0]);
                            for (i, rect) in rects.iter().enumerate() {
                                assert!(
                                    rect.width() >= 44. && rect.height() >= 44.,
                                    "target: {context} {rect:?}"
                                );
                                assert!(
                                    rect.left() >= h.state().2.left()
                                        && rect.right() <= h.state().2.right(),
                                    "bounds: {context} {rect:?}"
                                );
                                for other in &rects[..i] {
                                    let overlap = rect.intersect(*other);
                                    assert!(
                                        overlap.width() <= 0. || overlap.height() <= 0.,
                                        "overlap: {context} {rect:?} {other:?}"
                                    );
                                }
                            }
                            let stacked = rects[2].top() > rects[0].bottom() - 0.5;
                            if width == 260. && group && masks == 2 {
                                assert!(stacked);
                            }
                            if width >= 460. {
                                assert!(!stacked, "wide rows stay compact");
                            }
                            if width == 260. && group && masks == 2 && depth == 8 {
                                let ids: Vec<_> = labels
                                    .iter()
                                    .map(|label| h.get_by_label(label).accesskit_node().id())
                                    .collect();
                                h.set_size(vec2(756., 180.));
                                h.run_steps(3);
                                for (label, id) in labels.iter().zip(ids) {
                                    assert_eq!(
                                        h.get_by_label(label).accesskit_node().id(),
                                        id,
                                        "resize changed {label} identity"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn revealing_a_nested_masked_group_shows_both_lines_and_targets_stay_independent() {
        let mut app = fixture(true, 2);
        let target = app.session.active().unwrap().active_layer.unwrap();
        for i in 0..6 {
            app.run("layer.groupLayers", json!({"name":format!("Parent {i}")}))
                .unwrap();
        }
        let mut h = Harness::builder()
            .with_size(vec2(1194., 834.))
            .build_ui_state(
                |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
                (app, TabletUi::default()),
            );
        h.run_steps(3);
        h.state_mut()
            .0
            .run("layer.select", json!({"layer":target.0}))
            .unwrap();
        h.run_steps(4);
        for (size, scale) in [
            (vec2(1194., 834.), 1.),
            (vec2(834., 1194.), 1.),
            (vec2(1194., 834.), 1.),
        ] {
            h.set_size(size).set_pixels_per_point(scale);
            h.run_steps(4);
            h.get_by_label(&format!("Select layer: {NAME}"))
                .scroll_to_me();
            h.run_steps(3);
            let bounds = h.get_by_label(&format!("Select layer: {NAME}")).rect();
            assert!(
                h.output()
                    .shapes
                    .iter()
                    .any(|shape| name_is_painted(&shape.shape, bounds.intersect(shape.clip_rect))),
                "name not painted after resize: {size:?} scale {scale}"
            );
        }
        let name = h.get_by_label(&format!("Select layer: {NAME}")).rect();
        let image = h.get_by_label(&format!("Image: {NAME}")).rect();
        let footer = h.get_by_label("New layer").rect();
        assert!(name.width() >= 88.);
        assert!(image.top() >= name.bottom());
        assert!(name.top() >= h.get_by_label("Select multiple layers").rect().bottom());
        assert!(image.bottom() <= footer.top());
        let before = h.state().0.session.active().unwrap().doc.clone();
        for (label, pixel, vector) in [
            ("Mask", true, false),
            ("Vector mask", false, true),
            ("Image", false, false),
        ] {
            h.get_by_label(&format!("{label}: {NAME}")).click();
            h.run_steps(3);
            assert!(h.state().1.message.is_empty(), "{}", h.state().1.message);
            assert_eq!(h.state().0.ui.mask_target, pixel);
            assert_eq!(h.state().0.ui.vector_mask_target, vector);
            assert_eq!(h.state().0.session.active().unwrap().doc, before);
            assert_eq!(
                h.get_by_label(&format!("Select layer: {NAME}")).rect(),
                name
            );
            assert_eq!(h.get_by_label(&format!("Image: {NAME}")).rect(), image);
        }
        h.state_mut()
            .0
            .run("layer.setProps", json!({"layer":target.0,"name":"Short"}))
            .unwrap();
        h.run_steps(3);
        assert_eq!(h.get_by_label("Select layer: Short").rect(), name);
        assert_eq!(h.get_by_label("Image: Short").rect(), image);
        h.get_by_label("Visibility: Short").click();
        h.run_steps(3);
        let st = h.state().0.session.active().unwrap();
        assert!(!st.doc.layer(target).unwrap().visible);
        assert_eq!(st.active_layer, Some(target));
        assert!(!h.state().0.ui.mask_target && !h.state().0.ui.vector_mask_target);
    }
}
