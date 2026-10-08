//! Mask editing stays beside the canvas; viewing and painting targets are distinct.
use super::layers::LayerTarget;
use super::*;
use photocraft_doc::LayerId;
use photocraft_engine::mask_view_cmds::{self, MaskViewMode};

impl TabletUi {
    pub(super) fn open_mask_controls(
        &mut self,
        app: &mut PhotocraftApp,
        ctx: &egui::Context,
        layer: LayerId,
        target: LayerTarget,
    ) {
        self.target_layer(app, ctx, layer, target);
        if self.message.is_empty() {
            self.multi_select = false;
            self.mask_controls = Some(target);
            self.layer_properties = false;
            self.mask_actions_open = false;
            self.inspector = Inspector::Layers;
            self.inspector_open = true;
        }
    }

    /// False means the target no longer exists: return to the stack without editing anything.
    pub(super) fn mask_controls_panel(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) -> bool {
        let Some(st) = app.session.active() else {
            self.mask_controls = None;
            return false;
        };
        let doc = st.doc.clone();
        let Some(layer) = st.active_layer.and_then(|id| doc.layer(id)) else {
            self.mask_controls = None;
            return false;
        };
        let target = self.mask_controls.unwrap_or(LayerTarget::Mask);
        let painting = photocraft_ui_egui::canvas::paint_target(app);
        if (target == LayerTarget::Mask && painting != json!("mask"))
            || (target == LayerTarget::Vector
                && (!app.ui.vector_mask_target || painting != json!("pixels")))
        {
            // Another surface selected an alpha channel or Quick Mask. Show the stack's
            // actual target instead of leaving a misleading selected mask tab behind.
            self.mask_controls = None;
            return false;
        }
        let (enabled, linked, prefix) = match target {
            LayerTarget::Mask if layer.mask.is_some() => {
                let Some(mask) = &layer.mask else {
                    return false;
                };
                (mask.enabled, mask.linked, "layer.layerMask")
            }
            LayerTarget::Vector if layer.vector_mask.is_some() => {
                let Some(mask) = &layer.vector_mask else {
                    return false;
                };
                (mask.enabled, mask.linked, "layer.vectorMask")
            }
            _ => {
                self.mask_controls = None;
                return false;
            }
        };
        let view = mask_view_cmds::current(st).map(|v| v.mode);
        let mut properties = false;
        ui.horizontal(|ui| {
            let back = button(ui, "Back", false);
            back.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Back to layers")
            });
            if back.clicked() {
                self.mask_controls = None;
            }
            properties =
                navigation::icon_button(ui, "sliders-horizontal", "Layer properties", false, 44.)
                    .clicked();
            ui.add(egui::Label::new(egui::RichText::new(&layer.name).strong()).truncate());
        });
        if properties {
            self.open_layer_properties();
            return true;
        }
        let count =
            1 + usize::from(layer.mask.is_some()) + usize::from(layer.vector_mask.is_some());
        let width = (ui.available_width() - (count - 1) as f32 * 6.) / count as f32;
        ui.horizontal(|ui| {
            for (kind, label, present) in [
                (LayerTarget::Image, "Image", true),
                (LayerTarget::Mask, "Pixel mask", layer.mask.is_some()),
                (
                    LayerTarget::Vector,
                    "Vector mask",
                    layer.vector_mask.is_some(),
                ),
            ] {
                if present
                    && ui
                        .add_sized([width, 44.], Button::new(label).selected(target == kind))
                        .clicked()
                {
                    self.target_layer(app, ui.ctx(), layer.id, kind);
                    if self.message.is_empty() {
                        self.mask_controls = (kind != LayerTarget::Image).then_some(kind);
                        self.mask_actions_open = false;
                    }
                }
            }
        });
        let height = ui.available_height().max(1.);
        egui::ScrollArea::vertical()
            .id_salt(("ipad-mask-controls", doc.id.0, layer.id.0, target == LayerTarget::Vector))
            .scroll_source(egui::scroll_area::ScrollSource::ALL)
            .max_height(height)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if target == LayerTarget::Mask {
                    let width = (ui.available_width()-12.)/3.;
                    ui.horizontal(|ui| {
                        for (label, mode, selected) in [
                            ("Composite", "off", view.is_none()),
                            ("Mask only", "gray", view == Some(MaskViewMode::Gray)),
                            ("Overlay", "overlay", view == Some(MaskViewMode::Overlay)),
                        ] {
                            if ui.add_sized([width,44.],Button::new(label).selected(selected)).clicked() {
                                // Explicit mask view also resolves alpha / Quick Mask precedence.
                                self.target_layer(app, ui.ctx(), layer.id, LayerTarget::Mask);
                                if self.message.is_empty() {
                                    self.invoke(app, ui.ctx(), "view.layerMask", json!({"layer":layer.id.0,"mode":mode}));
                                }
                            }
                        }
                    });
                }
                ui.horizontal(|ui| {
                    let width = (ui.available_width()-6.)/2.;
                    for (label,key,value) in [("Enabled","enabled",enabled),("Linked","linked",linked)] {
                        if ui.add_sized([width,44.],Button::new(label).selected(value)).clicked() {
                            self.invoke(app,ui.ctx(),&format!("{prefix}.{key}"),json!({"layer":layer.id.0,key:!value}));
                        }
                    }
                });
                ui.label(if linked { "Moves with the layer." } else { "Moves independently of the layer." });
                if !enabled { ui.label("This mask is not applied."); }
                if target == LayerTarget::Vector {
                    ui.label("Path target · vector mask. Brushes paint the layer image.");
                    if full_button(ui,"Edit path",false).clicked() {
                        self.target_layer(app,ui.ctx(),layer.id,LayerTarget::Vector);
                        if self.message.is_empty() { app.ui.tool = Tool::DirectSelection; }
                    }
                    if layer.mask.is_none() && full_button(ui,"Add pixel mask",false).clicked() {
                        self.invoke(app,ui.ctx(),photocraft_ui_egui::layer_menu_ui::add_mask_command(doc.selection.is_some(),false),json!({"layer":layer.id.0}));
                        if self.message.is_empty() {
                            self.open_mask_controls(app,ui.ctx(),layer.id,LayerTarget::Mask);
                        }
                    }
                }
                ui.horizontal_wrapped(|ui| {
                    if button(ui,"Load selection",false).clicked() {
                        self.invoke(app,ui.ctx(),"select.loadSelection",json!({
                            "layer":layer.id.0,"channel":if target == LayerTarget::Mask { "mask" } else { "vectorMask" },"operation":"new"
                        }));
                    }
                    let label = if target == LayerTarget::Mask && doc.selection.is_some() { "Invert selected area" } else { "Invert mask" };
                    if button(ui,label,false).clicked() {
                        if target == LayerTarget::Mask {
                            self.target_layer(app,ui.ctx(),layer.id,LayerTarget::Mask);
                            if self.message.is_empty() {
                                self.invoke(app,ui.ctx(),"image.adjustments.invert",json!({"layer":layer.id.0,"target":"mask"}));
                            }
                        } else {
                            self.invoke(app,ui.ctx(),"layer.vectorMask.edit",json!({"layer":layer.id.0,"invert":true}));
                        }
                    }
                });
                ui.separator();
                if full_button(ui,"Mask actions…",self.mask_actions_open).clicked() {
                    self.mask_actions_open = !self.mask_actions_open;
                }
                if self.mask_actions_open {
                    let apply = if target == LayerTarget::Mask { "layer.layerMask.apply" } else { "layer.rasterize.vectorMask" };
                    let label = if target == LayerTarget::Mask { "Apply mask to image" } else { "Convert to pixel mask" };
                    if ui.add_enabled(menus::is_enabled(app,apply),Button::new(label).min_size(vec2(44.,44.))).clicked() {
                        self.invoke(app,ui.ctx(),apply,json!({"layer":layer.id.0}));
                        if self.message.is_empty() {
                            if target == LayerTarget::Vector {
                                self.open_mask_controls(app,ui.ctx(),layer.id,LayerTarget::Mask);
                            } else {
                                self.target_layer(app,ui.ctx(),layer.id,LayerTarget::Image);
                                self.mask_controls = None;
                            }
                        }
                    }
                    if button(ui,"Delete mask",false).clicked() {
                        self.invoke(app,ui.ctx(),&format!("{prefix}.delete"),json!({"layer":layer.id.0}));
                        if self.message.is_empty() {
                            self.target_layer(app,ui.ctx(),layer.id,LayerTarget::Image);
                            self.mask_controls = None;
                        }
                    }
                }
            });
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };

    fn app() -> PhotocraftApp {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        app.run(
            "file.new",
            json!({"width":32,"height":32,"background":"transparent"}),
        )
        .unwrap();
        app.run("layer.setProps", json!({"name":"Ink"})).unwrap();
        app.run("edit.fill", json!({"contents":"color","color":"#ff0000"}))
            .unwrap();
        app.run("select.rect", json!({"x":16,"y":0,"width":16,"height":32}))
            .unwrap();
        app.run("layer.layerMask.revealSelection", json!({}))
            .unwrap();
        app.run("select.deselect", json!({})).unwrap();
        app
    }
    fn harness(
        app: PhotocraftApp,
        size: egui::Vec2,
    ) -> Harness<'static, (PhotocraftApp, TabletUi)> {
        let mut h = Harness::builder().with_size(size).build_ui_state(
            |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
            (app, TabletUi::default()),
        );
        PhotocraftApp::setup_context(&h.ctx, photocraft_ui_egui::theme::ThemeKind::Pro);
        h.run_steps(3);
        h.get_by_label("Mask controls").click();
        h.run_steps(3);
        assert!(h.state().1.message.is_empty(), "{}", h.state().1.message);
        h
    }

    #[test]
    fn properties_and_mask_controls_switch_without_changing_the_paint_target_or_document() {
        let app = app();
        let before = app.session.active().unwrap().doc.clone();
        let before_history = app.session.active().unwrap().history.past_len();
        let active = app.session.active().unwrap().active_layer;
        let selected = app.session.active().unwrap().selected_layers();
        let mut h = harness(app, vec2(507., 768.));
        let target = photocraft_ui_egui::canvas::paint_target(&h.state().0);

        h.get_by_label("Layer properties").click();
        h.run_steps(3);
        assert!(h.state().1.layer_properties);
        assert!(h.state().1.mask_controls.is_none());
        assert_eq!(
            photocraft_ui_egui::canvas::paint_target(&h.state().0),
            target
        );
        h.get_by_label("Back to layers").click();
        h.run_steps(3);
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
        assert_eq!(
            h.state().0.session.active().unwrap().history.past_len(),
            before_history
        );
        assert_eq!(h.state().0.session.active().unwrap().active_layer, active);
        assert_eq!(
            h.state().0.session.active().unwrap().selected_layers(),
            selected
        );

        h.get_by_label("Mask controls").click();
        h.run_steps(3);
        assert!(h.state().1.mask_controls.is_some());
        h.get_by_label("Layer properties").click();
        h.run_steps(3);
        assert!(h.state().1.layer_properties);
        assert!(h.state().1.mask_controls.is_none());
        assert_eq!(
            photocraft_ui_egui::canvas::paint_target(&h.state().0),
            target
        );
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
        assert_eq!(
            h.state().0.session.active().unwrap().history.past_len(),
            before_history
        );
    }

    #[test]
    fn mask_view_modes_preserve_document_and_pixel_invert_undo_is_isolated() {
        let mut app = app();
        let st = app.session.active_mut().unwrap();
        st.saved_revision = st.revision;
        let before = st.doc.clone();
        let steps = st.history.entries().len();
        let mut h = harness(app, vec2(1194., 834.));
        for (label, want) in [
            ("Mask only", Some(MaskViewMode::Gray)),
            ("Overlay", Some(MaskViewMode::Overlay)),
            ("Composite", None),
        ] {
            h.get_by_label(label).click();
            h.run_steps(3);
            let st = h.state().0.session.active().unwrap();
            assert_eq!(mask_view_cmds::current(st).map(|v| v.mode), want);
            assert_eq!(st.doc, before);
            assert_eq!(st.history.entries().len(), steps);
            assert!(!st.is_dirty());
            assert_eq!(
                photocraft_ui_egui::canvas::paint_target(&h.state().0),
                json!("mask")
            );
        }
        h.get_by_label("Invert mask").click();
        h.run_steps(3);
        assert!(h.state().1.message.is_empty(), "{}", h.state().1.message);
        let st = h.state().0.session.active().unwrap();
        assert_eq!(st.doc.layers[0].surface(), before.layers[0].surface());
        let mask = st.doc.layers[0].mask.as_ref().unwrap();
        assert!(mask.value(4, 4) > 0.99 && mask.value(24, 4) < 0.01);
        h.state_mut().0.run("edit.undo", json!({})).unwrap();
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
        h.run_steps(3);
        for (label, mode) in [
            ("Mask only", MaskViewMode::Gray),
            ("Overlay", MaskViewMode::Overlay),
        ] {
            h.get_by_label(label).click();
            h.run_steps(3);
            h.get_by_label("Invert mask").click();
            h.run_steps(3);
            assert_eq!(
                mask_view_cmds::current(h.state().0.session.active().unwrap()).map(|v| v.mode),
                Some(mode)
            );
            h.state_mut().0.run("edit.undo", json!({})).unwrap();
            h.run_steps(3);
            assert_eq!(h.state().0.session.active().unwrap().doc, before);
            h.get_by_label("Back to layers").click();
            h.run_steps(3);
            h.get_by_label("Mask controls").click();
            h.run_steps(3);
            assert_eq!(
                mask_view_cmds::current(h.state().0.session.active().unwrap()).map(|v| v.mode),
                Some(mode),
                "reopening controls preserves view"
            );
        }
        h.get_by_label("Load selection").click();
        h.run_steps(3);
        assert!(h.state().1.message.is_empty(), "{}", h.state().1.message);
        let st = h.state().0.session.active().unwrap();
        let selection = st.doc.selection.as_ref().unwrap();
        assert!(
            selection.sample_channel(4, 4, 0) < 0.01 && selection.sample_channel(24, 4, 0) > 0.99
        );
        assert!(h.query_by_label("Invert selected area").is_some());
    }

    #[test]
    fn pixel_and_vector_state_actions_touch_only_the_chosen_mask() {
        let mut app = app();
        app.run("layer.vectorMask.revealAll", json!({})).unwrap();
        let initial = app.session.active().unwrap().doc.clone();
        let mut h = harness(app, vec2(1194., 834.));
        h.get_by_label("Enabled").click();
        h.run_steps(3);
        h.get_by_label("Linked").click();
        h.run_steps(3);
        let layer = &h.state().0.session.active().unwrap().doc.layers[0];
        assert!(!layer.mask.as_ref().unwrap().enabled && !layer.mask.as_ref().unwrap().linked);
        assert_eq!(layer.vector_mask, initial.layers[0].vector_mask);
        h.get_by_label("Vector mask").click();
        h.run_steps(3);
        h.get_by_label("Enabled").click();
        h.run_steps(3);
        h.get_by_label("Linked").click();
        h.run_steps(3);
        let layer = &h.state().0.session.active().unwrap().doc.layers[0];
        assert!(
            !layer.vector_mask.as_ref().unwrap().enabled
                && !layer.vector_mask.as_ref().unwrap().linked
        );
        h.get_by_label("Edit path").click();
        h.run_steps(3);
        assert_eq!(h.state().0.ui.tool, Tool::DirectSelection);
        assert!(h.state().0.ui.vector_mask_target);
        assert!(!h.state().0.ui.mask_target);
        for _ in 0..4 {
            h.state_mut().0.run("edit.undo", json!({})).unwrap();
        }
        assert_eq!(h.state().0.session.active().unwrap().doc, initial);
    }

    #[test]
    fn apply_is_disabled_for_group_masks_and_delete_returns_to_stack_with_undo() {
        let mut app = app();
        let id = app.run("layer.new.group", json!({"name":"Group"})).unwrap()["layer"]
            .as_u64()
            .unwrap();
        app.run("layer.layerMask.revealAll", json!({})).unwrap();
        let mut h = harness(app, vec2(1194., 834.));
        h.get_by_label("Mask actions…").click();
        h.run_steps(3);
        assert!(
            h.get_by_label("Apply mask to image")
                .accesskit_node()
                .is_disabled()
        );
        h.get_by_label("Delete mask").click();
        h.run_steps(3);
        assert!(h.state().1.message.is_empty(), "{}", h.state().1.message);
        assert!(h.state().1.mask_controls.is_none());
        assert!(
            h.state()
                .0
                .session
                .active()
                .unwrap()
                .doc
                .layer(LayerId(id))
                .unwrap()
                .mask
                .is_none()
        );
        h.state_mut().0.run("edit.undo", json!({})).unwrap();
        assert!(
            h.state()
                .0
                .session
                .active()
                .unwrap()
                .doc
                .layer(LayerId(id))
                .unwrap()
                .mask
                .is_some()
        );
    }

    #[test]
    fn applying_pixel_mask_changes_only_masked_pixels_and_undo_restores_both() {
        let app = app();
        let before = app.session.active().unwrap().doc.clone();
        let mut h = harness(app, vec2(1194., 834.));
        h.get_by_label("Mask actions…").click();
        h.run_steps(3);
        h.get_by_label("Apply mask to image").click();
        h.run_steps(3);
        assert!(h.state().1.message.is_empty(), "{}", h.state().1.message);
        let layer = &h.state().0.session.active().unwrap().doc.layers[0];
        assert!(layer.mask.is_none());
        let pixels = layer.surface().unwrap();
        assert!(pixels.sample_channel(4, 4, 3) < 0.01);
        assert!(pixels.sample_channel(24, 4, 3) > 0.99);
        assert!(h.state().1.mask_controls.is_none());
        assert_eq!(
            photocraft_ui_egui::canvas::paint_target(&h.state().0),
            json!("pixels")
        );
        h.state_mut().0.run("edit.undo", json!({})).unwrap();
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
    }

    #[test]
    fn converting_vector_mask_opens_pixel_controls_and_undo_restores_vector() {
        let mut app = app();
        app.run("layer.layerMask.delete", json!({})).unwrap();
        app.run("layer.vectorMask.revealAll", json!({})).unwrap();
        let before = app.session.active().unwrap().doc.clone();
        let mut h = harness(app, vec2(1194., 834.));
        h.get_by_label("Mask actions…").click();
        h.run_steps(3);
        h.get_by_label("Convert to pixel mask").click();
        h.run_steps(3);
        assert!(h.state().1.message.is_empty(), "{}", h.state().1.message);
        let layer = &h.state().0.session.active().unwrap().doc.layers[0];
        assert!(layer.vector_mask.is_none());
        assert!(layer.mask.as_ref().unwrap().value(4, 4) > 0.99);
        assert_eq!(layer.surface(), before.layers[0].surface());
        assert!(h.query_by_label("Mask only").is_some());
        assert_eq!(
            photocraft_ui_egui::canvas::paint_target(&h.state().0),
            json!("mask")
        );
        h.state_mut().0.run("edit.undo", json!({})).unwrap();
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
    }

    #[test]
    fn mask_targets_and_primary_controls_fit_and_back_preserves_the_editing_target() {
        for size in [vec2(1194., 834.), vec2(834., 1194.), vec2(507., 768.)] {
            let mut h = harness(app(), size);
            let top = h.get_by_label("Back to layers").rect();
            let footer = h
                .query_all_by_label("Tool settings")
                .map(|node| node.rect().top())
                .fold(0., f32::max);
            for label in [
                "Back to layers",
                "Image",
                "Pixel mask",
                "Composite",
                "Mask only",
                "Overlay",
                "Enabled",
                "Linked",
            ] {
                let rect = h.get_by_label(label).rect();
                assert!(
                    rect.left() >= 0. && rect.right() <= size.x && rect.bottom() <= footer,
                    "{size:?} {label}: {rect:?}"
                );
                assert!(rect.height() >= 44., "{label} must be Pencil/finger sized");
            }
            h.get_by_label("Mask actions…").scroll_to_me();
            h.run_steps(3);
            assert_eq!(h.get_by_label("Back to layers").rect(), top);
            h.get_by_label("Back to layers").click();
            h.run_steps(3);
            assert!(h.state().1.mask_controls.is_none());
            assert_eq!(
                photocraft_ui_egui::canvas::paint_target(&h.state().0),
                json!("mask")
            );
        }
    }

    #[test]
    fn vector_only_layers_can_add_pixel_masks_and_external_targets_leave_mask_details() {
        let mut app = app();
        app.run("layer.layerMask.delete", json!({})).unwrap();
        app.run("layer.vectorMask.revealAll", json!({})).unwrap();
        let vector = app.session.active().unwrap().doc.layers[0]
            .vector_mask
            .clone();
        let mut h = harness(app, vec2(1194., 834.));
        h.get_by_label("Add pixel mask").click();
        h.run_steps(3);
        assert!(h.state().1.message.is_empty(), "{}", h.state().1.message);
        let layer = &h.state().0.session.active().unwrap().doc.layers[0];
        assert!(layer.mask.is_some());
        assert_eq!(layer.vector_mask, vector);
        assert_eq!(
            photocraft_ui_egui::canvas::paint_target(&h.state().0),
            json!("mask")
        );
        h.state_mut().0.run("channel.new", json!({})).unwrap();
        h.run_steps(3);
        assert!(h.state().1.mask_controls.is_none());
        assert!(h.query_by_label("Target · Alpha channel").is_some());
        h.get_by_label("Mask controls").click();
        h.run_steps(3);
        h.state_mut()
            .0
            .run("select.editInQuickMaskMode", json!({"on":true}))
            .unwrap();
        h.run_steps(3);
        assert!(h.state().1.mask_controls.is_none());
        assert!(h.query_by_label("Target · Quick Mask").is_some());
    }

    #[test]
    fn adding_a_mask_finishes_quick_mask_before_reading_the_selection() {
        let mut app = app();
        app.run("layer.layerMask.delete", json!({})).unwrap();
        app.run("select.rect", json!({"x":0,"y":0,"width":16,"height":32}))
            .unwrap();
        app.run("select.editInQuickMaskMode", json!({"on":true}))
            .unwrap();
        app.run(
            "paint.stroke",
            json!({"points":[[8,8]],"size":4,"hardness":1.,"color":"#000000","target":"quickMask"}),
        )
        .unwrap();
        let quick = app.session.active().unwrap().doc.quick_mask.clone();
        let pixels = app.session.active().unwrap().doc.layers[0]
            .surface()
            .cloned();
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
        let doc = &h.state().0.session.active().unwrap().doc;
        assert!(doc.quick_mask.is_none());
        assert_eq!(doc.layers[0].surface(), pixels.as_ref());
        let mask = doc.layers[0].mask.as_ref().unwrap();
        assert!(mask.value(2, 2) > 0.99);
        assert!(mask.value(8, 8) < 0.01 && mask.value(24, 24) < 0.01);
        h.state_mut().0.run("edit.undo", json!({})).unwrap();
        assert!(
            h.state().0.session.active().unwrap().doc.layers[0]
                .mask
                .is_none()
        );
        h.state_mut().0.run("edit.undo", json!({})).unwrap();
        assert_eq!(h.state().0.session.active().unwrap().doc.quick_mask, quick);
    }

    #[test]
    fn multiselect_thumbnails_toggle_membership_without_replacing_other_layers() {
        let mut app = app();
        let ink = app.session.active().unwrap().active_layer.unwrap();
        app.run("layer.vectorMask.revealAll", json!({})).unwrap();
        let other = app.run("layer.new.layer", json!({"name":"Other"})).unwrap()["layer"]
            .as_u64()
            .unwrap();
        let mut h = Harness::builder()
            .with_size(vec2(1194., 834.))
            .build_ui_state(
                |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
                (app, TabletUi::default()),
            );
        h.run_steps(3);
        h.get_by_label("Select multiple layers").click();
        h.run_steps(3);
        for label in ["Image: Ink", "Mask: Ink", "Vector mask: Ink"] {
            h.get_by_label(label).click();
            h.run_steps(3);
            let selected = h.state().0.session.active().unwrap().selected_layers();
            assert!(
                selected.contains(&LayerId(other)),
                "{label} replaced the existing selection"
            );
        }
        assert!(
            h.state()
                .0
                .session
                .active()
                .unwrap()
                .selected_layers()
                .contains(&ink)
        );
        assert!(!h.state().0.ui.mask_target && !h.state().0.ui.vector_mask_target);
    }
}
