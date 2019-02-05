// This DS serves two purposes. First, it forms multiple longest-chain blocktrees.
// Second, it links all blocks together following the DAG rule defined in the
// paper - the set of nodes are all blocks known to the host, and the set of edges
// are the reference links from a proposer block, as well as reference link from
// a voter block to its parent block
use std::rc::Rc;

type Link = Rc<Node>;       // a link is a smart pointer to a node

pub struct Node {
    pub parent: Option<Link>,   // links to its parent in the blockchain
    pub references: Vec<Link>,  // reference links in the DAG. see the comment above.
    // block: Rc<some block type here>
}

pub struct BlockTree {
    pub genesis: Link,          // points to the genesis block of this tree (chain)
    pub head: Link,
}

impl BlockTree {
    pub fn new() -> Self {
        let genesis = Node {
            parent: None,
            references: vec![],
        };
        let to_genesis = Rc::new(genesis);
        return BlockTree {
            genesis: Rc::clone(&to_genesis),
            head: Rc::clone(&to_genesis),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let tree = BlockTree::new();
        let genesis = Rc::clone(&tree.genesis);
        let head = Rc::clone(&tree.head);
        assert_eq!(genesis.parent.is_none(), true); // check whether it's genesis
        assert_eq!(head.parent.is_none(), true);    // check whether it's genesis
    }

    #[test]
    fn ref_count() {
        let tree = BlockTree::new();
        {
            let _genesis = Rc::clone(&tree.genesis);
            let _head = Rc::clone(&tree.head);
            assert_eq!(Rc::strong_count(&tree.genesis), 4);
            assert_eq!(Rc::strong_count(&tree.head), 4);
        }
        assert_eq!(Rc::strong_count(&tree.genesis), 2);
        assert_eq!(Rc::strong_count(&tree.head), 2);
    }
}
