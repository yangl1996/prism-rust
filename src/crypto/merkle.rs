use super::hash::H256;

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    nodes: Vec<Vec<H256>>,
}

impl MerkleTree {
    pub fn new(data: Vec<H256>) -> Self {
        let mut prev_layer_size: usize = data.len();
        let mut nodes: Vec<Vec<H256>> = vec![data];
        let mut layer: usize = 0;
        loop {
            if prev_layer_size < 2 {
                break;
            }
            let mut this_layer: Vec<H256> = vec![];
            // below `- 1` is for odd size case
            for i in (0..prev_layer_size - 1).step_by(2) {
                let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
                let left_hash: [u8; 32] = (&nodes[layer][i]).into();
                let right_hash: [u8; 32] = (&nodes[layer][i+1]).into();
                ctx.update(&left_hash[..]);
                ctx.update(&right_hash[..]);
                let digest = ctx.finish();
                this_layer.push(digest.into());
            }
            if prev_layer_size & 0x01 == 1 {
                // if layer size is odd, we duplicate the last node
                let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
                let left_hash: [u8; 32] = (&nodes[layer][prev_layer_size - 1]).into();
                ctx.update(&left_hash[..]);
                ctx.update(&left_hash[..]);
                let digest = ctx.finish();
                this_layer.push(digest.into());
            } 
            prev_layer_size = this_layer.len();
            nodes.push(this_layer);
            layer += 1;
        }

        MerkleTree {
            nodes,
        }
    }

    pub fn append(&mut self, data: &mut Vec<H256>) {
        if data.is_empty() {return;}
        let mut append_begin = self.nodes[0].len();
        // if it's odd, minus 1 to make it even
        append_begin -= append_begin & 0x01;
        self.nodes[0].append(data);
        let mut prev_layer_size: usize = self.nodes[0].len();
        let mut layer: usize = 1;
        loop {
            if prev_layer_size < 2 {
                break;
            }
            if layer == self.nodes.len() {
                self.nodes.push(vec![]);
            }
            let origin_this_layer_size = self.nodes[layer].len();
            // below `- 1` is for odd size case
            for i in (append_begin ..prev_layer_size - 1).step_by(2) {
                let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
                let left_hash: [u8; 32] = (&self.nodes[layer - 1][i]).into();
                let right_hash: [u8; 32] = (&self.nodes[layer - 1][i+1]).into();
                ctx.update(&left_hash[..]);
                ctx.update(&right_hash[..]);
                let digest = ctx.finish();
                if origin_this_layer_size > 0 && (i >> 1) == origin_this_layer_size - 1 {
                    self.nodes[layer][origin_this_layer_size - 1] = digest.into();
                } else {
                    self.nodes[layer].push(digest.into());
                }
            }
            if prev_layer_size & 0x01 == 1 {
                // if layer size is odd, we duplicate the last node
                let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
                let left_hash: [u8; 32] = (&self.nodes[layer - 1][prev_layer_size - 1]).into();
                ctx.update(&left_hash[..]);
                ctx.update(&left_hash[..]);
                let digest = ctx.finish();
                if origin_this_layer_size > 0 && prev_layer_size >> 1 == origin_this_layer_size - 1 {
                    self.nodes[layer][origin_this_layer_size - 1] = digest.into();
                } else {
                    self.nodes[layer].push(digest.into());
                }
            } 
            prev_layer_size = self.nodes[layer].len();
            layer += 1;
            append_begin = append_begin >> 1;
            append_begin -= append_begin & 0x01;
        }
    }

    pub fn root(&self) -> H256 {
        if let Some(vec) = self.nodes.last() {
            if let Some(root) = vec.first() {
                return *root;
            }
        }
        Default::default()
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        if self.nodes.len() == 1 || index >= self.nodes[0].len() { return vec![]; }
        let mut results = vec![];
        let mut layer: usize = 0;
        let mut index = index;
        loop {
            if layer == self.nodes.len() - 1 {
                break;
            }
            results.push(
                if index == self.nodes[layer].len() - 1 && index & 0x01 == 0 {
                    // special case for odd number, duplicate itself
                    self.nodes[layer][index]
                } else {
                    let sibling_index = match index & 0x01 {
                        1 => index - 1,
                        _ => index + 1,
                    };
                    //DELETE:println!("I'm at {}, h: {}, sibling at {}, h: {}",nodes_index,self.nodes[nodes_index],sibling_index, self.nodes[sibling_index]);
                    self.nodes[layer][sibling_index]
                }
            );
            layer += 1;
            index = index >> 1;
        }
        results
    }

