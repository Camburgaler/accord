use eframe::{App, Frame, egui};
use egui::{ColorImage, TextureHandle, ViewportBuilder};
use std::{
    io::Read,
    net::TcpListener,
    sync::{Arc, Mutex},
};
use telemetry::{SEMANTIC_TELEMETRY_ADDRESS, SemanticV1};

const MAP_WIDTH_PX: f32 = 7588.0;
const MAP_HEIGHT_PX: f32 = 7140.0;

// Where (0,0) lives in the image (derived through trial and error)
// TODO: Redo this, since ER calculates map origin based on where the actor spawned
// const MAP_ZERO_X: f32 = MAP_WIDTH_PX * 297.0 / 768.0;
// const MAP_ZERO_Y: f32 = MAP_HEIGHT_PX * 153.0 / 192.0;
// const MAP_ZERO_X: f32 = MAP_WIDTH_PX * 288.0 / 768.0; // prev: 290 / 768
// const MAP_ZERO_Y: f32 = MAP_HEIGHT_PX * 287.0 / 384.0; // prev: 288 / 384
// const MAP_ZERO_X: f32 = MAP_WIDTH_PX / 2.0;
// const MAP_ZERO_Y: f32 = MAP_HEIGHT_PX / 2.0;
const MAP_ZERO_X: f32 = 0.0;
const MAP_ZERO_Z: f32 = MAP_HEIGHT_PX;

// World units → pixels (derived through trial and error)
const WORLD_TO_PX: f32 = 25.0 / 32.0; // prev: 26 / 32
// const WORLD_TO_PX: f32 = 1.0;

// const BLOCK_ORIGIN_SCALE_X: f32 = MAP_WIDTH_PX / 256.0;
// const BLOCK_ORIGIN_SCALE_Z: f32 = MAP_HEIGHT_PX / 256.0;
const BLOCK_ORIGIN_SCALE: f32 = 32.0;
// const BLOCK_ORIGIN_OFFSET_X: f32 = MAP_WIDTH_PX / 2.0; // prev: 1 / 1
// const BLOCK_ORIGIN_OFFSET_Z: f32 = MAP_HEIGHT_PX / 2.0; // prev: 1 / 1

struct GuiApp {
    latest: Arc<Mutex<Option<SemanticV1>>>,
    map: TextureHandle,
    zoom: f32,
}

impl App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            ui.heading("Elden Ring Telemetry");

            if let Some(frame) = self.latest.lock().unwrap().as_ref() {
                let panel_rect: egui::Rect = ui.max_rect();
                let screen_center: egui::Pos2 = panel_rect.center();
                let block_center: egui::Vec2 = block_to_map_px(
                    frame.block_center_offset[0] as f32,
                    frame.block_center_offset[1] as f32,
                );
                let player_px: egui::Vec2 =
                    block_center + world_to_map_px(frame.position.0, frame.position.2);
                let map_draw_pos: egui::Pos2 = screen_center - player_px * self.zoom;
                let map_size: egui::Vec2 =
                    egui::vec2(MAP_WIDTH_PX * self.zoom, MAP_HEIGHT_PX * self.zoom);

                ui.allocate_ui(ui.available_size(), |ui| {
                    ui.painter().image(
                        self.map.id(),
                        egui::Rect::from_min_size(map_draw_pos, map_size),
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                });

                let yaw: f32 = frame.rotation;
                let dir: egui::Vec2 = egui::vec2(yaw.cos(), yaw.sin());

                let size: f32 = 8.0;
                let points: [egui::Pos2; 3] = [
                    screen_center + dir * size,
                    screen_center + egui::vec2(-dir.y, dir.x) * size * 0.6,
                    screen_center + egui::vec2(dir.y, -dir.x) * size * 0.6,
                ];

                ui.painter().add(egui::Shape::convex_polygon(
                    points.to_vec(),
                    egui::Color32::RED,
                    egui::Stroke::NONE,
                ));

                let scroll_y = ctx.input(|i| i.smooth_scroll_delta.y);
                if scroll_y != 0.0 {
                    self.zoom *= 1.0 + scroll_y * 0.001;
                    self.zoom = self.zoom.clamp(0.05, 2.0);
                }
            } else {
                ui.label("Waiting for telemetry…");
            }
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let latest: Arc<Mutex<Option<SemanticV1>>> = Arc::new(Mutex::new(None));
    let latest_bg: Arc<Mutex<Option<SemanticV1>>> = latest.clone();

    std::thread::spawn(move || {
        let listener: TcpListener =
            TcpListener::bind(SEMANTIC_TELEMETRY_ADDRESS).expect("Failed to bind GUI listener");
        println!("Telemetry GUI listening on 127.0.0.1:5556");

        let (mut stream, addr) = listener.accept().expect("Failed to accept connection");
        println!("Bridge connected from {:?}", addr);

        let mut buf: [u8; SemanticV1::WIRE_SIZE as usize] = [0u8; SemanticV1::WIRE_SIZE as usize];

        loop {
            if stream.read_exact(&mut buf).is_err() {
                break;
            }

            let frame: SemanticV1 =
                unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const SemanticV1) };

            *latest_bg.lock().unwrap() = Some(frame);
        }
    });

    let options: eframe::NativeOptions = eframe::NativeOptions {
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
        Box::new(|_cc: &eframe::CreationContext<'_>| {
            let map = load_map(&_cc.egui_ctx);
            Ok(Box::new(GuiApp {
                latest,
                map,
                zoom: 1.0,
            }))
        }),
    )
}

fn load_map(ctx: &egui::Context) -> TextureHandle {
    // TODO: Make this filepath relative
    // TODO: Make the TCP connections bidirectional so that the GUI can request the raw map tile images from ER's files (is this possible?)
    let image = image::open(
        "C:/Users/camer/Documents/GitHub/accord/crates/gui/src/assets/tlb_overworld_map.jpg",
    )
    .expect("map image")
    .to_rgba8();

    let size: [usize; 2] = [image.width() as usize, image.height() as usize];
    let pixels = image.into_raw();

    let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);

    ctx.load_texture("elden_ring_map", color_image, egui::TextureOptions::LINEAR)
}

fn world_to_map_px(x: f32, z: f32) -> egui::Vec2 {
    egui::vec2(
        MAP_ZERO_X + x * WORLD_TO_PX,
        MAP_ZERO_Z - z * WORLD_TO_PX, // inverted
    )
}

fn block_to_map_px(x: f32, z: f32) -> egui::Vec2 {
    egui::vec2(
        x * BLOCK_ORIGIN_SCALE,
        z * -BLOCK_ORIGIN_SCALE, // inverted
    )
}
