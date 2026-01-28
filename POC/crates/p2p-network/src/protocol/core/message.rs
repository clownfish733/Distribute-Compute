use serde::{Serialize, Deserialize};

use crate::types::peer::Nonce;

#[derive(Serialize, Deserialize, Debug)]
pub enum CoreMessage{
    Handshake(HandshakeMessage),
    Ping(PingMessage),
    Pong(PongMessage),
    GetPeers(GetPeersMessage),
    Peers(PeersMessage),
    Disconnect(DisconnectMessage),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HandshakeMessage;

#[derive(Serialize, Deserialize, Debug)]
pub struct PingMessage{
    nonce: Nonce,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PongMessage{
    nonce: Nonce,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPeersMessage;

#[derive(Serialize, Deserialize, Debug)]
pub struct PeersMessage;

#[derive(Serialize, Deserialize, Debug)]
pub struct DisconnectMessage;