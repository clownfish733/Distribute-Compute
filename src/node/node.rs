use serde::{Serialize, Deserialize};

use std::{fmt::Debug, path::Path};

use anyhow::Result;

use crate::network::{NetworkMessage, ResponseHandler};

use async_trait::async_trait;

#[async_trait]
pub trait Node<B: Block, M: NetworkMessage>: Default + Serialize + for<'de>Deserialize<'de> + Clone + Debug + ResponseHandler<M>{
    fn get_chain_height(&self) -> usize;
    fn save<P>(&self, path: P) -> Result<()> where P: AsRef<Path>;
    fn load<P>(path: P) -> Result<Self> where P: AsRef<Path>;
    fn validate_block(&self, block: B) -> bool;
    fn add_block(&mut self, block: B) -> Result<()>;
}

pub trait Block{

}