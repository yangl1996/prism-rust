extern crate ring;
extern crate byteorder;
use byteorder::{ByteOrder, LittleEndian};

pub struct Block {
    //pub transactions: [Transaction; 16], // each block holds 16 txn
    pub parent: BlockHash,
    //pub transaction_blocks: [BlockHash; 4], // each block points to 4 txn blocks
    pub nonce: u32,
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Block {{\n")?;
        //write!(f, "  transactions: not implemented")?;
        write!(f, "  parent: {}\n", self.parent)?;
        //write!(f, "  transaction blocks: not implemented")?;
        write!(f, "  nonce: {}\n", self.nonce)?;
        write!(f, "}}")
    }
}

impl Block {
    pub fn serialized(&self) -> [u8; 36] {
        // little-endian parent hash + little-endian nonce
        let mut serialized: [u8; 36] = [0; 36];
        for idx in 0..32 {
            serialized[idx] = self.parent.0[31-idx];
        }
        LittleEndian::write_u32(&mut serialized[32..36], self.nonce);
        return serialized;
    }

    pub fn hash(&self) -> BlockHash {
        // TODO: we don't specifically arrange the bytes in the Block
        // struct, so the hash depends on how ring serializes the bytes
        let serialized = self.serialized();
        let digest = ring::digest::digest(&ring::digest::SHA256, &serialized);
        let mut raw_hash: [u8; 32] = [0; 32];
        raw_hash[..32].clone_from_slice(digest.as_ref());
        return BlockHash(raw_hash);
    }
}

#[derive(Eq)]
pub struct BlockHash(pub [u8; 32]); // big endian u256

impl Ord for BlockHash {
    fn cmp(&self, other: &BlockHash) -> std::cmp::Ordering {
        for byte_idx in 0..31 {
            let res = self.0[byte_idx].cmp(&other.0[byte_idx]);
            match res {
                std::cmp::Ordering::Equal => {
                    continue;
                },
                _ => {
                    return res;
                },
            }
        }
        return self.0[31].cmp(&other.0[31]);
    }
}

impl PartialOrd for BlockHash {
    fn partial_cmp(&self, other: &BlockHash) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BlockHash {
    fn eq(&self, other: &BlockHash) -> bool {
        for byte_idx in 0..32 {
            if self.0[byte_idx] != other.0[byte_idx] {
                return false;
            }
        }
        return true;
    }
}

impl std::fmt::Display for BlockHash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte_idx in 0..32 {
            write!(f, "{:>02x}", self.0[byte_idx])?;
        }
        Ok(())
    }
}

//pub struct Transaction;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blockhash_ordering() {
        let bigger_blockhash = BlockHash([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                         0, 0, 0, 0, 0, 0, 0, 1]);
        let smaller_blockhash = BlockHash([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                          0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bigger_blockhash > smaller_blockhash, true);

        let bigger_blockhash = BlockHash([0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                         0, 0, 0, 0, 0, 0, 0, 0]);
        let smaller_blockhash = BlockHash([0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                          0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bigger_blockhash > smaller_blockhash, true);

        let some_blockhash = BlockHash([0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                         0, 0, 0, 0, 0, 0, 0, 0]);
        let same_blockhash = BlockHash([0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                          0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(some_blockhash >= same_blockhash, true);
        assert_eq!(some_blockhash <= same_blockhash, true);
        assert_eq!(some_blockhash == same_blockhash, true);
    }

    #[test]
    fn block_serialization() {
        let block = Block {
            parent: BlockHash([10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
                               10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
                               10, 10, 10, 10, 10, 10, 10, 10, 10, 9]),
            nonce: 12345,
        };
        let serialized = block.serialized();
        let should_be: [u8; 36] = [9, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
                                   10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
                                   10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 57,
                                   48, 0,  0];
        let mut comp = true;
        for idx in 0..36 {
            print!("| {}, {} |", serialized[idx], should_be[idx]);
            if serialized[idx] != should_be[idx] {
                comp = false;
            }
        }
        assert_eq!(comp, true);
    }

    #[test]
    fn block_hash() {
        let block = Block {
            parent: BlockHash([10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
                               10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
                               10, 10, 10, 10, 10, 10, 10, 10, 10, 9]),
            nonce: 12345,
        };
        let hash = block.hash();
        let should_be = BlockHash([0x58, 0x87, 0xa8, 0xbc, 0xba, 0xde, 0x11,
                                   0x0e, 0x4b, 0xa7, 0x8f, 0x95, 0x01, 0xe6,
                                   0x39, 0xd0, 0x5d, 0x1f, 0xf9, 0xdf, 0xd6,
                                   0xab, 0xd2, 0x1f, 0x93, 0x69, 0x56, 0xa8,
                                   0x08, 0xab, 0xbb, 0xfb]);
        assert_eq!(hash == should_be, true);
    }
}
