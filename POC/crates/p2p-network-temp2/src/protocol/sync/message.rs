use serde::{Serialize, Deserialize};

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