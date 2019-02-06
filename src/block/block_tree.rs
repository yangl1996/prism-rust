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

    pub fn append(&mut self, new: &Rc<Block>) -> Link {
        let parent_ptr = self.nodes.get(new.parent()).unwrap();
        let mut ref_ptrs = Vec::new();
        for ref_hash in new.reference_links() {
            let ref_ptr = self.nodes.get(ref_hash).unwrap();
            ref_ptrs.push(Rc::clone(ref_ptr));
        }
        let new_node = Node {
            parent: Some(Rc::clone(parent_ptr)),
            references: ref_ptrs,
            block: Rc::clone(new),
        };
        let pointer_to_new_node = Rc::new(new_node);
        self.nodes.insert(new.hash(), Rc::clone(&pointer_to_new_node));
        return pointer_to_new_node;
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
        ( $parent_hash:expr, $ref_links:expr, $nonce:expr ) => {{
            let metadata = proposer::ProposerMetadata {
                level_cert: $parent_hash,
                ref_links: $ref_links,
            };
            proposer::ProposerBlock {
                header: block_header::BlockHeader {
                    voter_hash: hash::Hash([0; 32]),
                    proposal_hash: metadata.hash(),
                    transactions_hash: hash::Hash([0; 32]),
                    nonce: $nonce,
                },
                transactions: vec![],
                metadata: metadata,
            }
        }};
    }

    macro_rules! fake_voter {
        ( $parent_hash:expr, $nonce:expr ) => {{
            let metadata = voter::VoterMetadata {
                votes: vec![],
                parent_merkle_root: hash::Hash([0; 32]),
                parent_proofs: vec![],
                parent: $parent_hash,
            };
            voter::VoterBlock {
                header: block_header::BlockHeader {
                    voter_hash: metadata.hash(),
                    proposal_hash: hash::Hash([0; 32]),
                    transactions_hash: hash::Hash([0; 32]),
                    nonce: $nonce,
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
            vec![],
            1
        );
        let proposer_gptr: Rc<Block> = Rc::new(genesis_proposer);

        let mut voter_gptrs = Vec::new();
        let g_voter_1 = fake_voter!(hash::Hash(hex!(
            "1111111111111111111111111111111111111111111111111111111111111111"
        )), 2);
        let g_voter_2 = fake_voter!(hash::Hash(hex!(
            "1111111111111111111111111111111111111111111111111111111111111112"
        )), 3);
        let g_voter_3 = fake_voter!(hash::Hash(hex!(
            "1111111111111111111111111111111111111111111111111111111111111113"
        )), 4);
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

    #[test]
    fn append() {
        let gp = fake_proposer!(
            hash::Hash(hex!(
                "1122334455667788112233445566778811223344556677881122334455667788"
            )),
            vec![], 1
        );
        let p_gp: Rc<Block> = Rc::new(gp);

        let mut voter_gptrs = Vec::new();
        let gv1 = fake_voter!(hash::Hash(hex!(
            "1111111111111111111111111111111111111111111111111111111111111111"
        )), 2);
        let gv2 = fake_voter!(hash::Hash(hex!(
            "1111111111111111111111111111111111111111111111111111111111111112"
        )), 3);
        let gv3 = fake_voter!(hash::Hash(hex!(
            "1111111111111111111111111111111111111111111111111111111111111113"
        )), 4);
        let p_gv1: Rc<Block> = Rc::new(gv1);
        voter_gptrs.push(&p_gv1);
        let p_gv2: Rc<Block> = Rc::new(gv2);
        voter_gptrs.push(&p_gv2);
        let p_gv3: Rc<Block> = Rc::new(gv3);
        voter_gptrs.push(&p_gv3);

        let mut dag = BlockDAG::new(&p_gp, voter_gptrs);

        let p1 = fake_proposer!(p_gp.hash(), vec![], 5);
        let p_p1: Rc<Block> = Rc::new(p1);
        let n_p1 = dag.append(&p_p1);
        let v1 = fake_voter!(p_gv1.hash(), 6);
        let p_v1: Rc<Block> = Rc::new(v1);
        let n_v1 = dag.append(&p_v1);
        let v2 = fake_voter!(p_v1.hash(), 7);
        let p_v2: Rc<Block> = Rc::new(v2);
        let n_v2 = dag.append(&p_v2);
        let v3 = fake_voter!(p_v1.hash(), 8);
        let p_v3: Rc<Block> = Rc::new(v3);
        let n_v3 = dag.append(&p_v3);
        let p2 = fake_proposer!(p_p1.hash(), vec![p_v1.hash(), p_v2.hash()], 9);
        let p_p2: Rc<Block> = Rc::new(p2);
        let n_p2 = dag.append(&p_p2);
        // should look like this
        //        GP          GV1         GV2          GV3
        //        |           |
        //        p1    --->  v1
        //        |     |    / \
        //        |     |   /   \
        //        p2 ----> v2   v3
        
        // check total number of nodes
        assert_eq!(dag.nodes.len(), 9);

        // check parent ptrs
        assert_eq!(Rc::ptr_eq(n_p1.parent.as_ref().unwrap(), &dag.proposer_tree.genesis), true);
        assert_eq!(Rc::ptr_eq(n_v1.parent.as_ref().unwrap(), &dag.voter_trees[0].genesis), true);
        assert_eq!(Rc::ptr_eq(n_p2.parent.as_ref().unwrap(), &n_p1), true);
        assert_eq!(Rc::ptr_eq(n_v2.parent.as_ref().unwrap(), &n_v1), true);
        assert_eq!(Rc::ptr_eq(n_v3.parent.as_ref().unwrap(), &n_v1), true);

        // TODO: check reference links
        assert_eq!(n_p1.references.len(), 0);
        assert_eq!(n_p2.references.len(), 2);
        assert_eq!(Rc::ptr_eq(&n_p2.references[0], &n_v1), true);
        assert_eq!(Rc::ptr_eq(&n_p2.references[1], &n_v2), true);
        assert_eq!(n_v1.references.len(), 1);
        assert_eq!(Rc::ptr_eq(&n_v1.references[0], &dag.voter_trees[0].genesis), true);
        assert_eq!(n_v2.references.len(), 1);
        assert_eq!(Rc::ptr_eq(&n_v2.references[0], &n_v1), true);
        assert_eq!(n_v3.references.len(), 1);
        assert_eq!(Rc::ptr_eq(&n_v3.references[0], &n_v1), true);
    }
}
