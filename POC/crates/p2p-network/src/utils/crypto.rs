use sha2::{Sha256, Digest};


pub fn sha256(message: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.finalize().into()
}