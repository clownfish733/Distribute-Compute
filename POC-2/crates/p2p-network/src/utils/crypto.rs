use sha2::{Sha256, Digest};

use crate::{core::types::Checksum};

pub fn sha256(message: &[u8]) -> [u8;32]{
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.finalize().into()
}


pub fn calc_checksum(message: &[u8]) -> Checksum{
    let hash = sha256(message);
    Checksum::from_be_bytes(hash[..4].try_into().unwrap())
}