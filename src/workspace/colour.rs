//! Bounded colour controls, with independent FG/BG intent and browser-owned swatches.
use super::*;
use egui::{Color32, Mesh, Rect, Sense, Stroke, StrokeKind};
use photocraft_ui_egui::color_picker_ui::{hex, hsv_to_rgb, parse_hex, rgb_to_hsv};

const MAX_SWATCHES: usize = 64;
const MAX_BYTES: usize = 16_384;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum ColourView {
    #[default]
    Picker,
    Values,
    Swatches,
}
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Model {
    #[default]
    Rgb,
    Hsb,
    Lab,
}

#[derive(Default)]
struct ColourMemory {
    source: Option<[f32; 3]>,
    hsv: [f32; 3],
}
impl ColourMemory {
    fn sync(&mut self, rgb: [f32; 3]) {
        if self.source == Some(rgb) {
            return;
        }
        let mut next = rgb_to_hsv(rgb);
        if next[1] <= f32::EPSILON || next[2] <= f32::EPSILON {
            next[0] = self.hsv[0];
        }
        if next[2] <= f32::EPSILON {
            next[1] = self.hsv[1];
        }
        self.hsv = next;
        self.source = Some(rgb);
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Swatch {
    name: String,
    rgb: [f32; 3],
}

#[derive(Default)]
pub(super) struct ColourState {
    pub view: ColourView,
    model: Model,
    memory: [ColourMemory; 2],
    lab: [Option<([f32; 3], [f32; 3])>; 2],
    swatches: Vec<Swatch>,
    swatch_name: String,
    selected_swatch: Option<usize>,
    target_background: Option<bool>,
    loaded: bool,
    dirty: bool,
    storage_message: String,
    storage_failed: bool,
}

fn decode_swatches(text: &str) -> Result<Vec<Swatch>, &'static str> {
    if text.len() > MAX_BYTES {
        return Err("Saved swatches are too large to load.");
    }
    let value: Value =
        serde_json::from_str(text).map_err(|_| "Saved swatches could not be read.")?;
    if value["version"] != 1 {
        return Err("Saved swatches use an unsupported version.");
    }
    let items = value["swatches"]
        .as_array()
        .ok_or("Saved swatches are invalid.")?;
    if items.len() > MAX_SWATCHES {
        return Err("Saved swatches exceed the 64-colour limit.");
    }
    let mut out = Vec::<Swatch>::new();
    for item in items {
        let name = item["name"]
            .as_str()
            .ok_or("A saved swatch has no name.")?
            .trim();
        let rgb: [f32; 3] = if let Some(rgb) = item.get("rgb") {
            serde_json::from_value(rgb.clone())
                .map_err(|_| "A saved swatch has an invalid colour.")?
        } else {
            // Accept palettes saved by the first local preview of this surface.
            item["hex"]
                .as_str()
                .and_then(parse_hex)
                .ok_or("A saved swatch has an invalid colour.")?
        };
        if rgb
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err("A saved swatch has an invalid colour.");
        }
        if name.is_empty() || name.chars().count() > 32 || out.iter().any(|s| s.name == name) {
            return Err("Saved swatch names are invalid or repeated.");
        }
        out.push(Swatch {
            name: name.into(),
            rgb,
        });
    }
    Ok(out)
}

impl TabletUi {
    pub fn swatches_loaded(&self) -> bool {
        self.colour.loaded
    }
    pub fn restore_swatches(&mut self, text: Option<&str>) {
        if self.colour.loaded {
            return;
        }
        self.colour.loaded = true;
        if let Some(text) = text {
            match decode_swatches(text) {
                Ok(swatches) => self.colour.swatches = swatches,
                Err(error) => {
                    self.colour.storage_message = format!("{error} Stored data was left unchanged.")
                }
            }
        }
    }
    pub fn swatches_to_store(&self) -> Option<String> {
        self.colour.dirty.then(|| {
            json!({"version":1,"swatches":self.colour.swatches.iter().map(|s|
            json!({"name":s.name,"rgb":s.rgb})).collect::<Vec<_>>()})
            .to_string()
        })
    }
    pub fn swatches_stored(&mut self, success: bool) {
        self.colour.dirty = false;
        self.colour.storage_failed = !success;
        self.colour.storage_message = if success {
            String::new()
        } else {
            "Swatches are available this session, but browser storage could not save them.".into()
        };
    }

