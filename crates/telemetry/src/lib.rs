use mio::net::TcpStream;
use std::io::Write;

pub struct TelemetrySocket {
    pub stream: Option<TcpStream>,
}

impl TelemetrySocket {
    pub fn new() -> Self {
        Self { stream: None }
    }

    pub fn set_stream(&mut self, stream: TcpStream) {
        let _ = stream.set_nodelay(true);
        self.stream = Some(stream);
    }

    pub fn clear(&mut self) {
        self.stream = None;
    }

    pub fn send(&mut self, bytes: &[u8]) {
        let Some(stream) = self.stream.as_mut() else {
            return;
        };

        match stream.write(bytes) {
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => {
                self.stream = None;
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TelemetryV1 {
    pub timestamp_ms: u64,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl TelemetryV1 {
    pub const SIZE: usize = std::mem::size_of::<TelemetryV1>();
}
