use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum CustomMessage{
    Unknown(CustomUnknown)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomUnknown{
    msg_type: u8,
    payload: Vec<u8>
}