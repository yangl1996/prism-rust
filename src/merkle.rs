use crate::hash::{self, Hashable};

extern crate ring;

/// A Merkle tree.
#[derive(Debug)]
pub struct MerkleTree<'a, T: Hashable> {
    data: &'a [T],
    nodes: Vec<hash::SHA256>,
}

impl<'a, T: Hashable> MerkleTree<'a, T> {
    pub fn new(data: &'a [T]) -> Self {
        // calculate the size of the tree
        let mut this_layer_size = data.len();
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
        let mut nodes: Vec<hash::SHA256> = vec![Default::default(); tree_size];

        // construct the tree
        let mut layer_start = tree_size;
        let mut layers = layer_size.iter().zip(data_size.iter());

        // fill in the bottom layer
        let (l, d) = layers.next().unwrap();
        layer_start -= l;
        let hashed_data: Vec<hash::SHA256> = data.iter().map(|x| x.sha256()).collect();
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
                ctx.update(&nodes[last_layer_start + (i << 1)].0);
                ctx.update(&nodes[last_layer_start + (i << 1) + 1].0);
                let digest = ctx.finish();
                nodes[layer_start + i] = digest.into();
            }
            if l != d {
                nodes[layer_start + l - 1] = nodes[layer_start + d - 1];
            }
        }

        return MerkleTree {
            data: data,
            nodes: nodes,
        };
    }

    fn root(&self) -> &hash::SHA256 {
        return &self.nodes[0];
    }

    fn proof(&self, data: &T) -> Vec<&hash::SHA256> {
        let mut results = vec![];
        let data_index = self
            .data
            .iter()
            .position(|r| std::ptr::eq(r, data))
            .unwrap();
        let mut known_index = if self.data.len() & 0x01 == 1 {
            self.nodes.len() - self.data.len() - 1 + data_index
        }
        else {
            self.nodes.len() - self.data.len() + data_index
        };
        loop {
            if known_index == 0 {
                break;
            }
            let sibling_index = match known_index & 0x01 {
                1 => known_index + 1,
                _ => known_index - 1,
            };
            results.push(&self.nodes[sibling_index]);
            known_index = (known_index - 1) >> 1;
        }
        return results;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                hash::SHA256(hex!(
                    "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
                )),
                hash::SHA256(hex!(
                    "0102010201020102010201020102010201020102010201020102010201020102"
                )),
                hash::SHA256(hex!(
                    "0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b"
                )),
                hash::SHA256(hex!(
                    "0403020108070605040302010807060504030201080706050403020108070605"
                )),
                hash::SHA256(hex!(
                    "1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a"
                )),
                hash::SHA256(hex!(
                    "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
                )),
                hash::SHA256(hex!(
                    "0000000100000001000000010000000100000001000000010000000100000001"
                )),
            ]
        }};
    }

    #[test]
    fn new_tree() {
        let input_data = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        assert_eq!(merkle_tree.nodes.len(), 15);
        assert_eq!(
            merkle_tree.nodes[0],
            hash::SHA256(hex!(
                "9d8f0638fa3d46f618dea970df55b53a02f4aa924e8d598af6b5f296fdaabce5"
            ))
        );
        assert_eq!(
            merkle_tree.nodes[13],
            hash::SHA256(hex!(
                "b8027a4fc86778e60f636c12e67d03b7356f1d6d8a8ff486bcdaa3dcf81b714b"
            ))
        );
    }

    #[test]
    fn root() {
        let input_data = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            &hash::SHA256(hex!(
                "9d8f0638fa3d46f618dea970df55b53a02f4aa924e8d598af6b5f296fdaabce5"
            ))
        );
    }

    #[test]
    fn proof() {
        let input_data = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(&input_data[2]);
        assert_eq!(proof[0], &merkle_tree.nodes[10]);
        assert_eq!(proof[1], &merkle_tree.nodes[3]);
        assert_eq!(proof[2], &merkle_tree.nodes[2]);
        assert_eq!(proof.len(), 3);
    }
}
