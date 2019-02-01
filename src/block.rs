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
        let mut serialized: [u8; 36] = [0; 36];
        serialized[..32].clone_from_slice(&self.parent.0);
        LittleEndian::write_u32(&mut serialized[32..36], self.nonce);
        return serialized;
    }

    /*
    fn hash(&self) -> BlockHash {
        // TODO: we don't specifically arrange the bytes in the Block
        // struct, so the hash depends on how ring serializes the bytes
        let digest = ring::digest::(&ring::digest::SHA256, Block)
    }
    */
}

#[derive(Eq)]
pub struct BlockHash(pub [u8; 32]); // little endian u256

impl Ord for BlockHash {
    fn cmp(&self, other: &BlockHash) -> std::cmp::Ordering {
        for byte_idx in (1..32).rev() {
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
        return self.0[0].cmp(&other.0[0]);
    }
}

impl PartialOrd for BlockHash {
    fn partial_cmp(&self, other: &BlockHash) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BlockHash {
    fn eq(&self, other: &BlockHash) -> bool {
        for byte_idx in (0..32).rev() {
            if self.0[byte_idx] != other.0[byte_idx] {
                return false;
            }
        }
        return true;
    }
}

impl std::fmt::Display for BlockHash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte_idx in (0..32).rev() {
            write!(f, "{:>02x}", self.0[byte_idx])?;
        }
        Ok(())
    }
}

//pub struct Transaction;

