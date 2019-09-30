use crate::crypto::hash::H256;
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Ping(String),
    Pong(String),
    NewBlockHashes(Vec<H256>),
    GetBlocks(Vec<H256>),
    Blocks(Vec<Vec<u8>>),
    NewTransactionHashes(Vec<H256>),
    GetTransactions(Vec<H256>),
    Transactions(Vec<Transaction>),
    Bootstrap(H256),
}
