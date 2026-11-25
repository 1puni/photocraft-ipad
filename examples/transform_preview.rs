//! Offscreen visual QA using the real canvas and tablet workspace.
//! cargo run --example transform_preview -- .cache/transform-visuals
use egui_kittest::{Harness, kittest::Queryable};
use photocraft_engine::Session;
use photocraft_ipad::workspace::TabletUi;
use photocraft_ui_egui::{PhotocraftApp, Services, state::Tool};
use serde_json::json;

struct Preview {
    app: PhotocraftApp,
    workspace: TabletUi,
}
impl eframe::App for Preview {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.logic(ctx, frame);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        photocraft_ui_egui::touch_ui::set_enabled(ui.ctx(), true);
        self.app
            .ui_with_workspace(ui, |app, ui| self.workspace.show(app, ui));
    }
}
fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".cache/transform-visuals".into());
    std::fs::create_dir_all(&out).unwrap();
    let theme = std::env::args()
        .nth(2)
        .and_then(|name| photocraft_ui_egui::theme::ThemeKind::from_name(&name))
        .unwrap_or_default();
    for (name, width, height) in [
        ("landscape", 1194., 834.),
        ("portrait", 834., 1194.),
        ("split", 507., 768.),
    ] {
        let mut h = Harness::builder()
            .with_size(egui::vec2(width, height))
            .wgpu()
            .build_eframe(|_| {
                let mut app = PhotocraftApp::new(Session::new(), Services::default());
                app.run(
                    "file.new",
                    json!({"width":800,"height":600,"background":"transparent"}),
                )
                .unwrap();
                app.run("edit.fill", json!({"color":"#e9e4d8"})).unwrap();
                app.run("layer.new.layer", json!({"name":"Ochre tile"}))
                    .unwrap();
                app.run(
                    "select.rect",
                    json!({"x":200,"y":150,"width":300,"height":240}),
                )
                .unwrap();
                app.run("edit.fill", json!({"color":"#ce693b"})).unwrap();
                app.run("select.deselect", json!({})).unwrap();
                app.ui.tool = Tool::Move;
                app.ui.tool_options.move_show_transform = true;
                Preview {
                    app,
                    workspace: TabletUi::default(),
                }
            });
        h.run_steps(12);
        let ctx = h.ctx.clone();
        h.state_mut().app.set_theme(&ctx, theme);
        h.run_steps(3);
        h.render()
            .unwrap()
            .save(format!("{out}/{name}-selected.png"))
            .unwrap();
        h.get_by_label("Transform layer").click();
        h.run_steps(12);
        for mode in ["Resize", "Distort", "Warp"] {
            h.get_by_label(mode).click();
            h.hover_at(egui::pos2(width - 8., 8.));
            h.run_steps(8);
            h.render()
                .unwrap()
                .save(format!("{out}/{name}-{}.png", mode.to_lowercase()))
                .unwrap();
        }
        h.get_by_label("Cancel").click();
        h.run_steps(3);
        h.state_mut().app.ui.tool = Tool::Crop;
        h.run_steps(8);
        h.get_by_label("Crop: Free").click();
        h.run_steps(3);
        h.get_by_label("Square").click();
        h.run_steps(3);
        h.get_by_label("Outside: Delete").click();
        h.run_steps(3);
        h.get_by_label("Keep cropped pixels").click();
        h.hover_at(egui::pos2(width - 8., 8.));
        h.run_steps(8);
        h.render()
            .unwrap()
            .save(format!("{out}/{name}-crop.png"))
            .unwrap();
    }
}
