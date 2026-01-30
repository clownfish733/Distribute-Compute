use crate::{
    wire_message,
    message_dispatch,
};
use crate::protocol::Message;
use crate::protocol::CoreMessage;

use crate::protocol::core::{
    HandshakeMessage,
    PingMessage,
    PongMessage,
    GetPeersMessage,
    PeersMessage,
    DisconnectMessage,
};

use crate::protocol::message::{
    
}

wire_message!(CoreMessage {
    Handshake(HandshakeMessage) = 0x00, 
    Ping(PingMessage) = 0x01,
    Pong(PongMessage) = 0x02,
    GetPeers(GetPeersMessage) = 0x03,
    Peers(PeersMessage) = 0x04,
    Disconnect(DisconnectMessage) = 0x05
});

message_dispatch!(Message {
    Core(CoreMessage) = 0x20,

});