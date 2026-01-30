use serde::{Serialize, de::DeserializeOwned};

use crate::core::P2PError;

pub trait WireEncodable: Sized{
    fn encode(&self) -> Result<Vec<u8>, P2PError>;
    fn decode(payload: &[u8]) -> Result<Self, P2PError>;
}

impl <T> WireEncodable for T
where T: Serialize + DeserializeOwned{
    fn encode(&self) -> Result<Vec<u8>, P2PError> {
        Ok(postcard::to_allocvec(self)?)
    }
    fn decode(payload: &[u8]) -> Result<Self, P2PError> {
        Ok(postcard::from_bytes(payload)?)
    }
}
