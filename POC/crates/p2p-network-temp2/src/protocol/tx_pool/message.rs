use serde::{Serialize, Deserialize};

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