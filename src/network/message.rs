use crate::crypto::hash::H256;
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Ping(String),
    Pong(String),
    NewBlockHashes(Vec<H256>),
    GetBlocks(Vec<H256>),
    ProposerVoterBlocks(Vec<Vec<u8>>),
    TransactionBlocks(Vec<Vec<u8>>),
    NewTransactionHashes(Vec<H256>),
    GetTransactions(Vec<H256>),
    Transactions(Vec<Transaction>),
}

impl Message {
    pub fn priority(&self) -> usize {
        match self {
            Message::Ping(_) | Message::Pong(_) | Message::NewBlockHashes(_) | Message::GetBlocks(_) => 0,
            Message::ProposerVoterBlocks(_) => 1,
            Message::NewTransactionHashes(_) | Message::GetTransactions(_) | Message::Transactions(_) | Message::TransactionBlocks(_) => 2,
        }
    }
}
