use thiserror::Error;

#[derive(Error, Debug)]
pub enum P2PError{
    #[error("Unknown message type: 0x{0:02x}")]
    UnknownMessageType(u8),

    #[error("Unknow message category: 0x{0:02x}")]
    UnknownMessageCategory(u8),

    #[error("Invalid message size: {0}")]
    InvalidMessageSize(usize),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IOError(String),

    #[error("Protocol Violation")]
    ProtocolViolation,

    #[error("Peer Disconnected")]
    PeerDisconnect,

    #[error("Connection timeout")]
    Timeout,

    #[error("Peer Already Connected")]
    PeerAlreadyConnected,

    #[error("Peer Not Found")]
    PeerNotFound
}   

impl From<postcard::Error> for P2PError{
    fn from(e: postcard::Error) -> Self {
        P2PError::SerializationError(format!("Postcard error: {:?}", e))   
    }
}

impl From<std::io::Error> for P2PError{
    fn from(e: std::io::Error) -> Self {
        P2PError::IOError(format!("IO Error: {}", e))
    }
}