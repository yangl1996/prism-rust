// This DS serves two purposes. First, it forms multiple longest-chain blocktrees.
// Second, it links all blocks together following the DAG rule defined in the
// paper - the set of nodes are all blocks known to the host, and the set of edges
// are the reference links from a proposer block, as well as reference link from
// a voter block to its parent block
use super::Block;
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
    pub fn new(genesis: &Rc<Block>) -> Self {
        let genesis_node = Node {
            parent: None,
            references: vec![],
            block: Rc::clone(genesis),
        };
        let pointer_to_genesis_node = Rc::new(genesis_node);
        return BlockTree {
            genesis: Rc::clone(&pointer_to_genesis_node),
        };
    }

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

#[cfg(test)]
mod tests {
    use super::super::block_header;
    use super::super::hash;
    use super::super::proposer;
    use super::super::Block;
    use super::*;
    use std::rc::Rc;

    macro_rules! fake_proposer {
        () => {
            proposer::ProposerBlock {
                header: block_header::BlockHeader {
                    voter_hash: hash::Hash([1; 32]),
                    proposal_hash: hash::Hash([2; 32]),
                    transactions_hash: hash::Hash([3; 32]),
                    nonce: 12345,
                },
                transactions: vec![],
                metadata: proposer::ProposerMetadata {
                    level_cert: hash::Hash(hex!(
                        "0102030405060708010203040506070801020304050607080102030405060708"
                    )),
                    ref_links: vec![],
                },
            }
        };
    }

    #[test]
    fn new() {
        let genesis_proposer = fake_proposer!();
        let genesis_pointer: Rc<Block> = Rc::new(genesis_proposer);
        let tree = BlockTree::new(&genesis_pointer);
        let genesis = Rc::clone(&tree.genesis);
        assert_eq!(genesis.parent.is_none(), true);
        assert_eq!(
            genesis.block.hash(),
            hash::Hash(hex!(
                "29e6703a080f122e9ac455aedfbe9bd1974492df74f88ad970c07b824d4ea292"
            ))
        );
    }

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
}
