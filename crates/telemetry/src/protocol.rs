use crate::frames::{RawV1, SemanticV1};
use log::error;
use std::io::{Error, ErrorKind, Read, Result};

pub const FRAME_MAGIC: u32 = 0xCAFEBEEF;
pub const RAW_TELEMETRY_ADDRESS: &str = "127.0.0.1:5555";
pub const SEMANTIC_TELEMETRY_ADDRESS: &str = "127.0.0.1:5556";

/// Represents the header of a telemetry frame
/// Serialize this as "Part 1" of the frame
#[derive(Debug, Copy, Clone)]
pub struct FrameHeader {
    pub magic: u32,        // 0xCAFEBEEF
    pub payload_size: u32, // size of the payload
    pub frame_type: u16,
    pub version: u16,
    pub timestamp_ms: u64,
}

impl FrameHeader {
    // pub const SIZE: u32 = std::mem::size_of::<FrameHeader>() as u32;
    pub const WIRE_SIZE: u32 = 4 // magic
        + 4 // payload size
        + 2 // frame type
        + 2 // version
        + 8; // timestamp_ms

    pub fn to_bytes(&self) -> [u8; Self::WIRE_SIZE as usize] {
        let mut out: [u8; Self::WIRE_SIZE as usize] = [0u8; Self::WIRE_SIZE as usize];
        let mut i: usize = 0;

        out[i..i + 4].copy_from_slice(&self.magic.to_le_bytes());
        i += 4;

        out[i..i + 4].copy_from_slice(&(self.payload_size).to_le_bytes());
        i += 4;

        out[i..i + 2].copy_from_slice(&self.frame_type.to_le_bytes());
        i += 2;

        out[i..i + 2].copy_from_slice(&self.version.to_le_bytes());
        i += 2;

        out[i..i + 8].copy_from_slice(&self.timestamp_ms.to_le_bytes());

        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::WIRE_SIZE as usize {
            return None;
        }

        let mut i: usize = 0;

        let magic: u32 = u32::from_le_bytes(bytes[i..i + 4].try_into().ok()?);
        i += 4;

        let size: u32 = u32::from_le_bytes(bytes[i..i + 4].try_into().ok()?);
        i += 4;

        let frame_type: u16 = u16::from_le_bytes(bytes[i..i + 2].try_into().ok()?);
        i += 2;

        let version: u16 = u16::from_le_bytes(bytes[i..i + 2].try_into().ok()?);
        i += 2;

        let timestamp_ms: u64 = u64::from_le_bytes(bytes[i..i + 8].try_into().ok()?);

        Some(Self {
            magic,
            payload_size: size,
            frame_type,
            version,
            timestamp_ms,
        })
    }
}

pub trait FramePayload: Sized {
    const FRAME_TYPE: u16;
    const VERSION: u16;

    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Option<Self>;
}

/// The catcher's mit.
/// Deserialize the header and payload into this.
#[derive(Debug, Clone)]
pub struct FrameEnvelope {
    pub header: FrameHeader,
    pub payload: Vec<u8>,
}

fn header_sanity_check(header: &FrameHeader) -> bool {
    env_logger::try_init().ok();

    if header.magic != FRAME_MAGIC {
        // log error
        error!("Invalid frame magic: {}", header.magic);
        return false;
    }

    match header.frame_type {
        RawV1::FRAME_TYPE => match header.version {
            RawV1::VERSION => {
                if header.payload_size != RawV1::WIRE_SIZE {
                    error!("Invalid frame size for RawV1: {}", header.payload_size);
                    return false;
                }
            }
            _ => {
                error!("Invalid frame version for RawV1: {}", header.version);
                return false;
            }
        },
        SemanticV1::FRAME_TYPE => match header.version {
            SemanticV1::VERSION => {
                if header.payload_size != SemanticV1::WIRE_SIZE {
                    error!("Invalid frame size for SemanticV1: {}", header.payload_size);
                    return false;
                }
            }
            _ => {
                error!("Invalid frame version for SemanticV1: {}", header.version);
                return false;
            }
        },
        _ => {
            error!("Invalid frame type: {}", header.frame_type);
            return false;
        }
    }

    if header.timestamp_ms == 0 || header.timestamp_ms == u64::MAX {
        error!("Invalid frame timestamp: {}", header.timestamp_ms);
        return false;
    }

    true
}

pub fn read_frame<R: Read>(reader: &mut R) -> Result<FrameEnvelope> {
    // 1. Read header bytes
    let mut header_buf: [u8; FrameHeader::WIRE_SIZE as usize] =
        [0u8; FrameHeader::WIRE_SIZE as usize];
    reader.read_exact(&mut header_buf)?;

    // 2. Ensure frame header length
    if header_buf.len() != FrameHeader::WIRE_SIZE as usize {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Invalid frame header length",
        ));
    }

    // 3. Reinterpret bytes as FrameHeader
    let header: FrameHeader = FrameHeader::from_bytes(&header_buf)
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Failed to parse frame header"))?;

    // 4. Sanity check
    if !header_sanity_check(&header) {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid frame header"));
    }

    // 5. Read payload
    let mut payload: Vec<u8> = vec![0u8; header.payload_size as usize];
    reader.read_exact(&mut payload)?;

    Ok(FrameEnvelope { header, payload })
}
