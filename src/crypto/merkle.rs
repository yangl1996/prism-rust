use super::hash::{Hashable, H256};

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    data_size: Vec<usize>,
    nodes: Vec<H256>,
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable {
        // calculate the size of the tree
        let mut this_layer_size = data.len();

        // todo: Added by Vivek. Lei check this
        // What default behaviour do we want?
        if this_layer_size == 0 {
            return Self {
                data_size: vec![this_layer_size],
                nodes: vec![],
            };
        }
        let mut layer_size = vec![]; // size after dup
        let mut data_size = vec![]; // size before dup
        loop {
            data_size.push(this_layer_size);
            if this_layer_size == 1 {
                layer_size.push(this_layer_size);
                break;
            }
            if this_layer_size & 0x01 == 1 {
                this_layer_size += 1;
            }
            layer_size.push(this_layer_size);
            this_layer_size = this_layer_size >> 1;
        }
        let tree_size = layer_size.iter().sum();

        // allocate the tree
        let mut nodes: Vec<H256> = vec![Default::default(); tree_size];

        // construct the tree
        let mut layer_start = tree_size;
        let mut layers = layer_size.iter().zip(data_size.iter());

        // fill in the bottom layer
        let (l, d) = layers.next().unwrap();
        layer_start -= l;
        let hashed_data: Vec<H256> = data.iter().map(|x| x.hash()).collect();
        nodes[layer_start..layer_start + d].copy_from_slice(&hashed_data);
        if l != d {
            nodes[layer_start + l - 1] = nodes[layer_start + d - 1];
        }

        // fill in other layers
        for (l, d) in layers {
            let last_layer_start = layer_start;
            layer_start -= l;
            for i in 0..*d {
                let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
                let left_hash: [u8; 32] = (&nodes[last_layer_start + (i << 1)]).into();
                let right_hash: [u8; 32] = (&nodes[last_layer_start + (i << 1) + 1]).into();
                ctx.update(&left_hash[..]);
                ctx.update(&right_hash[..]);
                let digest = ctx.finish();
                nodes[layer_start + i] = digest.into();
            }
            if l != d {
                nodes[layer_start + l - 1] = nodes[layer_start + d - 1];
            }
        }

        return MerkleTree {
            data_size: data_size,
            nodes: nodes,
        };
    }

    pub fn root(&self) -> H256 {
        if self.nodes.len() == 0 {
            return (&[0; 32]).into();
        } else {
            return self.nodes[0];
        }
    }

    /// Returns the Merkle Proof of data at index i
    // todo: Lei check this
    pub fn proof(&self, index: usize) -> Vec<H256> {
        if self.data_size.len() == 1 || index >= self.data_size[0] { return vec![]; }
        let mut results = vec![];
        let mut known_index = if self.data_size[0] & 0x01 == 1 {
            self.nodes.len() - self.data_size[0] - 1 + index
        } else {
            self.nodes.len() - self.data_size[0] + index
        };
        loop {
            if known_index == 0 {
                break;
            }
            let sibling_index = match known_index & 0x01 {
                1 => known_index + 1,
                _ => known_index - 1,
            };
            results.push(self.nodes[sibling_index]);
            known_index = (known_index - 1) >> 1;
        }
        results
    }

    pub fn update<T>(&mut self, index: usize, data: &T) where T: Hashable {
        if index >= self.data_size[0] { return; }
        if self.data_size[0] == 1 {
            self.nodes[0] = data.hash();
            return;
        }
        let last_layer_start = if self.data_size[0] & 0x01 == 1 {
            self.nodes.len() - self.data_size[0] - 1
        } else {
            self.nodes.len() - self.data_size[0]
        };
        let mut known_index = last_layer_start + index;
        let mut layer_start = last_layer_start;
        let mut idx = 0usize;
        loop {
            if known_index >= last_layer_start {
                self.nodes[known_index] = data.hash();
            } else {
                self.nodes[known_index] = {
                    let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
                    let left_hash: [u8; 32] = (&self.nodes[(known_index << 1) + 1]).into();
                    let right_hash: [u8; 32] = (&self.nodes[(known_index << 1) + 2]).into();
                    ctx.update(&left_hash[..]);
                    ctx.update(&right_hash[..]);
                    let digest = ctx.finish();
                    digest.into()
                };
            }
            if known_index == 0 {
                break;
            }
            if known_index == layer_start + self.data_size[idx] - 1 && self.data_size[idx] & 0x01 == 1 {
                self.nodes[known_index + 1] = self.nodes[known_index];
            }
            layer_start = (layer_start - 1) >> 1;
            idx += 1;
            known_index = (known_index - 1) >> 1;
        }
    }
}

