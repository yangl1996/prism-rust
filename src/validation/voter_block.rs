use super::super::config;
use super::data_availability::*;
use super::*;
use crate::block::Block;
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use std::sync::{Arc, Mutex};

/// Checks data availability and prism logic
pub struct VoterBlockRule {
    blockchain: Arc<Mutex<BlockChain>>,
    block_db: Arc<BlockDatabase>,
}
impl BlockRule for VoterBlockRule {
    fn result(&self, block: &Block) -> BlockRuleResult {
        let content = block.get_voter_content();
        let chain_number = content.chain_number;
        //1. Check if the chain number is valid
        if chain_number < 0 || chain_number > config::NUM_VOTER_CHAINS {
            return BlockRuleResult::False;
        }
        let mut blocks_not_in_db: Vec<H256> = vec![];
        let mut blocks_not_in_blockchain: Vec<H256> = vec![];

        //2. Check if the parent voter block is available
        let mut latest_level_voted_by_ancestor: usize = 0;
        let voter_parent =
            get_available_block(content.voter_parent_hash, &self.blockchain, &self.block_db);
        match voter_parent {
            BlockDataAvailability::NotInDB => {
                // The voter parent should be requested from the network
                blocks_not_in_db.push(content.voter_parent_hash);
            }
            BlockDataAvailability::NotInBlockchain => {
                // The voter parent should be added to the blockchain first
                blocks_not_in_blockchain.push(content.voter_parent_hash);
            }
            BlockDataAvailability::Block(voter_parent_block) => {
                latest_level_voted_by_ancestor = latest_level_voted_on_chain(
                    &voter_parent_block,
                    &self.blockchain,
                    &self.block_db,
                );
            }
        }

        //3. Check if all voted proposer blocks are available and have continuous by level from latest_level_voted_by_ancestor onwards
        for (index, proposer_vote) in content.proposer_block_votes.iter().enumerate() {
            let proposer_block =
                get_available_block(*proposer_vote, &self.blockchain, &self.block_db);
            match proposer_block {
                BlockDataAvailability::NotInDB => {
                    // The voter parent should be requested from the network
                    blocks_not_in_db.push(content.voter_parent_hash);
                }
                BlockDataAvailability::NotInBlockchain => {
                    // The voter parent should be added to the blockchain first
                    blocks_not_in_blockchain.push(content.voter_parent_hash);
                }
                BlockDataAvailability::Block(block) => {
                    let blockchain_l = self.blockchain.lock().unwrap();
                    let level = blockchain_l.prop_node_data(&block.hash()).level as usize;
                    drop(blockchain_l);
                    if level != index + 1 + latest_level_voted_by_ancestor {
                        //The votes  are not on contigous levels
                        return BlockRuleResult::False;
                    }
                }
            }
        }
        // The block is valid if all the data is available
        if blocks_not_in_db.len() == 0 && blocks_not_in_blockchain.len() == 0 {
            return BlockRuleResult::True;
        } else {
            return BlockRuleResult::MissingReferencesInDBandBC(
                blocks_not_in_db,
                blocks_not_in_blockchain,
            );
        }
    }
}

/// Returns the last proposer level voted by the voter chain until 'voter_block'
fn latest_level_voted_on_chain(
    voter_block: &Block,
    blockchain: &Mutex<BlockChain>,
    block_db: &BlockDatabase,
) -> usize {
    let mut content = voter_block.get_voter_content();
    // Find the latest ancestor on the chain which has voted
    loop {
        // If the ancestor has any votes, then return the latest vote
        if content.proposer_block_votes.len() > 0 {
            let latest_prop_voted = content.proposer_block_votes.last().unwrap();
            let blockchain_l = blockchain.lock().unwrap();
            return blockchain_l.prop_node_data(latest_prop_voted).level as usize;
        } else {
            let voter_parent = get_available_block(content.voter_parent_hash, blockchain, block_db);
            match voter_parent {
                BlockDataAvailability::Block(voter_block) => unimplemented!(), //content = voter_block.get_voter_content(),
                _ => panic!("This shouldn't have happened"),
            }
        }
    }
}

// TODO: Add tests
