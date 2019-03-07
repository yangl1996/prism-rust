use super::hash::{Hashable, H256};

/// A Merkle tree.
#[derive(Debug)]
pub struct MerkleTree<'a, T: Hashable> {
    data: &'a [T],
    nodes: Vec<H256>,
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

    fn root(&self) -> &H256 {
        return &self.nodes[0];
    }

    fn proof(&self, data: &T) -> Vec<&H256> {
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
