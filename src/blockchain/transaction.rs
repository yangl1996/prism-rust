// todo: Txblock currently has no metadata.


//use crate::crypto::hash::{H256};
//
//
//#[derive(Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash, Default)]
//pub struct Tx {
//    /// Parent prop node hash
//    pub parent_prop_node: Option<H256>,
//    /// Prop node which refers this node
//    pub child_prop_node: Option<H256>,
//}
//
//
//impl Default for Tx {
//    fn default() -> Self {
//        let level = 0;
//        let leadership_status = PropBlockLeaderStatus::NotALeader;
//        return Proposer {level, leadership_status};
//    }
//}