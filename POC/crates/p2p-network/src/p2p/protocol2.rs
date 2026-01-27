const MAX_MESSAGE_SIZE: usize = 65536;

const GET_PEERS_VOLUME: usize = 20;

use tokio::{
    io::AsyncReadExt, 
    net::TcpListener, 
    sync::{RwLock,mpsc}
};

use std::{
    collections::HashMap, 
    sync::Arc,
    time::Duration,
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use thiserror::Error;

use async_trait::async_trait;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message{
    Core(CoreMessage),
    Sync(SyncMessage),
    TxPool(TxPoolMessage),
    Layer2(Layer2Message),
    Custom(CustomMessage),
}

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
    nonce: [u8; 32]
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PongMessage{
    nonce: [u8; 32]
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPeersMessage;

#[derive(Serialize, Deserialize, Debug)]
pub struct PeersMessage;

#[derive(Serialize, Deserialize, Debug)]
pub struct DisconnectMessage;

#[derive(Serialize, Deserialize, Debug)]
pub enum SyncMessage{
    GetHeaders(GetHeadersRequest),
    Headers(HeadersResponse),
    GetBlocks(GetBlocksRequest),
    Blocks(BlocksResponse),
    GetState(GetStateRequests),
    StateChunk(ChunkStateResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetHeadersRequest;

#[derive(Serialize, Deserialize, Debug)]
pub struct HeadersResponse;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetBlocksRequest;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlocksResponse;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetStateRequests;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkStateResponse;

#[derive(Serialize, Deserialize, Debug)]
pub enum TxPoolMessage{
    NewTransaction(Transaction),
    GetPooledTransactions(GetPooledTransactionRequest),
    PoolTransactions(PoolTransactionResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPooledTransactionRequest;

#[derive(Serialize, Deserialize, Debug)]
pub struct PoolTransactionResponse;

#[derive(Serialize, Deserialize, Debug)]
pub enum Layer2Message{
    Placeholder(Layer2Placeholder)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Layer2Placeholder;

#[derive(Serialize, Deserialize, Debug)]
pub enum CustomMessage{
    Unknown(CustomUnknown)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomUnknown{
    msg_type: u8,
    payload: Vec<u8>
}

trait WireEncodable: Sized{
    fn encode(&self) -> Result<Vec<u8>, P2PError>;
    fn decode(payload: &[u8]) -> Result<Self, P2PError>;
}

impl <T> WireEncodable for T
where T: Serialize + DeserializeOwned{
    fn encode(&self) -> Result<Vec<u8>, P2PError> {
        Ok(postcard::to_allocvec(self)?)
    }
    fn decode(payload: &[u8]) -> Result<Self, P2PError> {
        Ok(postcard::from_bytes(payload)?)
    }
}

macro_rules! wire_message {
    ($enum_name:ident {
        $($variant:ident($type:ty) =$id:expr), * $(,)?
    }) => {
        impl $enum_name{
            fn to_wire(&self) -> Result<(u8, Vec<u8>), P2PError>{
                match self {
                    $(
                        $enum_name::$variant(msg) => Ok(($id, msg.encode()?)),
                    )*
                }
            }

            fn from_wire(type_id: u8, payload: &[u8]) -> Result<Self, P2PError>
            where 
                $($type: WireEncodable, )*
            {
                match type_id{
                    $(
                        $id => Ok($enum_name::$variant(<$type>::decode(payload)?)),
                    )*
                    _ => Err(P2PError::UnknownMessageType(type_id))
                }
            }
        }
    }
}

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

macro_rules! message_dispatch {
    ($($variant:ident($type:ty) = $category:expr), * $(,)?) => {
        impl Message{
            pub fn to_bytes(&self) -> Result<Vec<u8>, P2PError> {
                let (category, type_id, payload) = match self {
                    $(
                        Message::$variant(msg) => {
                            let (id, payload) = msg.to_wire()?;
                            ($category, id, payload)
                        }
                    )*
                };

                let msg_type = category + type_id;
                let total_len = 1 + payload.len();
                let mut buf = Vec::with_capacity(4 + total_len);
                buf.extend_from_slice(&(total_len as u32).to_be_bytes());
                buf.push(msg_type);
                buf.extend_from_slice(&payload);
                Ok(buf)
            }

            pub async fn from_stream<R: AsyncReadExt + Unpin>(
                stream: &mut R,
            ) -> Result<Self, P2PError> {
                let mut len_buf = [0u8; 4];
                stream.read_exact(&mut len_buf).await?;
                let msg_len = u32::from_be_bytes(len_buf) as usize;

                if msg_len == 0 || msg_len > MAX_MESSAGE_SIZE {
                    return Err(P2PError::InvalidMessageSize(msg_len))
                }

                let msg_type = stream.read_u8().await?;
                let payload_len = msg_len - 1;
                let mut payload = vec![0u8; payload_len];
                stream.read_exact(&mut payload).await?;

                let category = msg_type & 0xF0;
                let type_id = msg_type & 0x0F;

                match category{
                    $(
                        $category => Ok(Message::$variant(<$type>::from_wire(type_id, &payload)?)),
                    )*
                    _ => Err(P2PError::UnknownMessageCategory(category))
                }
        }
        }

        
    }
}

message_dispatch!(
    Core(CoreMessage) = 0x00,
    Sync(SyncMessage) = 0x20,
    TxPool(TxPoolMessage) = 0x40,
    Layer2(Layer2Message) = 0x50,
    Custom(CustomMessage) = 0x70,
);

#[derive(Error, Debug)]
pub enum P2PError{
    #[error("Unknown message type: 0x{0:02x}")]
    UnknownMessageType(u8),

    #[error("Unknow message category: 0x{0:02x}")]
    UnknownMessageCategory(u8),

    #[error("Invalid message size: {0}")]
    InvalidMessageSize(usize),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IOError(String),

    #[error("Protocol Violation")]
    ProtocolViolation,

    #[error("Peer Disconnected")]
    PeerDisconnect
}

impl From<postcard::Error> for P2PError{
    fn from(e: postcard::Error) -> Self {
        P2PError::SerializationError(format!("Postcard error: {:?}", e))   
    }
}

impl From<std::io::Error> for P2PError{
    fn from(e: std::io::Error) -> Self {
        P2PError::IOError(format!("IO Error: {}", e))
    }
}


async fn handle_message(
    peer_id: [u8; 32],
    msg: Message,
    network: &Network
) -> Result<(), P2PError> {
    match msg {
        Message::Core(core_msg) => handle_core_message(peer_id, core_msg, network).await,
        Message::Sync(sync_msg) => handle_sync_message(peer_id, sync_msg, network).await,
        Message::TxPool(tx_msg) => handle_txpool_message(peer_id, tx_msg, network).await,
        Message::Layer2(l2_msg) => handle_layer2_message(peer_id, l2_msg, network).await,
        Message::Custom(custom_msg) => handle_custom_message(peer_id, custom_msg, network).await,
    }
}

async fn handle_core_message(
    peer_id: [u8; 32],
    msg: CoreMessage,
    network: &Network
) -> Result<(), P2PError>{
    match msg{
        CoreMessage::Handshake(_) => {
            Err(P2PError::ProtocolViolation)
        }

        CoreMessage::Ping(ping) => {
            let pong = PongMessage { nonce: ping.nonce};
            network.send_to_peer(peer_id, Message::Core(CoreMessage::Pong(pong))).await
        }
        CoreMessage::Pong(pong) => {
            network.handle_pong(peer_id, pong).await;
            Ok(())
        }
        CoreMessage::GetPeers(_) => {
            let peers = network.get_known_peers(GET_PEERS_VOLUME).await;
            network.send_to_peer(peer_id, Message::Core(CoreMessage::Peers(peers))).await
        }
        CoreMessage::Peers(peers) => {
            network.add_discovered_peers(peers).await;
            Ok(())
        }
        CoreMessage::Disconnect(msg) => {
            println!("Peer {} disconnection: {:?}", hex::encode(peer_id), msg.reason);
            Err(P2PError::PeerDisconnect)
        }
    }
}

async fn handle_sync_message(
    peer_id: [u8; 32],
    msg: SyncMessage,
    network: &Network
) -> Result<(), P2PError> {
    network.sync_handler.handle(peer_id, msg).await
}

async fn handle_txpool_message(
    peer_id: [u8; 32],
    msg: TxPoolMessage,
    network: &Network
) -> Result<(), P2PError> {
    network.txpool_handler.handle(peer_id, msg).await
}

async fn handle_layer2_message(
    peer_id: [u8; 32],
    msg: Layer2Message,
    network: &Network
) -> Result<(), P2PError> {
    network.layer2_handler.handle(peer_id, msg).await
}

async fn handle_custom_message(
    peer_id: [u8; 32],
    msg: CustomMessage,
    network: &Network
) -> Result<(), P2PError> {
    println!("Received custom message from peer {}: {:?}", hex::encode(peer_id), msg);
    Ok(())
}

#[async_trait]
pub trait MessageHandler<M> {
    async fn handle(&self, peer_id: [u8; 32], msg: M) -> Result<(), P2PError>;
}

pub struct SyncHandler {
    blockchain: Arc<Blockchain>,
}

#[async_trait]
impl MessageHandler<SyncMessage> for SyncHandler {
    async fn handle(&self, peer_id: [u8; 32], msg: SyncMessage) -> Result<(), P2PError> {
        match msg {
            SyncMessage::GetHeaders(req) => {
                let headers = self.blockchain.get_headers(req.start, req.limit).await?;
                // Send response back to peer
                Ok(())
            }
            SyncMessage::Headers(resp) => {
                self.blockchain.process_headers(peer_id, resp.headers).await?;
                Ok(())
            }
            SyncMessage::GetBlocks(req) => {
                // Handle block request
                Ok(())
            }
            SyncMessage::Blocks(resp) => {
                // Process blocks
                Ok(())
            }
            SyncMessage::GetState(req) => {
                // Handle state request
                Ok(())
            }
            SyncMessage::StateChunk(resp) => {
                // Process state chunk
                Ok(())
            }
        }
    }
}

struct Blockchain;

pub struct Network {
    pub node_id: [u8; 32],

    pub config: NetworkConfig,

    peers: Arc<RwLock<PeerDatabase>>,

    peer_db: Arc<RwLock<PeerDatabase>>,

    pub sync_handler: Arc<SyncHandler>,
    pub txpool_handler: Arc<TxPoolHandler>,
    pub layer2_handler: Arc<Layer2Handler>,

    pub blockchain: Arc<Blockchain>,

    shutdown_tx: mpsc::Sender<()>,
}


#[derive(Clone)]
pub struct NetworkConfig{
    pub protocol_version: u32,
    pub min_protocol_version: u32,

    pub network_id: u64,
    
    pub listen_addr: String,
    pub listen_port: u16,

    pub external_addr: Option<String>,

    pub max_peers: usize,
    pub max_inbound_peers: usize,
    pub max_outbound_peers: usize,

    pub handshake_timeout_secs: u64,
    pub ping_interval_secs: u64,
    pub ping_timeout_secs: u64,

    pub capabilities: Vec<String>,
    pub required_capabilities: Vec<String>,

    pub bootstrap_nodes: Vec<String>
}

impl Default for NetworkConfig{
    fn default() -> Self {
        Self { 
            protocol_version: 1, 
            min_protocol_version: 1, 
            network_id: 1, 
            listen_addr: "0.0.0.0".to_string(), 
            listen_port: 30303, 
            external_addr: None, 
            max_peers: 50, 
            max_inbound_peers: 25, 
            max_outbound_peers: 25, 
            handshake_timeout_secs: 10, 
            ping_interval_secs: 15, 
            ping_timeout_secs: 30, 
            capabilities: vec!["sync/1".to_string(), "txpool/1".to_string()], 
            required_capabilities: vec!["sync/1".to_string()], 
            bootstrap_nodes: vec![] 
        }
    }
}

pub struct Peer{
    pub peer_id: [u8; 32],
    pub address: String,
    pub port: u16,
    pub outbount_tx: mpsc::Sender<Message>,
    pub is_outbound: bool,
    pub protocol_version: u32,
    pub capabilities: Vec<String>,
    pub best_block_height: u64,
    pub best_block_hash: [u8; 32],
    pub connect_at: std::time::Instant,
    pub last_seen: Arc<RwLock<std::time::Instant>>,
}

impl Network{
    pub fn new(
        node_id: [u8; 32],
        config: NetworkConfig,
        blockchain: Arc<Blockchain>
    )-> Arc<Self> {
        let (shutdown_tx, _) = mpsc::channel(1);

        let peer_db = Arc::new(RwLock::new(PeerDatabase::new()));

        Arc::new(Self {
            node_id,
            config,
            peers: Arc::new(RwLock::new(HashMap::new())),
            peer_db,
            sync_handler: Arc::new(SyncHandler::new(Arc::clone(&blockchain))),
            txpool_handler: Arc::new(TxPoolHandler::new()),
            layer2_handler: Arc::new(Layer2Handler::new()),
            blockchain,
            shutdown_tx,
        })

    }   
    pub async fn start(self: Arc<Self>) -> Result<(), P2PError>{
        let listener = TcpListener::bind(
            format!("{}:{}", self.config.listen_addr, self.config.listen_port)
        ).await?;

        println!("P2P listening on {}:{}", self.config.listen_addr, self.config.listen_port);

        let network = Arc::clone(&self);
        tokio::spawn(async move {
            network.accept_loop(listener).await;
        });

        let network = Arc::clone(&self);
        tokio::spawn(async move {
            network.connection_manager().await;
        });

        let network = Arc::clone(&self);
        tokio::spawn(async move {
            network.ping_manager().await;
        });

        self.bootstrap().await;

        Ok(())
    }


    async fn accept_loop(self: Arc<Self>, listener: TcpListener) {
        loop{
            match listener.accept().await{
                Ok((stream, addr)) => {
                    println!("Incoming connection from {}", addr);

                    let peer_count = self.peers.read().await.len();
                    let inbount_count = self.peers.read().await
                        .values()
                        .filter(|p| !p.is_outbound)
                        .count();

                    if peer_count >= self.config.max_peers || 
                        inbount_count >= self.config.max_peers {
                       
                        println!("Rejecting connection from {} - too many peers", addr);
                        drop(stream);
                        continue;
                    }

                    let network = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, false, network).await{
                            eprintln!("Connection error: {:?}", e);
                        }
                    });


        
                }
                Err(e) => {
                    eprintln!("Accept errror: {:?}", e);
                }
            }
        }
    }

    async fn connection_manager(self: Arc<Self>) {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop{
            interval.tick().await;

            let peer_count: usize = self.peers.read().await.len();
            let outbound_count = self.peers.read().await
                .values()
                .filter(|p| p.is_outbound)
                .count();

            if outbound_count < self.config.max_outbound_peers &&
                peer_count < self.config.max_peers
        }
    }
}