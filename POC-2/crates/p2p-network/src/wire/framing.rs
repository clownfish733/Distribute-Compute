use crate::{
    core::{
        constants::{FRAME_HEADER_SIZE, MAGIC, MAX_PAYLOAD_SIZE}, 
        types::{Checksum, Magic},
        error::P2PError,
    }, 

    protocol::message::Message, 

    utils::crypto::calc_checksum
};

use tokio::io::AsyncReadExt;

pub struct Frame{
    magic: Magic,
    message_type: u32,
    payload_size: u32,
    checksum: Checksum,
    payload: Vec<u8>,
}

impl Frame{
    pub fn from_message(message: &Message) -> Result<Self, P2PError> {
        let message_type = message.message_type();
        let payload      = message.encode()?;
        let payload_size = payload.len() as u32;
        let checksum     = calc_checksum(&payload);

        Ok(Self {
            magic: MAGIC,
            message_type,
            payload_size,
            checksum,
            payload,
        })
    }

    pub fn validate_checksum(&self) -> Result<(), P2PError>{
        if self.checksum != calc_checksum(&self.payload){
            return Err(P2PError::InvalidChecksum)
        }
        return Ok(())
    }

    pub fn validate_header(&self) -> Result<(), P2PError>{
        if self.magic != MAGIC{
            return Err(P2PError::InvalidMagic(self.magic as usize));
        } 
        if self.payload_size > MAX_PAYLOAD_SIZE || self.payload_size == 0{
            return Err(P2PError::InvalidMessageSize(self.payload_size as usize))
        }        
        Ok(())
    }

    pub fn add_payload(&mut self, payload: Vec<u8>){
        self.payload = payload;
    }

    pub fn payload(&self) -> Vec<u8>{
        self.payload.clone()
    }

    pub fn message_type(&self) -> u32 {
        self.message_type
    }

    pub fn to_bytes(&self) -> Vec<u8>{
        let total_len = self.payload_size as usize + FRAME_HEADER_SIZE;
        
        let mut buf: Vec<u8> = Vec::with_capacity(total_len);

        buf.extend_from_slice(&self.magic.to_be_bytes());
        buf.extend_from_slice(&self.message_type.to_be_bytes());
        buf.extend_from_slice(&self.payload_size.to_be_bytes());
        buf.extend_from_slice(&self.checksum.to_be_bytes());
        buf.extend_from_slice(&self.payload);

        buf
    }

    pub fn header_from_bytes(buf: &[u8; FRAME_HEADER_SIZE]) -> Self{
        Self { 
            magic: Magic::from_be_bytes(buf[..4].try_into().unwrap()), 
            message_type: u32::from_be_bytes(buf[4..8].try_into().unwrap()), 
            payload_size: u32::from_be_bytes(buf[8..12].try_into().unwrap()), 
            checksum: Checksum::from_be_bytes(buf[12..16].try_into().unwrap()), 
            payload: Vec::new() 
        }
    }

    pub async fn from_stream<R: AsyncReadExt + Unpin>(
        stream: &mut R,
    ) -> Result<Self, P2PError> {
        let mut header_buf = [0u8; FRAME_HEADER_SIZE];
        
        stream.read_exact(&mut header_buf).await?;

        let mut frame = Self::header_from_bytes(&header_buf);
        frame.validate_header()?;

        let mut payload = vec![0u8; frame.payload_size as usize];
        stream.read_exact(&mut payload).await?;

        frame.add_payload(payload);

        frame.validate_checksum()?;

        Ok(frame)
    }
}


/*
// Send
let frame = Frame::from_message(&message)?;
stream.write_all(&frame.to_bytes()).await?;

// Receive
let message = Frame::from_stream(&mut stream).await?.into_message()?;
*/