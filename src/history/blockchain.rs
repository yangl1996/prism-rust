use super::block::block::{Block, BlockType};
use super::crypto::hash::{self, Hashable, SHA256};
use std::collections::{HashSet};

pub enum BlockId {
    Hash(SHA256),
}


/*
Ignore this
/// The status of different blocks.
pub enum TxBlockRefStatus{
    /// When a proposer block has referenced it
    Referenced,
    /// When none of the proposer blocks have referenced it
    UnReferenced
}
pub type PropBlockRefStatus = TxBlockRefStatus;


/// The content of blocks. This is a placeholder until Guilia is done with Mining function.
pub struct TxBlockContent{
    // List of votes on prop blocks.
}

impl Hashable for VoterBlockContent {
    fn sha256(&self) -> SHA256 {
        let x: [u8; 32] = [0; 32]; // Default (wrong) behaviour
        return SHA256(x);
    }
}

pub type PropBlockContent = TxBlockContent;
pub type VoterBlockContent = TxBlockContent;


type TxBlock = Block<TxBlockContent>;
type PropBlock = Block<PropBlockContent>;
type VoterBlock = Block<VoterBlockContent>;

*/

pub enum PropBlockLeaderStatus{
    Leader,
    MaybeLeader,
    NotLeader
}

pub enum VoterBlockStatus{
    OnMainChain,
    Orphan
}

// Todo: Import enum from block
pub enum NodeType{
    Transaction,
    Proposer,
    Voter,
}

pub trait Node{
    fn get_type() -> NodeType;
}

pub struct TxNode{
    /// Block Id
    block_id : BlockId,
    /// Parent prop block
    parent_prop_block_id: PropNode,
    /// Prop block which refers this block
    child_prop_block_id: PropNode,
}

impl Node for TxNode{
    fn get_type() -> NodeType{ return NodeType::Transaction }
}



pub struct PropNode{
    /// Block Id
    block_id : BlockId,
    /// Parent prop block
    parent_prop_block_id: Box<PropNode>,
    /// Level of the proposer block
    level: u32,
    /// List of Prop blocks which refer this block
    children_prop_block_id: Vec<Box<PropNode>>,
    /// List of Prop blocks referred by this block
    referred_prop_block_ids: Vec<Box<PropNode>>,
    /// List of Voter blocks voted
    votes_block_ids: Vec<VoterNode>,
    /// Leadership Status
    leadership_status: PropBlockLeaderStatus
}

impl PropNode{
    fn change_leadership_status(&mut self, new_status: PropBlockLeaderStatus){
        self.leadership_status = new_status;
    }
}

impl Node for PropNode{
    fn get_type() -> NodeType{ return NodeType::Proposer }
}



pub struct VoterNode{
    /// The chain of the voter block
    chain_id: u16,
    /// Block Id
    block_id : BlockId,
    /// The parent on its chain
    parent: Box<VoterNode>,
    /// Height from the genesis block
    level: u32
}


impl Node for VoterNode{
    fn get_type() -> NodeType{ return NodeType::Voter }
}


/// Stores all the tx nodes
pub struct TxPool{
    /// Set of all transaction nodes
    tx_nodes: HashSet<TxNode>
}

impl TxPool{
    /// Initialize Tx pool
//    pub fn new() -> Self{
//        let tx_nodes: HashSet<TxNode> = HashSet::new();
//        return TxPool{tx_nodes};
//    }

    /// Add a tx block
    pub fn add_tx_block(&mut self, node: TxNode){
//        self.tx_nodes.insert(node); Todo: Define a hash insert ???
    }
}
/// Stores all the prop nodes
pub struct PropTree{
    /// Genesis block
    genesis_block: PropNode,
    /// Best block on the main chain
    best_block: PropNode,
    /// Proposer blocks stored level wise
    prop_nodes: Vec< Vec<PropNode> >,
    /// Leader blocks
    leader_nodes : Vec< Option<PropNode> >
}

impl PropTree{
    /// Get the best block
    pub fn get_best_block(&self) -> &PropNode {
        return &self.best_block;
    }

    /// Get the level of the best block
    pub fn get_best_level(&self) -> &u32 {
        return &self.best_block.level;
    }

