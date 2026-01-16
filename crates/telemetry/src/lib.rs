use std::io::Write;
use std::net::TcpStream;

pub struct TelemetrySocket {
    pub stream: Option<TcpStream>,
}

impl TelemetrySocket {
    pub fn new() -> Self {
        Self { stream: None }
    }

    pub fn set_stream(&mut self, stream: TcpStream) {
        let _ = stream.set_nonblocking(true);
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
    pub yaw: f32, // direction the player is facing in radians
    pub x: f32,   // horizontal map space
    pub y: f32,   // vertical 3D space
    pub z: f32,   // vertical map space
}

impl TelemetryV1 {
    pub const SIZE: usize = std::mem::size_of::<TelemetryV1>();
}
