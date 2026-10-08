//! Mask editing stays beside the canvas; viewing and painting targets are distinct.
use super::layers::LayerTarget;
use super::*;
use photocraft_doc::{Document, LayerId};
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
                self.mask_adjustment_controls(app, ui, &doc, layer, target);
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

    fn mask_adjustment_controls(
        &mut self,
        app: &mut PhotocraftApp,
        ui: &mut Ui,
        doc: &Document,
        layer: &photocraft_doc::Layer,
        target: LayerTarget,
    ) {
        let (prefix, kind, density, feather) = match target {
            LayerTarget::Mask => {
                let Some(mask) = &layer.mask else { return };
                ("layer.layerMask", "pixel", mask.density, mask.feather)
            }
            LayerTarget::Vector => {
                let Some(mask) = &layer.vector_mask else {
                    return;
                };
                ("layer.vectorMask", "vector", mask.density, mask.feather)
            }
            LayerTarget::Image => return,
        };
        let density_label = format!(
            "{} mask density (%)",
            if kind == "pixel" { "Pixel" } else { "Vector" }
        );
        let mut density_percent = density * 100.;
        let label = ui.label(&density_label);
        let mut density_changed = false;
        ui.horizontal(|ui| {
            let value_width = 80.;
            let slider_width =
                (ui.available_width() - value_width - ui.spacing().item_spacing.x).max(44.);
            let default_slider_width = ui.spacing().slider_width;
            ui.spacing_mut().slider_width = slider_width;
            let slider = ui
                .add_sized(
                    [slider_width, 44.],
                    egui::Slider::new(&mut density_percent, 0.0..=100.0)
                        .step_by(1.0)
                        .show_value(false),
                )
                .labelled_by(label.id);
            ui.spacing_mut().slider_width = default_slider_width;
            density_changed |= slider.changed();
            density_changed |= ui
                .add_sized(
                    [value_width, 44.],
                    egui::DragValue::new(&mut density_percent)
                        .range(0.0..=100.0)
                        .speed(1.0)
                        .fixed_decimals(0)
                        .suffix("%"),
                )
                .labelled_by(label.id)
                .changed();
        });
        if density_changed {
            self.invoke(
                app,
                ui.ctx(),
                &format!("{prefix}.edit"),
                json!({
                    "layer": layer.id.0,
                    "density": density_percent,
                    "coalesce": format!("ipad-mask:{}:{}:{kind}:density:{}", doc.id.0, layer.id.0, self.edit_gesture),
                }),
            );
        }
        let feather_label = format!(
            "{} mask feather (px)",
            if kind == "pixel" { "Pixel" } else { "Vector" }
        );
        let mut next_feather = feather;
        let label = ui.label(&feather_label);
        let mut feather_changed = false;
        ui.horizontal(|ui| {
            let value_width = 80.;
            let slider_width =
                (ui.available_width() - value_width - ui.spacing().item_spacing.x).max(44.);
            let default_slider_width = ui.spacing().slider_width;
            ui.spacing_mut().slider_width = slider_width;
            let slider = ui
                .add_sized(
                    [slider_width, 44.],
                    egui::Slider::new(&mut next_feather, 0.0..=1000.0)
                        .logarithmic(true)
                        .smallest_positive(0.1)
                        .show_value(false),
                )
                .labelled_by(label.id);
            ui.spacing_mut().slider_width = default_slider_width;
            feather_changed |= slider.changed();
            feather_changed |= ui
                .add_sized(
                    [value_width, 44.],
                    egui::DragValue::new(&mut next_feather)
                        .range(0.0..=1000.0)
                        .speed(0.1)
                        .min_decimals(1)
                        .max_decimals(2)
                        .suffix(" px"),
                )
                .labelled_by(label.id)
                .changed();
        });
        if feather_changed {
            self.invoke(
                app,
                ui.ctx(),
                &format!("{prefix}.edit"),
                json!({
                    "layer": layer.id.0,
                    "feather": next_feather,
                    "coalesce": format!("ipad-mask:{}:{}:{kind}:feather:{}", doc.id.0, layer.id.0, self.edit_gesture),
                }),
            );
        }
        ui.label("Feather softens the mask edge; 0 px keeps it crisp.");
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

    fn drag_control(
        h: &mut Harness<'static, (PhotocraftApp, TabletUi)>,
        label: &str,
        fraction: f32,
    ) {
        {
            let control = h.get_by_role_and_label(egui::accesskit::Role::Slider, label);
            control.scroll_to_me();
        }
        h.run_steps(3);
        let rect = h
            .get_by_role_and_label(egui::accesskit::Role::Slider, label)
            .rect();
        let start = egui::pos2(rect.right() - 8., rect.center().y);
        let end = egui::pos2(rect.left() + rect.width() * fraction, rect.center().y);
        h.hover_at(start);
        h.run_steps(1);
        h.drag_at(start);
        h.run_steps(1);
        h.hover_at(end);
        h.run_steps(2);
        h.drop_at(end);
        h.run_steps(3);
    }

    fn mask_controls_app() -> PhotocraftApp {
        let mut app = app();
        app.run("layer.vectorMask.revealAll", json!({})).unwrap();
        app
    }

    fn select_mask_target(h: &mut Harness<'static, (PhotocraftApp, TabletUi)>, vector: bool) {
        if vector {
            h.get_by_label("Vector mask").click();
            h.run_steps(3);
        }
    }

    #[test]
    fn mask_density_and_feather_drags_undo_once_without_touching_other_pixels_or_masks() {
        for (vector, control, label) in [
            (false, "density", "Pixel mask density (%)"),
            (false, "feather", "Pixel mask feather (px)"),
            (true, "density", "Vector mask density (%)"),
            (true, "feather", "Vector mask feather (px)"),
        ] {
            let app = mask_controls_app();
            let layer = app.session.active().unwrap().active_layer.unwrap();
            let mut h = harness(app, vec2(1194., 834.));
            select_mask_target(&mut h, vector);
            let before_target = photocraft_ui_egui::canvas::paint_target(&h.state().0);
            let before = h.state().0.session.active().unwrap().doc.clone();
            let image = before.layer(layer).unwrap().surface().unwrap().clone();
            let pixel_mask = before.layer(layer).unwrap().mask.clone().unwrap();
            let vector_mask = before.layer(layer).unwrap().vector_mask.clone().unwrap();
            let disabled = if vector {
                "layer.vectorMask.enabled"
            } else {
                "layer.layerMask.enabled"
            };
            h.state_mut()
                .0
                .run(disabled, json!({"layer":layer.0,"enabled":false}))
                .unwrap();
            h.run_steps(3);
            assert!(h.query_by_label("This mask is not applied.").is_some());
            let before = h.state().0.session.active().unwrap().doc.clone();
            let before_steps = h.state().0.session.active().unwrap().history.past_len();
            let fraction = if control == "density" { 0.45 } else { 0.55 };
            drag_control(&mut h, label, fraction);

            let st = h.state().0.session.active().unwrap();
            let edited_layer = st.doc.layer(layer).unwrap();
            assert_eq!(edited_layer.surface().unwrap(), &image);
            if vector {
                assert_eq!(edited_layer.mask.as_ref().unwrap(), &pixel_mask);
                let edited = edited_layer.vector_mask.as_ref().unwrap();
                assert!(!edited.enabled);
                let (actual, original) = if control == "density" {
                    (edited.density, vector_mask.density)
                } else {
                    (edited.feather, vector_mask.feather)
                };
                assert_ne!(actual, original, "{label} drag did not change the value");
                assert_eq!(edited.path, vector_mask.path);
            } else {
                assert_eq!(edited_layer.vector_mask.as_ref().unwrap(), &vector_mask);
                let edited = edited_layer.mask.as_ref().unwrap();
                assert!(!edited.enabled);
                let (actual, original) = if control == "density" {
                    (edited.density, pixel_mask.density)
                } else {
                    (edited.feather, pixel_mask.feather)
                };
                assert_ne!(actual, original, "{label} drag did not change the value");
                assert_eq!(edited.surface, pixel_mask.surface);
            }
            assert_eq!(
                st.history.past_len(),
                before_steps + 1,
                "{label} should coalesce per drag"
            );
            assert_eq!(
                photocraft_ui_egui::canvas::paint_target(&h.state().0),
                before_target
            );

            h.state_mut().0.run("edit.undo", json!({})).unwrap();
            assert_eq!(
                h.state().0.session.active().unwrap().doc,
                before,
                "{label} undo"
            );
            assert_eq!(
                photocraft_ui_egui::canvas::paint_target(&h.state().0),
                before_target
            );
        }
    }

    #[test]
    fn mask_tabs_load_each_masks_latest_density_and_feather_values() {
        let mut app = mask_controls_app();
        let layer = app.session.active().unwrap().active_layer.unwrap();
        app.run(
            "layer.layerMask.edit",
            json!({"layer":layer.0,"density":35,"feather":4}),
        )
        .unwrap();
        app.run(
            "layer.vectorMask.edit",
            json!({"layer":layer.0,"density":82,"feather":64}),
        )
        .unwrap();
        let mut h = harness(app, vec2(507., 768.));
        assert_eq!(
            h.get_by_role_and_label(egui::accesskit::Role::Slider, "Pixel mask density (%)")
                .accesskit_node()
                .numeric_value(),
            Some(35.0)
        );
        assert_eq!(
            h.get_by_role_and_label(egui::accesskit::Role::Slider, "Pixel mask feather (px)")
                .accesskit_node()
                .numeric_value(),
            Some(4.0)
        );
        h.get_by_label("Vector mask").click();
        h.run_steps(3);
        assert_eq!(
            h.get_by_role_and_label(egui::accesskit::Role::Slider, "Vector mask density (%)")
                .accesskit_node()
                .numeric_value(),
            Some(82.0)
        );
        assert_eq!(
            h.get_by_role_and_label(egui::accesskit::Role::Slider, "Vector mask feather (px)")
                .accesskit_node()
                .numeric_value(),
            Some(64.0)
        );
        h.get_by_label("Pixel mask").click();
        h.run_steps(3);
        assert_eq!(
            h.get_by_role_and_label(egui::accesskit::Role::Slider, "Pixel mask density (%)")
                .accesskit_node()
                .numeric_value(),
            Some(35.0)
        );
    }

    #[test]
    fn feather_numeric_entry_accepts_exact_pixel_radius_and_undoes_once() {
        let mut h = harness(mask_controls_app(), vec2(507., 768.));
        let layer = h.state().0.session.active().unwrap().active_layer.unwrap();
        let before = h.state().0.session.active().unwrap().doc.clone();
        let before_steps = h.state().0.session.active().unwrap().history.past_len();
        {
            let field = h.get_by_role_and_label(
                egui::accesskit::Role::SpinButton,
                "Pixel mask feather (px)",
            );
            field.focus();
        }
        h.run_steps(1);
        assert!(
            h.get_by_role_and_label(egui::accesskit::Role::SpinButton, "Pixel mask feather (px)")
                .is_focused()
        );
        h.get_by_role_and_label(egui::accesskit::Role::SpinButton, "Pixel mask feather (px)")
            .type_text("12.5");
        h.run_steps(1);
        h.key_press(egui::Key::Enter);
        h.run_steps(3);

        assert_eq!(
            h.state()
                .0
                .session
                .active()
                .unwrap()
                .doc
                .layer(layer)
                .unwrap()
                .mask
                .as_ref()
                .unwrap()
                .feather,
            12.5
        );
        assert_eq!(
            h.state().0.session.active().unwrap().history.past_len(),
            before_steps + 1
        );
        h.state_mut().0.run("edit.undo", json!({})).unwrap();
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
    }

    #[test]
    fn mask_adjustments_scroll_in_portrait_without_moving_pinned_navigation() {
        let mut h = harness(mask_controls_app(), vec2(507., 768.));
        let back = h.get_by_label("Back to layers").rect();
        let tabs = h.get_by_label("Pixel mask").rect();
        let control =
            h.get_by_role_and_label(egui::accesskit::Role::Slider, "Pixel mask feather (px)");
        control.scroll_to_me();
        h.run_steps(3);
        let feather = h
            .get_by_role_and_label(egui::accesskit::Role::Slider, "Pixel mask feather (px)")
            .rect();
        assert!(feather.height() >= 44.);
        assert!(feather.left() >= 0. && feather.right() <= 507.);
        assert_eq!(h.get_by_label("Back to layers").rect(), back);
        assert_eq!(h.get_by_label("Pixel mask").rect(), tabs);
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