/// Verify that the data hash with a vector of proofs will produce the Merkle root. Also need the
/// index of data and `data_size`, the total number of leaves.
pub fn verify(root: &H256, data: &H256, proof: &[H256], index: usize, data_size: usize) -> bool {
    let mut this_layer_size = data_size;
    let mut layer_size = vec![];
    loop {
        if this_layer_size == 1 {
            layer_size.push(this_layer_size);
            break;
        }
        if this_layer_size & 0x01 == 1 {
            this_layer_size += 1;
        }
        layer_size.push(this_layer_size);
        this_layer_size = this_layer_size >> 1;
    }
    if layer_size.len() != proof.len() + 1 { return false; }
    let mut iter = layer_size.into_iter();
    iter.next();
    let mut index: usize = iter.sum::<usize>() + index;
    let mut acc = *data;
    for h in proof.iter() {
        if index == 0 { return false; }
        let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
        let acc_: [u8; 32] = (&acc).into();
        let h: [u8; 32] = h.into();
        if index & 0x01 == 1 {
            ctx.update(&acc_[..]);
            ctx.update(&h[..]);
        } else {
            ctx.update(&h[..]);
            ctx.update(&acc_[..]);
        }
        let digest = ctx.finish();
        acc = digest.into();
        index = (index - 1) >> 1;
    }
    acc == *root
}

#[cfg(test)]
mod tests {
    use super::super::hash;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (&hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (&hex!("0102010201020102010201020102010201020102010201020102010201020102")).into(),
                (&hex!("0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b")).into(),
                (&hex!("0403020108070605040302010807060504030201080706050403020108070605")).into(),
                (&hex!("1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a")).into(),
                (&hex!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")).into(),
                (&hex!("0000000100000001000000010000000100000001000000010000000100000001")).into(),
            ]
        }};
    }

    #[test]
    fn new_tree() {
        let input_data: Vec<hash::H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        assert_eq!(merkle_tree.nodes.len(), 15);
        assert_eq!(
            merkle_tree.nodes[0],
            (&hex!("9d8f0638fa3d46f618dea970df55b53a02f4aa924e8d598af6b5f296fdaabce5")).into()
        );
        assert_eq!(
            merkle_tree.nodes[13],
            (&hex!("b8027a4fc86778e60f636c12e67d03b7356f1d6d8a8ff486bcdaa3dcf81b714b")).into()
        );
    }

    #[test]
    fn root() {
        let input_data: Vec<hash::H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (&hex!("9d8f0638fa3d46f618dea970df55b53a02f4aa924e8d598af6b5f296fdaabce5")).into()
        );
    }

    #[test]
    fn proof() {
        let input_data: Vec<hash::H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(2);
        assert_eq!(proof[0], merkle_tree.nodes[10]);
        assert_eq!(proof[1], merkle_tree.nodes[3]);
        assert_eq!(proof[2], merkle_tree.nodes[2]);
        assert_eq!(proof.len(), 3);
        assert!(verify(&merkle_tree.root(), &input_data[2].hash(), &proof, 2, input_data.len()));

        let proof = merkle_tree.proof(6);
        assert_eq!(proof[0], merkle_tree.nodes[14]);
        assert_eq!(proof[1], merkle_tree.nodes[5]);
        assert_eq!(proof[2], merkle_tree.nodes[1]);
        assert_eq!(proof.len(), 3);
        assert!(verify(&merkle_tree.root(), &input_data[6].hash(), &proof, 6, input_data.len()));

        let wrong_proof: Vec<H256> = proof.iter().take(2).cloned().collect();
        assert!(!verify(&merkle_tree.root(), &input_data[6].hash(), &wrong_proof, 6, input_data.len()));
        let mut wrong_proof: Vec<H256> = proof.clone();
        wrong_proof[0] = [09u8; 32].into();
        assert!(!verify(&merkle_tree.root(), &input_data[6].hash(), &wrong_proof, 6, input_data.len()));

    }

    #[test]
    fn proof_and_update() {
        for top in 0..=7usize {
            let input_data: Vec<hash::H256> = gen_merkle_tree_data!().into_iter().take(top).collect();
            let merkle_tree = MerkleTree::new(&input_data);
            for idx in 0..input_data.len() {
                //proof
                let proof = merkle_tree.proof(idx);
                assert!(verify(&merkle_tree.root(), &input_data[idx].hash(), &proof, idx, input_data.len()));
                //update
                let mut input_data_mut = input_data.clone();
                input_data_mut[idx] = [09u8; 32].into();
                let mut merkle_tree_mut = MerkleTree::new(&input_data_mut);
                assert_ne!(merkle_tree.root(), merkle_tree_mut.root());
                merkle_tree_mut.update(idx, &input_data[idx]);
                assert_eq!(merkle_tree.root(), merkle_tree_mut.root());
            }
            if top>1 {
                let input_data_: Vec<hash::H256> = input_data.iter().rev().cloned().collect();
                let mut merkle_tree_ = MerkleTree::new(&input_data_);
                assert_ne!(merkle_tree.root(), merkle_tree_.root());
                for idx in 0..input_data.len() {
                    merkle_tree_.update(idx, &input_data[idx]);
                }
                assert_eq!(merkle_tree.root(), merkle_tree_.root());
            }
        }
    }
}
