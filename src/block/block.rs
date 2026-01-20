use serde::{Deserialize, Serialize};

use std::{fmt::Debug, ops::Add};

use anyhow::Result;

pub trait Block: Debug + Clone + Serialize + for<'de>Deserialize<'de>{
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(&mut self, bytes: Vec<u8>) -> Result<Self>;
}

type Hash = [u8; 32];
type Address = [u8; 20];
type Signature = Vec<u8>;
type PublicKey = Vec<u8>;
type Bytes = Vec<u8>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PoSBlock{
    header: BlockHeader,
    body: BlockBody,
    signature: Signature,
}

#[derive(Serialize, Deserialize ,Debug, Clone)]
struct BlockHeader{
    block_number: u64,
    timetamp: u64,
    parent_hash: Hash,
    block_hash: Hash,

    state_root: Hash,
    transaction_root: Hash,
    receipts_root: Hash,

    proposer_id: u64,
    proposer: Address,
    slot: u64,
    epoch: u64,

    extra_data: Bytes,

    randomness:Hash,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BlockBody{
    transactions: Vec<Transaction>,
    
    attestations: Vec<Attestation>,
    validator_deposits: Vec<ValidatorDeposit>,
    validator_withdrawels: Vec<ValidatorWithdrawl>,
    validator_exits: Vec<ValidatorExit>,

    slashings: Vec<Slashing>,

    randomness_reveal: Bytes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Transaction{
    hash: Hash,
    nonce: u64,

    form: Address,
    to: Option<Address>,

    value: u128,

    data: Bytes,

    fee: TransactionFee,

    signature: TransactionSignature,

    tx_type: u8,

    
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AggregateAttestation{
    slot: u64,
    block_hash: Hash,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ValidatorDeposit{

}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ValidatorWithdrawl{

}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ValidatorExit{

}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Slashing{

}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TransactionFee{
    gas_limit: u64,
    gas_price: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TransactionSignature{
    v: u8, //recovery ID
    r: Hash, //sig component
    s: Hash, //sig component
    public_key: Option<PublicKey>
}