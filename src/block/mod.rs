pub mod address;
pub mod block_header;
pub mod block_tree;
pub mod hash;
pub mod proposer;
pub mod transaction;
pub mod voter;

pub trait Block {
    fn header(&self) -> &block_header::BlockHeader;
    fn hash(&self) -> hash::Hash;
    fn reference_links(&self) -> &[hash::Hash];
    fn parent(&self) -> &hash::Hash;
}
