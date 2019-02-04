pub mod block_header;
pub mod hash;
pub mod address;

pub trait Block {
    fn header(&self) -> &block_header::BlockHeader;
}
