use std::{io::Read, net::TcpListener};
use telemetry::TelemetryV1;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5555")?;
    println!("Telemetry bridge listening on 127.0.0.1:5555");

    let (mut stream, addr) = listener.accept()?;
    println!("Client connected from {:?}", addr);

    let mut buf = [0u8; TelemetryV1::SIZE];

    loop {
        stream.read_exact(&mut buf)?;

        let frame: TelemetryV1 =
            unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const TelemetryV1) };

        println!(
            "t={} pos=({:.2}, {:.2}, {:.2}) rot=({:.2}, {:.2}, {:.2})",
            frame.timestamp_ms, frame.x, frame.y, frame.z, frame.pitch, frame.yaw, frame.roll
        );
    }
}