    /// Get all the proposer blocks at a level
    pub fn get_all_block_at_level(&self, level: u32) -> &Vec<PropNode> {
        return &self.prop_nodes[level as usize];
    }

    /// Get all potential leader blocks at a level. Used for List Ledger Decoding
    pub fn get_proposer_list_at_level(&self, level: u32) -> Vec<&PropNode> {
        let all_blocks: &Vec<PropNode> = self.get_all_block_at_level(level);
        let mut potential_leaders: Vec<&PropNode> = Vec::new();
        // Todo: filter proposer blocks with maybe leadership status
        return potential_leaders;
    }

    /// Get the proposer block list sequence up to a level. Used for List Ledger Decoding
    pub fn get_proposer_block_sequence(&self, level: u32) -> Vec<Vec<&PropNode>>{
        let best_level = self.get_best_level();
        let mut proposer_list_sequence :Vec<Vec<&PropNode>> = vec![];
        for l in 0..*best_level {
            proposer_list_sequence.push(self.get_proposer_list_at_level(l));
        }
        return proposer_list_sequence;
    }

    /// Get the leader block at a level
    pub fn get_leader_block_at_level(&self, level: u32) -> &Option<PropNode>{
        return &self.leader_nodes[level as usize];
    }

    /// Get the leader block sequence up to a level
    pub fn get_leader_block_sequence(&self, level: u32) -> Vec<&Option<PropNode>>{
        let best_level = self.get_best_level();
        let mut leader_sequence :Vec<&Option<PropNode>> = vec![];
        for l in 0..*best_level {
            leader_sequence.push(self.get_leader_block_at_level(l));
        }
        return leader_sequence;
    }

    /// Add proposer block
    pub fn add_proposer_block(&mut self, node: PropNode) {
        self.prop_nodes[node.level as usize].push(node);
    }

}

/// Stores all the voter nodes
pub struct VoterChain{
    /// Voter chain id
    id: u16,
    /// Genesis block
    genesis_block: VoterNode,
    /// Best block on the main chain
    best_block: VoterNode,
    /// Set of all Voter nodes
    voter_nodes: HashSet<VoterNode>
}

impl VoterChain {
//    /// Initialize the voter tree
//    pub fn new(id: u16, genesis_block: VoterNode) -> Self {
//        let best_block: VoterNode =  genesis_block.clone(); Todo: Define clone
//        let voter_nodes: HashSet<VoterNode> = HashSet::new();
//        voter_nodes.insert(genesis_block.clone()); Todo: Define insert
//        return VoterChain {id, genesis_block, best_block, voter_nodes};
//    }

    /// Get the best block
    pub fn get_best_block(&self) -> &VoterNode {
        return &self.best_block;
    }

    /// Get the level of the best block
    pub fn get_chain_length(&self) -> &u32 {
        return &self.best_block.level;
    }

    /// Add  voter block
    pub fn add_voter_block(&mut self, node: VoterNode){
//        self.tx_nodes.insert(node); Todo: Define a hash insert??

//        if node.parent == self.best_block{
//            self.best_block = node;
//        }
//        else if node.level > self.best_block.level +1 {
//            //  Todo: Reorg!! Return Success status?
//        }


    }
}

pub struct BlockChainGraph{
    tx_block_pool: TxPool,
    prop_block_tree: PropTree,
    voter_chains: Vec<VoterChain>
}

impl BlockChainGraph{

//    pub fn new(number_of_voter_chains: u32) -> Self {
//        let mut voter_chains: Vec<VoterChain> =  vec![];
//
//        /// Initializing voter chains
//        for i in 0..number_of_voter_chains{
//            //Todo: Generate random genesis block
//            // let v_chain = VoterChain::new(i, genesis_block);
//            // voter_chains.push(v_chain);
//        }
//
//        let tx_block_pool: TxPool = TxPool::new();
//        //Todo: Generate random genesis block
//        // let prop_block_tree: TxPool = PropTree::new();
//
//        return BlockChainGraph{}
//    }

    pub fn get_number_of_voter_chains(&self) -> u32{
        return self.voter_chains.len() as u32;
    }

    pub fn add_block<T: Node>(&mut self, node: T){
        // Check the node type and perform the required function.
    }
}