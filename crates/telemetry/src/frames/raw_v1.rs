use std::fmt::Display;

use eldenring::{cs::BlockId, position::HavokPosition};
use fromsoftware_shared::F32Vector4;

use crate::FramePayload;

/// One possible frame payload.
/// Serialize this as "Part 2" of a frame.
/// Describes telemetry in a form that is meaningful to Elden Ring.
#[derive(Debug, Copy, Clone)]
pub struct RawV1 {
    pub block_center: HavokPosition, // coordinates of the center of the block the player is currently in
    pub block_id: BlockId,
    pub chunk_position: F32Vector4, // coordinates of the center of the chunk relative to the center of the block
    pub position: HavokPosition,    // coordinates of the player relative to the center of the chunk
    pub initial_position: HavokPosition, // coordinates of the player relative to ???
    pub yaw: f32,                   // direction the player is facing in radians, minus PI/2
}

impl RawV1 {
    // pub const SIZE: u32 = std::mem::size_of::<RawV1>() as u32;
    pub const WIRE_SIZE: u32 = 16 // block_center
        + 4 // block_id
        + 16 // chunk_position
        + 16 // position
        + 16 // initial_position
        + 4; // yaw
}

impl FramePayload for RawV1 {
    const FRAME_TYPE: u16 = 0;
    const VERSION: u16 = 1;

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(RawV1::WIRE_SIZE as usize);
        bytes.extend_from_slice(&self.block_center.0.to_le_bytes());
        bytes.extend_from_slice(&self.block_center.1.to_le_bytes());
        bytes.extend_from_slice(&self.block_center.2.to_le_bytes());
        bytes.extend_from_slice(&self.block_id.0.to_le_bytes());
        bytes.extend_from_slice(&self.chunk_position.0.to_le_bytes());
        bytes.extend_from_slice(&self.chunk_position.1.to_le_bytes());
        bytes.extend_from_slice(&self.chunk_position.2.to_le_bytes());
        bytes.extend_from_slice(&self.position.0.to_le_bytes());
        bytes.extend_from_slice(&self.position.1.to_le_bytes());
        bytes.extend_from_slice(&self.position.2.to_le_bytes());
        bytes.extend_from_slice(&self.initial_position.0.to_le_bytes());
        bytes.extend_from_slice(&self.initial_position.1.to_le_bytes());
        bytes.extend_from_slice(&self.initial_position.2.to_le_bytes());
        bytes.extend_from_slice(&self.yaw.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut i: usize = 0;

        let block_center: HavokPosition = HavokPosition(
            f32::from_le_bytes(bytes[i..i + 4].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 4..i + 8].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 8..i + 12].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 12..i + 16].try_into().ok()?),
        );
        i += 16;

        let block_id: BlockId = BlockId(i32::from_le_bytes(bytes[i..i + 4].try_into().ok()?));
        i += 4;

        let chunk_position: F32Vector4 = F32Vector4(
            f32::from_le_bytes(bytes[i..i + 4].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 4..i + 8].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 8..i + 12].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 12..i + 16].try_into().ok()?),
        );
        i += 16;

        let position: HavokPosition = HavokPosition(
            f32::from_le_bytes(bytes[i..i + 4].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 4..i + 8].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 8..i + 12].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 12..i + 16].try_into().ok()?),
        );
        i += 16;

        let initial_position: HavokPosition = HavokPosition(
            f32::from_le_bytes(bytes[i..i + 4].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 4..i + 8].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 8..i + 12].try_into().ok()?),
            f32::from_le_bytes(bytes[i + 12..i + 16].try_into().ok()?),
        );
        i += 16;

        let yaw: f32 = f32::from_le_bytes(bytes[i..i + 4].try_into().ok()?);

        Some(RawV1 {
            block_center,
            block_id,
            chunk_position,
            position,
            initial_position,
            yaw,
        })
    }
}

impl Display for RawV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "block_center=({:.2}, {:.2}, {:.2}), block_id={}, chunk_position=({:.2}, {:.2}, {:.2}), position=({:.2}, {:.2}, {:.2}), initial_position=({:.2}, {:.2}, {:.2}), yaw={:.2}",
            self.block_center.0,
            self.block_center.1,
            self.block_center.2,
            self.block_id,
            self.chunk_position.0,
            self.chunk_position.1,
            self.chunk_position.2,
            self.position.0,
            self.position.1,
            self.position.2,
            self.initial_position.0,
            self.initial_position.1,
            self.initial_position.2,
            self.yaw
        )
    }
}
