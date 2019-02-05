pub mod address;
pub mod block_header;
pub mod hash;
pub mod proposer;
pub mod transaction;
pub mod voter;
pub mod block_tree;

pub trait Block {
    fn header(&self) -> &block_header::BlockHeader;
    fn transactions(&self) -> &[transaction::Transaction];
}