    pub(super) fn color(&mut self, app: &mut PhotocraftApp, ui: &mut Ui) {
        let width = ui.available_width();
        ui.horizontal(|ui| {
            for (background, name, rgba) in [
                (false,"Foreground",app.session.tools.foreground),
                (true,"Background",app.session.tools.background),
            ] {
                let response = ui.add_sized([((width-56.)/2.).max(44.),44.],
                    Button::new(format!("{name}\n{}",hex([rgba[0],rgba[1],rgba[2]])))
                        .selected(self.color_background == background));
                response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button,true,self.color_background == background,name));
                if response.clicked() { self.color_background=background; self.color_hex.clear(); }
            }
            if navigation::icon_button(ui,"arrow-left-right","Swap colours",false,44.).clicked() {
                self.invoke(app,ui.ctx(),"tools.setColors",json!({"foreground":app.session.tools.background,"background":app.session.tools.foreground}));
                if self.message.is_empty() { photocraft_ui_egui::type_tool::foreground_changed(app); }
                self.color_hex.clear();
            }
        });
        ui.horizontal(|ui| {
            for (view, label) in [
                (ColourView::Picker, "Picker"),
                (ColourView::Values, "Values"),
                (ColourView::Swatches, "Swatches"),
            ] {
                if ui
                    .add_sized(
                        [(width - 12.) / 3., 44.],
                        Button::new(label).selected(self.colour.view == view),
                    )
                    .clicked()
                {
                    self.colour.view = view;
                }
            }
        });
        let source = if self.color_background {
            app.session.tools.background
        } else {
            app.session.tools.foreground
        };
        let rgb = [source[0], source[1], source[2]];
        if self.colour.target_background != Some(self.color_background)
            || self
                .colour
                .selected_swatch
                .and_then(|i| self.colour.swatches.get(i))
                .is_some_and(|s| s.rgb != rgb)
        {
            self.colour.selected_swatch = None;
        }
        self.colour.target_background = Some(self.color_background);
        self.colour.memory[usize::from(self.color_background)].sync(rgb);
        if self.color_source != Some(source) || self.color_hex.is_empty() {
            self.color_hex = hex(rgb);
            self.color_source = Some(source);
        }
        if self.colour.view == ColourView::Swatches && !self.colour.storage_message.is_empty() {
            ui.label(&self.colour.storage_message);
            if self.colour.storage_failed && button(ui, "Retry saving swatches", false).clicked() {
                self.colour.dirty = true;
            }
        }
        let height = ui.available_height().max(1.);
        match self.colour.view {
            ColourView::Picker => self.colour_picker(app, ui, width, height),
            ColourView::Values | ColourView::Swatches => {
                egui::ScrollArea::vertical()
                    .id_salt(("ipad-colour-body", self.colour.view as u8))
                    .scroll_source(egui::scroll_area::ScrollSource::ALL)
                    .max_height(height)
                    .min_scrolled_height(height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.colour.view {
                        ColourView::Values => self.colour_values(app, ui, rgb),
                        ColourView::Swatches => self.colour_swatches(app, ui, rgb),
                        ColourView::Picker => {}
                    });
            }
        }
    }

    fn colour_picker(&mut self, app: &mut PhotocraftApp, ui: &mut Ui, width: f32, height: f32) {
        let target = usize::from(self.color_background);
        let mut hsv = self.colour.memory[target].hsv;
        let mut changed = false;
        let height = height.min(250.);
        let wide = width > 440.;
        let field_width = if wide {
            (height * 1.25).min(300.).min(width - 220.)
        } else {
            width - 52.
        };
        ui.horizontal_top(|ui| {
            let (rect, response) =
                ui.allocate_exact_size(vec2(field_width.max(1.), height), Sense::click_and_drag());
            ui.painter().add(colour_mesh(rect, 24, 24, |x, y| {
                hsv_to_rgb(hsv[0], x, 1. - y)
            }));
            let marker = rect.min + vec2(hsv[1] * rect.width(), (1. - hsv[2]) * rect.height());
            ui.painter()
                .circle_stroke(marker, 5., Stroke::new(2., Color32::WHITE));
            ui.painter()
                .circle_stroke(marker, 7., Stroke::new(1., Color32::BLACK));
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Slider,
                    true,
                    "Saturation and brightness",
                )
            });
            if let Some(p) = response
                .interact_pointer_pos()
                .filter(|_| response.dragged() || response.clicked())
            {
                hsv[1] = ((p.x - rect.left()) / rect.width()).clamp(0., 1.);
                hsv[2] = (1. - (p.y - rect.top()) / rect.height()).clamp(0., 1.);
                changed = true;
            }
            let (rect, response) =
                ui.allocate_exact_size(vec2(44., height), Sense::click_and_drag());
            ui.painter().add(colour_mesh(rect, 1, 36, |_, y| {
                hsv_to_rgb(y * 360., 1., 1.)
            }));
            let y = rect.top() + hsv[0] / 360. * rect.height();
            ui.painter().rect_stroke(
                Rect::from_center_size(egui::pos2(rect.center().x, y), vec2(rect.width(), 6.)),
                1.,
                Stroke::new(2., Color32::WHITE),
                StrokeKind::Inside,
            );
            response
                .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Slider, true, "Hue"));
            if let Some(p) = response
                .interact_pointer_pos()
                .filter(|_| response.dragged() || response.clicked())
            {
                hsv[0] = ((p.y - rect.top()) / rect.height()).clamp(0., 0.999_99) * 360.;
                changed = true;
            }
            if wide {
                ui.allocate_ui_with_layout(
                    vec2((width - field_width - 56.).max(1.), height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("ipad-quick-swatches")
                            .max_height(height)
                            .scroll_source(egui::scroll_area::ScrollSource::ALL)
                            .show(ui, |ui| {
                                ui.label("Quick swatches");
                                self.essential_swatches(app, ui, true);
                            });
                    },
                );
            }
        });
        if changed {
            self.colour.memory[target].hsv = hsv;
            self.set_color(app, ui.ctx(), hsv_to_rgb(hsv[0], hsv[1], hsv[2]));
        }
    }

    fn colour_values(&mut self, app: &mut PhotocraftApp, ui: &mut Ui, rgb: [f32; 3]) {
        ui.horizontal(|ui| {
            for (model, label) in [
                (Model::Rgb, "RGB"),
                (Model::Hsb, "HSB"),
                (Model::Lab, "Lab"),
            ] {
                if button(ui, label, self.colour.model == model).clicked() {
                    self.colour.model = model;
                }
            }
        });
        let target = usize::from(self.color_background);
        let mut values = match self.colour.model {
            Model::Rgb => rgb.map(|v| v * 255.),
            Model::Hsb => {
                let h = self.colour.memory[target].hsv;
                [h[0], h[1] * 100., h[2] * 100.]
            }
            Model::Lab => self.colour.lab[target]
                .filter(|(source, _)| *source == rgb)
                .map_or_else(
                    || photocraft_color::convert::srgb_to_lab(rgb),
                    |(_, values)| values,
                ),
        };
        let (names, ranges) = match self.colour.model {
            Model::Rgb => (["Red", "Green", "Blue"], [(0., 255.); 3]),
            Model::Hsb => (
                ["Hue", "Saturation %", "Brightness %"],
                [(0., 359.99), (0., 100.), (0., 100.)],
            ),
            Model::Lab => (
                ["Lightness", "a", "b"],
                [(0., 100.), (-128., 127.), (-128., 127.)],
            ),
        };
        let mut changed = false;
        if ui.available_width() > 440. {
            ui.columns(3, |columns| {
                for i in 0..3 {
                    let response = columns[i].add_sized(
                        [columns[i].available_width(), 44.],
                        egui::DragValue::new(&mut values[i])
                            .prefix(format!("{} ", names[i]))
                            .range(ranges[i].0..=ranges[i].1)
                            .clamp_existing_to_range(false)
                            .speed(0.5)
                            .max_decimals(2),
                    );
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::DragValue, true, names[i])
                    });
                    changed |= response.changed();
                }
            });
        } else {
            egui::Grid::new("ipad-colour-values")
                .num_columns(2)
                .min_col_width(100.)
                .show(ui, |ui| {
                    for i in 0..3 {
                        ui.label(names[i]);
                        let response = ui.add_sized(
                            [96., 44.],
                            egui::DragValue::new(&mut values[i])
                                .range(ranges[i].0..=ranges[i].1)
                                .clamp_existing_to_range(false)
                                .speed(0.5)
                                .max_decimals(2),
                        );
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::DragValue, true, names[i])
                        });
                        changed |= response.changed();
                        ui.end_row();
                    }
                });
        }
        if changed {
            let next = match self.colour.model {
                Model::Rgb => values.map(|v| v / 255.),
                Model::Hsb => {
                    let h = [values[0], values[1] / 100., values[2] / 100.];
                    self.colour.memory[target].hsv = h;
                    hsv_to_rgb(h[0], h[1], h[2])
                }
                Model::Lab => photocraft_color::convert::lab_to_srgb(values),
            };
            self.set_color(app, ui.ctx(), next);
            if self.colour.model == Model::Lab && self.message.is_empty() {
                self.colour.lab[target] = Some((next.map(|v| v.clamp(0., 1.)), values));
            }
        }
        ui.horizontal(|ui| {
            ui.add_sized(
                [130., 44.],
                egui::TextEdit::singleline(&mut self.color_hex).hint_text("#rrggbb"),
            );
            if button(ui, "Set hex", false).clicked() {
                if let Some(rgb) = parse_hex(&self.color_hex) {
                    self.set_color(app, ui.ctx(), rgb);
                } else {
                    self.message = "Enter a six-digit hex colour, for example #245d85.".into();
                }
            }
        });
        ui.label(if self.colour.model == Model::Lab {
            "Lab D50 converted to the sRGB tool colour."
        } else {
            "sRGB tool colour"
        });
        if button(ui, "Sample foreground", app.ui.tool == Tool::Eyedropper).clicked() {
            self.color_background = false;
            self.color_hex.clear();
            app.ui.tool = Tool::Eyedropper;
        }
    }

    fn colour_swatches(&mut self, app: &mut PhotocraftApp, ui: &mut Ui, rgb: [f32; 3]) {
        ui.label("Essentials");
        self.essential_swatches(app, ui, false);
        ui.separator();
        ui.label("Saved on this browser");
        ui.horizontal_wrapped(|ui| {
            for (i, sw) in self.colour.swatches.clone().iter().enumerate() {
                if swatch(ui, &sw.name, sw.rgb, self.colour.selected_swatch == Some(i)).clicked() {
                    self.set_color(app, ui.ctx(), sw.rgb);
                    if self.message.is_empty() {
                        self.colour.selected_swatch = Some(i);
                    }
                }
            }
        });
        if let Some(swatch) = self
            .colour
            .selected_swatch
            .and_then(|i| self.colour.swatches.get(i))
        {
            ui.label(format!("{} · {}", swatch.name, hex(swatch.rgb)));
        }
        ui.add_sized(
            [ui.available_width(), 44.],
            egui::TextEdit::singleline(&mut self.colour.swatch_name).hint_text("Name this colour"),
        );
        ui.horizontal_wrapped(|ui| {
            if button(ui, "Save swatch", false).clicked() {
                self.save_swatch(rgb);
            }
            if ui
                .add_enabled(
                    self.colour.selected_swatch.is_some(),
                    Button::new("Remove swatch").min_size(vec2(44., 44.)),
                )
                .clicked()
                && let Some(i) = self
                    .colour
                    .selected_swatch
                    .take()
                    .filter(|i| *i < self.colour.swatches.len())
            {
                self.colour.swatches.remove(i);
                self.colour.dirty = true;
            }
        });
    }

    fn essential_swatches(&mut self, app: &mut PhotocraftApp, ui: &mut Ui, quick: bool) {
        ui.horizontal_wrapped(|ui| {
            for (name, value) in [
                ("Ink", "#161616"),
                ("Paper", "#ffffff"),
                ("Slate", "#64748b"),
                ("Warm grey", "#a8a29e"),
                ("Ochre", "#d1a054"),
                ("Rust", "#b84a32"),
                ("Rose", "#d8798a"),
                ("Plum", "#7c4774"),
                ("Indigo", "#504ca3"),
                ("Blue", "#245d85"),
                ("Teal", "#287d7e"),
                ("Leaf", "#5b7948"),
            ] {
                if quick && !["Ink", "Paper", "Ochre", "Rust", "Blue", "Teal"].contains(&name) {
                    continue;
                }
                let colour = parse_hex(value).expect("built-in hex");
                if swatch(ui, name, colour, false).clicked() {
                    self.set_color(app, ui.ctx(), colour);
                }
            }
        });
    }

    fn save_swatch(&mut self, rgb: [f32; 3]) {
        let name = self.colour.swatch_name.trim();
        let name = if name.is_empty() {
            hex(rgb)
        } else {
            name.into()
        };
        if self.colour.swatches.len() >= MAX_SWATCHES {
            self.colour.storage_message =
                "The palette holds 64 swatches. Remove one to add another.".into();
            return;
        }
        if name.chars().count() > 32 {
            self.colour.storage_message = "Use a name of 32 characters or fewer.".into();
            return;
        }
        if self.colour.swatches.iter().any(|s| s.name == name) {
            self.colour.storage_message = "That name is already saved. Choose another name.".into();
            return;
        }
        self.colour.swatches.push(Swatch { name, rgb });
        self.colour.selected_swatch = Some(self.colour.swatches.len() - 1);
        self.colour.swatch_name.clear();
        self.colour.storage_message.clear();
        self.colour.dirty = true;
    }

    pub(super) fn set_color(
        &mut self,
        app: &mut PhotocraftApp,
        ctx: &egui::Context,
        rgb: [f32; 3],
    ) {
        let rgb = rgb.map(|v| v.clamp(0., 1.));
        let target = if self.color_background {
            "background"
        } else {
            "foreground"
        };
        self.invoke(
            app,
            ctx,
            "tools.setColors",
            json!({target:[rgb[0],rgb[1],rgb[2],1.]}),
        );
        if self.message.is_empty() {
            self.colour.selected_swatch = None;
            self.colour.lab[usize::from(self.color_background)] = None;
            self.colour.memory[usize::from(self.color_background)].sync(rgb);
            self.color_hex = hex(rgb);
            if !self.color_background {
                photocraft_ui_egui::type_tool::foreground_changed(app);
            }
        }
    }
}