    pub fn update(&mut self, index: usize, data: H256) {
        if index >= self.nodes[0].len() { return; }
        let mut layer: usize = 0;
        let mut index = index;
        loop {
            if layer == self.nodes.len() {
                break;
            }
            self.nodes[layer][index] = 
                if layer == 0 {
                    data
                } else {
                    let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
                    let left_index = index << 1;
                    if left_index < self.nodes[layer - 1].len() - 1 {
                        let left_hash: [u8; 32] = (&self.nodes[layer - 1][left_index]).into();
                        let right_hash: [u8; 32] = (&self.nodes[layer - 1][left_index + 1]).into();
                        ctx.update(&left_hash[..]);
                        ctx.update(&right_hash[..]);
                    } else {
                        // special case for odd number, we duplicate this node
                        let left_hash: [u8; 32] = (&self.nodes[layer - 1][left_index]).into();
                        ctx.update(&left_hash[..]);
                        ctx.update(&left_hash[..]);
                    }
                    let digest = ctx.finish();
                    digest.into()
                };
            layer += 1;
            index = index >> 1;
        }
    }
}

/// Verify that the data hash with a vector of proofs will produce the Merkle root. Also need the
/// index of data and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, data: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    if index >= leaf_size { return false; }
    let mut this_layer_size = leaf_size;
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
    //DELETE:println!("Verify, layer size len: {}, proof len: {}", layer_size.len(), proof.len());
    if layer_size.len() != proof.len() + 1 { return false; }
    let mut iter = layer_size.iter();
    iter.next();
    let mut layer_start = iter.sum::<usize>();
    let mut index: usize = index;
    let mut layer = 0;
    let mut acc = *data;
    for h in proof.iter() {
        let nodes_index = layer_start + index;
        if nodes_index  == 0 { return false; }
        let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
        //DELETE:println!("{} and {}", acc, h);
        let acc_: [u8; 32] = (&acc).into();
        let h: [u8; 32] = h.into();
        if nodes_index & 0x01 == 1 {
            ctx.update(&acc_[..]);
            ctx.update(&h[..]);
        } else {
            ctx.update(&h[..]);
            ctx.update(&acc_[..]);
        }
        let digest = ctx.finish();
        acc = digest.into();
        //DELETE:println!("\t= {}", acc);
        layer += 1;
        layer_start = layer_start - layer_size[layer];
        index = index  >> 1;
    }
    acc == *root
}

