use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Layer2Message{
    Placeholder(Layer2Placeholder)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Layer2Placeholder;