use fromsoftware_shared::F32Vector3;
use std::{
    f32::consts::PI,
    net::{TcpListener, TcpStream},
    sync::Mutex,
    time::Duration,
};
use telemetry::{
    FRAME_MAGIC, FrameEnvelope, FrameHeader, FramePayload, RAW_TELEMETRY_ADDRESS, RawV1,
    SEMANTIC_TELEMETRY_ADDRESS, SemanticV1, TelemetrySocket, read_frame,
};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(RAW_TELEMETRY_ADDRESS)?;
    println!("Telemetry bridge listening on {}", RAW_TELEMETRY_ADDRESS);

    let (mut lstream, addr) = listener.accept()?;
    println!("Client connected from {:?}", addr);

    let socket = std::sync::Arc::new(Mutex::new(TelemetrySocket::new()));
    let socket_bg = socket.clone();

    std::thread::spawn(move || {
        loop {
            // Attempt connect WITHOUT holding the lock
            let new_stream = TcpStream::connect(SEMANTIC_TELEMETRY_ADDRESS).ok();

            if let Some(stream) = new_stream {
                let mut sock = socket_bg.lock().unwrap();
                if sock.stream.is_none() {
                    sock.set_stream(stream);
                }
            }

            std::thread::sleep(Duration::from_secs(5));
        }
    });

    loop {
        let frame_envelope: FrameEnvelope = match read_frame(&mut lstream) {
            Ok(frame) => frame,
            Err(e) => {
                println!("Failed to read frame: {}", e);
                continue;
            }
        };
        let raw_frame_header: FrameHeader = frame_envelope.header;
        let raw_frame: RawV1 = match RawV1::from_bytes(&frame_envelope.payload) {
            Some(frame) => frame,
            None => {
                println!("Failed to parse raw frame");
                continue;
            }
        };

        // println!("{}", raw_frame);

        let semantic_frame: SemanticV1 = SemanticV1 {
            rotation: raw_frame.yaw + PI / 2.0,
            block_center_offset: [raw_frame.block_id.block(), raw_frame.block_id.region()],
            position: F32Vector3(
                raw_frame.position.0 - raw_frame.chunk_position.0 - raw_frame.block_center.0
                    + raw_frame.initial_position.0,
                raw_frame.position.1 - raw_frame.chunk_position.1 - raw_frame.block_center.1
                    + raw_frame.initial_position.1,
                raw_frame.position.2 - raw_frame.chunk_position.2 - raw_frame.block_center.2
                    + raw_frame.initial_position.2,
            ),
        };

        println!("{}", semantic_frame);

        let frame_header: FrameHeader = FrameHeader {
            magic: FRAME_MAGIC,
            payload_size: SemanticV1::WIRE_SIZE,
            frame_type: SemanticV1::FRAME_TYPE,
            version: SemanticV1::VERSION,
            timestamp_ms: raw_frame_header.timestamp_ms,
        };

        let bytes: Vec<u8> = SemanticV1::to_bytes(&semantic_frame);

        if let Ok(mut sock) = socket.lock() {
            sock.queue_frame(&frame_header, &bytes.as_slice());
            sock.flush();
        }
    }
}
