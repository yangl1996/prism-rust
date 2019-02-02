extern crate ring;

#[derive(Eq, Serialize, Deserialize)]
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
}
