use crate::{
    message_dispatch, wire_message
};
use crate::protocol::Message;

use crate::core::traits::WireEncodable;

use crate::protocol::core::{
    HandshakeMessage,
    PingMessage,
    PongMessage,
    GetPeersMessage,
    PeersMessage,
    DisconnectMessage,
};

use crate::protocol::sync::{
   GetHeadersRequest,
   HeadersResponse,
   GetBlocksRequest,
   BlocksResponse,
   GetStateRequests,
   ChunkStateResponse,
};

use crate::protocol::tx_pool::{
    Transaction,
    GetPooledTransactionRequest,
    PoolTransactionResponse,
};

use crate::protocol::layer2::{
    Layer2Placeholder,
};

use crate::protocol::custom::{
    CustomUnknown,
};

use crate::protocol::{
    CoreMessage,
    SyncMessage,
    TxPoolMessage,
    Layer2Message,
    CustomMessage,
};

wire_message!(CoreMessage {
    Handshake(HandshakeMessage) = 0x00, 
    Ping(PingMessage) = 0x01,
    Pong(PongMessage) = 0x02,
    GetPeers(GetPeersMessage) = 0x03,
    Peers(PeersMessage) = 0x04,
    Disconnect(DisconnectMessage) = 0x05
});

wire_message!(SyncMessage {
    GetHeaders(GetHeadersRequest) = 0x00,
    Headers(HeadersResponse) = 0x01,
    GetBlocks(GetBlocksRequest) = 0x02,
    Blocks(BlocksResponse) = 0x03,
    GetState(GetStateRequests) = 0x04,
    StateChunk(ChunkStateResponse) = 0x05,
});

wire_message!(TxPoolMessage {
    NewTransaction(Transaction) = 0x00,
    GetPooledTransactions(GetPooledTransactionRequest) = 0x01,
    PoolTransactions(PoolTransactionResponse) = 0x02,
});

wire_message!(Layer2Message {
    Placeholder(Layer2Placeholder) = 0x00,
});

wire_message!(CustomMessage {
    Unknown(CustomUnknown) = 0x00,
});

message_dispatch!(
    Core(CoreMessage) = 0x00,
    Sync(SyncMessage) = 0x20,
    TxPool(TxPoolMessage) = 0x40,
    Layer2(Layer2Message) = 0x50,
    Custom(CustomMessage) = 0x70,
);