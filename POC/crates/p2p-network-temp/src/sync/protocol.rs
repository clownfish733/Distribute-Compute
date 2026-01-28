use tokio::io::AsyncReadExt;

pub enum Message{
    Core(Box<CoreMessage>),
    Sync(Box<SyncMessage>),
    TxPool(Box<TxPoolMessage>),
    Layer2(Box<Layer2Message>),
    Custom(Box<CustomMessage>),
}

pub enum CoreMessage{
    Handshake(HandshakeMessage),
    Ping(PingMessage),
    Pong(PongMessage),
    GetPeers,
    Peers(Vec<PeerInfo>),
    Disconnect(DisconnectMessage),
}


pub struct HandshakeMessage;

pub struct PingMessage;

struct PongMessage;

pub struct PeerInfo;

pub struct DisconnectMessage;

pub enum SyncMessage{
    GetHeaders(GetHeadersRequest),
    Headers(HeadersResponse),
    GetBlocks(GetBlocksRequest),
    Blocks(BlocksResponse),
    GetState(GetStateRequests),
    StateChunk(ChunkStateResponse),
}

pub struct GetHeadersRequest;

pub struct HeadersResponse;

pub struct GetBlocksRequest;

pub struct BlocksResponse;

pub struct GetStateRequests;

pub struct ChunkStateResponse;

pub enum TxPoolMessage{
    NewTransaction(Transaction),
    GetPooledTransactions(Vec<[u8; 32]>),
    PoolTransactions(Vec<Transaction>),
}

pub struct Transaction;

pub enum Layer2Message{
    Placeholder(Vec<u8>)
}

pub enum CustomMessage{
    Unknown {msg_type: u8, payload: Vec<u8>}
}
/*

impl Message{
    pub fn to_bytes(&self) -> Result<Vec<u8>, P2PError>{
        let (category, type_id, payload) = match self{
            Message::Core(msg) => {
                let (id, payload) = msg.to_wire()?;
                (0x00u8, id, payload)
            }
            Message::Sync(msg) => {
                let (id, payload) = msg.to_wire()?;
                (0x20u8, id, payload)
            }
            Message::TxPool(msg) => {
                let (id, payload) = msg.to_wire()?;
                (0x40u8, id, payload)
            }
            Message::Layer2(msg) => {
                let (id, payload) = msg.to_wire()?;
                (0x50u8, id, payload)
            }
            Message::Custom(msg) => {
                let (id, payload) = msg.to_wire()?;
                (0x70u8, id, payload)
            }
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
        stream: &mut R
    ) -> Result<Self, P2PError> {

        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        if msg_len ==0 || msg_len > MAX_MESSAGE_SIZE {
            return Err(P2PError::InvalidMessageSize(msg_len));
        }

        let msg_type = stream.read_u8().await?;

        let payload_len = msg_len - 1;
        let mut payload = vec![0u8; payload_len];
        stream.read_exact(&mut payload).await?;

        let category = msg_type & 0xF0;
        let type_id = msg_type & 0x0F;

        match category{
            0x00 => Ok(Message::Core(CoreMessage::from_wire(type_id, &payload)?)),
            0x20 => Ok(Message::Sync(SyncMessage::from_wire(type_id, &payload)?)),
            0x40 => Ok(Message::TxPool(TxPoolMessage::from_wire(type_id, &payload)?)),
            0x50 => Ok(Message::Layer2(Layer2Message::from_wire(type_id, &payload)?)),
            0x70..=0xFF => Ok(Message::Custom(CustomMessage::from_wire(type_id, &payload)?)),
            _ => Err(P2PError::UnknownMessageCategory(category))
        }

    }
}

impl CoreMessage{
    fn to_wire(&self) -> Result<(u8, Vec<u8>), P2PError>{
        match self {
            CoreMessage::Handshake(msg) => Ok((0x00, msg.encode()?)),
            CoreMessage::Ping(msg) => Ok((0x01, msg.encode()?)),
            CoreMessage::Pong(msg) => Ok((0x02, msg.encode()?)),
            CoreMessage::GetPeers => Ok((0x03, vec![])),
            CoreMessage::Peers(peers) => Ok((0x04, bincode::serialize(peers)?)),
            CoreMessage::Disconnect(msg) => Ok((0x05, msg.encode()?)),
        }
    }
    
    fn from_wire(type_id: u8, payload: &[u8]) -> Result<Self, P2PError>{
        match type_id{
            0x00 => Ok(CoreMessage::Handshake(HandshakeMessage::decode(payload)?)),
            0x01 => Ok(CoreMessage::Ping(PingMessage::decode(payload)?)),
            0x02 => Ok(CoreMessage::Pong(PongMessage::decode(payload)?)),
            0x03 => Ok(CoreMessage::GetPeers),
            0x04 => Ok(CoreMessage::Peers(bincode::deserialize(payload)?)),
            0x05 => Ok(CoreMessage::Disconnect(DisconnectMessage::decode(payload)?)),
            _ => Err(P2PError::UnknownMessageType(type_id)),
        }
    }
}

*/