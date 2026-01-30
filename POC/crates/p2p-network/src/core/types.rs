//config
pub type NodeId = [u8; 32];

pub type ProtocolVersion = u32;

pub type NetworkId = u64;

pub type Timing = u64;

pub type Capability = String; //change this to an enum later

//network
use core::fmt;

use std::{ops::Add};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port(u16);


#[derive(Clone, Serialize, Deserialize)]
pub struct SocketAddress{
    pub address: Address,
    pub port: Port,
}

impl Add<Port> for Address{
    type Output = SocketAddress;
    fn add(self, port: Port) -> Self::Output {
        SocketAddress{
            address: self,
            port
        }
    }
}

impl fmt::Debug for SocketAddress{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.address.0, self.port.0)
    }
}

impl fmt::Display for SocketAddress{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.address.0, self.port.0)
    }
}

impl From<SocketAddress> for String {
    fn from(addr: SocketAddress) -> String{
        format!("{}:{}", addr.address.0, addr.port.0)
    }
}


//peer
pub type PeerId = [u8; 32];

pub type BlockHash = [u8; 32];

pub type Nonce = [u8; 32];