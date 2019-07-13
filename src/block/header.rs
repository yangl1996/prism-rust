use crate::crypto::hash::{Hashable, H256};
use crate::crypto::vrf;
use crate::crypto::vrf::{VrfPublicKey, VrfSecretKey, VrfInput, VrfProof, VrfOutput};
use super::pos_metadata::Metadata;

// TODO: Add the address of the miner


/// The header of a block.
#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub struct Header {
    /// Hash of the parent proposer block.
    pub parent: H256,
    /// Proof of stake leader election.
    pub pos_metadata: Metadata,
    /// Merkle root of the block content.
    pub content_root: H256,
    /// Extra content for debugging purposes.
    pub extra_content: [u8; 32],
    /// POS mining difficulty of this block.
    pub difficulty: H256,
    /// Signature on the header.
    pub header_signature: Vec<u8>,
}

impl Header {
    /// Create a new block header.
    pub fn new(
        parent: H256,
        pos_metadata: Metadata,
        content_root: H256,
        extra_content: [u8; 32],
        difficulty: H256,
        header_signature: Vec<u8>,
    ) -> Self {
        Self {
            parent,
            pos_metadata,
            content_root,
            extra_content,
            difficulty,
            header_signature
        }
    }

    pub fn pos_hash(&self) -> VrfOutput {
        return self.pos_metadata.vrf_output;
    }
}

impl Hashable for Header {
    fn hash(&self) -> H256 {
        let serialized = bincode::serialize(self).unwrap();
        let digest = ring::digest::digest(&ring::digest::SHA256, &serialized);
        return digest.into();
    }
}




//
//#[cfg(test)]
//pub mod tests {
//    use super::super::header::Header;
//    use super::super::proposer::Content as ProposerContent;
//    use super::super::transaction::Content as tx_Content;
//    use super::super::voter::Content as voter_Content;
//    use super::super::{Block, Content};
//    use std::cell::RefCell;
//
//    use crate::crypto::hash::{Hashable, H256};
//    use crate::crypto::merkle::MerkleTree;
//    use crate::transaction::{Authorization, CoinId, Input, Output, Transaction};
//
//    ///The hash should match
//    #[test]
//    fn test_hash() {
//        let header = sample_header();
//        let header_hash_should_be = sample_header_hash_should_be();
//        assert_eq!(header.hash(), header_hash_should_be);
//    }
//
//    /// Any changes to the header should change the hash value.
//    #[test]
//    fn fake_parent() {
//        let mut header = sample_header();
//        let header_hash_should_be = sample_header_hash_should_be();
//        let fake_parent_hash: H256 =
//            (&hex!("0102010201020102010201020102010291790343908920102010201020102454")).into();
//        header.parent = fake_parent_hash;
//        assert_ne!(header.hash(), header_hash_should_be);
//    }
//
//    #[test]
//    fn fake_timestamp() {
//        let mut header = sample_header();
//        let header_hash_should_be = sample_header_hash_should_be();
//        let fake_timestamp: u128 = 73948732;
//        header.timestamp = fake_timestamp;
//        assert_ne!(header.hash(), header_hash_should_be);
//    }
//
//    #[test]
//    fn fake_nonce() {
//        let mut header = sample_header();
//        let header_hash_should_be = sample_header_hash_should_be();
//        let fake_nonce: u32 = 209830934;
//        header.nonce = fake_nonce;
//        assert_ne!(header.hash(), header_hash_should_be);
//    }
//
//    #[test]
//    fn fake_content_root() {
//        let mut header = sample_header();
//        let header_hash_should_be = sample_header_hash_should_be();
//        let fake_content_root: H256 =
//            (&hex!("beefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef")).into();
//        header.content_merkle_root = fake_content_root;
//        assert_ne!(header.hash(), header_hash_should_be);
//    }
//
//    #[test]
//    fn fake_extra_content() {
//        let mut header = sample_header();
//        let header_hash_should_be = sample_header_hash_should_be();
//        let fake_extra_content: [u8; 32] = [
//            1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//            0, 0, 1,
//        ];
//        header.extra_content = fake_extra_content;
//        assert_ne!(header.hash(), header_hash_should_be);
//    }
//
//    #[test]
//    fn fake_difficulty() {
//        let mut header = sample_header();
//        let header_hash_should_be = sample_header_hash_should_be();
//        let fake_difficulty: [u8; 32] = [
//            1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//            0, 0, 1,
//        ];
//        header.difficulty = (&fake_difficulty).into();
//        assert_ne!(header.hash(), header_hash_should_be);
//    }
//
//    #[macro_export]
//    macro_rules! gen_hashed_data {
//        () => {{
//            vec![
//                (&hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
//                (&hex!("0102010201020102010201020102010201020102010201020102010201020102")).into(),
//                (&hex!("0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b0a0a0a0a0b0b0b0b")).into(),
//                (&hex!("0403020108070605040302010807060504030201080706050403020108070605")).into(),
//                (&hex!("1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a1a2a3a4a")).into(),
//                (&hex!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")).into(),
//                (&hex!("0000000100000001000000010000000100000001000000010000000100000001")).into(),
//            ]
//        }};
//    }
//
//    // Header stuff
//    pub fn sample_header() -> Header {
//        let parent_hash: H256 =
//            (&hex!("0102010201020102010201020102010201020102010201020102010201020102")).into();
//        let timestamp: u128 = 7094730;
//        let nonce: u32 = 839782;
//        let content_root: H256 =
//            (&hex!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")).into();
//        let extra_content: [u8; 32] = [
//            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//            0, 0, 0,
//        ];
//        let difficulty: [u8; 32] = [
//            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//            0, 20, 10,
//        ];
//        let difficulty = (&difficulty).into();
//        let header = Header::new(
//            parent_hash,
//            timestamp,
//            nonce,
//            content_root,
//            extra_content,
//            difficulty,
//        );
//        return header;
//    }
//
//    pub fn sample_header_hash_should_be() -> H256 {
//        let header_hash_should_be =
//            (&hex!("1d6e0f5f11248d070c6b15d103ead72a8802279c600a60b40b38510b582aacbf")).into(); // Calculated on Mar 15, 2019
//        return header_hash_should_be;
//    }
//
//}
