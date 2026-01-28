const MAX_MESSAGE_SIZE: usize = 65536;

const GET_PEERS_VOLUME: usize = 20;

use tokio::{
    io::AsyncReadExt, 
    net::{TcpListener, TcpStream}, 
    sync::{RwLock,mpsc::{self, UnboundedReceiver}}
};

use std::{
    collections::HashMap, 
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
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
    PeerDisconnect,

    #[error("Connection timeout")]
    Timeout,

    #[error("Peer Already Connected")]
    PeerAlreadyConnected,

    #[error("Peer Not Found")]
    PeerNotFound
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

    peers: Arc<RwLock<HashMap<[u8;32], Peer>>>,

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
                peer_count < self.config.max_peers {
                 let needed = self.config.max_outbound_peers - outbound_count;

                 let candidates = self.peer_db.read().await
                    .get_candidates(needed)
                    .await;   
                }

                for addr in candidates{
                    let network = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = network.connect_to_peer(&addr).await {
                            eprintln!("Failed to connect to {}: {:?}", addr, e);
                        }
                    });
                }
        }
    }

    pub async fn connect_to_peer(self: &Arc<Self>, addr: &str) -> Result<(), P2PError> {
        println!("Connecting to peer: {}", addr);

        let stream = tokio::time::timeout(
            Duration::from_secs(10),
             TcpStream::connect(addr)
            ).await
                .map_err(|_| P2PError::Timeout)??;
            handle_connection(stream, true, Arc::clone(&self)).await
    }

    async fn bootstrap(self: &Arc<Self>) {
        for addr in &self.config.bootstrap_nodes {
            let network = Arc::clone(&self);
            let addr = addr.clone();
            
            tokio::spawn(async move {
                if let Err(e) = network.connect_to_peer(&addr).await {
                    eprintln!("Failed to connect to bootstrap node {}: {:?}", addr, e);
                }
            });
        }
    }

    async fn ping_manager(self: Arc<Self>) {
        let mut interval = tokio::time::interval(
            Duration::from_secs(self.config.ping_interval_secs)
        );

        loop {
            interval.tick().await;

            let peers: Vec<[u8; 32]> = self.peers.read().await
                .keys()
                .copied()
                .collect();

            for peer_id in peers {
                let nonce = rand::random::<u64>();
                let ping = PingMessage { nonce }
                
                if let Err(e) = self.send_to_peer(
                    peer_id,
                    Message::Core(CoreMessage::Ping(ping))
                ).await {
                    eprintln!("Failed to ping peer {:?}: {:?}", peer_id, e);
                }
            }
        }
    }

    pub async fn add_peer(&self, peer: Peer) -> Result<(), P2PError> {
        let peer_id = peer.peer_id;

        if self.peers.read().await.contains_key(&peer_id) {
            return Err(P2PError::PeerAlreadyConnected)
        }

        self.peer_db.write().await.add_peer(
            peer_id,
            peer.address.clone(),
            peer.port,
        ).await;

        println!("Peer {} connected (protocol v{}, {} capabilities",
            hex::encode(peer_id),
            peer.protocol_version,
            peer.capabilities.len()
        );

        self.peers.write().await.insert(peer_id, peer);

        Ok(())
    }

    pub async fn remove_peer(&self, peer_id: [u8; 32]) {
        if let Some(peer) = self.peers.write().await.remove(&peer_id) {
            println!("Peer {} disconnected", hex::encode(peer_id));
            
            self.peer_db.write().await.mark_disconnected(peer_id).await;
        }
    }

    pub async fn send_to_peer(
        &self,
        peer_id: [u8; 32],
        msg: Message,
    ) -> Result<(), P2PError> {
        let peers = self.peers.read().await;

        if let Some(peer) = peers.get(&peer_id) {
            peer.outbount_tx.send(msg).await
                .map_err(|_| P2PError::PeerDisconnect)
        } else {
            Err(P2PError::PeerNotFound)
        }
    }

    pub async fn broadcast(&self, msg: Message) {
        let peers = self.peers.read().await;

        for peer in peers.values(){
            let msg_clone = match &msg{
                Message::Core(m) => Message::Core(m.clone()),
                Message::Sync(m) => Message::Sync(m.clone()),
                Message::TxPool(m) => Message::TxPool(m.clone()),
                Message::Layer2(m) => Message::Layer2(m.clone()),
                Message::Custom(m) => Message::Custom(m.clone()),
            };

            if let Err(e) = peer.outbount_tx.send(msg_clone).await {
                eprintln!("Failed to broadcast to peer {:?}: {:?}", peer.peer_id, e)
            }
        }
    }

    pub async fn broadcast_to_capable(&self, capability: &str, msg: Message) {
        let peers = self.peers.read().await;

        for peer in peers.values() {
            if !peer.capabilities.contains(&capability.to_string()) {
                continue;
            }

            let msg_clone = match &msg {
                Message::Core(m) => Message::Core(m.clone()),
                Message::Sync(m) => Message::Sync(m.clone()),
                Message::TxPool(m) => Message::TxPool(m.clone()),
                Message::Layer2(m) => Message::Layer2(m.clone()),
                Message::Custom(m) => Message::Custom(m.clone()),
            };

            if let Err(e) = peer.outbount_tx.send(msg_clone).await {
                eprintln!("Failed to broadcast to peer {:?}: {:?}", peer.peer_id, e)
            }
        }
    }

    pub async fn get_known_peers(&self, limit: usize) -> Vec<PeerInfo> {
        self.peer_db.read().await.get_best_peers(limit).await
    }

    pub async fn add_discovered_peers(&self, peers: Vec<PeerInfo>) {
        let mut db = self.peer_db.write().await;
        for peer in peers {
            db.add_peer(peer.peer_id, peer.address, peer.port).await;
        }
    }

    pub async fn peer_count(&self) -> usize{
        self.peers.read().await.len()
    }

    pub async fn get_peer_stats(&self) -> PeerStats{
        let peers = self.peers.read().await;

        let total = peers.len();
        let inbound = peers.values().filter(|p| !p.is_outbound).count();
        let outbound = peers.values().filter(|p| p.is_outbound).count();

        PeerStats{
            total,
            inbound,
            outbound
        }
    }
}

