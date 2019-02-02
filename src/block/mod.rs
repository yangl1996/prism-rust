mod block_hash;
mod core_block;

trait Block {
    fn serialize(&self) -> Vec<u8>;
    fn hash(&self) -> block_hash::BlockHash;
}
