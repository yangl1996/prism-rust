extern crate bincode;

use super::block_hash::BlockHash;

const NUM_TXN_BLOCKS: usize = 4; // how many txn blocks in a core block

#[derive(Serialize, Deserialize)]
pub struct CoreBlock {
    pub parent: BlockHash,
    pub transaction_blocks: [BlockHash; NUM_TXN_BLOCKS],
    pub nonce: u32,
}

impl std::fmt::Display for CoreBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{\n")?;
        //write!(f, "  transactions: not implemented")?;
        write!(f, "  parent: {}\n", self.parent)?;
        write!(f, "  transaction blocks: {{\n")?;
        for txn in &self.transaction_blocks {
            write!(f, "    {},\n", txn)?;
        }
        write!(f, "  }}\n")?;
        write!(f, "  nonce: {}\n", self.nonce)?;
        write!(f, "}}")
    }
}

impl super::Block for CoreBlock {
    fn serialize(&self) -> Vec<u8> {
        return bincode::serialize(&self).unwrap();
    }

    fn hash(&self) -> BlockHash {
        // TODO: we don't specifically arrange the bytes in the Block
        // struct, so the hash depends on how ring serializes the bytes
        let serialized = self.serialize();
        let digest = ring::digest::digest(&ring::digest::SHA256, &serialized);
        let mut raw_hash: [u8; 32] = [0; 32];
        raw_hash[..32].clone_from_slice(digest.as_ref());
        return BlockHash(raw_hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::Block;

    #[test]
    fn block_hash() {
        let block = CoreBlock {
            parent: BlockHash([5; 32]),
            transaction_blocks: [
                BlockHash([1; 32]),
                BlockHash([2; 32]),
                BlockHash([4; 32]),
                BlockHash([3; 32]),
            ],
            nonce: 12345,
        };
        let hash = block.hash();
        let should_be = BlockHash([
            0xa3, 0x90, 0x46, 0xd3, 0xaf, 0xfa, 0x8b, 0x05, 0xe6, 0x20, 0x80, 0xe2, 0x67, 0x21,
            0x92, 0xef, 0x04, 0x7a, 0x15, 0xf9, 0xd7, 0x81, 0x84, 0xcb, 0x0b, 0x0c, 0x0d, 0x30,
            0xdf, 0x8f, 0x8e, 0x55,
        ]);
        assert_eq!(hash == should_be, true);
    }
}