#[derive(Debug, Clone)]
pub struct PeerStats{
    pub total: usize,
    pub inbound: usize,
    pub outbound: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PeerInfo{
    pub peer_id: [u8; 32],
    pub address: String,
    pub port: u16,
}

struct PeerRecord{
    peer_id: [u8; 32],
    address: String,
    port: u16,
    last_seen: u64,
    last_attempted: u64,
    successful_connections: u32,
    failed_connections: u32,
    reputation: i32,
}

pub struct PeerDatabase{
    peers: HashMap<[u8;32], PeerRecord>
}

impl PeerDatabase{
    pub fn new() -> Self {
        Self { 
            peers: HashMap::new() 
        }
    }

    pub async fn add_peer(&mut self, peer_id: [u8; 32], address: String, port: u16) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.peers.entry(peer_id) 
            .and_modify(|record| {
                record.last_seen = now;
                record.successful_connections += 1;
                record.reputation += 1;
            })
            .or_insert(PeerRecord {
                 peer_id, 
                 address, 
                 port, 
                 last_seen: now, 
                 last_attempted: 0, 
                 successful_connections: 1, 
                 failed_connections: 0, 
                 reputation: 10 
                });
    }

    pub async fn mark_disconnected(&mut self, peer_id: [u8; 32]) {
        if let Some(record) = self.peers.get_mut(&peer_id) {
            record.failed_connections += 1;
            record.reputation -= 2;

            let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        record.last_seen = now;
        }
    }

    pub async fn mark_failed(&mut self, peer_id: [u8; 32]) {
        if let Some(record) = self.peers.get_mut(&peer_id) {
            record.failed_connections += 1;
            record.reputation -= 2;

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            record.last_attempted = now;
        }
    }

    pub async fn get_candidates(&self, count: usize) -> Vec<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut candidates: Vec<_> = self.peers.values()
            .filter(|r| {
                r.reputation > 0 &&
                now - r.last_attempted > 60
            })
            .collect();

        candidates.sort_by_key(|r| -r.reputation);

        candidates.iter()
            .take(count)
            .map(|r| format!("{}:{}", r.address, r.port))
            .collect()
    }

    pub async fn get_best_peers(&self, limit: usize) -> Vec<PeerInfo> {
        let mut peers: Vec<_> = self.peers.values()
            .filter(|r| r.reputation > 0)
            .collect();

        peers.sort_by_key(|r| -r.reputation);

        peers.iter()
            .take(limit)
            .map(|r| PeerInfo {
                peer_id: r.peer_id,
                address: r.address.clone(),
                port: r.port,
            })
            .collect()
    }
}