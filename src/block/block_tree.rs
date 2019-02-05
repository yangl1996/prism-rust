// This DS serves two purposes. First, it forms multiple longest-chain blocktrees.
// Second, it links all blocks together following the DAG rule defined in the
// paper - the set of nodes are all blocks known to the host, and the set of edges
// are the reference links from a proposer block, as well as reference link from
// a voter block to its parent block
use super::hash::Hash;
use super::Block;
use std::collections::HashMap;
use std::rc::Rc;

type Link = Rc<Node>; // a link is a smart pointer to a node

pub struct Node {
    pub parent: Option<Link>,  // links to its parent in the blockchain
    pub references: Vec<Link>, // reference links in the DAG. see the comment above.
    pub block: Rc<Block>,      // pointer to the block struct itself
}

pub struct BlockTree {
    pub genesis: Link, // points to the genesis block of this tree (chain)
}

impl BlockTree {
    pub fn append(&self, parent: &Link, new: &Rc<Block>) -> Link {
        let new_node = Node {
            parent: Some(Rc::clone(parent)),
            references: vec![],
            block: Rc::clone(new),
        };
        let pointer_to_new_node = Rc::new(new_node);
        return pointer_to_new_node;
    }
}

pub struct BlockDAG {
    pub proposer_tree: BlockTree,
    pub voter_trees: Vec<BlockTree>,
    pub nodes: HashMap<Hash, Link>,
}

