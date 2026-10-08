use super::*;
use photocraft_ui_egui::icons;

// Every tool is directly available; separators keep related instruments together.
pub(super) const TOOL_GROUPS: &[&[Tool]] = &[
    &[Tool::Move, Tool::Crop, Tool::Hand, Tool::Zoom],
    &[
        Tool::RectMarquee,
        Tool::EllipseMarquee,
        Tool::Lasso,
        Tool::PolygonLasso,
        Tool::MagneticLasso,
        Tool::ObjectSelection,
        Tool::QuickSelection,
        Tool::MagicWand,
    ],
    &[
        Tool::Brush,
        Tool::Pencil,
        Tool::MixerBrush,
        Tool::Eraser,
        Tool::BackgroundEraser,
        Tool::MagicEraser,
        Tool::Gradient,
        Tool::PaintBucket,
    ],
    &[
        Tool::SpotHealing,
        Tool::Healing,
        Tool::Patch,
        Tool::ContentAwareMove,
        Tool::CloneStamp,
        Tool::HistoryBrush,
        Tool::Blur,
        Tool::Sharpen,
        Tool::Smudge,
        Tool::Dodge,
        Tool::Burn,
        Tool::Sponge,
    ],
    &[
        Tool::Pen,
        Tool::PathSelection,
        Tool::DirectSelection,
        Tool::Rectangle,
        Tool::EllipseShape,
        Tool::Triangle,
        Tool::Polygon,
        Tool::Line,
        Tool::CustomShape,
    ],
    &[
        Tool::Type,
        Tool::VerticalType,
        Tool::Eyedropper,
        Tool::Ruler,
        Tool::Note,
        Tool::Count,
        Tool::Slice,
        Tool::SliceSelect,
    ],
];

pub(super) fn icon_button(
    ui: &mut Ui,
    icon: &str,
    label: &str,
    selected: bool,
    size: f32,
) -> egui::Response {
    let tint = Tokens::get(ui.ctx()).text;
    let response = ui
        .scope(|ui| {
            ui.spacing_mut().button_padding = vec2(6., 6.);
            ui.add_sized(
                vec2(size, size),
                Button::image(icons::image(icon, 22., tint))
                    .frame(selected)
                    .selected(selected),
            )
        })
        .inner;
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
    });
    response.on_hover_text(label)
}

impl TabletUi {
    pub(super) fn tool_rail(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let tokens = Tokens::get(ui.ctx());
        egui::Panel::left("ipad-tool-rail")
            .exact_size(96.)
            .frame(Frame::NONE.fill(tokens.chrome).inner_margin(4))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(2., 2.);
                egui::Panel::bottom("ipad-rail-colour")
                    .frame(Frame::NONE)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (background, short, label, color) in [
                                (
                                    false,
                                    "FG",
                                    "Foreground colour",
                                    app.session.tools.foreground,
                                ),
                                (
                                    true,
                                    "BG",
                                    "Background colour",
                                    app.session.tools.background,
                                ),
                            ] {
                                let (rect, response) =
                                    ui.allocate_exact_size(vec2(42., 48.), egui::Sense::click());
                                let swatch = egui::Rect::from_min_size(
                                    rect.min + vec2(3., 2.),
                                    vec2(36., 30.),
                                );
                                photocraft_ui_egui::widgets::checker(ui.painter(), swatch, 6.);
                                let [r, g, b, a] =
                                    color.map(|v| (v.clamp(0., 1.) * 255.).round() as u8);
                                ui.painter().rect_filled(
                                    swatch,
                                    2.,
                                    egui::Color32::from_rgba_unmultiplied(r, g, b, a),
                                );
                                ui.painter().rect_stroke(
                                    swatch,
                                    2.,
                                    egui::Stroke::new(1., tokens.text_dim),
                                    egui::StrokeKind::Inside,
                                );
                                ui.painter().text(
                                    rect.center_bottom() - vec2(0., 2.),
                                    egui::Align2::CENTER_BOTTOM,
                                    short,
                                    egui::FontId::proportional(10.),
                                    tokens.text,
                                );
                                response.widget_info(|| {
                                    egui::WidgetInfo::labeled(
                                        egui::WidgetType::Button,
                                        ui.is_enabled(),
                                        label,
                                    )
                                });
                                if response
                                    .on_hover_text(format!("{label} · #{r:02X}{g:02X}{b:02X}"))
                                    .clicked()
                                {
                                    self.color_background = background;
                                    self.color_hex.clear();
                                    self.inspector = Inspector::Color;
                                    self.inspector_open = true;
                                }
                            }
                        });
                    });
                egui::ScrollArea::vertical()
                    .scroll_source(egui::scroll_area::ScrollSource::ALL)
                    .id_salt("ipad-all-tools")
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .show(ui, |ui| {
                        for (group, tools) in TOOL_GROUPS.iter().enumerate() {
                            if group > 0 {
                                ui.separator();
                            }
                            egui::Grid::new(("ipad-rail-group", group))
                                .spacing(vec2(2., 2.))
                                .show(ui, |ui| {
                                    for (index, tool) in tools.iter().enumerate() {
                                        if icon_button(
                                            ui,
                                            icons::tool_icon(*tool),
                                            tool.label(),
                                            app.ui.tool == *tool,
                                            40.,
                                        )
                                        .clicked()
                                        {
                                            app.ui.tool = *tool;
                                        }
                                        if index % 2 == 1 {
                                            ui.end_row();
                                        }
                                    }
                                });
                        }
                    });
            });
    }

    pub(super) fn command_navigation(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            if !self.command_path.is_empty() && button(ui, "Back", false).clicked() {
                self.command_path.pop();
            }
            ui.strong(if self.command_path.is_empty() {
                "Studio".into()
            } else {
                self.command_path.join(" / ")
            });
            if icon_button(ui, "search", "Search Studio commands", false, 44.).clicked() {
                self.sheet = None;
                self.command_tray_open = true;
                self.command_focus_requested = true;
            }
        });
    }

    pub(super) fn command_browser(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) -> bool {
        let items = menus::menu_items(app);
        let mut groups = Vec::<String>::new();
        let mut matches = Vec::new();
        for item in &items {
            if item.id.is_empty() || item.label == "---" || !menus::is_live(&item.id) {
                continue;
            }
            if item.path.starts_with(&self.command_path) {
                if let Some(group) = item.path.get(self.command_path.len()) {
                    if !groups.contains(group) {
                        groups.push(group.clone());
                    }
                } else {
                    matches.push(item);
                }
            }
        }
        if !groups.is_empty() {
            let width = (ui.available_width() - 8.) / 2.;
            egui::Grid::new("ipad-command-folders")
                .spacing(vec2(8., 8.))
                .show(ui, |ui| {
                    for (index, group) in groups.iter().enumerate() {
                        let response = ui.add_sized([width, 52.], Button::new(group));
                        if response.clicked() {
                            self.command_path.push(group.clone());
                        }
                        if index % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });
            ui.separator();
        }
        for item in matches {
            if ui
                .add_enabled(
                    item.enabled,
                    Button::new(&item.label).min_size(vec2(ui.available_width(), 44.)),
                )
                .clicked()
            {
                self.invoke(app, ui.ctx(), &item.id, json!({}));
                return true;
            }
        }
        false
    }
}
