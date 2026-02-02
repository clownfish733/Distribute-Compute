use thiserror::Error;

#[derive(Error, Debug)]
pub enum P2PError{
    #[error("Unknown message type: 0x{0:02x}")]
    UnknownMessageType(u8),

    #[error("Unknown message category: 0x{0:02x}")]
    UnknownMessageCategory(u8),

    #[error("Invalid message size: {0}")]
    InvalidMessageSize(usize),

    #[error("Invalidate magic: {0}")]
    InvalidMagic(usize),

    #[error("Invalid checksum")]
    InvalidChecksum,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IOError(String),

    #[error("Protocol violation")]
    ProtocolViolation,

    #[error("Peer disconnected")]
    PeerDisconnect,

    #[error("Connection timeout")]
    Timeout,

    #[error("Peer already connected")]
    PeerAlreadyConnected,

    #[error("Peer not found")]
    PeerNotFound
}   

impl From<postcard::Error> for P2PError{
    fn from(e: postcard::Error) -> Self {
        P2PError::SerializationError(format!("Postcard error: {:?}", e))   
    }
}

impl From<std::io::Error> for P2PError{
    fn from(e: std::io::Error) -> Self {
        P2PError::IOError(format!("IO error: {}", e))
    }
}