extern crate ring;

pub struct Block {
    transactions: [Transaction; 16], // each block holds 16 txn
    parent: BlockHash,
    transaction_blocks: [BlockHash; 4], // each block points to 4 txn blocks
    nonce: u32,
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

pub struct Transaction;

