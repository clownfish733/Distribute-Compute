use crate::protocol::Message;

use crate::core::constants::MAGIC;

pub struct MessageFrame{
    magic: u32,
    payload_size: u32,
    checksum: u32, // sha256 payload first 4 bytes
    payload: Vec<u8>,
}

impl MessageFrame{
}