pub mod address;
pub mod block_header;
pub mod hash;
pub mod transaction;

pub trait Block {
    fn header(&self) -> &block_header::BlockHeader;
}