fn c32(rgb: [f32; 3]) -> Color32 {
    Color32::from_rgb(
        (rgb[0] * 255.).round() as u8,
        (rgb[1] * 255.).round() as u8,
        (rgb[2] * 255.).round() as u8,
    )
}
fn swatch(ui: &mut Ui, name: &str, rgb: [f32; 3], selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(44., 44.), Sense::click());
    ui.painter().rect_filled(rect.shrink(3.), 3., c32(rgb));
    if selected {
        ui.painter().rect_stroke(
            rect,
            4.,
            Stroke::new(2., Tokens::get(ui.ctx()).accent),
            StrokeKind::Inside,
        );
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Button,
            true,
            selected,
            format!("Swatch: {name}"),
        )
    });
    response.on_hover_text(format!("{name} · {}", hex(rgb)))
}
fn colour_mesh(rect: Rect, nx: u32, ny: u32, color: impl Fn(f32, f32) -> [f32; 3]) -> Mesh {
    let mut mesh = Mesh::default();
    for y in 0..=ny {
        for x in 0..=nx {
            let (x, y) = (x as f32 / nx as f32, y as f32 / ny as f32);
            mesh.colored_vertex(
                rect.min + vec2(x * rect.width(), y * rect.height()),
                c32(color(x, y)),
            );
        }
    }
    for y in 0..ny {
        for x in 0..nx {
            let a = y * (nx + 1) + x;
            mesh.add_triangle(a, a + 1, a + nx + 2);
            mesh.add_triangle(a, a + nx + 2, a + nx + 1);
        }
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};
    fn app() -> PhotocraftApp {
        PhotocraftApp::new(photocraft_engine::Session::new(), Default::default())
    }

    #[test]
    fn black_retains_hue_and_saturation_separately_for_foreground_and_background() {
        let mut memory = [ColourMemory::default(), ColourMemory::default()];
        memory[0].sync(hsv_to_rgb(125., 0.7, 0.8));
        memory[1].sync(hsv_to_rgb(280., 0.4, 0.6));
        for m in &mut memory {
            m.sync([0.; 3]);
        }
        assert!((memory[0].hsv[0] - 125.).abs() < 0.01);
        assert!((memory[0].hsv[1] - 0.7).abs() < 0.01);
        assert!((memory[1].hsv[0] - 280.).abs() < 0.01);
        assert!((memory[1].hsv[1] - 0.4).abs() < 0.01);
        memory[0].sync([0.4; 3]);
        assert!((memory[0].hsv[0] - 125.).abs() < 0.01);
        assert_eq!(memory[0].hsv[1], 0.);
    }

    #[test]
    fn swatches_round_trip_and_reject_invalid_stored_data_without_writing() {
        let mut w = TabletUi::default();
        w.colour.swatch_name = "Muted blue".into();
        w.save_swatch([0.123_456_79, 0.234_567_9, 0.876_543_2]);
        let json = w.swatches_to_store().unwrap();
        let mut restored = TabletUi::default();
        restored.restore_swatches(Some(&json));
        assert_eq!(restored.colour.swatches, w.colour.swatches);
        assert!(restored.swatches_to_store().is_none());
        for bad in [
            "not JSON".into(),
            "x".repeat(MAX_BYTES + 1),
            json!({"version":2,"swatches":[]}).to_string(),
            json!({"version":1,"swatches":[{"name":"a","rgb":[-1,0,0]}]}).to_string(),
            json!({"version":1,"swatches":[{"name":"a","rgb":[0,0,0]},{"name":"a","rgb":[1,1,1]}]})
                .to_string(),
        ] {
            let mut invalid = TabletUi::default();
            invalid.restore_swatches(Some(&bad));
            assert!(invalid.colour.swatches.is_empty());
            assert!(!invalid.colour.storage_message.is_empty());
            assert!(invalid.swatches_to_store().is_none());
        }
        w.colour.swatch_name = "Muted blue".into();
        w.save_swatch([1., 0., 0.]);
        assert_eq!(w.colour.swatches.len(), 1);
        assert!(w.colour.storage_message.contains("already saved"));
        w.swatches_stored(false);
        assert_eq!(w.colour.swatches.len(), 1);
        assert!(w.colour.storage_message.contains("this session"));
    }

    #[test]
    fn colour_views_keep_targets_and_switches_pinned_in_portrait_and_split_view() {
        for size in [vec2(1194., 834.), vec2(834., 1194.), vec2(507., 768.)] {
            let mut w = TabletUi {
                inspector: Inspector::Color,
                ..Default::default()
            };
            w.restore_swatches(None);
            let mut h = Harness::builder().with_size(size).build_ui_state(
                |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
                (app(), w),
            );
            h.run_steps(3);
            let target = h.get_by_label("Foreground").rect();
            let picker = h.get_by_label("Picker").rect();
            for label in ["Values", "Swatches", "Picker"] {
                h.get_by_label(label).click();
                h.run_steps(3);
                assert_eq!(
                    h.get_by_label("Foreground").rect(),
                    target,
                    "{size:?} {label}"
                );
                assert_eq!(h.get_by_label("Picker").rect(), picker, "{size:?} {label}");
                if label == "Values" {
                    let footer = h
                        .query_all_by_label("Tool settings")
                        .map(|node| node.rect().top())
                        .fold(0., f32::max);
                    for channel in ["Red", "Green", "Blue"] {
                        assert!(
                            h.query_all_by_label(channel)
                                .map(|node| node.rect().bottom())
                                .reduce(f32::max)
                                .expect("visible numeric channel")
                                <= footer,
                            "{size:?}: {channel} must be visible without scrolling"
                        );
                    }
                }
            }
            for label in [
                "Foreground",
                "Background",
                "Swap colours",
                "Picker",
                "Values",
                "Swatches",
                "Saturation and brightness",
                "Hue",
            ] {
                let rect = h.get_by_label(label).rect();
                assert!(
                    rect.left() >= 0. && rect.right() <= size.x + 1. && rect.bottom() <= size.y,
                    "{size:?} {label}: {rect:?}"
                );
            }
        }
    }

    #[test]
    fn opening_numeric_models_never_clamps_or_changes_the_current_colour() {
        for rgb in [hsv_to_rgb(359.995, 0.9, 0.8), [1.; 3]] {
            let mut app = app();
            app.run(
                "tools.setColors",
                json!({"foreground":[rgb[0],rgb[1],rgb[2],1.]}),
            )
            .unwrap();
            let before = app.session.tools.foreground;
            let mut w = TabletUi {
                inspector: Inspector::Color,
                ..Default::default()
            };
            w.colour.view = ColourView::Values;
            let mut h = Harness::builder()
                .with_size(vec2(1194., 834.))
                .build_ui_state(
                    |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
                    (app, w),
                );
            h.run_steps(3);
            for label in ["HSB", "Lab", "RGB"] {
                h.get_by_label(label).click();
                h.run_steps(3);
                assert_eq!(
                    h.state().0.session.tools.foreground,
                    before,
                    "opening {label} changed {rgb:?}"
                );
            }
        }
    }

    #[test]
    fn failed_storage_retry_stays_reachable_above_a_full_palette() {
        for size in [vec2(1194., 834.), vec2(834., 1194.), vec2(507., 768.)] {
            let mut w = TabletUi {
                inspector: Inspector::Color,
                ..Default::default()
            };
            w.colour.view = ColourView::Swatches;
            for i in 0..MAX_SWATCHES {
                w.colour.swatch_name = format!("Colour {i}");
                w.save_swatch([i as f32 / MAX_SWATCHES as f32, 0.2, 0.5]);
            }
            w.swatches_stored(false);
            let mut h = Harness::builder().with_size(size).build_ui_state(
                |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
                (app(), w),
            );
            h.run_steps(3);
            let retry = h.get_by_label("Retry saving swatches");
            assert!(retry.rect().bottom() <= size.y);
            retry.click();
            h.run_steps(3);
            let saved = h
                .state()
                .1
                .swatches_to_store()
                .expect("retry queues the unchanged palette");
            assert_eq!(decode_swatches(&saved).unwrap().len(), MAX_SWATCHES);
        }
    }

    #[test]
    fn swatch_recolours_selected_type_even_when_it_matches_foreground_and_undo_restores_text() {
        let mut app = app();
        app.run("file.new", json!({"width":400,"height":200}))
            .unwrap();
        let id = app
            .run(
                "type.create",
                json!({"text":"Hello world","size":40,"x":20,"y":100,"color":"#ffffff"}),
            )
            .unwrap()["layer"]
            .as_u64()
            .unwrap();
        app.run("tools.setColors", json!({"foreground":"#245d85"}))
            .unwrap();
        app.ui.tool = Tool::Type;
        app.ui.text_edit = Some(photocraft_ui_egui::state::TextEdit {
            layer: id,
            caret: 0,
            anchor: 5,
            session: "ipad-colour-test".into(),
            created: false,
            dragging: false,
            resize: None,
            preedit: None,
        });
        let before = app.session.active().unwrap().doc.clone();
        let fg = app.session.tools.foreground;
        let mut w = TabletUi {
            inspector: Inspector::Color,
            ..Default::default()
        };
        w.colour.view = ColourView::Swatches;
        let mut h = Harness::builder()
            .with_size(vec2(1194., 834.))
            .build_ui_state(
                |ui, (app, w): &mut (PhotocraftApp, TabletUi)| w.show(app, ui),
                (app, w),
            );
        h.run_steps(3);
        for label in ["Values", "Picker", "Swatches"] {
            h.get_by_label(label).click();
            h.run_steps(3);
        }
        assert_eq!(
            h.state().0.session.active().unwrap().doc,
            before,
            "idle and view switches must not recolour text"
        );
        h.get_by_label("Swatch: Blue").click();
        h.run_steps(3);
        assert_eq!(h.state().0.session.tools.foreground, fg);
        let after = h.state().0.session.active().unwrap().doc.clone();
        let photocraft_doc::LayerContent::Text(text) =
            &after.layer(photocraft_doc::LayerId(id)).unwrap().content
        else {
            panic!("text")
        };
        let runs = text.char_runs();
        assert_eq!(runs[0].len, 5);
        assert_eq!(runs[0].style.color.to_rgba8(), [36, 93, 133, 255]);
        assert_eq!(runs[1].style.color.to_rgba8(), [255, 255, 255, 255]);
        h.get_by_label("Background").click();
        h.run_steps(3);
        h.get_by_label("Swatch: Rust").click();
        h.run_steps(3);
        assert_eq!(
            h.state().0.session.active().unwrap().doc,
            after,
            "BG must not recolour selected text"
        );
        photocraft_ui_egui::type_tool::commit(&mut h.state_mut().0);
        h.state_mut().0.run("edit.undo", json!({})).unwrap();
        assert_eq!(h.state().0.session.active().unwrap().doc, before);
    }
}
