//! Vertical per-tool controls using the same view options consumed by the canvas.
use super::*;
fn choice(ui: &mut Ui, label: &str, value: &mut String, options: &[(&str, &str)]) {
    ui.label(label);
    ui.horizontal_wrapped(|ui| {
        for (key, name) in options {
            if button(ui, name, value == key).clicked() {
                *value = (*key).into();
            }
        }
    });
}
fn number(ui: &mut Ui, label: &str, value: &mut f32, range: std::ops::RangeInclusive<f32>) {
    ui.label(label);
    ui.add_sized(
        [ui.available_width(), 44.],
        egui::DragValue::new(value).range(range).speed(0.5),
    );
}
impl TabletUi {
    pub(super) fn options(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let tool = app.ui.tool;
        ui.heading(tool.label());
        let before = app.ui.tool_options.clone();
        let o = &mut app.ui.tool_options;
        if photocraft_ui_egui::tool_feedback::is_selection_tool(tool) {
            number(ui, "Feather (px)", &mut o.feather, 0.0..=1000.);
            ui.checkbox(&mut o.anti_alias, "Anti-alias edges");
        }
        match tool {
            Tool::Move => {
                ui.checkbox(&mut o.move_auto_select, "Automatically select layers");
                choice(
                    ui,
                    "Select",
                    &mut o.move_target,
                    &[("layer", "Layer"), ("group", "Group")],
                );
                ui.checkbox(&mut o.move_show_transform, "Show transform controls");
            }
            Tool::RectMarquee | Tool::EllipseMarquee => {
                choice(
                    ui,
                    "Shape constraint",
                    &mut o.marquee_style,
                    &[
                        ("normal", "Free"),
                        ("fixedRatio", "Ratio"),
                        ("fixedSize", "Fixed pixels"),
                    ],
                );
                if o.marquee_style != "normal" {
                    number(ui, "Width", &mut o.marquee_width, 0.01..=300_000.);
                    number(ui, "Height", &mut o.marquee_height, 0.01..=300_000.);
                }
            }
            Tool::MagneticLasso => {
                number(
                    ui,
                    "Detection width (px)",
                    &mut o.magnetic_width,
                    1.0..=256.,
                );
                number(
                    ui,
                    "Edge contrast (%)",
                    &mut o.magnetic_contrast,
                    1.0..=100.,
                );
                number(ui, "Point frequency", &mut o.magnetic_frequency, 0.0..=100.);
                ui.checkbox(&mut o.magnetic_pressure, "Pressure narrows detection");
            }
            Tool::MagicWand | Tool::PaintBucket | Tool::MagicEraser => {
                number(ui, "Tolerance", &mut o.tolerance, 0.0..=255.);
                ui.checkbox(&mut o.contiguous, "Contiguous area");
                ui.checkbox(&mut o.sample_all_layers, "Sample all layers");
                if tool == Tool::PaintBucket {
                    ui.checkbox(&mut o.bucket_fill_pattern, "Fill with selected pattern");
                    number(ui, "Opacity (%)", &mut o.fill_opacity, 1.0..=100.);
                }
                if tool == Tool::MagicEraser {
                    number(ui, "Opacity (%)", &mut o.magic_eraser_opacity, 0.0..=100.);
                }
            }
            Tool::BackgroundEraser => {
                choice(
                    ui,
                    "Sampling",
                    &mut o.bg_sampling,
                    &[
                        ("continuous", "Continuous"),
                        ("once", "Once"),
                        ("backgroundSwatch", "Background colour"),
                    ],
                );
                choice(
                    ui,
                    "Limits",
                    &mut o.bg_limits,
                    &[
                        ("discontiguous", "All"),
                        ("contiguous", "Contiguous"),
                        ("findEdges", "Find edges"),
                    ],
                );
                number(ui, "Tolerance (%)", &mut o.bg_tolerance, 0.0..=100.);
                ui.checkbox(&mut o.bg_protect_fg, "Protect foreground colour");
            }
            Tool::Gradient => {
                choice(
                    ui,
                    "Shape",
                    &mut o.gradient_style,
                    &[
                        ("linear", "Linear"),
                        ("radial", "Radial"),
                        ("angle", "Angle"),
                        ("reflected", "Reflected"),
                        ("diamond", "Diamond"),
                    ],
                );
                ui.checkbox(&mut o.gradient_classic, "Paint pixels (classic)");
                ui.checkbox(&mut o.gradient_reverse, "Reverse colours");
                ui.checkbox(&mut o.gradient_dither, "Dither");
                number(ui, "Opacity (%)", &mut o.fill_opacity, 1.0..=100.);
            }
            Tool::Crop => {
                choice(
                    ui,
                    "Aspect ratio",
                    &mut o.crop_ratio,
                    &[
                        ("", "Free"),
                        ("original", "Original"),
                        ("1:1", "Square"),
                        ("4:3", "4:3"),
                        ("3:4", "3:4"),
                        ("16:9", "16:9"),
                    ],
                );
                ui.checkbox(&mut o.crop_delete, "Delete cropped pixels");
            }
            Tool::Healing | Tool::CloneStamp => {
                ui.checkbox(&mut o.clone_aligned, "Keep source aligned");
                choice(
                    ui,
                    "Sample",
                    &mut o.clone_sample,
                    &[
                        ("current", "Current layer"),
                        ("currentAndBelow", "Current and below"),
                        ("all", "All layers"),
                    ],
                );
                ui.label("Enable Alt, tap the source with Pencil, then disable Alt to paint.");
            }
            Tool::SpotHealing => {
                choice(
                    ui,
                    "Healing",
                    &mut o.spot_type,
                    &[
                        ("contentAware", "Content aware"),
                        ("createTexture", "Create texture"),
                        ("proximityMatch", "Proximity match"),
                    ],
                );
                ui.checkbox(&mut o.sample_all_layers, "Sample all layers");
            }
            Tool::Patch => choice(
                ui,
                "Patch",
                &mut o.patch_mode,
                &[
                    ("source", "Repair selection"),
                    ("destination", "Repair destination"),
                ],
            ),
            Tool::ContentAwareMove => {
                choice(
                    ui,
                    "Behaviour",
                    &mut o.cam_mode,
                    &[("move", "Move"), ("extend", "Extend")],
                );
                number(ui, "Structure", &mut o.cam_structure, 1.0..=7.);
                number(ui, "Colour", &mut o.cam_color, 0.0..=10.);
            }
            Tool::Dodge | Tool::Burn => {
                choice(
                    ui,
                    "Tones",
                    &mut o.tone_range,
                    &[
                        ("shadows", "Shadows"),
                        ("midtones", "Midtones"),
                        ("highlights", "Highlights"),
                    ],
                );
                number(ui, "Exposure (%)", &mut o.exposure, 0.0..=100.);
                ui.checkbox(&mut o.protect_tones, "Protect tones");
            }
            Tool::Sponge => {
                choice(
                    ui,
                    "Mode",
                    &mut o.sponge_mode,
                    &[("desaturate", "Desaturate"), ("saturate", "Saturate")],
                );
                ui.checkbox(&mut o.vibrance, "Vibrance");
            }
            Tool::Blur | Tool::Sharpen | Tool::Smudge => {
                number(ui, "Strength (%)", &mut o.strength, 0.0..=100.);
                ui.checkbox(&mut o.sample_all_layers, "Sample all layers");
                if tool == Tool::Sharpen {
                    ui.checkbox(&mut o.protect_detail, "Protect detail");
                }
                if tool == Tool::Smudge {
                    ui.checkbox(&mut o.finger_painting, "Finger painting");
                }
            }
            Tool::QuickSelection => {
                ui.checkbox(&mut o.enhance_edge, "Enhance edge");
                ui.checkbox(&mut o.sample_all_layers, "Sample all layers");
            }
            Tool::Pen
            | Tool::Rectangle
            | Tool::EllipseShape
            | Tool::Triangle
            | Tool::Polygon
            | Tool::Line
            | Tool::CustomShape => {
                choice(
                    ui,
                    "Create",
                    &mut o.vector_mode,
                    &[("shape", "Shape layer"), ("path", "Path")],
                );
                ui.checkbox(&mut o.shape_fill, "Fill shape");
                number(ui, "Stroke width", &mut o.stroke_width, 0.0..=1000.);
                if tool == Tool::Rectangle {
                    number(ui, "Corner radius", &mut o.corner_radius, 0.0..=1000.);
                }
                if tool == Tool::Polygon {
                    ui.label("Sides");
                    ui.add_sized(
                        [160., 44.],
                        egui::DragValue::new(&mut o.polygon_sides).range(3..=100),
                    );
                }
                if tool == Tool::Line {
                    number(ui, "Line weight", &mut o.line_weight, 0.1..=1000.);
                }
            }
            Tool::Pencil => {
                ui.checkbox(&mut o.pencil_auto_erase, "Auto erase foreground colour");
            }
            Tool::Zoom => {
                ui.checkbox(&mut o.zoom_scrubby, "Drag to zoom");
            }
            _ => {}
        }
        if tool == Tool::Gradient {
            photocraft_ui_egui::gradient_ui::options_changed(app, &before);
        }
        if tool.is_brushlike() && button(ui, "Brush settings", false).clicked() {
            self.inspector = Inspector::Brush;
        }
        if tool.is_type() {
            photocraft_ui_egui::type_tool::character_panel(app, ui, false);
            ui.separator();
            photocraft_ui_egui::type_tool::character_panel(app, ui, true);
        }
        if photocraft_ui_egui::tool_feedback::is_selection_tool(tool) {
            self.command(app, ui, "Select and mask…", "select.selectAndMask");
            self.command(app, ui, "Select subject", "select.subject");
        }
        if tool == Tool::Move {
            self.command(app, ui, "Free transform", "edit.freeTransform");
        }
    }
}
