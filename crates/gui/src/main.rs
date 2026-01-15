use eframe::{App, Frame, egui};
use egui::ViewportBuilder;
use telemetry::TelemetryV1;

#[derive(Default)]
struct GuiApp {
    last_frame_time: u64,
    x: f32,
    y: f32,
    z: f32,
    latest: std::sync::Arc<std::sync::Mutex<Option<TelemetryV1>>>,
}

impl App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Elden Ring Telemetry");
            ui.label("GUI is alive.");

            ui.monospace(format!(
                "t = {}\npos = ({:.2}, {:.2}, {:.2})",
                self.last_frame_time, self.x, self.y, self.z
            ));

            let painter = ui.painter();
            let center = ui.max_rect().center();
            painter.circle_filled(center, 5.0, egui::Color32::RED);

            self.last_frame_time += 1;
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder {
            title: Some("Accord - Elden Ring Telemetry".to_owned()),
            inner_size: Some(egui::vec2(800.0, 600.0)),
            ..Default::default()
        },
        ..Default::default()
    };

    eframe::run_native(
        "Accord – Elden Ring Telemetry",
        options,
        Box::new(|_cc| Ok(Box::new(GuiApp::default()))),
    )
}
