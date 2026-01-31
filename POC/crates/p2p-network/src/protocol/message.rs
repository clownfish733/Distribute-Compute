use serde::{Serialize, Deserialize};

use super::{
    core::CoreMessage,
    sync::SyncMessage,
    tx_pool::TxPoolMessage,
    layer2::Layer2Message,
    custom::CustomMessage,
};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message{
    Core(CoreMessage),
    Sync(SyncMessage),
    TxPool(TxPoolMessage),
    Layer2(Layer2Message),
    Custom(CustomMessage),
}

