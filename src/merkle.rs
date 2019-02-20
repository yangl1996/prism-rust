use crate::hash::{self, Hashable};

extern crate ring;

/// A Merkle tree.
#[derive(Debug)]
pub struct MerkleTree<'a, T: Hashable> {
    data: &'a [T],
    proof: Vec<hash::Hash>,
}

#[inline]
fn find_parent(me: usize) -> usize {
    return (me - 1) >> 1;
}

#[inline]
fn find_buddy(me: usize) -> usize {
    match me & 0x1 {
        1 => return me + 1,
        _ => return me - 1,
    };
}

#[inline]
fn find_kids(me: usize) -> (usize, usize) {
    return ((me << 1) + 1, (me << 1) + 2);
}

impl<'a, T: Hashable> MerkleTree<'a, T> {
    fn new(data: &'a [T]) -> Self {
        let mut proof: Vec<hash::Hash> = vec![];
        let mut last_row: Vec<hash::Hash> = data.iter().map(|x| x.hash()).collect();
        let mut last_row_size = last_row.len();
        let mut last_row_begin = 0;

        loop {
            // if the last row contains only one element, append it and we're done
            if last_row_size == 1 {
                proof.append(&mut last_row);
                break;
            }
            // if the last row contains odd num of elements, dup the last one
            else if last_row_size & 0x1 == 1 {
                // TODO: more idiomatic way of doing this?
                last_row.push(last_row.last().cloned().unwrap());
                last_row_size += 1;
            }
            // append the last row to the proof
            last_row.reverse();
            proof.append(&mut last_row);

            // construct the next row
            let new_row_size = last_row_size >> 1;
            for i in 0..new_row_size {
                // hash the two kids
                let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
                ctx.update(&proof[last_row_begin + last_row_size - 1 - (i << 1)].0);
                ctx.update(&proof[last_row_begin + last_row_size - 2 - (i << 1)].0);
                let digest = ctx.finish();
                let mut raw_hash: [u8; 32] = [0; 32];
                raw_hash[0..32].copy_from_slice(digest.as_ref());
                last_row.push(raw_hash.into());
            }

            // update ptrs
            last_row_begin += last_row_size;
            last_row_size = new_row_size;
        }
        proof.reverse();
        return MerkleTree {
            data: data,
            proof: proof,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::{self, Hashable};

    #[test]
    fn new_tree() {
        let input_data = vec![
            hash::Hash(hex!(
                "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
            )),
            hash::Hash(hex!(
                "0102010201020102010201020102010201020102010201020102010201020102"
            )),
            hash::Hash(hex!(
                "0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b"
            )),
            hash::Hash(hex!(
                "0403020108070605040302010807060504030201080706050403020108070605"
            )),
            hash::Hash(hex!(
                "1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a"
            )),
            hash::Hash(hex!(
                "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
            )),
            hash::Hash(hex!(
                "0000000100000001000000010000000100000001000000010000000100000001"
            )),
        ];
        let merkle_tree = MerkleTree::new(&input_data);
        assert_eq!(merkle_tree.proof.len(), 15);
        assert_eq!(merkle_tree.proof[0], hash::Hash(hex!("9d8f0638fa3d46f618dea970df55b53a02f4aa924e8d598af6b5f296fdaabce5")));
        assert_eq!(merkle_tree.proof[13], hash::Hash(hex!("b8027a4fc86778e60f636c12e67d03b7356f1d6d8a8ff486bcdaa3dcf81b714b")));
    }
}
