/*
Validation for proposer blocks
*/

use crate::block::{Block,Content,PROPOSER_INDEX};
use crate::blockchain::{BlockChain,NUM_VOTER_CHAINS};
use crate::crypto::hash::{Hashable,H256};


pub struct ProposerBlockValidator<'a> {
    // Database of known blocks
    pub blockchain: &'a BlockChain
}

impl<'a> super::Validator<'a> for ProposerBlockValidator<'a> {

    fn new(blockchain: &'a BlockChain) -> Self {
        ProposerBlockValidator { blockchain: blockchain }
    }

    fn is_valid(&self, block: &'a Block) -> bool {
        
        if (
            self.is_duplicate(&block) || // 1. Check duplicate
            self.is_empty(&block) ||  // 2. Check if empty reflinks
            !self.is_coinbase_valid(&block) || // 3. Check coinbase validity
            !self.is_pow_valid(&block) // 4. check pow validity, sortition
        ) {
            return false;
        }
        return true;
    }

    fn is_duplicate(&self, block: &'a Block) -> bool {
        // Checks if we already have a copy of this block in storage
        return self.blockchain.proposer_node_data_map.contains_key(&block.hash())
    }

    fn is_empty(&self, block: &'a Block) -> bool {
        // Checks if (a)  this is a proposer block, and (b) the proposer 
        // reflinks at least are nonempty 
        match &block.content {
            Content::Transaction(c) => return true,
            Content::Voter(c) => return true,
            Content::Proposer(c) => {
                return (c.proposer_block_hashes.is_empty() &&   
                        c.transaction_block_hashes.is_empty())
            }
        }
        return true;
    }

    fn is_coinbase_valid(&self, block: &'a Block) -> bool {
        // TODO: replace with coinbase transaction validity check once  
        // coinbase tx gets added
        return true;
    }

    fn is_pow_valid(&self, block: &'a Block) -> bool {
        let header_hash: H256 = block.header.hash();

        // Check that the sortition is in the correct range
        let num_chains = NUM_VOTER_CHAINS + 2;
        let difficulty = block.header.difficulty;
        // let ratio = difficulty * PROPOSER_INDEX / num_chains;
        // if (
        //     header_hash < (difficulty * PROPOSER_INDEX / num_chains) || 
        //     header_hash >= (difficulty * (PROPOSER_INDEX + 1) / num_chains)
        // ) {
        //     return false;
        // }
        return true;
    }


}
