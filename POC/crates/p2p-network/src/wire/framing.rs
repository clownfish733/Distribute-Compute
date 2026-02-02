use crate::protocol::Message;

use crate::core::constants::MAGIC;

use crate::utils::crypto::sha256;

pub struct MessageFrame{
    magic: u32,
    payload_size: u32,
    checksum: u32, // sha256 payload first 4 bytes
    payload: Vec<u8>,
}

impl MessageFrame{
    fn from_message(message: &Message) -> Self{
        let payload = message.to_bytes().expect("Failed to serialize message");
        let hash = sha256(&payload);
        let checksum = u32::from_be_bytes(hash[..4].try_into().unwrap());

        Self { 
            magic: MAGIC, 
            payload_size: payload.len() as u32, 
            checksum, 
            payload
        }
    }
}