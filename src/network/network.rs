use std::net::SocketAddr;

use anyhow::Result;

use tokio::sync::mpsc;

use async_trait::async_trait;

pub trait NetworkMessage: Send + Sync + 'static{
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: Vec<u8>) -> Result<Self> where Self: Sized;
}

pub enum EventCommand{
    Message(SocketAddr, Vec<u8>),
    Close(SocketAddr),
}

pub enum ResponseCommand{
    Message(Vec<u8>),
    Close,
}

#[async_trait]
pub trait PeerManager: Default + Send + Sync + 'static{
    fn add_peer(&mut self, peer: &SocketAddr, response_tx: mpsc::Sender<ResponseCommand>) -> Result<()>;
    //sends close channel then removes it.
    async fn remove_peer(&mut self, peer: &SocketAddr) -> Result<()>;
    async fn send(&self, peer: &SocketAddr, message: Vec<u8>) -> Result<()>;
    async fn broadcast(&self, message: Vec<u8>) -> Result<()>;
    async fn update(&mut self) -> Result<()>;
}

pub enum NetworkOption<M: NetworkMessage>{
    Send(M),
    Broadcast(M),
    None,
}

pub trait ResponseHandler<M: NetworkMessage>: Send + Sync + 'static{
    fn handle_message(&mut self, message: M) -> NetworkOption<M>;
}

