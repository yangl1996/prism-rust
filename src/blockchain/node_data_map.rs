use super::database::{BlockChainDatabase, PROPOSER_NODE_DATA_CF, VOTER_NODE_DATA_CF};
use super::proposer::NodeData as ProposerNodeData;
use super::proposer::Status as ProposerStatus;
use super::voter::NodeData as VoterNodeData;
use super::voter::NodeStatus as VoterNodeUpdateStatus;
use bincode::{serialize, deserialize};
use rocksdb::WriteBatch;
use serde::{Deserialize, Serialize};

use crate::crypto::hash::{Hashable, H256};
use std::sync::{Arc, Mutex};


pub struct NodeDataMap {
    pub db: Arc<Mutex<BlockChainDatabase>>,
}

impl NodeDataMap {
    pub fn insert_voter(&self, hash: &H256, node_data: VoterNodeData) {
        let db = self.db.lock().unwrap();
        db.insert(VOTER_NODE_DATA_CF, hash, node_data);
    }

    pub fn insert_proposer(&self, hash: &H256, node_data: ProposerNodeData) {
        let db = self.db.lock().unwrap();
        db.insert(PROPOSER_NODE_DATA_CF, hash, node_data);
    }

    pub fn get_voter(&self, hash: &H256) -> VoterNodeData {
        let db = self.db.lock().unwrap();
        let hash_u8: [u8; 32] = hash.into();
        let cf = db.handle.cf_handle(VOTER_NODE_DATA_CF).unwrap();
        let serialized_result = db.handle.get_cf(cf, &hash_u8);
        match serialized_result {
            Err(e) => panic!("Database error"),
            Ok(serialized_option) => match serialized_option {
                None => panic!("Node data not present"),
                Some(s) => return deserialize(&s).unwrap(),
            },
        }
    }

    pub fn get_proposer(&self, hash: &H256) -> ProposerNodeData {
        let db = self.db.lock().unwrap();
        let hash_u8: [u8; 32] = hash.into();
        let cf = db.handle.cf_handle(PROPOSER_NODE_DATA_CF).unwrap();
        let serialized_result = db.handle.get_cf(cf, &hash_u8);
        match serialized_result {
            Err(e) => panic!("Database error"),
            Ok(serialized_option) => match serialized_option {
                None => panic!("Node data not present"),
                Some(s) => return deserialize(&s).unwrap(),
            },
        }
    }

    pub fn contains_proposer(&self, hash: &H256) -> bool {
        let db = self.db.lock().unwrap();
        match db.check(PROPOSER_NODE_DATA_CF, hash) {
            Err(e) => panic!("Database error"),
            Ok(b) => return b,
        };
    }

    pub fn contains_voter(&self, hash: &H256) -> bool {
        let db = self.db.lock().unwrap();
        match db.check(VOTER_NODE_DATA_CF, hash) {
            Err(e) => panic!("Database error"),
            Ok(b) => return b,
        }
    }


    fn edit<D: Serialize>(&self, cf_name: &str, hash: &H256, data: D) {
        let db = self.db.lock().unwrap();
        let hash_u8: [u8; 32] = hash.into();
        let cf = db.handle.cf_handle(cf_name).unwrap();
        let serialized_data = serialize(&data).unwrap();
        let mut batch = WriteBatch::default();
        batch.delete_cf(cf, hash_u8);
        batch.put_cf(cf, hash_u8, serialized_data);
        db.handle.write(batch);
    }
}

// Proposer Node Data edits
impl NodeDataMap {
    pub fn give_proposer_leader_status(&self, hash: &H256) {
        let mut prop_node_data = self.get_proposer(hash);
        prop_node_data.leadership_status = ProposerStatus::Leader;
        self.edit(PROPOSER_NODE_DATA_CF, hash, prop_node_data);
    }

    pub fn give_proposer_potential_leader_status(&self, hash: &H256) {
        let mut prop_node_data = self.get_proposer(hash);
        prop_node_data.leadership_status = ProposerStatus::PotentialLeader;
        self.edit(PROPOSER_NODE_DATA_CF, hash, prop_node_data);
    }

    pub fn give_proposer_not_leader_status(&self, hash: &H256) {
        let mut prop_node_data = self.get_proposer(hash);
        prop_node_data.leadership_status = ProposerStatus::NotLeaderUnconfirmed;
        self.edit(PROPOSER_NODE_DATA_CF, hash, prop_node_data);
    }

    pub fn give_proposer_not_leader_confirmed_status(&self, hash: &H256) {
        let mut prop_node_data = self.get_proposer(hash);
        prop_node_data.leadership_status = ProposerStatus::NotLeaderAndConfirmed;
        self.edit(PROPOSER_NODE_DATA_CF, hash, prop_node_data);
    }

    pub fn proposer_increment_vote(&self, hash: &H256) {
        let mut prop_node_data = self.get_proposer(hash);
        prop_node_data.votes += 1;
        self.edit(PROPOSER_NODE_DATA_CF, hash, prop_node_data);
    }

    pub fn proposer_decrement_vote(&self, hash: &H256) {
        let mut prop_node_data = self.get_proposer(hash);
        prop_node_data.votes -= 1;
        self.edit(PROPOSER_NODE_DATA_CF, hash, prop_node_data);
    }

}

// Voter Node Data edits
impl NodeDataMap {
    /// Mark the block as orphan.
    pub fn voter_make_orphan(&self, hash: &H256) {
        let mut voter_node_data = self.get_voter(hash);
        voter_node_data.status = VoterNodeUpdateStatus::Orphan;
        self.edit(VOTER_NODE_DATA_CF, hash, voter_node_data);
    }

    /// Mark the block as on the main chain.
    pub fn voter_make_on_main_chain(&mut self, hash: &H256) {
        let mut voter_node_data = self.get_voter(hash);
        voter_node_data.status = VoterNodeUpdateStatus::OnMainChain;
        self.edit(VOTER_NODE_DATA_CF, hash, voter_node_data);

    }


}