#[cfg(test)]
mod tests {
    use super::super::hash::{Hashable, H256};
    use super::*;
    use crate::crypto::hash::tests::generate_random_hash;

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
/*
    #[test]
    fn new_tree() {
        let input_data: Vec<hash::H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        assert_eq!(
            merkle_tree.nodes[0],
            (&hex!("9d8f0638fa3d46f618dea970df55b53a02f4aa924e8d598af6b5f296fdaabce5")).into()
        );
        assert_eq!(
            merkle_tree.nodes[13],
            (&hex!("b8027a4fc86778e60f636c12e67d03b7356f1d6d8a8ff486bcdaa3dcf81b714b")).into()
        );
    }
*/
    #[test]
    fn root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let input_hash: Vec<H256> = input_data.iter().map(|x|x.hash()).collect();
        let merkle_tree = MerkleTree::new(input_hash);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (&hex!("9d8f0638fa3d46f618dea970df55b53a02f4aa924e8d598af6b5f296fdaabce5")).into()
        );
    }
    #[test]
    fn proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let input_hash: Vec<H256> = input_data.iter().map(|x|x.hash()).collect();
        let merkle_tree = MerkleTree::new(input_hash);
        let proof = merkle_tree.proof(2);
        assert_eq!(proof.len(), 3);
        assert!(verify(&merkle_tree.root(), &input_data[2].hash(), &proof, 2, input_data.len()));

        let proof = merkle_tree.proof(6);
        assert_eq!(proof.len(), 3);
        assert!(verify(&merkle_tree.root(), &input_data[6].hash(), &proof, 6, input_data.len()));

        let wrong_proof: Vec<H256> = proof.iter().take(2).cloned().collect();
        assert!(!verify(&merkle_tree.root(), &input_data[6].hash(), &wrong_proof, 6, input_data.len()));
        let mut wrong_proof: Vec<H256> = proof.clone();
        wrong_proof[0] = [09u8; 32].into();
        assert!(!verify(&merkle_tree.root(), &input_data[6].hash(), &wrong_proof, 6, input_data.len()));

    }

    #[test]
    fn large_proof() {
        let limit = 100usize;
        let mut input_data = vec![];
        for _ in 0..limit {
            input_data.push(generate_random_hash());
        }
        let input_hash: Vec<H256> = input_data.iter().map(|x|x.hash()).collect();
        let merkle_tree = MerkleTree::new(input_hash);
        for idx in 0..limit {
            let proof = merkle_tree.proof(idx);
            assert!(verify(&merkle_tree.root(), &input_data[idx].hash(), &proof, idx, input_data.len()));
        }
    }

    #[test]
    fn update() {
        for top in 0..=7usize {
            let input_data: Vec<H256> = gen_merkle_tree_data!().into_iter().take(top).collect();
            let input_hash: Vec<H256> = input_data.iter().map(|x|x.hash()).collect();
            let merkle_tree = MerkleTree::new(input_hash);
            for idx in 0..input_data.len() {
                //update
                let mut input_data_mut = input_data.clone();
                input_data_mut[idx] = [09u8; 32].into();
                let input_hash: Vec<H256> = input_data_mut.iter().map(|x|x.hash()).collect();
                let mut merkle_tree_mut = MerkleTree::new(input_hash);
                assert_ne!(merkle_tree.root(), merkle_tree_mut.root());
                merkle_tree_mut.update(idx, input_data[idx].hash());
                assert_eq!(merkle_tree.root(), merkle_tree_mut.root());
            }
        }
    }

    #[test]
    fn large_update() {
        let limit = 100usize;
        let mut input_data = vec![];
        for _ in 0..limit {
            input_data.push(generate_random_hash());
        }
        let input_hash: Vec<H256> = input_data.iter().map(|x|x.hash()).collect();
        let mut merkle_tree = MerkleTree::new(input_hash);
        let root = merkle_tree.root();
        for idx in 0..limit {
            merkle_tree.update(idx, [09u8;32].into());
            assert_ne!(merkle_tree.root(), root);
            merkle_tree.update(idx, input_data[idx].hash());
            assert_eq!(merkle_tree.root(), root);
        }
    }

    #[test]
    fn append() {
        let top = 6usize;
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let input_data_minus: Vec<H256> = input_data.iter().take(top).cloned().collect();
        let input_hash: Vec<H256> = input_data.iter().map(|x|x.hash()).collect();
        let input_hash_minus: Vec<H256> = input_data_minus.iter().map(|x|x.hash()).collect();
        let merkle_tree = MerkleTree::new(input_hash);
        let mut merkle_tree_minus = MerkleTree::new(input_hash_minus);
        assert_ne!(merkle_tree.root(), merkle_tree_minus.root());
        merkle_tree_minus.append(&mut vec![input_data[top].hash()]);
        assert_eq!(merkle_tree.root(), merkle_tree_minus.root());
    }

    #[test]
    fn large_append() {
        let limit = 100usize;
        let mut input_data = vec![];
        for _ in 0..limit {
            input_data.push(generate_random_hash());
        }
        let input_hash: Vec<H256> = input_data.iter().map(|x|x.hash()).collect();
        let merkle_tree = MerkleTree::new(input_hash.clone());
        for top in 0..limit-1 {
            //create merkle tree using only top inputs
            let input_hash_minus: Vec<H256> = input_hash.iter().take(top).cloned().collect();
            let mut merkle_tree_minus = MerkleTree::new(input_hash_minus);
            assert_ne!(merkle_tree.root(), merkle_tree_minus.root());
            //append the other inputs
            let mut input_hash_plus: Vec<H256> = input_hash.iter().skip(top).cloned().collect();
            merkle_tree_minus.append(&mut input_hash_plus);
            //check result should be the same
            assert_eq!(merkle_tree.root(), merkle_tree_minus.root());
        }
    }
}
