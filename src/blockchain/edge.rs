/// This type is used to represent the different types of edge in the Prism blockchain.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub enum Edge {
    // Tx edge types
    /// From a transaction block to the proposer block parent on which it is mined.
    TransactionToProposerParent,
    // Prop edge types
    /// From a proposer block to the proposer block parent on which it is mined
    ProposerToProposerParent,
    // Voter edge types
    /// From a voter block to the proposer block parent on which it is mined.
    VoterToProposerParent,
    /// From a voter block to the proposer block it votes for.
    VoterToProposerVote,
    /// Acts both as `VoterToProposerParent` and `VoterToProposerVote`.
    VoterToProposerParentAndVote,
    /// From a voter block to the voter block parent on which it is mined.
    VoterToVoterParent,

    /// FEW REVERSE EDGES
    VoterFromProposerVote,
    /// The reverse of `VoterToProposerParentAndVote`.
    VoterFromProposerParentAndVote,

    /// EDGES which have an edge weight with it
    /// From a proposer block to another proposer block it refers to.
    ProposerToProposerReference,
    /// From a proposer block to a transaction block it refers to. The `u32` content is used for
    /// ordering among all types of reference links.
    ProposerToTransactionReference,
    /// From a leader proposer block to the transaction blocks it includes in the ledger.
    // TODO: does not seem to be used anywhere
    ProposerToTransactionLeaderReference,
    /// Acts both as `ProposerToTransactionReference` and `ProposerToTransactionLeaderReference`.
    ProposerToTransactionReferenceAndLeaderReference,
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Edge::TransactionToProposerParent => write!(f, "Tx2PropParent"),
            Edge::ProposerToProposerParent => write!(f, "Prop2PropParent"),
            Edge::ProposerToProposerReference => write!(f, "Prop2PropRef"),
            Edge::ProposerToTransactionReference => write!(f, "Prop2TxRef"),
            Edge::ProposerToTransactionLeaderReference => write!(f, "Prop2TxLeaderRef"),
            Edge::ProposerToTransactionReferenceAndLeaderReference => {
                write!(f, "Prop2TxRefAndLeaderRef")
            }
            Edge::VoterToProposerParent => write!(f, "V2PropParent"),
            Edge::VoterToVoterParent => write!(f, "V2VParent"),
            Edge::VoterToProposerVote => write!(f, "V2PropVote"),
            Edge::VoterToProposerParentAndVote => write!(f, "V2PropParent_and_Vote"),
            // Reverse Edges
            Edge::VoterFromProposerVote => write!(f, "VFromPropVote"),
            Edge::VoterFromProposerParentAndVote => write!(f, "VFromPropParent_and_Vote"),
            _ => write!(f, "Not supported!"),
        }
    }
}

impl Edge {
    /// Returns the variant of the reversed edge.
    pub fn reverse_edge(&self) -> Edge {
        match self {
            Edge::VoterToProposerVote => Edge::VoterFromProposerVote,
            Edge::VoterToProposerParentAndVote => Edge::VoterFromProposerParentAndVote,
            // Reverse Edges
            Edge::VoterFromProposerVote => Edge::VoterToProposerVote,
            Edge::VoterFromProposerParentAndVote => Edge::VoterToProposerParentAndVote,
            _ => panic!("Reverse fot this edge type is not supported"),
        }
    }
}