impl BlockDAG {
    pub fn new(proposer_genesis: &Rc<Block>, voter_genesises: Vec<&Rc<Block>>) -> Self {
        // init hashmap
        let mut nodes: HashMap<Hash, Link> = HashMap::new();

        // init proposer tree
        let proposer_genesis_node = Node {
            parent: None,
            references: vec![],
            block: Rc::clone(proposer_genesis),
        };
        let ptr_proposer_genesis_node = Rc::new(proposer_genesis_node);
        let proposer_tree = BlockTree {
            genesis: Rc::clone(&ptr_proposer_genesis_node),
        };
        nodes.insert(proposer_genesis.hash(), ptr_proposer_genesis_node);

        // init voter trees
        let mut voter_trees = Vec::new();
        for voter_g in voter_genesises {
            let voter_genesis_node = Node {
                parent: None,
                references: vec![],
                block: Rc::clone(voter_g),
            };
            let ptr_voter_genesis_node = Rc::new(voter_genesis_node);
            voter_trees.push(BlockTree {
                genesis: Rc::clone(&ptr_voter_genesis_node),
            });
            nodes.insert(voter_g.hash(), ptr_voter_genesis_node);
        }
        return BlockDAG {
            proposer_tree: proposer_tree,
            voter_trees: voter_trees,
            nodes: nodes,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::super::block_header;
    use super::super::hash;
    use super::super::hash::Hashable;
    use super::super::proposer;
    use super::super::voter;
    use super::super::Block;
    use super::*;
    use std::rc::Rc;

    macro_rules! fake_proposer {
        ( $parent_hash:expr, $ref_links:expr ) => {{
            let metadata = proposer::ProposerMetadata {
                level_cert: $parent_hash,
                ref_links: $ref_links,
            };
            proposer::ProposerBlock {
                header: block_header::BlockHeader {
                    voter_hash: hash::Hash([0; 32]),
                    proposal_hash: metadata.hash(),
                    transactions_hash: hash::Hash([0; 32]),
                    nonce: 12345,
                },
                transactions: vec![],
                metadata: metadata,
            }
        }};
    }

    macro_rules! fake_voter {
        ( $parent_hashes:expr ) => {{
            let metadata = voter::VoterMetadata {
                votes: vec![],
                parent_links: $parent_hashes,
            };
            voter::VoterBlock {
                header: block_header::BlockHeader {
                    voter_hash: metadata.hash(),
                    proposal_hash: hash::Hash([0; 32]),
                    transactions_hash: hash::Hash([0; 32]),
                    nonce: 54321,
                },
                transactions: vec![],
                metadata: metadata,
            }
        }};
    }

    #[test]
    fn new() {
        let genesis_proposer = fake_proposer!(
            hash::Hash(hex!(
                "1122334455667788112233445566778811223344556677881122334455667788"
            )),
            vec![]
        );
        let proposer_gptr: Rc<Block> = Rc::new(genesis_proposer);

        let mut voter_gptrs = Vec::new();
        let g_voter_1 = fake_voter!(vec![hash::Hash(hex!(
            "1111111111111111111111111111111111111111111111111111111111111111"
        ))]);
        let g_voter_2 = fake_voter!(vec![hash::Hash(hex!(
            "1111111111111111111111111111111111111111111111111111111111111112"
        ))]);
        let g_voter_3 = fake_voter!(vec![hash::Hash(hex!(
            "1111111111111111111111111111111111111111111111111111111111111113"
        ))]);
        let gptr_voter_1: Rc<Block> = Rc::new(g_voter_1);
        voter_gptrs.push(&gptr_voter_1);
        let gptr_voter_2: Rc<Block> = Rc::new(g_voter_2);
        voter_gptrs.push(&gptr_voter_2);
        let gptr_voter_3: Rc<Block> = Rc::new(g_voter_3);
        voter_gptrs.push(&gptr_voter_3);

        let dag = BlockDAG::new(&proposer_gptr, voter_gptrs);

        // check all four blocks we constructed exists in the hashmap
        assert_eq!(dag.nodes.contains_key(&proposer_gptr.hash()), true);
        assert_eq!(dag.nodes.contains_key(&gptr_voter_1.hash()), true);
        assert_eq!(dag.nodes.contains_key(&gptr_voter_2.hash()), true);
        assert_eq!(dag.nodes.contains_key(&gptr_voter_3.hash()), true);
        // check for a random hash
        assert_eq!(
            dag.nodes.contains_key(&hash::Hash(hex!(
                "1234123412341234123412341234123412341234123412341234123412341234"
            ))),
            false
        );

        // check all genesis blocks are in place
        assert_eq!(dag.proposer_tree.genesis.block.hash(), proposer_gptr.hash());
        assert_eq!(dag.voter_trees[0].genesis.block.hash(), gptr_voter_1.hash());
        assert_eq!(dag.voter_trees[1].genesis.block.hash(), gptr_voter_2.hash());
        assert_eq!(dag.voter_trees[2].genesis.block.hash(), gptr_voter_3.hash());
    }

    /*
           #[test]
           fn ref_count() {
           let genesis_proposer = fake_proposer!();
           let genesis_pointer: Rc<Block> = Rc::new(genesis_proposer);
           let tree = BlockTree::new(&genesis_pointer);
           {
           let _genesis = Rc::clone(&tree.genesis);
           assert_eq!(Rc::strong_count(&tree.genesis.block), 2);
           assert_eq!(Rc::strong_count(&tree.genesis), 2);
           }
           assert_eq!(Rc::strong_count(&tree.genesis.block), 2);
           assert_eq!(Rc::strong_count(&tree.genesis), 1);
           }

           #[test]
           fn append() {
           let genesis_proposer = fake_proposer!();
           let genesis_pointer: Rc<Block> = Rc::new(genesis_proposer);
           let tree = BlockTree::new(&genesis_pointer);
           let block_1 = fake_proposer!();
           let block_1_ptr: Rc<Block> = Rc::new(block_1);
           let block_1_node = tree.append(&tree.genesis, &block_1_ptr);
           let block_2 = fake_proposer!();
           let block_2_ptr: Rc<Block> = Rc::new(block_2);
           let block_2_node = tree.append(&block_1_node, &block_2_ptr);
           let block_3 = fake_proposer!();
           let block_3_ptr: Rc<Block> = Rc::new(block_3);
           let block_3_node = tree.append(&block_1_node, &block_3_ptr);
    // should look like this
    //             ----3
    // G----1----<
    //             ----2
    assert_eq!(Rc::ptr_eq(block_3_node.parent.as_ref().unwrap(), &block_1_node), true);
    assert_eq!(Rc::ptr_eq(block_2_node.parent.as_ref().unwrap(), &block_1_node), true);
    assert_eq!(Rc::ptr_eq(block_1_node.parent.as_ref().unwrap(), &tree.genesis), true);
    }
         */
}
