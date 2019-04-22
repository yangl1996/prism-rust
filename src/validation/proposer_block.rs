use super::super::config;
use super::data_availability::*;
use super::*;
use crate::block::Block;
use crate::blockchain::BlockChain;
use crate::blockchain::utils::get_proposer_genesis_hash;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use std::sync::{Arc, Mutex};

// TODO: To check more rules to detect adversarial behaviour
/// Checks data availability.
pub struct ProposerBlockRule {
    blockchain: Arc<Mutex<BlockChain>>,
    block_db: Arc<BlockDatabase>,
}
impl BlockRule for ProposerBlockRule {
    fn result(&self, block: &Block) -> BlockRuleResult {
        let content = block.get_proposer_content();
        let mut blocks_not_in_db: Vec<H256> = vec![];
        let mut blocks_not_in_blockchain: Vec<H256> = vec![];

        // Check 1: Check data availability of tx blocks referenced
        for tx_block_hash in content.transaction_block_hashes.iter(){
            let tx_block =
                get_available_block(*tx_block_hash, &self.blockchain, &self.block_db);
            match tx_block {
                BlockDataAvailability::NotInDB => {
                    // The voter parent should be requested from the network
                    blocks_not_in_db.push(*tx_block_hash);
                }
                BlockDataAvailability::NotInBlockchain => {
                    // The voter parent should be added to the blockchain first
                    blocks_not_in_blockchain.push(*tx_block_hash);
                }
                BlockDataAvailability::Block(_) => {
                    // do nothing. this is the best case
                }
            }
        }

        // Check 2: Check data availability of prop blocks referenced
        for prop_block_hash in content.proposer_block_hashes.iter(){
            let prop_block =
                get_available_block(*prop_block_hash, &self.blockchain, &self.block_db);
            match prop_block {
                BlockDataAvailability::NotInDB => {
                    // The voter parent should be requested from the network
                    blocks_not_in_db.push(*prop_block_hash);
                }
                BlockDataAvailability::NotInBlockchain => {
                    // The voter parent should be added to the blockchain first
                    blocks_not_in_blockchain.push(*prop_block_hash);
                }
                BlockDataAvailability::Block(_) => {
                    // do nothing. this is the best case
                }
            }
        }

        //Final result: If all the data is available then the block is valid
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


// TODO: Add tests
