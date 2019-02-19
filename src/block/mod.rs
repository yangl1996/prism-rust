pub mod address;
pub mod block_header;
pub mod proposer;
pub mod transaction;
pub mod voter;

use crate::hash::{self, Hashable};

pub enum Metadata {
    VoterMetadata(voter::VoterMetadata),
    ProposerMetadata(proposer::ProposerMetadata),
}

impl Hashable for Metadata {
    fn hash(&self) -> hash::Hash {
        match &self {
            Metadata::VoterMetadata(mt) => return mt.hash(),
            Metadata::ProposerMetadata(mt) => return mt.hash(),
        }
    }
}

/// A block stores the essential information that will be used in the DAG
pub struct Block {
    header: block_header::BlockHeader,
    transactions: Vec<transaction::Transaction>,
    metadata: Metadata,
}

impl Block {
    pub fn parent(&self) -> &hash::Hash {
        match &self.metadata {
            Metadata::VoterMetadata(mt) => return &mt.parent,
            Metadata::ProposerMetadata(mt) => return &mt.level_cert,
        };
    }

    pub fn references(&self) -> &[hash::Hash] {
        match &self.metadata {
            Metadata::VoterMetadata(mt) => return std::slice::from_ref(&mt.parent),
            Metadata::ProposerMetadata(mt) => return &mt.ref_links,
        };
    }

    pub fn hash(&self) -> hash::Hash {
        return self.header.hash();
    }
}

#[cfg(test)]
mod tests {
    use super::block_header::BlockHeader;
    use super::hash::{self, Hashable};
    use super::proposer::ProposerMetadata;
    use super::voter::VoterMetadata;
    use super::Block;
    use super::Metadata;

    macro_rules! fake_proposer {
        ( $parent_hash:expr, $ref_links:expr, $nonce:expr ) => {{
            let metadata = Metadata::ProposerMetadata(ProposerMetadata {
                level_cert: $parent_hash,
                ref_links: $ref_links,
            });
            Block {
                header: BlockHeader {
                    voter_hash: hash::Hash([0; 32]),
                    proposal_hash: metadata.hash(),
                    transactions_hash: hash::Hash([0; 32]),
                    nonce: $nonce,
                },
                transactions: vec![],
                metadata: metadata,
            }
        }};
    }

    macro_rules! fake_voter {
        ( $parent_hash:expr, $nonce:expr ) => {{
            let metadata = Metadata::VoterMetadata(VoterMetadata {
                votes: vec![],
                parent_merkle_root: hash::Hash([0; 32]),
                parent_proofs: vec![],
                parent: $parent_hash,
            });
            Block {
                header: BlockHeader {
                    voter_hash: metadata.hash(),
                    proposal_hash: hash::Hash([0; 32]),
                    transactions_hash: hash::Hash([0; 32]),
                    nonce: $nonce,
                },
                transactions: vec![],
                metadata: metadata,
            }
        }};
    }

    #[test]
    fn parent() {
        let voter_blk = fake_voter!(
            hash::Hash(hex!(
                "1122334455667788112233445566778811223344556677881122334455667788"
            )),
            1
        );
        assert_eq!(voter_blk.parent(), &hash::Hash(hex!("1122334455667788112233445566778811223344556677881122334455667788")));

        let proposer_blk = fake_proposer!(
            hash::Hash(hex!(
                "1122334455667788112233445566778811223344556677881122334455667788"
            )),
            vec![],
            1
        );
        assert_eq!(proposer_blk.parent(), &hash::Hash(hex!("1122334455667788112233445566778811223344556677881122334455667788")));
    }

    #[test]
    fn hash() {
        let voter_blk = fake_voter!(
            hash::Hash(hex!(
                "1122334455667788112233445566778811223344556677881122334455667788"
            )),
            1
        );
        assert_eq!(voter_blk.hash(), voter_blk.header.hash());

        let proposer_blk = fake_proposer!(
            hash::Hash(hex!(
                "1122334455667788112233445566778811223344556677881122334455667788"
            )),
            vec![],
            1
        );
        assert_eq!(proposer_blk.hash(), proposer_blk.header.hash());
    }

    #[test]
    fn references() {
        let voter_blk = fake_voter!(
            hash::Hash(hex!(
                "1122334455667788112233445566778811223344556677881122334455667788"
            )),
            1
        );
        let voter_ref = [hash::Hash(hex!("1122334455667788112233445566778811223344556677881122334455667788"))];
        assert_eq!(voter_blk.references(), &voter_ref);

        let proposer_blk = fake_proposer!(
            hash::Hash(hex!(
                "1122334455667788112233445566778811223344556677881122334455667788"
            )),
            vec![hash::Hash(hex!("0000000011111111000000001111111100000000111111110000000011111111")), 
            hash::Hash(hex!("1010101010101010101010101010101010101010101010101010101010101010"))],
            1
        );
        let proposer_ref = [hash::Hash(hex!("0000000011111111000000001111111100000000111111110000000011111111")), hash::Hash(hex!("1010101010101010101010101010101010101010101010101010101010101010"))];
        assert_eq!(proposer_blk.references(), &proposer_ref);
    }
}
