use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message{
    Core(CoreMessage),
    Sync(SyncMessage),
    TxPool(TxPoolMessage),
    Layer2(Layer2Message),
    Custom(CustomMessage),
}