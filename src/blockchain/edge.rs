/// This type is used to represent the different types of edge in the Prism blockchain.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub enum Edge {
    // Tx edge types
    /// From a transaction block to the proposer block parent on which it is mined.
    TransactionToProposerParent,
    // Prop edge types
    /// From a proposer block to the proposer block parent on which it is mined
    ProposerToProposerParent,
    /// From a proposer block to another proposer block it refers to. The `u32` content
    /// is used for ordering among all types of reference links.
    ProposerToProposerReference(u32),
    /// From a proposer block to a transaction block it refers to. The `u32` content is used for
    /// ordering among all types of reference links.
    ProposerToTransactionReference(u32),
    /// From a leader proposer block to the transaction blocks it includes in the ledger.
    // TODO: does not seem to be used anywhere
    ProposerToTransactionLeaderReference(u32),
    /// Acts both as `ProposerToTransactionReference` and `ProposerToTransactionLeaderReference`.
    ProposerToTransactionReferenceAndLeaderReference(u32),
    // Voter edge types
    /// From a voter block to the proposer block parent on which it is mined.
    VoterToProposerParent,
    /// From a voter block to the proposer block it votes for.
    VoterToProposerVote,
    /// Acts both as `VoterToProposerParent` and `VoterToProposerVote`.
    VoterToProposerParentAndVote,
    /// From a voter block to the voter block parent on which it is mined.
    VoterToVoterParent,

    /// The reverse of `TransactionToProposerParent`.
    TransactionFromProposerParent,
    /// The reverse of `ProposerToProposerParent`.
    ProposerFromProposerParent,
    /// The reverse of `ProposerToProposerReference`.
    ProposerFromProposerReference(u32),
    /// The reverse of `ProposerToTransactionReference`.
    ProposerFromTransactionReference(u32),
    /// The reverse of `ProposerToTransactionLeaderReference`.
    ProposerFromTransactionLeaderReference(u32),
    /// The reverse of `ProposerToTransactionReferenceAndLeaderReference`.
    ProposerFromTransactionReferenceAndLeaderReference(u32),
    // Voter edge types
    /// The reverse of `VoterToProposerParent`.
    VoterFromProposerParent,
    /// The reverse of `VoterToProposerVote`.
    VoterFromProposerVote,
    /// The reverse of `VoterToProposerParentAndVote`.
    VoterFromProposerParentAndVote,
    /// The reverse of `VoterToVoterParent`.
    VoterFromVoterParent,
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Edge::TransactionToProposerParent => write!(f, "Tx2PropParent"),
            Edge::ProposerToProposerParent => write!(f, "Prop2PropParent"),
            Edge::ProposerToProposerReference(_position) => write!(f, "Prop2PropRef"),
            Edge::ProposerToTransactionReference(_position) => write!(f, "Prop2TxRef"),
            Edge::ProposerToTransactionLeaderReference(_position) => write!(f, "Prop2TxLeaderRef"),
            Edge::ProposerToTransactionReferenceAndLeaderReference(_position) => {
                write!(f, "Prop2TxRefAndLeaderRef")
            }
            Edge::VoterToProposerParent => write!(f, "V2PropParent"),
            Edge::VoterToVoterParent => write!(f, "V2VParent"),
            Edge::VoterToProposerVote => write!(f, "V2PropVote"),
            Edge::VoterToProposerParentAndVote => write!(f, "V2PropParent_and_Vote"),
            // Reverse Edges
            Edge::TransactionFromProposerParent => write!(f, "TxFromPropParent"),
            Edge::ProposerFromProposerParent => write!(f, "PropFromPropParent"),
            Edge::ProposerFromProposerReference(_position) => write!(f, "PropFromPropRef"),
            Edge::ProposerFromTransactionReference(_position) => write!(f, "PropFromTxRef"),
            Edge::ProposerFromTransactionLeaderReference(_position) => {
                write!(f, "PropFromTxLeaderRef")
            }
            Edge::ProposerFromTransactionReferenceAndLeaderReference(_position) => {
                write!(f, "Prop2TxRefAndLeaderRef")
            }
            Edge::VoterFromProposerParent => write!(f, "VFromPropParent"),
            Edge::VoterFromVoterParent => write!(f, "VFromVParent"),
            Edge::VoterFromProposerVote => write!(f, "VFromPropVote"),
            Edge::VoterFromProposerParentAndVote => write!(f, "VFromPropParent_and_Vote"),
        }
    }
}

impl Edge {
    /// Returns the variant of the reversed edge.
    pub fn reverse_edge(&self) -> Edge {
        match self {
            Edge::TransactionToProposerParent => Edge::TransactionFromProposerParent,
            Edge::ProposerToProposerParent => Edge::ProposerFromProposerParent,
            Edge::ProposerToProposerReference(position) => {
                Edge::ProposerFromProposerReference(*position)
            }
            Edge::ProposerToTransactionReference(position) => {
                Edge::ProposerFromTransactionReference(*position)
            }
            Edge::ProposerToTransactionLeaderReference(position) => {
                Edge::ProposerFromTransactionLeaderReference(*position)
            }
            Edge::ProposerToTransactionReferenceAndLeaderReference(position) => {
                Edge::ProposerFromTransactionReferenceAndLeaderReference(*position)
            }
            Edge::VoterToProposerParent => Edge::VoterFromProposerParent,
            Edge::VoterToVoterParent => Edge::VoterFromVoterParent,
            Edge::VoterToProposerVote => Edge::VoterFromProposerVote,
            Edge::VoterToProposerParentAndVote => Edge::VoterFromProposerParentAndVote,
            // Reverse Edges
            Edge::TransactionFromProposerParent => Edge::TransactionToProposerParent,
            Edge::ProposerFromProposerParent => Edge::ProposerToProposerParent,
            Edge::ProposerFromProposerReference(position) => {
                Edge::ProposerToProposerReference(*position)
            }
            Edge::ProposerFromTransactionReference(position) => {
                Edge::ProposerToTransactionReference(*position)
            }
            Edge::ProposerFromTransactionLeaderReference(position) => {
                Edge::ProposerToTransactionLeaderReference(*position)
            }
            Edge::ProposerFromTransactionReferenceAndLeaderReference(position) => {
                Edge::ProposerToTransactionReferenceAndLeaderReference(*position)
            }
            Edge::VoterFromProposerParent => Edge::VoterToProposerParent,
            Edge::VoterFromVoterParent => Edge::VoterToVoterParent,
            Edge::VoterFromProposerVote => Edge::VoterToProposerVote,
            Edge::VoterFromProposerParentAndVote => Edge::VoterToProposerParentAndVote,
        }
    }
}
