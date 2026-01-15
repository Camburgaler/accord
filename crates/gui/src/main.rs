use eframe::{App, Frame, egui};
use egui::ViewportBuilder;
use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};
use telemetry::TelemetryV1;

#[derive(Default)]
struct GuiApp {
    last_frame_time: u64,
    x: f32,
    y: f32,
    z: f32,
    listener: Option<TcpStream>,
    latest: Arc<Mutex<Option<TelemetryV1>>>,
}

impl App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Elden Ring Telemetry");

            if let Some(frame) = self.latest.lock().unwrap().as_ref() {
                ui.monospace(format!(
                    "t = {}\npos = ({:.2}, {:.2}, {:.2})",
                    frame.timestamp_ms, frame.x, frame.y, frame.z
                ));

                let painter = ui.painter();
                let center = ui.max_rect().center();
                painter.circle_filled(center, 5.0, egui::Color32::RED);
            } else {
                ui.label("Waiting for telemetry…");
            }
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let latest = Arc::new(Mutex::new(None));
    let latest_bg = latest.clone();

    std::thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:5556").expect("Failed to bind GUI listener");
        println!("Telemetry GUI listening on 127.0.0.1:5556");

        let (mut stream, addr) = listener.accept().expect("Failed to accept connection");
        println!("Bridge connected from {:?}", addr);

        let mut buf = [0u8; TelemetryV1::SIZE];

        loop {
            if stream.read_exact(&mut buf).is_err() {
                break;
            }

            let frame = unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const TelemetryV1) };

            *latest_bg.lock().unwrap() = Some(frame);
        }
    });

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
        Box::new(|_cc| {
            Ok(Box::new(GuiApp {
                latest,
                ..Default::default()
            }))
        }),
    )
}
