use std::{
    io::{ErrorKind, Write},
    net::TcpStream,
};

use crate::FrameHeader;

const MAX_BUFFERED_BYTES: usize = 1024 * 1024; // 1 MB

pub struct TelemetrySocket {
    pub stream: Option<TcpStream>,
    write_buf: Vec<u8>,
}

impl TelemetrySocket {
    pub fn new() -> Self {
        Self {
            stream: None,
            write_buf: Vec::new(),
        }
    }

    pub fn set_stream(&mut self, stream: TcpStream) {
        let _ = stream.set_nonblocking(true);
        let _ = stream.set_nodelay(true);
        self.stream = Some(stream);
        self.write_buf.clear();
    }

    pub fn clear(&mut self) {
        self.stream = None;
    }

    // pub fn write_frame(&mut self, header: &FrameHeader, payload: &[u8]) -> Result<()> {
    //     let Some(stream) = self.stream.as_mut() else {
    //         return Ok(());
    //     };
    //     debug_assert_eq!(payload.len(), header.payload_size as usize);
    //     stream.write_all(&header.to_bytes())?;
    //     stream.write_all(payload)?;
    //     Ok(())
    // }

    pub fn queue_frame(&mut self, header: &FrameHeader, payload: &[u8]) {
        debug_assert_eq!(payload.len(), header.payload_size as usize);

        if self.write_buf.len() > MAX_BUFFERED_BYTES {
            // Drop frames rather than blow memory or stall the game
            self.write_buf.clear();
            return;
        }

        self.write_buf.extend_from_slice(&header.to_bytes());
        self.write_buf.extend_from_slice(payload);
    }

    pub fn flush(&mut self) {
        let Some(stream) = self.stream.as_mut() else {
            self.write_buf.clear();
            return;
        };

        while !self.write_buf.is_empty() {
            match stream.write(&self.write_buf) {
                Ok(0) => {
                    // Socket closed by peer
                    self.stream = None;
                    self.write_buf.clear();
                    break;
                }
                Ok(n) => {
                    // Remove the bytes that were successfully written
                    self.write_buf.drain(0..n);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // OS send buffer full, try again later
                    break;
                }
                Err(_) => {
                    // Hard failure
                    self.stream = None;
                    self.write_buf.clear();
                    break;
                }
            }
        }
    }
}
