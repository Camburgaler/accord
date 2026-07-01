use std::fmt::Display;

use fromsoftware_shared::F32Vector3;

use crate::FramePayload;

/// One possible frame payload.
/// Serialize this as "Part 2" of a frame.
/// Describes telemetry in a form that is meaningful to the GUI.
#[derive(Debug, Copy, Clone)]
pub struct SemanticV1 {
    pub block_center_offset: [u8; 2], // block id x and y coordinates
    pub position: F32Vector3,         // (x, y, z) coordinates of the player character
    pub rotation: f32,                // direction the player is facing in radians
}

impl SemanticV1 {
    // pub const SIZE: u32 = std::mem::size_of::<SemanticV1>() as u32;
    pub const WIRE_SIZE: u32 = 2 // block_center_offset
        + 12 // position
        + 4; // rotation
}

impl FramePayload for SemanticV1 {
    const FRAME_TYPE: u16 = 1;
    const VERSION: u16 = 1;

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(SemanticV1::WIRE_SIZE as usize);
        bytes.extend_from_slice(&self.block_center_offset);
        bytes.extend_from_slice(&self.position.0.to_le_bytes());
        bytes.extend_from_slice(&self.position.1.to_le_bytes());
        bytes.extend_from_slice(&self.position.2.to_le_bytes());
        bytes.extend_from_slice(&self.rotation.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut i: usize = 0;
        let block_center_offset: [u8; 2] = bytes[i..i + 2].try_into().ok()?;
        i += 2;

        let position: F32Vector3 = F32Vector3(
            f32::from_le_bytes(bytes[i..i + 4].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 4..i + 8].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 8..i + 12].try_into().ok()?),
        );
        i += 12;

        let rotation: f32 = f32::from_le_bytes(bytes[i..i + 4].try_into().ok()?);

        Some(SemanticV1 {
            block_center_offset,
            position,
            rotation,
        })
    }
}

impl Display for SemanticV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "block_center_offset=({:.2}, {:.2}), position=({:.2}, {:.2}, {:.2}), rotation={:.2}",
            self.block_center_offset[0],
            self.block_center_offset[1],
            self.position.0,
            self.position.1,
            self.position.2,
            self.rotation
        )
    }
}
