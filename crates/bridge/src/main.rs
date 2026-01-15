use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    sync::Mutex,
    time::Duration,
};
use telemetry::{TelemetrySocket, TelemetryV1};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5555")?;
    println!("Telemetry bridge listening on 127.0.0.1:5555");

    let (mut lstream, addr) = listener.accept()?;
    println!("Client connected from {:?}", addr);

    let socket = std::sync::Arc::new(Mutex::new(TelemetrySocket::new()));
    let socket_bg = socket.clone();

    std::thread::spawn(move || {
        loop {
            // Attempt connect WITHOUT holding the lock
            let new_stream = TcpStream::connect("127.0.0.1:5556").ok();

            if let Some(stream) = new_stream {
                let mut sock = socket_bg.lock().unwrap();
                if sock.stream.is_none() {
                    sock.set_stream(stream);
                }
            }

            std::thread::sleep(Duration::from_secs(5));
        }
    });

    let mut buf = [0u8; TelemetryV1::SIZE];

    loop {
        lstream.read_exact(&mut buf)?;

        let frame: TelemetryV1 =
            unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const TelemetryV1) };

        println!(
            "t={} pos=({:.2}, {:.2}, {:.2}) rot=({:.2}, {:.2}, {:.2})",
            frame.timestamp_ms, frame.x, frame.y, frame.z, frame.pitch, frame.yaw, frame.roll
        );

        let bytes = unsafe {
            std::slice::from_raw_parts(&frame as *const TelemetryV1 as *const u8, TelemetryV1::SIZE)
        };

        if let Ok(mut sock) = socket.lock() {
            sock.send(bytes);
        }
    }
}
