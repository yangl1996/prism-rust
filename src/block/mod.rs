pub mod header;
pub mod pos_metadata;
pub mod proposer;
pub mod transaction;
pub mod voter;

use crate::crypto::hash::{Hashable, H256};
use crate::experiment::performance_counter::PayloadSize;
use crate::utxodb::Utxo;

/// A block in the Prism blockchain.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    /// The header of the block.
    pub header: header::Header,
    /// The content of the block. It could contain transactions, references, or votes, depending on
    /// the block type.
    pub content: Content,
}

impl Block {
    /// Create a new block.
    pub fn new(
        parent: H256,
        pos_metadata: pos_metadata::Metadata,
        content_root: H256,
        extra_content: [u8; 32],
        difficulty: H256,
        header_signature: Vec<u8>,
        content: Content,
    ) -> Self {
        let header = header::Header::new(
            parent,
            pos_metadata,
            content_root,
            extra_content,
            difficulty,
            header_signature,
        );
        Self {
            header,
            content,
        }
    }

    // TODO: use another name
    /// Create a new block from header.
    pub fn from_header(
        header: header::Header,
        content: Content,
    ) -> Self {
        Self {
            header,
            content,
        }
    }
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        // TODO: we are only hashing the header here.
        return self.header.hash();
    }
}

impl PayloadSize for Block {
    fn size(&self) -> usize {
        return std::mem::size_of::<header::Header>()
            + self.content.size()

    }
}

/// The content of a block. It could be transaction content, proposer content, or voter content,
/// depending on the type of the block.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Content {
    /// Transaction block content.
    Transaction(transaction::Content),
    /// Proposer block content.
    Proposer(proposer::Content),
    /// Voter block content.
    Voter(voter::Content),
}

impl Hashable for Content {
    fn hash(&self) -> H256 {
        match self {
            Content::Transaction(c) => c.hash(),
            Content::Proposer(c) => c.hash(),
            Content::Voter(c) => c.hash(),
        }
    }
}

impl PayloadSize for Content {
    fn size(&self) -> usize {
        // TODO: we are not counting the 2 bits that are used to store block type
        match self {
            Content::Transaction(c) => c.size(),
            Content::Proposer(c) => c.size(),
            Content::Voter(c) => c.size(),
        }
    }
}

#[cfg(any(test, feature = "test-utilities"))]
pub mod tests {
    use super::*;
    use super::pos_metadata::{TimeStamp, RandomSource, Metadata};
    use crate::config;
    use crate::transaction::Transaction;
    use rand::Rng;
    use rand::rngs::OsRng;
    use ed25519_dalek::{Keypair, PublicKey, Signature};
    use crate::crypto::vrf::{VrfSecretKey, VrfPublicKey, VrfInput, vrf_evaluate, VrfValue};

    macro_rules! random_source {
        () => {{
            let mut rng = rand::thread_rng();
            let mut random_source = [0u8;32];
            for i in 0..32 {
                random_source[i] = rng.gen();
            }
            random_source
        }};
    }

    pub fn proposer_block(
        parent: H256,
        timestamp: TimeStamp,
        proposer_refs: Vec<H256>,
        transaction_refs: Vec<H256>,
    ) -> Block {
        let content = Content::Proposer(proposer::Content {
            transaction_refs,
            proposer_refs,
        });
        let content_hash = content.hash();
        // let mut metadata = Metadata::default();
        // metadata.timestamp = timestamp;
        // metadata.random_source = random_source!();

        let mut csprng: OsRng = OsRng::new().unwrap();
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let vrf_pubkey: VrfPublicKey = (&keypair.public).into();
        
        let mut utxo = Utxo::default();
        utxo.owner = vrf_pubkey.hash();
        
        //random source from parent
        let random_source_mining = random_source!();

        let input = VrfInput {
            random_source: random_source_mining,
            time: timestamp,
            coin: utxo.coin.clone(),
        };
        let (vrf_value, vrf_proof) = vrf_evaluate(&vrf_pubkey, &keypair.secret, &input);

        let metadata = Metadata {
            vrf_proof,
            vrf_value,
            vrf_pubkey,
            utxo: utxo,
            parent_random_source: random_source_mining,
            timestamp: timestamp,
            random_source: random_source!(),
        };
        
        // sign the header with keypair
        let header = header::Header {
            parent: parent,
            pos_metadata: metadata.clone(),
            content_root: content_hash,
            extra_content: [0u8; 32],
            difficulty: *config::DEFAULT_DIFFICULTY,
            header_signature: vec![],
        };

        let raw_unsigned = bincode::serialize(&header).unwrap();
        let header_signature = keypair.sign(&raw_unsigned).to_bytes().to_vec();

        //assert!(keypair.verify(&raw_unsigned, &keypair.sign(&raw_unsigned)).is_ok());

        //let public_key: PublicKey = keypair.public;
        //assert!(public_key.verify(&raw_unsigned, &keypair.sign(&raw_unsigned)).is_ok());

        Block::new(
            parent,
            metadata,
            content_hash,
            [0u8; 32],
            *config::DEFAULT_DIFFICULTY,
            header_signature,
            content,
        )
    }

    pub fn voter_block(
        parent: H256,
        timestamp: u128,
        chain_number: u16,
        votes: Vec<H256>,
    ) -> Block {
        let content = Content::Voter(voter::Content {
            chain_number,
            votes,
        });
        let content_hash = content.hash();
        let mut metadata = Metadata::default();
        metadata.timestamp = timestamp;
        metadata.random_source = random_source!();
        Block::new(
            parent,
            metadata,
            content_hash,
            [0u8; 32],
            *config::DEFAULT_DIFFICULTY,
            vec![],
            content,
        )
    }

    pub fn transaction_block(
        parent: H256,
        timestamp: u128,
        transactions: Vec<Transaction>,
    ) -> Block {
        let content = Content::Transaction(transaction::Content { transactions });
        let content_hash = content.hash();
        let mut metadata = Metadata::default();
        metadata.timestamp = timestamp;
        metadata.random_source = random_source!();
        Block::new(
            parent,
            metadata,
            content_hash,
            [0u8; 32],
            *config::DEFAULT_DIFFICULTY,
            vec![],
            content,
        )
    }
}
